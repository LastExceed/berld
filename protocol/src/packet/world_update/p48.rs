use std::io::{Error, Read, Write};

use nalgebra::Point2;

use crate::packet::CwSerializable;
use crate::packet::world_update::P48;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

impl CwSerializable for P48 {
	fn read_from(readable: &mut impl Read) -> Result<Self, Error> {
		Ok(Self {
			zone: readable.read_struct::<Point2<i32>>()?,
			sub_packets: Vec::read_from(readable)?
		})
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), Error> {
		writable.write_struct(&self.zone)?;
		self.sub_packets.write_to(writable)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct P48sub(pub [u8; 16]);

impl CwSerializable for P48sub {}