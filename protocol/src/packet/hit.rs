#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum HitType {
	#[default]
	Default,
	Block,

	Miss = 3,
	Dodge,
	Absorb,
	Invisible
}