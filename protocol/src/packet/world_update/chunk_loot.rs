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
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		let chunk = reader.read_struct::<Point<i32, 2>>()?;

		let drop_count = reader.read_struct::<i32>()?;
		let mut drops = Vec::with_capacity(drop_count as usize);
		for _ in 0..drop_count {
			drops.push(reader.read_struct::<Drop>()?);
		};

		let instance = Self {
			chunk,
			drops
		};
		Ok(instance)
	}
	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		writer.write_struct(&self.chunk)?;
		writer.write_struct(&(self.drops.len() as i32))?;
		for drop in &self.drops {
			writer.write_struct(drop)?;
		}
		Ok(())
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