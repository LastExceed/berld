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

		if packet.text.starts_with('/') {
			handle_command(&self, &source, &packet);
			return Ok(());
		}

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

fn handle_command(server: &Server, source: &Player, packet: &ChatMessageFromClient) {
	let mut params = packet.text.strip_prefix("/").unwrap().split(" ");
	let Some(command) = params.next() else {
		//text was just / with nothing else
		return;
	};
	match command {
		"xp" => {
			let Some(amount) = params.next() else {
				source.notify("too few arguments".to_string());
				return;
			};
			let Ok(parsed_amount) = amount.parse::<i32>() else {
				source.notify("failed to parse amount".to_string());
				return;
			};
			let dummy = CreatureUpdate {
				id: CreatureId(9999),
				affiliation: Some(Affiliation::Enemy),
				..Default::default()
			};
			source.send_ignoring(&dummy);

			let kill = Kill {
				killer: source.creature.read().id,
				unknown: 0,
				victim: dummy.id,
				xp: parsed_amount
			};

			let world_update = WorldUpdate {
				kills: vec![kill],
				..Default::default()
			};
			source.send_ignoring(&world_update);
			source.notify("ok".to_string());
		}
		other => {dbg!(other);}
	}
}