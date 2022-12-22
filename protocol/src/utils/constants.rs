use crate::packet::common::item::{TypeMajor, TypeMinor};
use crate::packet::common::Race;
use crate::packet::common::Race::*;
use crate::packet::creature_update::{CombatClassMajor, CombatClassMinor};

pub mod item_types;
pub mod combat_classes;
pub mod animations;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct ItemType {
	pub major: TypeMajor,
	pub minor: TypeMinor
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct CombatClass {
	pub major: CombatClassMajor,
	pub minor: CombatClassMinor
}

pub const PLAYABLE_RACES: [Race; 16] = [
	ElfMale,
	ElfFemale,
	HumanMale,
	HumanFemale,
	GoblinMale,
	GoblinFemale,
	LizardmanMale,
	LizardmanFemale,
	DwarfMale,
	DwarfFemale,
	OrcMale,
	OrcFemale,
	FrogmanMale,
	FrogmanFemale,
	UndeadMale,
	UndeadFemale
];