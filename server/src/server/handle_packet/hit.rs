use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::Race::*;
use protocol::packet::hit;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::sound;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::*;
use protocol::utils::sound_position_of;

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
		let source_creature_guard = source.creature.read().await; //todo: why can't i inline this?

		balancing::adjust_hit(&mut packet, &source_creature_guard, &target_creature_guard);
		packet.flash = true;

		let sound_effects =
			impact_sounds(packet.kind, target_creature_guard.race)
				.iter()
				.map(|sound| Sound {
					position: sound_position_of(packet.position),
					kind: *sound,
					volume: 1.0,
					pitch: 1.0
				})
				.collect();

		let world_update = WorldUpdate {
			hits: vec![packet],
			sounds: sound_effects,
			..Default::default()
		};
		target.send_ignoring(&world_update).await; //todo: only target needs to receive this packet, but finding player by id is expensive atm
	}
}

fn impact_sounds(hit_kind: hit::Kind, target_race: Race) -> Vec<sound::Kind> {
	match hit_kind {
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
	}
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