use std::mem::size_of;
use std::net::TcpStream;
use std::sync::Mutex;
use protocol::packet::chat_message::ChatMessageFromServer;
use protocol::packet::creature_update::{CreatureId, CreatureUpdate};
use protocol::packet::PacketFromServer;

pub struct Player {
	pub creature: CreatureUpdate,
	stream: Mutex<TcpStream>,
}

impl Player {
	pub fn new(creature: CreatureUpdate, stream: &mut TcpStream) -> Self {
		Self {
			stream: Mutex::new(stream.try_clone().unwrap()),
			creature
		}
	}

	pub fn send<T: PacketFromServer>(&self, packet: &T)// -> Result<(), io::Error>
		where [(); size_of::<T>()]:
	{
		let _ = packet.write_to_with_id(&mut self.stream.lock().unwrap() as &mut TcpStream);
	}

	pub fn notify(&self, text: String) {
		self.send(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		});
	}
}