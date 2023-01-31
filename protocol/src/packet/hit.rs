#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Kind {
	#[default]
	Normal,
	Block,

	Miss = 3,
	Dodge,
	Absorb,
	Invisible
}