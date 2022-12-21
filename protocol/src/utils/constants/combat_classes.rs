use crate::packet::creature_update::CombatClassMajor::*;
use crate::packet::creature_update::CombatClassMinor::*;
use crate::utils::constants::CombatClass;

pub const BERSERKER: CombatClass = CombatClass {
	major: Warrior,
	minor: Default
};

pub const GUARDIAN: CombatClass = CombatClass {
	major: Warrior,
	minor: Alternative
};

pub const SNIPER: CombatClass = CombatClass {
	major: Ranger,
	minor: Default
};

pub const SCOUT: CombatClass = CombatClass {
	major: Ranger,
	minor: Default
};

pub const FIRE_MAGE: CombatClass = CombatClass {
	major: Mage,
	minor: Default
};

pub const WATER_MAGE: CombatClass = CombatClass {
	major: Mage,
	minor: Alternative
};

pub const ASSASSIN: CombatClass = CombatClass {
	major: Rogue,
	minor: Default
};

pub const NINJA: CombatClass = CombatClass {
	major: Rogue,
	minor: Alternative
};