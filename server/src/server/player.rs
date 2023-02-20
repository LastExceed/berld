use std::mem::size_of;
use std::sync::atomic::AtomicBool;

use tokio::io;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;

use crate::server::creature::Creature;

pub struct Player {
	pub id: CreatureId,
	pub creature: RwLock<Creature>,
	writer: RwLock<BufWriter<OwnedWriteHalf>>,
	pub should_disconnect: AtomicBool,
}

impl Player {
	pub fn new(id: CreatureId, creature: Creature, writer: BufWriter<OwnedWriteHalf>) -> Self {
		Self {
			id,
			creature: RwLock::new(creature),
			writer: RwLock::new(writer),
			should_disconnect: AtomicBool::new(false)
		}
	}

	pub async fn send<Packet: FromServer + Sync>(&self, packet: &Packet) -> io::Result<()>
		where [(); size_of::<Packet>()]:
	{
		let mut writer = self.writer.write().await;
		packet.write_to_with_id(&mut *writer).await?;
		writer.flush().await
	}

	///sends a packet to this player and ignores any io errors.
	///useful when errors are already handled by the reading thread
	pub async fn send_ignoring<Packet: FromServer + Sync>(&self, packet: &Packet)
		where [(); size_of::<Packet>()]:
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