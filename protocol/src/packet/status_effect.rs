#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Bulwalk = 1,
	WarFrenzy,
	Camouflage,
	Poison,

	ManaShield = 6,
	Affection,
	Anger,
	FireSpark,
	Intuition,
	Elusiveness,
	Swiftness
}