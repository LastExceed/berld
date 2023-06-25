#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Air,
	Solid,
	Liquid,
	Wet
}