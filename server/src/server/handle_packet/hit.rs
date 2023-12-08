use std::sync::Arc;
use tap::Tap;

use protocol::packet::{CreatureUpdate, Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::Race::*;
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

		let hits_vec: Vec<Hit> = balancing::apply_block(&mut packet, &source_character_guard, &target_character_guard);
		let hit_sounds = impact_sounds(&packet, target_character_guard.race);

		let mut next_health = target_character_guard.health;
		for hit in &hits_vec {
			next_health -= hit.damage;
		}

		target.send_ignoring(&WorldUpdate {
			sounds: hit_sounds, 	// the sound and hit effect can be heard/seen by every players
			hits: hits_vec,			// the damages are only shown to the target
			..Default::default()	// and the attacker damage is precalculated by the client
		}).await;

		if next_health <= 0.0 {
			self.broadcast(&WorldUpdate { // send death sound to all players
				sounds: vec![Sound::at(target_character_guard.position, Destroy2)],
				..Default::default()
			}, None).await;
		}
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