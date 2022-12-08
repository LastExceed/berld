use std::mem::size_of;
use std::net::TcpStream;

use parking_lot::{Mutex, RwLock};

use protocol::packet::{ChatMessageFromServer, PacketFromServer};
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

	pub fn send<Packet: PacketFromServer>(&self, packet: &Packet)// -> Result<(), io::Error>
		where [(); size_of::<Packet>()]:
	{
		let _ = packet.write_to_with_id(&mut self.stream.lock() as &mut TcpStream);
	}

	pub fn notify(&self, text: String) {
		self.send(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		});
	}
}