#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum BlockType {
	Air,
	Solid,
	Liquid,
	Wet
}