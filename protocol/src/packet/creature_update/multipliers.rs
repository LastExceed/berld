use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, EnumCount)]
pub enum Multiplier {
	Health,
	AttackSpeed,
	Damage,
	Armor,
	Resi
}

impl From<Multiplier> for usize {
	fn from(value: Multiplier) -> Self {
		value as Self
	}
}