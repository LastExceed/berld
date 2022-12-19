use std::io;

use protocol::packet::{ChatMessageFromClient, ChatMessageFromServer};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<ChatMessageFromClient> for Server {
	fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) -> Result<(), io::Error> {
		self.broadcast(
			&ChatMessageFromServer {
				source: source.creature.read().id,
				text: packet.text
			},
			None
		);

		Ok(())
	}
}