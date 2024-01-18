use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::io::ErrorKind::{InvalidData, InvalidInput, UnexpectedEof};
use std::net::SocketAddr;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use colour::dark_grey_ln;
use config::{Config, ConfigError};
use futures::future::join_all;
use tap::{Pipe, Tap};
use tokio::{io, select};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;
use tokio::time::sleep;

use protocol::{Packet, WriteCwData};
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::{*, Hit};
use protocol::packet::area_request::{Region, Zone};
use protocol::packet::common::{CreatureId, Item};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::loot::GroundItem;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::*;
use protocol::utils::constants::SIZE_ZONE;
use protocol::utils::io_extensions::{ReadPacket, WriteArbitrary, WritePacket};

use crate::addon::{Addons, freeze_time, announce_join_leave};
use crate::addon::pvp::map_head;
use crate::addon::pvp;
use crate::server::creature::Creature;
use crate::server::creature_id_pool::CreatureIdPool;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;

use self::utils::{extend_lifetime, log_error};

pub mod creature_id_pool;
pub mod player;
mod handle_packet;
pub mod creature;
pub mod utils;

pub struct Server {
	id_pool: RwLock<CreatureIdPool>,
	pub players: RwLock<Vec<Arc<Player>>>,
	loot: RwLock<HashMap<Point2<i32>, Vec<GroundItem>>>,
	pub mapseed: i32,
	pub addons: Addons
}

impl Server {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let instance = Self {
			id_pool: Default::default(),
			players: Default::default(),
			loot: Default::default(),
			mapseed: config.get("seed")?,
			addons: Addons::new(config)?,
		};

