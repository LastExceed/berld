use std::{io, thread};
use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use std::mem::size_of;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;

use protocol::nalgebra::{Point2, Point3};
use protocol::packet::*;
use protocol::packet::common::{CreatureId, Item, PacketId};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::drops::Drop;
use protocol::SIZE_ZONE;
use protocol::utils::io_extensions::{ReadExtension, WriteExtension};

use crate::creature::Creature;
use crate::creature_id_pool::CreatureIdPool;
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

	pub fn broadcast<P: PacketFromServer>(&self, packet: &P, player_to_skip: Option<&Arc<Player>>) where [(); size_of::<P>()]: {
		for player in self.players.read().iter() {
			if match player_to_skip {
				Some(player_to_skip) => Arc::ptr_eq(player, player_to_skip),
				None => false
			} { continue }
			player.send(packet);
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
		if stream.read_struct::<PacketId>()? != PacketId::ProtocolVersion
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

		//at this point the server needs to send an abnormal CreatureUpdate which
		// A.) is not compressed (and lacks the size prefix used for compressed packets)
		// B.) has no bitfield indicating the presence of its properties
		// C.) falls 8 bytes short of representing a full creature
		//unfortunately it is impossible to determine which bytes are missing exactly, as the only reference is pixxie from the vanilla server, which is almost completely zeroed
		//the last non-zero bytes in pixxie are the equipped weapons, which are positioned correctly. from that we can deduce that the missing bytes belong to the last 3 properties
		//it's probably a cut-off at the end resulting from an incorrectly sized buffer
		stream.write_struct(&PacketId::CreatureUpdate)?;
		stream.write_struct(&assigned_id)?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
		stream.write_struct(&[0u8; 0x1168])?; //so we can simply zero out everything else and not worry about the missing bytes

		if stream.read_struct::<PacketId>()? != PacketId::CreatureUpdate {
			return Err(io::Error::from(ErrorKind::InvalidData))
		}
		let mut full_creature_update = CreatureUpdate::read_from(stream)?;
		enable_pvp(&mut full_creature_update);

		let me = Player::new(
			Creature::maybe_from(&full_creature_update).ok_or_else(|| io::Error::from(ErrorKind::InvalidData))?,
			stream,
		);


		me.send(&MapSeed(225));
		me.send(&ChatMessageFromServer {
			source: CreatureId(0),
			text: "welcome to berld".to_string()
		});

		for player in self.players.read().iter() {
			me.send(&player.creature.read().to_update());
		}

		WorldUpdate {
			drops: self.drops.read()
				.clone()
				.into_iter()
				.collect(),
			..Default::default()
		}.write_to_with_id(stream)?;

		let player_arc = Arc::new(me);
		self.players.write().push(player_arc.clone());
		self.broadcast(&player_arc.creature.read().to_update(), None);

		self.read_packets(player_arc.clone(), stream).expect_err("impossible");

		{
			let mut players = self.players.write();
			let index = players.iter().position(|other_player| Arc::ptr_eq(&player_arc, other_player)).expect("player not found");
			players.swap_remove(index);
		};
		self.broadcast(&CreatureUpdate {
			id: assigned_id,
			health: Some(0f32),
			affiliation: Some(Affiliation::Neutral),
			..Default::default()
		}, None);

		Ok(())
	}

	fn read_packets<T: Read>(&self, source: Arc<Player>, readable: &mut T) -> Result<(), io::Error> {
		loop {
			let packet_id = readable.read_struct::<PacketId>()?;
			match packet_id {
				PacketId::CreatureUpdate  => self.on_creature_update (&source, CreatureUpdate       ::read_from(readable)?)?,
				PacketId::CreatureAction  => self.on_creature_action (&source, CreatureAction       ::read_from(readable)?)?,
				PacketId::Hit             => self.on_hit             (&source, Hit                  ::read_from(readable)?)?,
				PacketId::StatusEffect    => self.on_status_effect   (&source, StatusEffect         ::read_from(readable)?)?,
				PacketId::Projectile      => self.on_projectile      (&source, Projectile           ::read_from(readable)?)?,
				PacketId::ChatMessage     => self.on_chat_message    (&source, ChatMessageFromClient::read_from(readable)?)?,
				PacketId::ZoneDiscovery   => self.on_zone_discovery  (&source, ZoneDiscovery        ::read_from(readable)?)?,
				PacketId::RegionDiscovery => self.on_region_discovery(&source, RegionDiscovery      ::read_from(readable)?)?,
				_ => panic!("unexpected packet id {:?}", packet_id)
			}
		}
	}
}