use std::sync::atomic::Ordering;
use std::time::Duration;

use colour::white_ln;
use tokio::time::sleep;

use protocol::packet::{ChatMessageFromServer, CreatureUpdate, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::Kill;

use crate::server::player::Player;
use crate::server::Server;

impl Server {
	pub async fn announce(&self, text: impl Into<String>) {
		let text = text.into();//todo: is there a way to prevent this boilerplate?

		white_ln!("{}", text);
		self.addons.discord_integration.post(format!("*{text}*")).await;
		self.broadcast(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		}, None).await;
	}

	pub async fn kick(&self, player: &Player, reason: impl Into<String>) {
		self.announce(format!("kicked {} because {}", player.character.read().await.name, reason.into())).await;
		//wait a bit to make sure the message arrives at the player about to be kicked
		sleep(Duration::from_millis(100)).await;

		player.should_disconnect.store(true, Ordering::Relaxed);
		//remove_player will be called by the reading task
	}
}

pub async fn give_xp(player: &Player, experience: i32) {
	let dummy = CreatureUpdate {
		id: CreatureId(9999),
		affiliation: Some(Affiliation::Enemy),
		..Default::default()
	};
	player.send_ignoring(&dummy).await;

	let kill = Kill {
		killer: player.id,
		unknown: 0,
		victim: dummy.id,
		experience
	};

	player.send_ignoring(&WorldUpdate::from(kill)).await;
}