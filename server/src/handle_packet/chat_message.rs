use std::io;

use colour::{cyan, white_ln};

use protocol::packet::{ChatMessageFromClient, ChatMessageFromServer, CreatureUpdate, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::Kill;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<ChatMessageFromClient> for Server {
	fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) -> Result<(), io::Error> {
		cyan!("{}: ", source.creature.read().name);
		white_ln!("{}", packet.text);

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