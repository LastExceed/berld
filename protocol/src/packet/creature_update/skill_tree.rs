use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, EnumCount)]
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

impl From<Skill> for usize {
	fn from(value: Skill) -> Self {
		value as Self
	}
}