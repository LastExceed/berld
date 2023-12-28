use num_enum::IntoPrimitive;
use strum_macros::{EnumCount, EnumIter};

use crate::utils::ArrayWrapperIndex;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, EnumCount, IntoPrimitive)]
#[repr(usize)]
pub enum Skill {
	PetMaster,
	PetRiding,
	Sailing,
	Climbing,
	HangGliding,
	Swimming,
	Aility1,
	Ability2,
	Ability3,
	Ability4,
	Ability5
}

impl ArrayWrapperIndex for Skill {
	type Item = i32;
}