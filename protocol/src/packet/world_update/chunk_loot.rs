use std::io::{Error, Read, Write};

use nalgebra::{Point2, Point3};

use crate::packet::CwSerializable;
use crate::packet::Item;
use crate::packet::world_update::ChunkLoot;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

//todo: implementation is extremely similar to P48
impl CwSerializable for ChunkLoot {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(Self {
			chunk: reader.read_struct::<Point2<i32>>()?,
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
	pub position: Point3<i64>,
	pub rotation: f32,
	pub scale: f32,
	pub unknown_a: u8,
	//pad3
	pub droptime: i32,
	pub unknown_b: i32,
	//pad4
}

impl CwSerializable for Drop {}