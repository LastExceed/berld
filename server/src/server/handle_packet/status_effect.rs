use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use protocol::nalgebra::Vector3;
use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::packet::hit::Kind::*;
use protocol::packet::status_effect::Kind::*;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::*;

use crate::addons::balancing;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<StatusEffect> for Server {
	async fn handle_packet(&self, source: &Player, packet: StatusEffect) {
		match packet.kind {
			Poison => {
				let players_guard = self.players.read().await; //todo: do i really have to do this?

				let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {
					return; //can happen when the target disconnected in this moment
				};
				apply_poison(source, target.to_owned(), &packet).await;
			}
			WarFrenzy => {
				balancing::buff_warfrenzy(&packet, self).await;
			}
			_ => ()
		}


		self.broadcast(&WorldUpdate::from(packet), Some(source)).await;
	}
}

async fn apply_poison(source: &Player, target: Arc<Player>, status_effect: &StatusEffect) {
	let source_character_guard = source.character.read().await;
	let target_character_guard = target.character.read().await;

	let mut hit = Hit {
		attacker: source.id,//todo: check if this matters
		target: status_effect.target,
		damage: status_effect.modifier,
		critical: false,
		stuntime: 0,
		position: target_character_guard.position,
		direction: Vector3::zeros(),
		is_yellow: false,
		kind: Normal,
		flash: true,
	};

	balancing::adjust_hit(&mut hit, &source_character_guard, &target_character_guard);
	drop(source_character_guard);
	drop(target_character_guard);

	let world_update = WorldUpdate {
		sounds: vec![Sound::at(hit.position, SlimeGroan)],
		hits: vec![hit],
		..Default::default()
	};

	let tick_count = status_effect.duration / 500;

	tokio::spawn(async move {
		for i in 0..=tick_count {
			if i != 0 {
				sleep(Duration::from_millis(500)).await;
			}

			if let Err(_) = target.send(&world_update).await {
				//disconnects are handled in the reading thread
				break;
			};
		}
	});
}