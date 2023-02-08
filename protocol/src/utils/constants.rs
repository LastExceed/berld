use crate::packet::common::Race;
use crate::packet::common::Race::*;
use crate::packet::creature_update::{Occupation, Specialization};

pub mod combat_classes;
pub mod animations;
pub mod materials;

pub const SIZE_BLOCK: i64 = 65536;
pub const SIZE_MAPBLOCK: i64 = SIZE_BLOCK * 8;
pub const SIZE_CHUNK: i64 = SIZE_BLOCK * 32;
pub const SIZE_ZONE: i64 = SIZE_CHUNK * 8;//maptile
pub const SIZE_SECTOR: i64 = SIZE_ZONE * 8;//dungeon grid
pub const SIZE_REGION: i64 = SIZE_ZONE * 64;//biome
pub const SIZE_WORLD: i64 = SIZE_REGION * 1024;
pub const SIZE_UNIVERSE: i64 = SIZE_WORLD * 256;
//const SIZE_MULTIVERSE: i64 = SIZE_UNIVERSE * 65536; //overflows; it's basically u64::MAX + 1

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct CombatClass {
	pub occupation: Occupation,
	pub specialization: Specialization
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