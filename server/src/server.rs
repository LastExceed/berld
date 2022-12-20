use std::{io, ptr, thread};
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::mem::size_of;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use colour::white_ln;
use parking_lot::RwLock;

use protocol::{CwSerializable, packet, Packet, SIZE_ZONE};
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::*;
use protocol::packet::common::{CreatureId, Item};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::drops::Drop;
use protocol::utils::io_extensions::{ReadExtension, WriteExtension};

use crate::creature::Creature;
use crate::creature_id_pool::CreatureIdPool;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::pvp::enable_pvp;

pub struct Server {
	players: RwLock<Vec<Arc<Player>>>,
	id_pool: RwLock<CreatureIdPool>,
	drops: RwLock<HashMap<Point2<i32>, Vec<Drop>>>
}

const TIMEOUT: Duration = Duration::from_secs(5);

impl Server {
	pub fn new() -> Self {
		Self {
			players: RwLock::new(Vec::new()),
			id_pool: RwLock::new(CreatureIdPool::new()),
			drops: RwLock::new(HashMap::new())
		}
	}

	pub fn run(self) {
		self.id_pool.write().claim(); //reserve 0 for the server itself

		let listener = TcpListener::bind("0.0.0.0:12345").expect("unable to bind listening socket");

		let self_arc = Arc::new(self);

		loop {
			let (mut stream, _) = listener.accept().unwrap();
			stream.set_read_timeout(Some(TIMEOUT)).expect("read timeout rejected");
			stream.set_write_timeout(Some(TIMEOUT)).expect("write timeout rejected");

			let self_arc_clone = self_arc.clone();
			thread::spawn(move || {
				if let Err(_) = self_arc_clone.handle_new_connection(&mut stream) {
					//TODO: error logging
				}
				stream.shutdown(Shutdown::Both).expect("TODO: panic message");
			});
		}
	}

	pub fn broadcast<Packet: FromServer>(&self, packet: &Packet, player_to_skip: Option<&Player>) where [(); size_of::<Packet>()]: {
		for player in self.players.read().iter() {
			if match player_to_skip {
				Some(player_to_skip) => ptr::eq(player.as_ref(), player_to_skip),
				None => false
			} { continue }

			player.send_ignoring(packet);
		}
	}

	pub fn add_drop(&self, item: Item, position: Point3<i64>, rotation: f32) {
		let zone = position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32);

		let drops_to_send = {
			let mut drops_guard = self.drops.write();
			let zone_drops = drops_guard.entry(zone).or_insert(vec![]);
			zone_drops.push(Drop {
				item,
				position,
				rotation,
				scale: 0.1,
				unknown_a: 0,
				unknown_b: 0,
				droptime: 0
			});

			let mut zone_drops_copy = zone_drops.clone();
			zone_drops_copy[zone_drops.len() - 1].droptime = 500;

			zone_drops_copy
		};//scope ensures the guard is dropped asap

