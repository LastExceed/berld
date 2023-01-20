use std::collections::HashMap;
use std::io::ErrorKind;
use std::mem::{size_of, transmute};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use colour::white_ln;
use futures::future;
use tokio::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::sleep;

use protocol::{CwSerializable, packet, Packet};
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::*;
use protocol::packet::common::{CreatureId, Item};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::drops::Drop;
use protocol::packet::world_update::sound_effect::Sound;
use protocol::packet::world_update::SoundEffect;
use protocol::utils::constants::SIZE_ZONE;
use protocol::utils::io_extensions::{ReadStruct, WriteStruct};
use protocol::utils::sound_position_of;

use crate::creature::Creature;
use crate::creature_id_pool::CreatureIdPool;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::pvp::enable_pvp;

pub struct Server {
	pub players: RwLock<Vec<Arc<Player>>>,
	id_pool: RwLock<CreatureIdPool>,
	drops: RwLock<HashMap<Point2<i32>, Vec<Drop>>>
}

impl Server {
	pub fn new() -> Self {
		Self {
			players: RwLock::new(Vec::new()),
			id_pool: RwLock::new(CreatureIdPool::new()),
			drops: RwLock::new(HashMap::new())
		}
	}

	pub async fn run(self) {
		self.id_pool.write().await.claim(); //reserve 0 for the server itself

		let listener = TcpListener::bind("0.0.0.0:12345").await.expect("unable to bind listening socket");

		let self_arc = Arc::new(self);

		loop {
			let (stream, _) = listener.accept().await.unwrap();
			stream.set_nodelay(true).unwrap();

			let self_arc_clone = self_arc.clone();
			tokio::spawn(async move {
				let (read_half, write_half) = stream.into_split();
				let write_lock_arc = Arc::new(RwLock::new(write_half));
				if let Err(_) = self_arc_clone.handle_new_connection(read_half, write_lock_arc.clone()).await {
					//TODO: error logging
				}
				write_lock_arc.write().await.shutdown().await.expect("TODO: panic message");
			});
		}
	}

	async fn handle_new_connection(&self, mut read_half: OwnedReadHalf, write_half: Arc<RwLock<OwnedWriteHalf>>) -> io::Result<()> {
		if read_half.read_struct::<packet::Id>().await? != ProtocolVersion::ID
			|| ProtocolVersion::read_from(&mut read_half).await?.0 != 3 {
			return Err(io::Error::from(ErrorKind::InvalidData));
		}
		let assigned_id = self.id_pool.write().await.claim();
		let result = self.handle_new_player(read_half, write_half, assigned_id).await;
		self.id_pool.write().await.free(assigned_id);
		result
	}

	async fn handle_new_player(&self, mut read_half: OwnedReadHalf, write_half: Arc<RwLock<OwnedWriteHalf>>, assigned_id: CreatureId) -> io::Result<()> {
		{//todo: extract individual functions
			let write_half_lock = &mut write_half.write().await as &mut OwnedWriteHalf;
			ConnectionAcceptance {}.write_to_with_id(write_half_lock).await?;
			write_abnormal_creature_update(write_half_lock, assigned_id).await?;
		};

		if read_half.read_struct::<packet::Id>().await? != CreatureUpdate::ID {
			return Err(io::Error::from(ErrorKind::InvalidData))
		}
		let mut full_creature_update = CreatureUpdate::read_from(&mut read_half).await?;

		enable_pvp(&mut full_creature_update);

		let new_player = Player::new(
			Creature::maybe_from(&full_creature_update).ok_or_else(|| io::Error::from(ErrorKind::InvalidData))?,
			write_half.clone(),
		);
		new_player.send(&MapSeed(225)).await?;
		new_player.notify("welcome to berld").await;

		for existing_player in self.players.read().await.iter() {
			new_player.send(&existing_player.creature.read().await.to_update()).await?;
		}

		new_player.send(&WorldUpdate {
			drops: self.drops.read().await
				.clone()
				.into_iter()
				.collect(),
			..Default::default()
		}).await?;

		self.announce(format!("[+] {}", new_player.creature.read().await.name)).await;

		let new_player_arc = Arc::new(new_player);
		self.players.write().await.push(new_player_arc.clone());
		self.broadcast(&new_player_arc.creature.read().await.to_update(), None).await;

		let _ = self.read_packets_forever(&new_player_arc, &mut read_half).await
			.expect_err("impossible"); //TODO: check if error emerged from reading or writing

		self.remove_player(&new_player_arc).await;

		self.announce(format!("[-] {}", new_player_arc.creature.read().await.name)).await;

		Ok(())
	}

	pub async fn broadcast<Packet: FromServer + Sync>(&self, packet: &Packet, player_to_skip: Option<&Player>) where [(); size_of::<Packet>()]: {
		future::join_all(self.players.read().await.iter().filter_map(|player| {
			if let Some(pts) = player_to_skip && ptr::eq(player.as_ref(), pts) {
				return None;
			}

			Some(player.send_ignoring(packet))
		})).await;
	}

