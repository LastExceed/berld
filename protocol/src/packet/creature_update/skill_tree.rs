use std::ops::{Index, IndexMut};
use std::slice::Iter;

use strum_macros::EnumIter;

use crate::packet::creature_update::SkillTree;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
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

impl SkillTree {
	pub fn iter(&self) -> Iter<'_, <Self as Index<Skill>>::Output> {
		self.0.iter()
	}
}

impl Index<Skill> for SkillTree {
	type Output = i32;

	fn index(&self, index: Skill) -> &Self::Output {
		&self.0[index as usize]
	}
}

impl IndexMut<Skill> for SkillTree {
	fn index_mut(&mut self, index: Skill) -> &mut Self::Output {
		&mut self.0[index as usize]
	}
}

//todo: this entire file is almost a 1:1 copypaste of equipment.rs