use std::io::{Error, Read, Write};

use nalgebra::Point2;

use crate::io_extensions::{ReadExtension, WriteExtension};
use crate::packet::CwSerializable;
use crate::packet::world_update::P48;

impl CwSerializable for P48 {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(Self {
			chunk: reader.read_struct::<Point2<i32>>()?,
			sub_packets: Vec::read_from(reader)?
		})
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		writer.write_struct(&self.chunk)?;
		self.sub_packets.write_to(writer)
	}
}

pub struct P48sub([u8; 16]);

impl CwSerializable for P48sub {}