		self.broadcast(&WorldUpdate {
			drops: vec![(zone, drops_to_send)],
			..Default::default()
		}, None);
	}

	///returns none if a player picks up an item they dropped in single player
	pub fn remove_drop(&self, zone: Point2<i32>, item_index: usize) -> Option<Item> {
		let (remaining_zone_drops, removed_item) = {
			let mut drops_guard = self.drops.write();

			let Some(zone_drops) = drops_guard.get_mut(&zone) else { return None };

			let removed_drop = zone_drops.swap_remove(item_index);
			let zone_drops_owned = zone_drops.to_owned();
			if zone_drops.is_empty() {
				drops_guard.remove(&zone);
			}

			(zone_drops_owned, removed_drop.item)
		};//scope ensures the guard is dropped asap

		self.broadcast(&WorldUpdate {
			drops: vec![(zone, remaining_zone_drops)],
			..Default::default()
		}, None);

		Some(removed_item)
	}

	fn handle_new_connection(&self, stream: &mut TcpStream) -> Result<(), io::Error> {
		if stream.read_struct::<packet::Id>()? != ProtocolVersion::ID
			|| ProtocolVersion::read_from(stream)?.0 != 3 {
			return Err(io::Error::from(ErrorKind::InvalidData))
		}
		let assigned_id = self.id_pool.write().claim();
		let result = self.handle_new_player(stream, assigned_id);
		self.id_pool.write().free(assigned_id);
		result
	}

	fn handle_new_player(&self, stream: &mut TcpStream, assigned_id: CreatureId) -> Result<(), io::Error> {
		ConnectionAcceptance {}.write_to_with_id(stream)?;

		write_abnormal_creature_update(stream, assigned_id)?;

		if stream.read_struct::<packet::Id>()? != CreatureUpdate::ID {
			return Err(io::Error::from(ErrorKind::InvalidData))
		}
		let mut full_creature_update = CreatureUpdate::read_from(stream)?;
		enable_pvp(&mut full_creature_update);

		let new_player = Player::new(
			Creature::maybe_from(&full_creature_update).ok_or_else(|| io::Error::from(ErrorKind::InvalidData))?,
			stream,
		);


		new_player.send(&MapSeed(225))?;
		new_player.send(&ChatMessageFromServer {
			source: CreatureId(0),
			text: "welcome to berld".to_string()
		})?;

		for existing_player in self.players.read().iter() {
			new_player.send(&existing_player.creature.read().to_update())?;
		}

		WorldUpdate {
			drops: self.drops.read()
				.clone()
				.into_iter()
				.collect(),
			..Default::default()
		}.write_to_with_id(stream)?;

		let new_player_arc = Arc::new(new_player);
		self.players.write().push(new_player_arc.clone());
		self.broadcast(&new_player_arc.creature.read().to_update(), None);

		self.announce(format!("[+] {}", new_player_arc.creature.read().name));

		let _ = self.read_packets_forever(&new_player_arc, stream)
			.expect_err("impossible"); //TODO: check if error emerged from reading or writing

		self.remove_player(&new_player_arc);

		self.announce(format!("[-] {}", new_player_arc.creature.read().name));

		Ok(())
	}

	fn remove_player(&self, player_to_remove: &Player) {
		{
			let mut players = self.players.write();
			let index = players.iter().position(|player| ptr::eq(player_to_remove, player.as_ref())).expect("player not found");
			players.swap_remove(index);
		};
		self.remove_creature(&player_to_remove.creature.read().id);
	}

	fn remove_creature(&self, creature_id: &CreatureId) {
		//this is a shortcut, as the creature technically still exists
		//the proper way to remove a creature requires updating all remaining creatures which is expensive on bandwidth
		self.broadcast(&CreatureUpdate {
			id: creature_id.to_owned(),
			health: Some(0f32), //makes the creature intangible
			affiliation: Some(Affiliation::Neutral), //ensures it doesnt show up on the map
			..Default::default()
		}, None);
	}

	fn read_packets_forever<Readable: Read>(&self, source: &Player, readable: &mut Readable) -> Result<(), io::Error> {
		loop {
			//todo: copypasta
			match readable.read_struct::<packet::Id>()? {
				CreatureUpdate       ::ID => self.handle_packet(source, CreatureUpdate       ::read_from(readable)?)?,
				CreatureAction       ::ID => self.handle_packet(source, CreatureAction       ::read_from(readable)?)?,
				Hit                  ::ID => self.handle_packet(source, Hit                  ::read_from(readable)?)?,
				StatusEffect         ::ID => self.handle_packet(source, StatusEffect         ::read_from(readable)?)?,
				Projectile           ::ID => self.handle_packet(source, Projectile           ::read_from(readable)?)?,
				ChatMessageFromClient::ID => self.handle_packet(source, ChatMessageFromClient::read_from(readable)?)?,
				ZoneDiscovery        ::ID => self.handle_packet(source, ZoneDiscovery        ::read_from(readable)?)?,
				RegionDiscovery      ::ID => self.handle_packet(source, RegionDiscovery      ::read_from(readable)?)?,
				unexpected_packet_id => panic!("unexpected packet id {:?}", unexpected_packet_id)
			}
		}
	}

	fn announce(&self, text: String) {
		white_ln!("{}", text);
		self.broadcast(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		}, None);
	}
}

/// during new player setup the server needs to send an abnormal CreatureUpdate which:
/// * is not compressed (and lacks the size prefix used for compressed packets)
/// * has no bitfield indicating the presence of its properties
/// * falls 8 bytes short of representing a full creature
///
/// unfortunately it is impossible to determine which bytes are missing exactly,
/// as the only reference is pixxie from the vanilla server, which is almost completely zeroed.
/// the last non-zero bytes in pixxie are the equipped weapons, which are positioned correctly.
/// from that it can be deduced that the missing bytes belong to the last 3 properties.
/// it's probably a cut-off at the end resulting from an incorrectly sized buffer
fn write_abnormal_creature_update(writable: &mut impl Write, assigned_id: CreatureId) -> Result<(), io::Error> {
	writable.write_struct(&CreatureUpdate::ID)?;
	writable.write_struct(&assigned_id)?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
	writable.write_all(&[0u8; 4456]) //so we can simply zero out everything else and not worry about the missing bytes
	//TODO: move this to protocol crate and construct this from an actual [CreatureUpdate]
}