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
use tap::{Pipe as _, Tap as _};
use tokio::{io, select};
use tokio::io::{AsyncWrite, AsyncWriteExt as _};
use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

use protocol::{Packet as _, WriteCwData};
use protocol::nalgebra::{Point2, Point3, Vector3};
use protocol::packet::{*, Hit};
use protocol::packet::area_request::{Region, Zone};
use protocol::packet::common::{CreatureId, Item};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::loot::GroundItem;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::*;
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use protocol::utils::io_extensions::{ReadPacket, WriteArbitrary as _, WritePacket};

use crate::addon::{Addons, announce_join_leave};
use crate::addon::pvp::map_head;
use crate::addon::pvp;
use crate::server::creature::Creature;
use crate::server::creature_id_pool::CreatureIdPool;
use crate::server::handle_packet::HandlePacket as _;
use crate::server::player::Player;
use crate::SERVER;

use self::utils::log_error;

pub mod creature_id_pool;
pub mod player;
mod handle_packet;
pub mod creature;
pub mod utils;

const TIMEOUT: Duration = Duration::from_secs(30);

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

	pub async fn run(&self) -> ! {
		self.initialize_id_pool().await;
		self.addons.start().await;

		//cubeworld is incapable of ipv6 networking
		let listener = TcpListener
			::bind((Ipv4Addr::UNSPECIFIED, 12345))
			.await
			.expect("unable to bind listening socket");

		loop { // infinite
			_ = self.spawn_session(&listener).await;
		}
	}

	async fn initialize_id_pool(&self) {
		let mut id_pool = self.id_pool.write().await;
		let _ = id_pool.claim(); //reserve 0 for the server itself
		pvp::team::display::reserve_dummy_ids(&mut id_pool);
	}
	
	async fn spawn_session(&self, listener: &TcpListener) -> io::Result<()> {
		let (stream, address) = listener
			.accept()
			.await
			.inspect_err(|err| log_error("tcp-accept", err))
			?;

		dark_grey_ln!("new connection from {}", address);

		tokio::spawn(async move {
			_ = SERVER
				.initialize_session(stream, address)
				.await
				.inspect_err(|err| log_error("handle-new-connection", err));
		});
		
		Ok(())
	}

	async fn initialize_session(&self, stream: TcpStream, address: SocketAddr) -> io::Result<()> {
		let (mut reader, mut writer) = configure_stream(stream)?;

		match check_version(&mut reader, &mut writer).await {
			Ok(())                                      => writer.write_packet(&ConnectionAcceptance).await?,
			Err(error) if error.kind() == UnexpectedEof => return Ok(()), // prevent listforge's preiodic connections from flooding the logs,
			Err(error)                                  => return Err(error)
		}

		let assigned_id = self.assign_id(&mut writer).await?;
		let (initial_creature_update, character) = read_character_data(&mut reader).await?;

		let (player, kick_receiver) = Player::new(
			address,
			assigned_id,
			character,
			writer,
		);
		let player = Arc::new(player);

		self.players.write().await.push(Arc::clone(&player));

		self.handle_packet(&player, initial_creature_update).await;
		self.initialize_player(&player).await;
		
		select! {
			biased;
			_ = kick_receiver => {},
			() = self.read_packets_forever(&player, reader) => {}
		};

		self.remove_player(&player).await;
		self.id_pool.write().await.free(assigned_id);

		Ok(())
	}
	
	async fn assign_id<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<CreatureId> {
		let assigned_id = self
			.id_pool
			.write()
			.await
			.claim();

		// at this point the server needs to send an abnormal `CreatureUpdate` which:
		// * is not compressed (and lacks the size prefix used for compressed packets)
		// * has no bitfield indicating the presence of its properties
		// * falls 8 bytes short of representing a full creature
		//
		// unfortunately it is impossible to determine which bytes are missing exactly,
		// as the only reference is pixxie from the vanilla server, which is almost completely zeroed.
		// the last non-zero bytes in pixxie are the equipped weapons, which are positioned correctly.
		// from that it can be deduced that the missing bytes belong to the last 3 properties.
		// it's probably a cut-off at the end resulting from an incorrectly sized buffer
		writable.write_arbitrary(&CreatureUpdate::ID).await?;
		writable.write_arbitrary(&assigned_id).await?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
		writable.write_all(&[0_u8; 4456]).await?; //so we can simply zero out everything else and not worry about the missing bytes
		writable.flush().await?;
		//TODO: move this to protocol crate and construct this from an actual [CreatureUpdate]
		
		Ok(assigned_id)
	}

	async fn initialize_player(&self, player: &Player) {
		player.send_ignoring(&MapSeed(self.mapseed)).await;
		announce_join_leave(self, player, true).await;
		send_existing_creatures(self, player).await;
		self.send_motd(player).await;
	}

	async fn send_motd(&self, player: &Player) {
		let message = format!(
			"welcome to berld\n{} player(s) connected",
			self.players.read().await.len()
		);
		
		player.notify(message).await;
	}

	pub async fn broadcast<Packet: FromServer>(&self, packet: &Packet, player_to_skip: Option<&Player>)
		where Vec<u8>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		let mut data = vec![];
		
		data.write_packet(packet).await.expect("failed to serialize a packet in-memory");
		
		_ = self.players
			.read()
			.await
			.iter()
			.filter(|player| !player_to_skip.is_some_and(|pts| ptr::eq(player.as_ref(), pts)))
			.map(async |player| {
				let mut writer = player
					.writer
					.write()
					.await;

				writer.write_all(&data).await?;
				writer.flush().await
			})
			.pipe(join_all)
			.await;
	}

	pub async fn add_drop(&self, item: Item, position: Point3<i64>, rotation: f32) {
		let zone = position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32);

		let mut loot = self.loot.write().await;
		let zone_loot = loot.entry(zone).or_insert(vec![]);
		zone_loot.push(GroundItem {
			item,
			position: position + Vector3::new(0, 0, SIZE_BLOCK / 10),
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

		tokio::spawn(async move {
			sleep(Duration::from_millis(500)).await;
			SERVER.broadcast(&WorldUpdate::from(Sound::at(position, DropItem)), None).await;
		});
	}

	//returns none if a player picks up an item that doesn't exist
	//this can happen when the item was dropped in single player
	//or when spamming pickup really fast
	pub async fn remove_drop(&self, zone: Point2<i32>, item_index: usize) -> Option<Item> {
		let mut drops_guard = self.loot.write().await;
		let zone_drops = drops_guard.get_mut(&zone)?;
		if !(0..zone_drops.len()).contains(&item_index) {
			return None;
		}
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

	async fn read_packets_forever(&self, source: &Player, mut reader: BufReader<OwnedReadHalf>) {
		loop {
			let future = self.process1packet(source, &mut reader);

			let Ok(io_result) = timeout(TIMEOUT, future).await
			else {
				self.kick(source, "connection timeout").await;
				break
			};
			
			if io_result.is_err() { // player disconnected
				break // todo: distinguish error kinds?
			}
		}
	}
	
	async fn process1packet(&self, source: &Player, reader: &mut BufReader<OwnedReadHalf>) -> io::Result<()> {
		match reader.read_id().await? {
			CreatureUpdate       ::ID => reader.read_packet::<CreatureUpdate       >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			CreatureAction       ::ID => reader.read_packet::<CreatureAction       >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			Hit                  ::ID => reader.read_packet::<Hit                  >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			StatusEffect         ::ID => reader.read_packet::<StatusEffect         >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			Projectile           ::ID => reader.read_packet::<Projectile           >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			ChatMessageFromClient::ID => reader.read_packet::<ChatMessageFromClient>().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			AreaRequest::<Zone>  ::ID => reader.read_packet::<AreaRequest<Zone>    >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			AreaRequest::<Region>::ID => reader.read_packet::<AreaRequest<Region>  >().await?.pipe(|packet| self.handle_packet(source, packet)).await,
			_unexpected_packet_id => return Err(InvalidData.into())
		}
		
		Ok(())
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
		.map(async |existing_player| {
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

fn configure_stream(stream: TcpStream) -> io::Result<(BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>)>{
	stream.set_nodelay(true)?;

	let (read_half, write_half) = stream.into_split();

	let reader = BufReader::new(read_half);
	let writer = BufWriter::new(write_half);

	Ok((reader, writer))
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

async fn read_character_data(reader: &mut impl ReadPacket) -> io::Result<(CreatureUpdate, Creature)> {
	if reader.read_id().await? != CreatureUpdate::ID {
		return Err(InvalidData.into());
	}

	let creature_update = reader.read_packet::<CreatureUpdate>().await?;

	let character = Creature::maybe_from(&creature_update)
		.ok_or_else::<io::Error, _>(||InvalidInput.into())?;

	Ok((creature_update, character))
}