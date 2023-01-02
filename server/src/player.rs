use std::mem::size_of;
use std::sync::Arc;

use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;

use crate::creature::Creature;

pub struct Player {
	pub creature: RwLock<Creature>,
	write_half: Arc<RwLock<OwnedWriteHalf>>,
}

impl Player {
	pub fn new(creature: Creature, write_half: Arc<RwLock<OwnedWriteHalf>>) -> Self {
		Self {
			write_half,
			creature: RwLock::new(creature)
		}
	}

	pub async fn send<Packet: FromServer + Sync>(&self, packet: &Packet) -> io::Result<()>
		where [(); size_of::<Packet>()]:
	{
		packet.write_to_with_id(&mut self.write_half.write().await as &mut OwnedWriteHalf).await
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

	pub async fn close_connection(&self) {
		let _ = self.write_half.write().await.shutdown().await; //todo: error handling
	}
}