#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum HitType {
	Default,
	Block,

	Miss = 3,
	Dodge,
	Absorb,
	Invisible
}