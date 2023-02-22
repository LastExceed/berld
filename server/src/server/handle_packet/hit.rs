use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::Race::*;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::{Sound, sound};
use protocol::packet::world_update::sound::Kind::*;

use crate::addons::balancing;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: Hit) {
		let players_guard = self.players.read().await;
		let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {
			return; //can happen when the target disconnected in this moment
		};
		let target_creature_guard = target.creature.read().await;
		let source_creature_guard = source.creature.read().await; //todo: rename creature -> character

		balancing::adjust_hit(&mut packet, &source_creature_guard, &target_creature_guard);
		packet.flash = true;//todo: (re-)move

		target.send_ignoring(&WorldUpdate {
			sounds: impact_sounds(&packet, target_creature_guard.race),
			hits: vec![packet],
			..Default::default()
		}).await; //todo: verify that only target needs this packet
	}
}

pub fn impact_sounds(hit: &Hit, target_race: Race) -> Vec<Sound> {
	match hit.kind {
		Block |
		Miss => vec![sound::Kind::Block],

		Absorb => vec![sound::Kind::Absorb],

		Dodge |
		Invisible => vec![],

		Normal => {
			if let Some(groan) = groan_of(target_race) {
				vec![Punch1, groan]
			} else {
				vec![Punch1]
			}
		},
	}.into_iter()
		.map(|kind| Sound::at(hit.position, kind))
		.collect()
}

fn groan_of(race: Race) -> Option<sound::Kind> {
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