use std::sync::Arc;
use tap::Tap;

use protocol::packet::{CreatureUpdate, Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::item::Kind::Weapon;
use protocol::packet::common::item::kind::Weapon::Shield;
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::{Sound, sound};
use protocol::packet::world_update::sound::Kind::*;

use crate::addon::balancing;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: Hit) {
		//todo: duplicate code
		let Some(target) = self
			.players
			.read()
			.await
			.iter()
			.find(|player| { player.id == packet.target })
			.map(Arc::clone)
			else { return; };//can happen when the target disconnected in this moment
		let target_character_guard = target.character.read().await;
		let source_character_guard = source.character.read().await;

		balancing::adjust_hit(&mut packet, &source_character_guard, &target_character_guard);
		packet.flash = true;//todo: (re-)move


		source.send_ignoring(&CreatureUpdate { // Avoid the depletion of the target blocking gauge
			id: target.id,
			blocking_gauge: Some(target_character_guard.blocking_gauge),
			..Default::default()
		}).await;

		let mut hits_vec = vec![];
		let mut hit_sounds = impact_sounds(&packet, target_character_guard.race);

		if packet.kind == Block {
			let block_packet = Hit { // Show Block message when attack is Blocked
				kind: Block,
				damage: 0.0,
				..packet
			};
			hits_vec.push(block_packet); // To target

			let left_weapon = &target_character_guard.equipment[Slot::LeftWeapon];
			let right_weapon = &target_character_guard.equipment[Slot::RightWeapon];
			if left_weapon.kind != Weapon(Shield) && right_weapon.kind != Weapon(Shield) { // No shield blocking
				packet.damage /= 4.0;
				packet.kind = Normal;
				hits_vec.push(packet); // Normal hit packet, but with damage divided by 4
			}
		} else {
			hits_vec.push(packet);
		}

		let mut next_health = target_character_guard.health;
		for hit in &hits_vec {
			next_health -= hit.damage;
		}

		if next_health <= 0.0 {
			hit_sounds.push(Sound::at(target_character_guard.position, Destroy2)); // TODO: this sound is only hearable for the target.
		}

		target.send_ignoring(&WorldUpdate {
			sounds: hit_sounds,
			hits: hits_vec,
			..Default::default()
		}).await; //todo: verify that only target needs this packet*/ ToufouMaster: After investigating, it seem like the hit damages are only visible for the target even with broadcast, also the sound "Hit" seems to be hearable by everyone, but not the Destroy2 which need a broadcast, again cubeworld is well done ;D
	}
}

pub fn impact_sounds(hit: &Hit, target_race: Race) -> Vec<Sound> {
	match hit.kind {
		Block |
		Miss => vec![sound::Kind::Block],

		Absorb => vec![sound::Kind::Absorb],

		Dodge |
		Invisible => vec![],

		Normal => Vec::with_capacity(2)
			.tap_mut(|vec| {
				vec.push(Punch1);
				if let Some(groan) = groan_of(target_race) {
					vec.push(groan);
				}
			}),
	}.into_iter()
		.map(|kind| Sound::at(hit.position, kind))
		.collect()
}

const fn groan_of(race: Race) -> Option<sound::Kind> {
	match race {
		ElfMale         => Some(MaleGroan),
		ElfFemale       => Some(FemaleGroan),
		HumanMale       => Some(MaleGroan2),
		HumanFemale     => Some(FemaleGroan2),
		GoblinMale      => Some(GoblinMaleGroan),
		GoblinFemale    => Some(GoblinFemaleGroan),
		LizardmanMale   => Some(LizardMaleGroan),
		LizardmanFemale => Some(LizardFemaleGroan),
		DwarfMale       => Some(DwarfMaleGroan),
		DwarfFemale     => Some(DwarfFemaleGroan),
		OrcMale         => Some(OrcMaleGroan),
		OrcFemale       => Some(OrcFemaleGroan),
		FrogmanMale     => Some(FrogmanMaleGroan),
		FrogmanFemale   => Some(FrogmanFemaleGroan),
		UndeadMale      => Some(UndeadMaleGroan),
		UndeadFemale    => Some(UndeadFemaleGroan),
		_ => None
	}
}