		Ok(instance)
	}

	pub async fn run(self) -> ! {
		let mut id_pool = self.id_pool.write().await;
		let _ = id_pool.claim(); //reserve 0 for the server itself
		pvp::team::display::reserve_dummy_ids(&mut id_pool);
		drop(id_pool);

		//cubeworld is incapable of ipv6 networking
		let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 12345)).await.expect("unable to bind listening socket");

		self.addons.listforge_api.run(&self).await;
		self.addons.discord_integration.run(&self);
		freeze_time(&self);

		loop {
			let Ok((stream, address)) = listener
				.accept()
				.await
				.inspect_err(|err| log_error("tcp-accept", err))
				else { continue };
			dark_grey_ln!("new connection from {}", address);

			let self_static = extend_lifetime(&self);
			tokio::spawn(async move {
				let result = self_static.handle_new_connection(stream, address).await;
				if let Err(err) = result {
					log_error("handle-new-connection", err);
				}
			});
		}
	}

	async fn handle_new_connection(&self, stream: TcpStream, address: SocketAddr) -> io::Result<()> {
		stream.set_nodelay(true)?;
		let (mut reader, mut writer) = split_and_buffer(stream);

		let check_version_result = check_version(&mut reader, &mut writer).await;
		if let Err(ref error) = check_version_result && error.kind() == UnexpectedEof {
			//prevent listforge's preiodic connections from flooding the logs
			return Ok(());
		}
		check_version_result?;

		writer.write_packet(&ConnectionAcceptance).await?;

		let assigned_id = self.id_pool.write().await.claim();
		write_abnormal_creature_update(&mut writer, assigned_id).await?;

		let (initial_creature_update, character) = read_character_data(&mut reader).await?;

		let (new_player, kick_receiver) = Player::new(
			address,
			assigned_id,
			character,
			writer,
		);
		let player = Arc::new(new_player);
		self.players.write().await.push(Arc::clone(&player));
		announce_join_leave(self, &player, true).await;

		self.handle_packet(&player, initial_creature_update).await;

		select! {
			biased;
			_ = kick_receiver => (),
			() = self.handle_new_player(reader, &player) => ()
		}

		self.remove_player(&player).await;
		self.id_pool.write().await.free(assigned_id);

		Ok(())
	}

	async fn handle_new_player(&self, reader: BufReader<OwnedReadHalf>, player: &Player) {
		player.send_ignoring(&MapSeed(self.mapseed)).await;
		let player_count = self.players.read().await.len();
		player.notify(format!("welcome to berld\n{} player(s) connected", player_count)).await;
		send_existing_creatures(self, player).await;

		self.read_packets_forever(player, reader).await
			.expect_err("impossible");
	}

	pub async fn broadcast<Packet: FromServer>(&self, packet: &Packet, player_to_skip: Option<&Player>)
		where BufWriter<OwnedWriteHalf>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		self.players
			.read()
			.await
			.iter()
			.filter(|player| !player_to_skip.is_some_and(|pts| ptr::eq(player.as_ref(), pts)))
			.map(|player| player.send_ignoring(packet))
			.pipe(join_all)
			.await;
	}

	pub async fn add_drop(&self, item: Item, position: Point3<i64>, rotation: f32) {
		let zone = position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32);

		let mut loot = self.loot.write().await;
		let zone_loot = loot.entry(zone).or_insert(vec![]);
		zone_loot.push(GroundItem {
			item,
			position,
			rotation,
			scale: 0.1,
			unknown_a: 0,
			unknown_b: 0,
			droptime: 0
		});
		let mut zone_loot_copy = zone_loot.clone();
		zone_loot_copy[zone_loot.len() - 1].droptime = 500;
		drop(loot);

		self.broadcast(&WorldUpdate {
			loot: HashMap::from([(zone, zone_loot_copy)]),
			sounds: vec![Sound::at(position, Drop)],
			..Default::default()
		}, None).await;

		let server_static = extend_lifetime(self);
		tokio::spawn(async move {
			sleep(Duration::from_millis(500)).await;
			server_static.broadcast(&WorldUpdate::from(Sound::at(position, DropItem)), None).await;
		});
	}

	///returns none if a player picks up an item they dropped in single player
	pub async fn remove_drop(&self, zone: Point2<i32>, item_index: usize) -> Option<Item> {
		let mut drops_guard = self.loot.write().await;
		let Some(zone_drops) = drops_guard.get_mut(&zone) else { return None; };
		let removed_drop = zone_drops.swap_remove(item_index);
		let zone_drops_owned = zone_drops.clone();
		if zone_drops.is_empty() {
			drops_guard.remove(&zone);
		}
		drop(drops_guard);

		self.broadcast(&WorldUpdate::from((zone, zone_drops_owned)), None).await;

		Some(removed_drop.item)
	}

	async fn remove_player(&self, player_to_remove: &Player) {
		let mut players = self.players.write().await;
		let index = players
			.iter()
			.position(|player| ptr::eq(player_to_remove, player.as_ref()))
			.expect("this should be the only place where players get removed");
		let player = players.swap_remove(index);
		drop(players);
		announce_join_leave(self, &player, false).await;
		pvp::team::change_to(self, player_to_remove, None).await;
		self.remove_creature(&player_to_remove.id).await;
		self.broadcast(&pvp::map_head::create_toggle_packet(&player, false), None).await;
	}

	async fn remove_creature(&self, creature_id: &CreatureId) {
		//this is a shortcut, as the creature technically still exists
		//the proper way to remove a creature requires updating all remaining creatures which is expensive on bandwidth
		self.broadcast(&CreatureUpdate {
			id: *creature_id,
			health: Some(0.0), //makes the creature intangible
			affiliation: Some(Affiliation::Neutral), //ensures it doesnt show up on the map
			..Default::default()
		}, None).await;
	}

	async fn read_packets_forever(&self, source: &Player, mut reader: BufReader<OwnedReadHalf>) -> io::Result<()> {
		loop {
			let iteration = async {
				//todo: copypasta
				match reader.read_id().await? {
					CreatureUpdate       ::ID => self.handle_packet(source, reader.read_packet::<CreatureUpdate       >().await?).await,
					CreatureAction       ::ID => self.handle_packet(source, reader.read_packet::<CreatureAction       >().await?).await,
					Hit                  ::ID => self.handle_packet(source, reader.read_packet::<Hit                  >().await?).await,
					StatusEffect         ::ID => self.handle_packet(source, reader.read_packet::<StatusEffect         >().await?).await,
					Projectile           ::ID => self.handle_packet(source, reader.read_packet::<Projectile           >().await?).await,
					ChatMessageFromClient::ID => self.handle_packet(source, reader.read_packet::<ChatMessageFromClient>().await?).await,
					AreaRequest::<Zone>  ::ID => self.handle_packet(source, reader.read_packet::<AreaRequest<Zone>    >().await?).await,
					AreaRequest::<Region>::ID => self.handle_packet(source, reader.read_packet::<AreaRequest<Region>  >().await?).await,
					unexpected_packet_id => panic!("unexpected packet id {unexpected_packet_id:?}")
				};

				io::Result::<_>::Ok(()) //todo: why do we need explicit type annotation here?
			};

			select! {
				biased;
				result = iteration => { result?; continue; },
				() = sleep(Duration::from_secs(10)) => { self.kick(source, "connection timeout").await; }
			}
		}
	}
}

