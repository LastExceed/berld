use async_trait::async_trait;
use colour::{cyan, white_ln};
use tokio::io;

use protocol::packet::{ChatMessageFromClient, ChatMessageFromServer, ConnectionRejection, CreatureUpdate, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::Kill;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<ChatMessageFromClient> for Server {
	async fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) -> io::Result<()> {
		cyan!("{}: ", source.creature.read().await.name);
		white_ln!("{}", packet.text);

		if packet.text.starts_with('/') {
			handle_command(&self, &source, &packet).await;
			return Ok(());
		}

		self.broadcast(
			&ChatMessageFromServer {
				source: source.id,
				text: packet.text
			},
			None
		).await;

		Ok(())
	}
}

async fn handle_command(server: &Server, source: &Player, packet: &ChatMessageFromClient) {
	let mut params = packet.text.strip_prefix("/").unwrap().split(" ");
	let Some(command) = params.next() else {
		//text was just / with nothing else
		return;
	};
	match command {
		"a" => {
			source.send_ignoring(&ConnectionRejection{}).await;
		}
		"xp" => {
			let Some(amount) = params.next() else {
				source.notify("too few arguments").await;
				return;
			};
			let Ok(parsed_amount) = amount.parse::<i32>() else {
				source.notify("failed to parse amount").await;
				return;
			};
			let dummy = CreatureUpdate {
				id: CreatureId(9999),
				affiliation: Some(Affiliation::Enemy),
				..Default::default()
			};
			source.send_ignoring(&dummy).await;

			let kill = Kill {
				killer: source.id,
				unknown: 0,
				victim: dummy.id,
				xp: parsed_amount
			};

			source.send_ignoring(&WorldUpdate::from(kill)).await;
			source.notify("ok").await;
		}
		other => {dbg!(other);}
	}
}