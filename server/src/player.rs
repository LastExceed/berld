use std::io;
use std::mem::size_of;
use std::net::{Shutdown, TcpStream};

use parking_lot::{Mutex, RwLock};

use protocol::packet::{ChatMessageFromServer, FromServer};
use protocol::packet::common::CreatureId;

use crate::creature::Creature;

pub struct Player {
	pub creature: RwLock<Creature>,
	stream: Mutex<TcpStream>,
}

impl Player {
	pub fn new(creature: Creature, stream: &mut TcpStream) -> Self {
		Self {
			stream: Mutex::new(stream.try_clone().unwrap()),
			creature: RwLock::new(creature)
		}
	}

	pub fn send<Packet: FromServer>(&self, packet: &Packet) -> Result<(), io::Error>
		where [(); size_of::<Packet>()]:
	{
		packet.write_to_with_id(&mut self.stream.lock() as &mut TcpStream)
	}

	///sends a packet to this player and ignores any io errors.
	///useful when errors are already handled by the reading thread
	pub fn send_ignoring<Packet: FromServer>(&self, packet: &Packet)
		where [(); size_of::<Packet>()]:
	{
		let _ = self.send(packet);
	}

	pub fn notify(&self, text: String) {
		self.send_ignoring(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		});
	}
}