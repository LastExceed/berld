use std::ops::{Div, Mul};
use std::fs;
use std::error::Error;
use std::mem::transmute;
use std::time::{UNIX_EPOCH, SystemTime};
use std::sync::Arc;
use std::time::Duration;

use colour::{white_ln, red_ln};
use tokio::time::sleep;

use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use protocol::packet::Hit;
use protocol::packet::world_update::{Block, Particle, Sound};
use protocol::packet::world_update::sound;
use protocol::packet::hit::Kind::Normal;
use protocol::nalgebra::{Point3, Vector3};
use protocol::packet::{ChatMessageFromServer, CreatureUpdate, ServerTick, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::creature_update::Affiliation::Pet;
use protocol::packet::creature_update::Animation::Riding;
use protocol::packet::world_update::Kill;

use crate::server::player::Player;
use crate::addon::kill_feed;
use crate::server::Server;

use super::send_existing_creatures;
use super::creature::Creature;

impl Server {
	pub async fn announce(&self, text: impl Into<String>) {
		let text = text.into();//todo: is there a way to prevent this boilerplate?

		white_ln!("{}", text);
		self.addons.discord_integration.post(&format!("*{text}*"), false).await;
		self.broadcast(&ChatMessageFromServer {
			source: CreatureId(0),
			text
		}, None).await;
	}

	pub async fn kick(&self, player: &Player, reason: impl Into<String>) {
		self.announce(format!("kicked {} because {}", player.character.read().await.name, reason.into())).await;
		//wait a bit to make sure the message arrives at the player about to be kicked
		sleep(Duration::from_millis(100)).await;

		player
			.kick_sender
			.write()
			.await
			.take()
			.map(|sender| sender.send(()));
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
		send_existing_creatures(self, player).await;
	}

	pub async fn find_player_by_id(&self, id: CreatureId) -> Option<Arc<Player>> {
		self
			.players
			.read()
			.await
			.iter()
			.find(|player| { player.id == id })
			.map(Arc::clone)
	}

	pub async fn apply_dot(&self, source_character: &Creature, target: Arc<Player>, ticks: i32, delay: u64, damage: f32, sound_kind: sound::Kind, particles: Option<Vec<Particle>>) {
		let target_character_guard = target.character.read().await;

		let mut hit = Hit {
			attacker: CreatureId(0),//todo: check if this matters
			target: target.id,
			damage,
			critical: false,
			stuntime: 0,
			position: target_character_guard.position,
			direction: Vector3::zeros(),
			is_yellow: false,
			kind: Normal,
			flash: true,
		};
		self.addons.balancing.adjust_hit(&mut hit, &source_character, &target_character_guard);
		drop(target_character_guard);

		let mut world_update = WorldUpdate {
			sounds: vec![Sound::at(hit.position, sound_kind)],
			particles: particles.unwrap_or(vec![]),
			hits: vec![hit],
			..Default::default()
		};

		let attacker_name = source_character.name.clone();
		tokio::spawn(async move {
			let mut nth = 0;
			loop {
				nth += 1;

				let character = target.character.read().await;
				world_update.hits[0].position = character.position;
				for particle in world_update.particles.iter_mut() {
					particle.position = character.position - Vector3::new(0,0,0x20000);
					particle.velocity = character.velocity / 10.0 + Vector3::new(0.0,0.0, 2.0);
				}

				if character.health == 0.0 {
					break;
				}
				drop(character);

				kill_feed::set_last_attacker(&target, attacker_name.clone()).await;
				if target.send(&world_update).await.is_err() {
					break; //disconnects are handled in the reading task
				};

				if nth == ticks {
					break;
				}

				sleep(Duration::from_millis(delay)).await;
			}
		});
	}
}

pub fn log_error(description: &str, err: impl Error) {
	red_ln!("error at {description}: {err}");

	let filename = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("time went backwards")
		.as_millis()
		.to_string();

	let _result = fs::create_dir_all(format!("logs/{description}")); //if it already exists thats fine too

	fs::write(format!("logs/{description}/{filename}.log"), err.to_string())
		.expect("failed writing log file");

}

///terrible hack due to my inexperience. unsafe as fuck
pub fn extend_lifetime<T>(it: &T) -> &'static T {
	#[expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]
	unsafe { transmute(it) } //TODO: figure out scoped tasks
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

pub async fn checkerboard(server: &Server, character: &Creature) {
	let start = Point3::new(
		character.position.x
			.div(SIZE_ZONE)
			.mul(SIZE_ZONE)
			.div(SIZE_BLOCK) as i32 - 0x100,
		character.position.y
			.div(SIZE_ZONE)
			.mul(SIZE_ZONE)
			.div(SIZE_BLOCK) as i32 - 0x100,
		character.position.z.div(SIZE_BLOCK) as i32,
	);

	let mut blocks = Vec::with_capacity(100);

	for dx in 0..800 {
		for dy in 0..800 {
			let block_alt = (dx + dy) % 2 == 1;
			let mapblock_alt = ((dx / 8) + (dy / 8)) % 2 == 1;
			let chunk_alt = ((dx / 32) + (dy / 32)) % 2 == 1;
			let zone_alt = ((dx / 256) + (dy / 256)) % 2 == 1;

			#[expect(clippy::collapsible_else_if, reason = "TODO")]
			let color =
				if zone_alt {
					if chunk_alt {
						if mapblock_alt {
							if block_alt { [0, 192, 192] } else { [0, 255, 255] }
						} else {
							if block_alt { [0, 192, 0] } else { [0, 255, 0] }
						}
					} else {
						if mapblock_alt {
							if block_alt { [0, 96, 192] } else { [0, 128, 255] }
						} else {
							if block_alt { [0, 0, 128] } else { [0, 0, 255] }
						}
					}
				} else {
					if chunk_alt {
						if mapblock_alt {
							if block_alt { [192, 0, 96] } else { [255, 0, 128] }
						} else {
							if block_alt { [96, 0, 96] } else { [255, 0, 255] }
						}
					} else {
						if mapblock_alt {
							if block_alt { [192, 96, 0] } else { [255, 128, 0] }
						} else {
							if block_alt { [128, 0, 0] } else { [255, 0, 0] }
						}
					}
				}.into();

			let block = Block {
				position: start + Vector3::new(dx, dy, 0),
				kind: protocol::packet::world_update::block::Kind::Solid,
				color,
				padding: 0
			};

			blocks.push(block);
		}
	}

	server.broadcast(&WorldUpdate::from(blocks), None).await;
}