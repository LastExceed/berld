use std::sync::Arc;

use tap::Tap;

use protocol::utils::constants::combat_classes::WATER_MAGE;
use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::Race::*;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::{Sound, sound};
use protocol::packet::world_update::sound::Kind::*;

use crate::addon::{balancing, kill_feed};
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: Hit) {
		let Some(target) = self.find_player_by_id(packet.target).await
			else { return; };//can happen when the target disconnected in this moment
		let target_character_guard = target.character.read().await;
		let source_character_guard = source.character.read().await;

		let is_heal = packet.damage.is_sign_negative();
		let sounds = impact_sounds(&packet, target_character_guard.race, is_heal);

		self.addons.balancing.ignite(self, &packet, &source_character_guard, Arc::clone(&target)).await;

		self.addons.balancing.adjust_hit(&mut packet, &source_character_guard, &target_character_guard);
		balancing::adjust_blocking(&mut packet, source, &source_character_guard, &target_character_guard).await;
		packet.flash = true;//todo: (re-)move
		drop(target_character_guard);

		kill_feed::set_last_attacker(&target, source_character_guard.name.clone()).await;

		let mut wu_for_target = WorldUpdate::from(packet);

		if is_heal {
			if source_character_guard.combat_class() != WATER_MAGE {
				//dont emit sound for heals from unholy spirits
			} else {
				self.broadcast(&WorldUpdate::from(sounds), Some(source)).await; //healing sounds are already audible to the source
			}
		} else {
			wu_for_target.sounds = sounds; //damage sounds are already audible to everyone but the target
		}

		drop(source_character_guard);
		target.send_ignoring(&wu_for_target).await;
	}
}

pub fn impact_sounds(hit: &Hit, target_race: Race, is_heal: bool) -> Vec<Sound> {
	match hit.kind {
		Block |
		Miss => vec![sound::Kind::Block],

		Absorb => vec![sound::Kind::Absorb],

		Dodge |
		Invisible => vec![],

		Normal if is_heal => vec![Watersplash],
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