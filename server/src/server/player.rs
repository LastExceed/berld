use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;

use tokio::io;
use tokio::io::BufWriter;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;
use protocol::utils::io_extensions::WritePacket;
use protocol::WriteCwData;

use crate::server::creature::Creature;

pub struct Player {
	pub address: SocketAddr,
	pub id: CreatureId,
	pub character: RwLock<Creature>,
	writer: RwLock<BufWriter<OwnedWriteHalf>>,
	pub admin: AtomicBool,
	pub should_disconnect: AtomicBool,
}

impl Player {
	pub fn new(address: SocketAddr, id: CreatureId, creature: Creature, writer: BufWriter<OwnedWriteHalf>) -> Self {
		Self {
			address,
			id,
			character: RwLock::new(creature),
			writer: RwLock::new(writer),
			admin: AtomicBool::new(false),
			should_disconnect: AtomicBool::new(false)
		}
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
		let _ = self.send(packet).await;
	}

	pub async fn notify(&self, message: impl Into<String>) {
		self.send_ignoring(&ChatMessageFromServer {
			source: CreatureId(0),
			text: message.into()
		}).await;
	}
}