use protocol::packet::creature_update::{Animation, Equipment};
use protocol::packet::creature_update::CombatClassMajor::*;
use protocol::packet::creature_update::CombatClassMinor::*;
use protocol::utils::constants::{animations, CombatClass};
use protocol::utils::constants::animations::{abilities, m1, m2};
use protocol::utils::constants::combat_classes::*;
use protocol::utils::constants::item_types::*;

use crate::anti_cheat::PresentIn;

pub(super) fn animations_avilable_with(combat_class: CombatClass, equipment: &Equipment) -> Vec<Animation> {
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
	match combat_class.major {
		Warrior => &abilities::WARRIOR[..],
		Ranger  => &abilities::RANGER[..],
		Mage    => match combat_class.minor {
			Alternative => &abilities::WATER_MAGE[..],
			Default | _ => &abilities::FIRE_MAGE[..],
		}
		Rogue   => match combat_class.minor {
			Default         => &abilities::ASSASSIN[..],
			Alternative | _ => &abilities::NINJA[..],//no, this is not a bug. the game is actually that inconsistent
		}
		_ => &[][..]
	}
}

fn weapon_specific_animations(combat_class: CombatClass, equipment: &Equipment) -> (&'static [Animation], &'static [Animation]) {
	let right = equipment.right_weapon.item_type();
	let left  = equipment.left_weapon.item_type();

	let left_handed = left.present_in(&[BOW, CROSSBOW]);

	let (mainhand, offhand) =
		if left_handed { (left, right) }
		else           { (right, left) };

	match mainhand {
		GREATSWORD |
		GREATAXE   |
		GREATMACE  |
		PITCHFORK => (&m1::GREATWEAPON[..], &m2::GREATWEAPON[..]),
		DAGGER    => (&m1::DAGGER[..]     , &m2::DAGGER[..]),
		FIST      => (&m1::UNARMED[..]    , &m2::UNARMED[..]),//use redirecting constants?
		LONGSWORD => (&m1::LONGSWORD[..]  , &m2::LONGSWORD[..]),
		BOW       => (&m1::BOW[..]        , &m2::BOW[..]),
		CROSSBOW  => (&m1::CROSSBOW[..]   , &m2::CROSSBOW[..]),
		BOOMERANG => (&m1::BOOMERANG[..]  , &m2::BOOMERANG[..]),
		STAFF     => match combat_class.minor {
			Alternative => (&m1::STAFF_WATER[..]   , &m2::STAFF_WATER[..]),
			_           => (&m1::STAFF_FIRE[..]    , &m2::STAFF_FIRE[..])
		},
		WAND      => match combat_class.minor {
			Alternative => (&m1::WAND_WATER[..]    , &m2::WAND_WATER[..]),
			_           => (&m1::WAND_FIRE[..]     , &m2::WAND_FIRE[..])
		},
		BRACELET  => match combat_class.minor {
			Alternative => (&m1::BRACELET_WATER[..], &m2::BRACELET_WATER[..]),
			_           => (&m1::BRACELET_FIRE[..] , &m2::BRACELET_FIRE[..])
		},
		VOID => {
			let (mainhand_m1, mainhand_m2) = match combat_class {
				FIRE_MAGE  => (&m1::BRACELET_FIRE[..] , &m2::BRACELET_FIRE[..]),
				WATER_MAGE => (&m1::BRACELET_WATER[..], &m2::BRACELET_WATER[..]),
				_          => (&m1::UNARMED[..]       , &m2::UNARMED[..])
			};
			let m2 =
				match offhand {
					SHIELD => &m2::SHIELD[..],
					_      => mainhand_m2
				};

			(mainhand_m1, m2)
		}
//		SWORD | AXE | MACE |
//		SHIELD|
//		ARROW | QUIVER | PICKAXE | TORCH
		_ => match offhand {
			SHIELD => (&m1::SHIELD[..]   , &m2::SHIELD[..]),
			_      => (&m1::DUALWIELD[..], &m2::UNARMED[..])//use redirecting constant?
		}
	}
}