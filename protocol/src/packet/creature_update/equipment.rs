use num_enum::IntoPrimitive;
use strum_macros::{EnumCount, EnumIter};

use crate::utils::ArrayWrapperIndex;
use crate::packet::common::Item;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, EnumCount, IntoPrimitive)]
#[repr(usize)]
pub enum Slot {
	Unknown,
	Neck,
	Chest,
	Feet,
	Hands,
	Shoulder,
	LeftWeapon,
	RightWeapon,
	LeftRing,
	RightRing,
	Lamp,
	Special,
	Pet,
}

impl ArrayWrapperIndex for Slot {
	type Item = Item;
}