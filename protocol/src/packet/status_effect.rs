#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusEffectType {
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