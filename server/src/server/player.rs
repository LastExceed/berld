mod addon_data;

use std::io::ErrorKind::BrokenPipe;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;

use tokio::io;
use tokio::sync::{oneshot, RwLock};

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;
use protocol::WriteCwData;

use crate::server::creature::Creature;
use crate::server::outbound::{serialize, Outbound};
use crate::server::player::addon_data::AddonData;

#[derive(Debug)]
pub struct Player {
	pub address: SocketAddr,
	pub id: CreatureId,
	pub character: RwLock<Creature>,
	pub outbound: Outbound,
	pub admin: AtomicBool, //todo: move to AddonData
	pub ac_immune: AtomicBool,
	pub kick_sender: RwLock<Option<oneshot::Sender<()>>>,
	pub addon_data: RwLock<AddonData>
}

impl Player {
	pub fn new(address: SocketAddr, id: CreatureId, creature: Creature, outbound: Outbound) -> (Self, oneshot::Receiver<()>) {
		let (kick_sender, kick_receiver) = oneshot::channel();

		let instance = Self {
			address,
			id,
			character: RwLock::new(creature),
			outbound,
			admin: AtomicBool::default(),
			ac_immune: AtomicBool::default(),
			kick_sender: RwLock::new(Some(kick_sender)),
			addon_data: RwLock::default()
		};

		(instance, kick_receiver)
	}

	pub async fn send<Packet: FromServer>(&self, packet: &Packet) -> io::Result<()>
		where Vec<u8>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		if self.outbound.enqueue(serialize(packet).await) {
			Ok(())
		} else {
			Err(BrokenPipe.into())
		}
	}

	///sends a packet to this player and ignores any io errors.
	///useful when errors are already handled by the reading thread
	pub async fn send_ignoring<Packet: FromServer>(&self, packet: &Packet)
		where Vec<u8>: WriteCwData<Packet>//todo: specialization could obsolete this
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