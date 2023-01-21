use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, EnumCount)]
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

impl From<Slot> for usize {
	fn from(value: Slot) -> Self {
		value as Self
	}
}