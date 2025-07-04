use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::creature_update::{Animation, Equipment};
use protocol::packet::creature_update::equipment::Slot::*;
use protocol::packet::creature_update::Occupation::*;
use protocol::packet::creature_update::Specialization::*;
use protocol::utils::constants::{animations, CombatClass};
use protocol::utils::constants::animations::{abilities, m1, m2};
use protocol::utils::constants::combat_classes::*;

use crate::addon::anti_cheat::PresentIn as _;

pub fn animations_avilable_with(combat_class: CombatClass, equipment: &Equipment) -> Vec<Animation> {
	let abilities = class_specific_animations(combat_class);
	let (m1, m2) = weapon_specific_animations(combat_class, equipment);

	[
		&animations::GENERAL[..],
		abilities,
		m1,
		m2
	].concat()
}

fn class_specific_animations(combat_class: CombatClass) -> &'static [Animation] {
	#[expect(clippy::wildcard_in_or_patterns, reason="specialization enum is not exhaustive")] //todo
	match combat_class.occupation {
		Warrior => &abilities::WARRIOR[..],
		Ranger  => &abilities::RANGER[..],
		Mage    => match combat_class.specialization {
			Alternative => &abilities::WATER_MAGE[..],
			Default | _ => &abilities::FIRE_MAGE[..],
		}
		Rogue   => match combat_class.specialization {
			Default         => &abilities::ASSASSIN[..],
			Alternative | _ => &abilities::NINJA[..],//no, this is not a bug. the game is actually that inconsistent
		}
		_ => &[][..]
	}
}

fn weapon_specific_animations(combat_class: CombatClass, equipment: &Equipment) -> (&'static [Animation], &'static [Animation]) {
	let right = equipment[RightWeapon].kind;
	let left  = equipment[LeftWeapon].kind;

	let left_handed = left.present_in(&[Weapon(Bow), Weapon(Crossbow)]);

	let (mainhand, offhand) =
		if left_handed { (left, right) }
		else           { (right, left) };

	match mainhand {
		Weapon(Greatsword) |
		Weapon(Greataxe)   |
		Weapon(Greatmace)  |
		Weapon(Pitchfork) => (&m1::GREATWEAPON[..], &m2::GREATWEAPON[..]),
		Weapon(Dagger)    => (&m1::DAGGER[..]     , &m2::DAGGER[..]),
		Weapon(Fist)      => (&m1::UNARMED[..]    , &m2::FIST[..]),//use redirecting constant?
		Weapon(Longsword) => (&m1::LONGSWORD[..]  , &m2::LONGSWORD[..]),
		Weapon(Bow)       => (&m1::BOW[..]        , &m2::BOW[..]),
		Weapon(Crossbow)  => (&m1::CROSSBOW[..]   , &m2::CROSSBOW[..]),
		Weapon(Boomerang) => (&m1::BOOMERANG[..]  , &m2::BOOMERANG[..]),
		Weapon(Staff)     => match combat_class.specialization {
			Alternative => (&m1::STAFF_WATER[..]   , &m2::STAFF_WATER[..]),
			_           => (&m1::STAFF_FIRE[..]    , &m2::STAFF_FIRE[..])
		},
		Weapon(Wand)      => match combat_class.specialization {
			Alternative => (&m1::WAND_WATER[..]    , &m2::WAND_WATER[..]),
			_           => (&m1::WAND_FIRE[..]     , &m2::WAND_FIRE[..])
		},
		Weapon(Bracelet)  => match combat_class.specialization {
			Alternative => (&m1::BRACELET_WATER[..], &m2::BRACELET_WATER[..]),
			_           => (&m1::BRACELET_FIRE[..] , &m2::BRACELET_FIRE[..])
		},
		Void => {
			let (mainhand_m1, mainhand_m2) = match combat_class {
				FIRE_MAGE  => (&m1::BRACELET_FIRE[..] , &m2::UNARMED[..]),
				WATER_MAGE => (&m1::BRACELET_WATER[..], &m2::UNARMED[..]),
				_          => (&m1::UNARMED[..]       , &m2::UNARMED[..])
			};
			let m2 =
				match offhand {
					Weapon(Shield) => &m2::SHIELD[..],
					_              => mainhand_m2
				};

			(mainhand_m1, m2)
		}
//		SWORD | AXE | MACE |
//		SHIELD|
//		ARROW | QUIVER | PICKAXE | TORCH
		_ => match offhand {
			Weapon(Shield) => (&m1::SHIELD[..]   , &m2::SHIELD[..]),
			_              => (&m1::DUALWIELD[..], &m2::UNARMED[..])//use redirecting constant?
		}
	}
}