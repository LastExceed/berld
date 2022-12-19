#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum StatusEffectType {

	Bulwalk = 1,
	WarFrenzy,
	Camouflage,
	Poison,

	ManaShield = 6,


	FireSpark = 9,
	Intuition,
	Elusiveness,
	Swiftness
}