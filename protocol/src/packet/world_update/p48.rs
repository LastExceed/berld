use std::io::{Error, Read, Write};

use nalgebra::Point;

use crate::packet::CwSerializable;
use crate::utils::{ReadExtension, WriteExtension};

pub struct P48 {
	pub chunk: Point<i32, 2>,
	pub sub_packets: Vec<P48sub>
}

impl CwSerializable for P48 {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		Ok(Self {
			chunk: reader.read_struct::<Point<i32, 2>>()?,
			sub_packets: Vec::read_from(reader)?
		})
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		writer.write_struct(&self.chunk)?;
		self.sub_packets.write_to(writer)
	}
}

pub struct P48sub([u8; 16]);

impl CwSerializable for P48sub {}