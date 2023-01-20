use async_trait::async_trait;
use tokio::io;

use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::hit::HitType;
use protocol::packet::world_update::sound_effect::Sound;
use protocol::packet::world_update::SoundEffect;
use protocol::utils::sound_position_of;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: Hit) -> io::Result<()> {
		if packet.target == packet.attacker && packet.damage.is_sign_negative() {
			return Ok(()) //self-heal is already applied client-side (which is a bug) so we need to ignore it server-side
		}

		let players_guard = self.players.read().await;
		let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {
			return Ok(()) //can happen when the target disconnected in this moment
		};

		packet.flash = true;

		let sound_effects =
			impact_sounds(packet.type_, target.creature.read().await.race)
				.iter()
				.map(|sound| SoundEffect {
					position: sound_position_of(packet.position),
					sound: *sound,
					volume: 1.0,
					pitch: 1.0
				})
				.collect();

		let world_update = WorldUpdate {
			hits: vec![packet],
			sound_effects,
			..Default::default()
		};
		target.send_ignoring(&world_update).await; //todo: only target needs to receive this packet, but finding player by id is expensive atm

		Ok(())
	}
}

fn impact_sounds(hit_type: HitType, target_race: Race) -> Vec<Sound> {
	match hit_type {
		HitType::Block |
		HitType::Miss => vec![Sound::Block],

		HitType::Absorb => vec![Sound::Absorb],

		HitType::Dodge |
		HitType::Invisible => vec![],

		HitType::Normal => {
			if let Some(groan) = groan_of(target_race) {
				vec![Sound::Punch1, groan]
			} else {
				vec![Sound::Punch1]
			}
		},
	}
}

fn groan_of(race: Race) -> Option<Sound> {
	match race {
		Race::ElfMale         => Some(Sound::MaleGroan),
		Race::ElfFemale       => Some(Sound::FemaleGroan),
		Race::HumanMale       => Some(Sound::MaleGroan2),
		Race::HumanFemale     => Some(Sound::FemaleGroan2),
		Race::GoblinMale      => Some(Sound::GoblinMaleGroan),
		Race::GoblinFemale    => Some(Sound::GoblinFemaleGroan),
		Race::LizardmanMale   => Some(Sound::LizardMaleGroan),
		Race::LizardmanFemale => Some(Sound::LizardFemaleGroan),
		Race::DwarfMale       => Some(Sound::DwarfMaleGroan),
		Race::DwarfFemale     => Some(Sound::DwarfFemaleGroan),
		Race::OrcMale         => Some(Sound::OrcMaleGroan),
		Race::OrcFemale       => Some(Sound::OrcFemaleGroan),
		Race::FrogmanMale     => Some(Sound::FrogmanMaleGroan),
		Race::FrogmanFemale   => Some(Sound::FrogmanFemaleGroan),
		Race::UndeadMale      => Some(Sound::UndeadMaleGroan),
		Race::UndeadFemale    => Some(Sound::UndeadFemaleGroan),
		_ => None
	}
}