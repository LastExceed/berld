use nalgebra::Point;
use crate::packet::CwSerializable;

#[repr(C)]
pub struct WorldEdit {
	pub position: Point<i32, 3>,
	pub color: [u8; 3],//todo: type
	pub block_type: BlockType,
	pub padding: i32
}

impl CwSerializable for WorldEdit {}

#[repr(u8)]
pub enum BlockType {
	Air,
	Solid,
	Liquid,
	Wet
}