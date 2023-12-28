use strum_macros::{EnumCount, EnumIter};
use num_enum::IntoPrimitive;

use crate::utils::ArrayWrapperIndex;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, EnumCount, IntoPrimitive)]
#[repr(usize)]
pub enum Multiplier {
	Health,
	AttackSpeed,
	Damage,
	Armor,
	Resi
}

impl ArrayWrapperIndex for Multiplier {
	type Item = f32;
}