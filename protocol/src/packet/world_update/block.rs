#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
	Air,
	Solid,
	Liquid,
	Wet
}