mod addon_data;

use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;

use tokio::io;
use tokio::io::BufWriter;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{oneshot, RwLock};

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;
use protocol::utils::io_extensions::WritePacket as _;
use protocol::WriteCwData;

use crate::server::creature::Creature;
use crate::server::player::addon_data::AddonData;

#[derive(Debug)]
pub struct Player {
	pub address: SocketAddr,
	pub id: CreatureId,
	pub character: RwLock<Creature>,
	writer: RwLock<BufWriter<OwnedWriteHalf>>,
	pub admin: AtomicBool, //todo: move to AddonData
	pub ac_immune: AtomicBool,
	pub kick_sender: RwLock<Option<oneshot::Sender<()>>>,
	pub addon_data: RwLock<AddonData>
}

impl Player {
	pub fn new(address: SocketAddr, id: CreatureId, creature: Creature, writer: BufWriter<OwnedWriteHalf>) -> (Self, oneshot::Receiver<()>) {
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
		where BufWriter<OwnedWriteHalf>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		let mut writer = self.writer.write().await;
		(&mut writer as &mut BufWriter<OwnedWriteHalf>).write_packet(packet).await //todo: why is this cast necessary?
	}

	///sends a packet to this player and ignores any io errors.
	///useful when errors are already handled by the reading thread
	pub async fn send_ignoring<Packet: FromServer>(&self, packet: &Packet)
		where BufWriter<OwnedWriteHalf>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		#[expect(let_underscore_drop, clippy::let_underscore_must_use, reason="deliberate")]
		let _ = self.send(packet).await;
	}

	pub async fn notify(&self, message: impl Into<String>) {
		self.send_ignoring(&ChatMessageFromServer {
			source: CreatureId(0),
			text: message.into()
		}).await;
	}
}