	pub async fn add_drop(&self, item: Item, position: Point3<i64>, rotation: f32) {
		let zone = position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32);

		let drops_to_send = {
			let mut drops_guard = self.drops.write().await;
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
			sound_effects: vec![
				SoundEffect {
					position: sound_position_of(position),
					sound: Sound::Drop,
					pitch: 1f32,
					volume: 1f32
				}
			],
			..Default::default()
		}, None).await;

		let server_static: &'static Server = unsafe { transmute(self) }; //todo: scoped task
		tokio::spawn(async move {
			sleep(Duration::from_millis(500)).await;
			let sound_effect = SoundEffect {
				position: sound_position_of(position),
				sound: Sound::DropItem,
				pitch: 1f32,
				volume: 1f32
			};
			server_static.broadcast(&WorldUpdate::from(sound_effect), None).await;
		});
	}

	///returns none if a player picks up an item they dropped in single player
	pub async fn remove_drop(&self, zone: Point2<i32>, item_index: usize) -> Option<Item> {
		let (remaining_zone_drops, removed_item) = {
			let mut drops_guard = self.drops.write().await;

			let Some(zone_drops) = drops_guard.get_mut(&zone) else { return None };

			let removed_drop = zone_drops.swap_remove(item_index);
			let zone_drops_owned = zone_drops.to_owned();
			if zone_drops.is_empty() {
				drops_guard.remove(&zone);
			}

			(zone_drops_owned, removed_drop.item)
		};//scope ensures the guard is dropped asap

		self.broadcast(&WorldUpdate::from((zone, remaining_zone_drops)), None).await;

		Some(removed_item)
	}

	async fn remove_player(&self, player_to_remove: &Player) {
		{
			let mut players = self.players.write().await;
			let index = players.iter().position(|player| ptr::eq(player_to_remove, player.as_ref())).expect("player not found");
			players.swap_remove(index);
		};
		self.remove_creature(&player_to_remove.creature.read().await.id).await;
	}

	async fn remove_creature(&self, creature_id: &CreatureId) {
		//this is a shortcut, as the creature technically still exists
		//the proper way to remove a creature requires updating all remaining creatures which is expensive on bandwidth
		self.broadcast(&CreatureUpdate {
			id: creature_id.to_owned(),
			health: Some(0f32), //makes the creature intangible
			affiliation: Some(Affiliation::Neutral), //ensures it doesnt show up on the map
			..Default::default()
		}, None).await;
	}

	async fn read_packets_forever(&self, source: &Player, readable: &mut OwnedReadHalf) -> io::Result<()> {
		loop {
			//todo: copypasta
			match readable.read_struct::<packet::Id>().await? {
				CreatureUpdate       ::ID => self.handle_packet(source, CreatureUpdate       ::read_from(readable).await?).await?,
				CreatureAction       ::ID => self.handle_packet(source, CreatureAction       ::read_from(readable).await?).await?,
				Hit                  ::ID => self.handle_packet(source, Hit                  ::read_from(readable).await?).await?,
				StatusEffect         ::ID => self.handle_packet(source, StatusEffect         ::read_from(readable).await?).await?,
				Projectile           ::ID => self.handle_packet(source, Projectile           ::read_from(readable).await?).await?,
				ChatMessageFromClient::ID => self.handle_packet(source, ChatMessageFromClient::read_from(readable).await?).await?,
				ZoneDiscovery        ::ID => self.handle_packet(source, ZoneDiscovery        ::read_from(readable).await?).await?,
				RegionDiscovery      ::ID => self.handle_packet(source, RegionDiscovery      ::read_from(readable).await?).await?,
				unexpected_packet_id => panic!("unexpected packet id {:?}", unexpected_packet_id)
			}
		}
	}

	async fn announce(&self, text: String) {
		white_ln!("{}", text);
		self.broadcast(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		}, None).await;
	}

	pub(crate) async fn kick(&self, player: &Player, reason: String) {
		self.announce(format!("kicked {} because {}", player.creature.read().await.name, reason)).await;
		//wait a bit to make sure the message arrives at the player about to be kicked
		sleep(Duration::from_millis(100)).await;
		player.close_connection().await;
		//remove_player will be called by the reading thread
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
async fn write_abnormal_creature_update<Writable: AsyncWrite + Unpin + Send>(writable: &mut Writable, assigned_id: CreatureId) -> io::Result<()> {
	writable.write_struct(&CreatureUpdate::ID).await?;
	writable.write_struct(&assigned_id).await?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
	writable.write_all(&[0u8; 4456]).await //so we can simply zero out everything else and not worry about the missing bytes
	//TODO: move this to protocol crate and construct this from an actual [CreatureUpdate]
}