use std::{io, thread};
use std::time::Duration;

use protocol::nalgebra::Vector3;
use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::hit::HitType;
use protocol::packet::status_effect::StatusEffectType;
use protocol::packet::world_update::sound_effect::Sound;
use protocol::packet::world_update::SoundEffect;
use protocol::utils::sound_position_of;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<StatusEffect> for Server {
	fn handle_packet(&self, source: &Player, packet: StatusEffect) -> Result<(), io::Error> {
		self.broadcast(
			&WorldUpdate {
				status_effects: vec![packet.clone()],
				..Default::default()
			},
			Some(source)
		);

		match packet.type_ {
			StatusEffectType::Poison => {
				let players_guard = self.players.read(); //todo: do i really have to do this?

				let Some(target) = players_guard.iter().find(|player| player.creature.read().id == packet.target) else {//todo: very expensive because RwLocks
					return Ok(()); //todo: invalid input?
				};
				let target_owned = target.to_owned();
				thread::spawn(move || {
					apply_poison(&target_owned, &packet);
				});
			}

			_ => ()
		}

		Ok(())
	}
}

fn apply_poison(target: &Player, status_effect: &StatusEffect) {
	let tick_count = status_effect.duration / 500;

	for i in 0..=tick_count {
		if i != 0 {
			thread::sleep(Duration::from_millis(500));
		}

		let target_position = target.creature.read().position;

		let hit = Hit {
			attacker: CreatureId(0),//todo: check if this matters
			target: status_effect.target,
			damage: status_effect.modifier,
			critical: false,
			stuntime: 0,
			position: target_position,
			direction: Vector3::zeros(),
			is_yellow: false,
			type_: HitType::Default,
			flash: true,
		};

		let sound_effect = SoundEffect {
			position: sound_position_of(target_position),
			sound: Sound::Absorb,
			pitch: 1.0,
			volume: 1.0
		};

		let world_update = WorldUpdate {
			hits: vec![hit],
			sound_effects: vec![sound_effect],
			..Default::default()
		};

		if let Err(_) = target.send(&world_update) {
			//disconnects are handled in the reading thread
			break;
		};
	}
}