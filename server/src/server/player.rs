pub mod addon_data;

use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;

use tokio::io::{self, AsyncWriteExt as _, SimplexStream, WriteHalf};
use tokio::sync::{oneshot, RwLock};

use protocol::nalgebra::Point2;
use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;
use protocol::utils::io_extensions::WritePacket;
use protocol::utils::zone_of;
use protocol::WriteCwData;

use crate::server::creature::Creature;
use crate::server::player::addon_data::{AddonData, ZoneState, AWAITED_GRACE};

// current and adjacent zones revealed to the player client-side (limited by max render distance)
pub const ZONE_DATA_RADIUS: i32 = 1;
// any terrain beyond this radius is guaranteed to be unloaded client-side
pub const ZONE_RETENTION_RADIUS: i32 = 3;

#[derive(Debug)]
pub struct Player {
	pub address: SocketAddr,
	pub id: CreatureId,
	pub character: RwLock<Creature>,
	pub writer: RwLock<WriteHalf<SimplexStream>>,
	pub admin: AtomicBool, //todo: move to AddonData
	pub ac_immune: AtomicBool,
	pub kick_sender: RwLock<Option<oneshot::Sender<()>>>,
	pub addon_data: RwLock<AddonData>
}

impl Player {
	pub fn new(address: SocketAddr, id: CreatureId, creature: Creature, writer: WriteHalf<SimplexStream>) -> (Self, oneshot::Receiver<()>) {
		let (kick_sender, kick_receiver) = oneshot::channel();

		let instance = Self {
			address,
			id,
			character: RwLock::new(creature),
			writer: RwLock::new(writer),
			admin: AtomicBool::default(),
			ac_immune: AtomicBool::default(),
			kick_sender: RwLock::new(Some(kick_sender)),
			addon_data: RwLock::default()
		};

		(instance, kick_receiver)
	}

	pub async fn send<Packet: FromServer>(&self, packet: &Packet) -> io::Result<()>
		where WriteHalf<SimplexStream>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		let mut writer = self.writer.write().await;
		#[expect(trivial_casts, reason = "todo: why is this cast necessary?")]
		(&mut writer as &mut WriteHalf<SimplexStream>).write_packet(packet).await
	}

	///sends a packet to this player and ignores any io errors.
	///useful when errors are already handled by the reading thread
	pub async fn send_ignoring<Packet: FromServer>(&self, packet: &Packet)
		where WriteHalf<SimplexStream>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		#[expect(let_underscore_drop, clippy::let_underscore_must_use, reason="deliberate")]
		let _ = self.send(packet).await;
	}

	pub async fn send_raw(&self, data: &[u8]) -> io::Result<()> {
		let mut writer = self.writer.write().await;

		writer.write_all(data).await?;
		writer.flush().await
	}

	pub async fn is_near(&self, zone: Point2<i32>) -> bool {
		let distance = zone_of(self.character.read().await.position) - zone;

		distance.x.abs().max(distance.y.abs()) <= ZONE_DATA_RADIUS
	}

	// client readiness for zone update; pending delivery = server knows client is not ready
	// live updates must not jump the queue (another player dropping loot while client
	// is waiting for a zone update from the server would result in block updates getting lost)
	pub async fn is_ready_for(&self, zone: Point2<i32>) -> bool {
		if !self.is_near(zone).await {
			return false;
		}

		let state = self.addon_data.read().await.zone_states.get(&zone).copied();

		match state {
			Some(ZoneState::Pending) => false,
			Some(ZoneState::Awaited(since)) => since.elapsed() >= AWAITED_GRACE,
			_ => true
		}
	}

	pub async fn notify(&self, message: impl Into<String>) {
		self.send_ignoring(&ChatMessageFromServer {
			source: CreatureId(0),
			text: message.into()
		}).await;
	}
}