//todo: way too much pvp stuff in here
//todo: status effects (including team hearts)
async fn send_existing_creatures(server: &Server, player: &Player) {
	pvp::team::display::reload(player, &[]).await;
	let own_team = player.addon_data.read().await.team;
	server
		.players
		.read()
		.await
		.iter()
		.filter(|existing_player| !ptr::eq(existing_player.as_ref(), player))
		.map(|existing_player| async {
			let character = existing_player
				.character
				.read()
				.await;

			let other_team = existing_player.addon_data.read().await.team;
			let is_teammate = own_team.is_some() && own_team == other_team;

			let creature_update = character
				.to_update(existing_player.id)
				.tap_mut(|packet| {
					packet.affiliation = Some(if is_teammate { Affiliation::Player } else { Affiliation::Enemy });
					packet.rarity = Some(if is_teammate { 0 } else { 4 });
				});
			let map_head = map_head::create(&character, existing_player.id);
			drop(character);

			player.send_ignoring(&creature_update).await;
			player.send_ignoring(&map_head).await;
		})
		.pipe(join_all)
		.await;
}

fn split_and_buffer(stream: TcpStream) -> (BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>) {
	let (read_half, write_half) = stream.into_split();

	let reader = BufReader::new(read_half);
	let writer = BufWriter::new(write_half);

	(reader, writer)
}

async fn check_version(reader: &mut impl ReadPacket, writer: &mut impl WritePacket<ProtocolVersion>) -> io::Result<()> {
	if reader.read_id().await? != ProtocolVersion::ID {
		return Err(InvalidData.into());
	}

	if reader.read_packet::<ProtocolVersion>().await?.0 != 3 {
		writer.write_packet(&ProtocolVersion(3)).await?;
		return Err(InvalidInput.into());
	}

	Ok(())
}

/// during new player setup the server needs to send an abnormal `CreatureUpdate` which:
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
	writable.write_arbitrary(&CreatureUpdate::ID).await?;
	writable.write_arbitrary(&assigned_id).await?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
	writable.write_all(&[0_u8; 4456]).await?; //so we can simply zero out everything else and not worry about the missing bytes
	writable.flush().await
	//TODO: move this to protocol crate and construct this from an actual [CreatureUpdate]
}

async fn read_character_data(reader: &mut impl ReadPacket) -> io::Result<(CreatureUpdate, Creature)> {
	if reader.read_id().await? != CreatureUpdate::ID {
		return Err(InvalidData.into());
	}

	let creature_update = reader.read_packet::<CreatureUpdate>().await?;

	let Some(character) = Creature::maybe_from(&creature_update) else {
		return Err(InvalidInput.into());
	};

	Ok((creature_update, character))
}