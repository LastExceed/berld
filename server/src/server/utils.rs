use std::sync::atomic::Ordering;
use std::time::Duration;

use colour::white_ln;
use tokio::time::sleep;

use protocol::nalgebra::Point3;
use protocol::packet::{ChatMessageFromServer, CreatureUpdate, ServerTick, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::creature_update::Affiliation::Pet;
use protocol::packet::creature_update::Animation::Riding;
use protocol::packet::world_update::Kill;

use crate::server::player::Player;
use crate::server::Server;

impl Server {
	pub async fn announce(&self, text: impl Into<String>) {
		let text = text.into();//todo: is there a way to prevent this boilerplate?

		white_ln!("{}", text);
		self.addons.discord_integration.post(&format!("*{text}*")).await;
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

	pub async fn teleport(&self, player: &Player, destination: Point3<i64>) {
		let server_creature = CreatureUpdate {
			id: CreatureId(0),
			position: Some(destination),
			affiliation: Some(Pet),
			animation: Some(Riding),
			..Default::default()
		};
		player.send_ignoring(&server_creature).await;
		sleep(Duration::from_millis(100)).await;
		player.send_ignoring(&ServerTick).await;
		player.send_ignoring(&ServerTick).await;
		for other_player in self.players.read().await.iter() {//todo: "on_reload" ?
			player.send_ignoring(&other_player.character.read().await.to_update(other_player.id)).await;
		}
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