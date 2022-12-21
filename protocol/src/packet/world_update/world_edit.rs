#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlockType {
	Air,
	Solid,
	Liquid,
	Wet
}