use crate::packet::creature_update::Occupation::*;
use crate::packet::creature_update::Specialization::*;
use crate::utils::constants::CombatClass;

pub const BERSERKER: CombatClass = CombatClass {
	occupation: Warrior,
	specialization: Default
};

pub const GUARDIAN: CombatClass = CombatClass {
	occupation: Warrior,
	specialization: Alternative
};

pub const SNIPER: CombatClass = CombatClass {
	occupation: Ranger,
	specialization: Default
};

pub const SCOUT: CombatClass = CombatClass {
	occupation: Ranger,
	specialization: Alternative
};

pub const FIRE_MAGE: CombatClass = CombatClass {
	occupation: Mage,
	specialization: Default
};

pub const WATER_MAGE: CombatClass = CombatClass {
	occupation: Mage,
	specialization: Alternative
};

pub const ASSASSIN: CombatClass = CombatClass {
	occupation: Rogue,
	specialization: Default
};

pub const NINJA: CombatClass = CombatClass {
	occupation: Rogue,
	specialization: Alternative
};