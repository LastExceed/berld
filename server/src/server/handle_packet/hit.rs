use tap::Tap as _;

use protocol::{packet::creature_update::equipment::Slot, utils::constants::CombatClass};
use protocol::packet::common::item;
use protocol::packet::creature_update::{Occupation, Specialization};
use protocol::utils::constants::combat_classes::{WATER_MAGE, FIRE_MAGE};
use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::Race::*;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound;
use protocol::packet::world_update::sound::Kind::*;

use crate::server::creature::Creature;
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

		self.addons.balancing.on_hit(self, &mut packet, &source_character_guard, &target).await;
		balancing::adjust_blocking(&mut packet, &source_character_guard);
		packet.flash = true;//todo: (re-)move

		kill_feed::set_last_attacker(&target, source_character_guard.name.clone()).await;

		//bystanders (but not victims) generate groans and weapon sounds locally when they receive a hit packet. but making use of that would be a mess
		let sounds = sounds(&packet, &source_character_guard, target_character_guard.race);
		drop(source_character_guard);
		drop(target_character_guard);
		self.broadcast(&WorldUpdate::from(sounds), Some(source)).await;

		target.send_ignoring(&WorldUpdate::from(packet)).await;
	}
}

pub fn sounds(hit: &Hit, source_creature: &Creature, target_race: Race) -> Vec<Sound> {
	let heals = hit.damage.is_sign_negative();

	match hit.kind {
		Block |
		Miss => vec![sound::Kind::Block],

		Absorb => vec![sound::Kind::Absorb],

		Dodge |
		Invisible => vec![],

		Normal if heals && source_creature.combat_class() != WATER_MAGE => vec![], //unholy spirits should be silent
		Normal if heals => vec![Watersplash],
		Normal => Vec::with_capacity(2)
			.tap_mut(|vec| {
				let weapon_kind = if let item::Kind::Weapon(w) = source_creature.equipment[Slot::RightWeapon].kind {
					w
				} else {
					item::kind::Weapon::Quiver //todo: lazy hack
				};

				vec.push(impact_of(source_creature.combat_class(), weapon_kind, hit.critical));
				if hit.critical && let Some(groan) = groan_of(target_race) {
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

fn impact_of(combat_class: CombatClass, weapon_kind: item::kind::Weapon, critical: bool) -> sound::Kind {
	use item::kind::Weapon::*;
	use sound::Kind::*;

	//todo: wrong sound when
	// - mage unarmed M2
	// - all illegal weapons/skills etc

	let mage = combat_class.occupation == Occupation::Mage;

	//todo: move class-weapon association to constants?
	match combat_class {
		WATER_MAGE => WatersplashHit,
		FIRE_MAGE => FireHit,
		_ => match weapon_kind {
			Greatsword | Sword | Greataxe | Axe => if critical { Blade2           } else { Blade1     },
			Greatmace  | Mace  | Shield         => if critical { Hit2             } else { Hit1       },
			Dagger     | Fist                   => if critical { Punch2           } else { Punch1     },
			Longsword                           => if critical { LongBlade2       } else { LongBlade1 },
			Crossbow   | Bow   | Boomerang      => if critical { HitArrowCritical } else { HitArrow   },

			Bracelet   | Staff | Wand           => if combat_class.specialization == Specialization::Alternative { WatersplashHit } else { FireHit },
			_ if mage                           => impact_of(combat_class, Bracelet, critical),

			_                                   => if critical { Hit2             } else { Hit1       },
			//Arrow, quiver, pitchfork, pickaxe, torch
		}
	}


}