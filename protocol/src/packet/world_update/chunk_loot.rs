use std::io::{Error, Read, Write};

use nalgebra::Point;

use crate::packet::CwSerializable;
use crate::packet::Item;
use crate::utils::{ReadExtension, WriteExtension};

pub struct ChunkLoot {
	pub chunk: Point<i32, 2>,
	pub drops: Vec<Drop>
}

//todo: implementation is extremely similar to P48
impl CwSerializable for ChunkLoot {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(Self {
			chunk: reader.read_struct::<Point<i32, 2>>()?,
			drops: Vec::read_from(reader)?
		})
	}
	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		writer.write_struct(&self.chunk)?;
		self.drops.write_to(writer)
	}
}

#[repr(C)]
pub struct Drop {
	pub item: Item,
	pub position: Point<i64, 3>,
	pub rotation: f32,
	pub scale: f32,
	pub unknown_a: u8,
	//pad3
	pub droptime: i32,
	pub unknown_b: i32,
	//pad4
}

impl CwSerializable for Drop {}