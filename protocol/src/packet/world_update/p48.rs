use std::io::{Error, Read, Write};
use nalgebra::Point;
use crate::packet::CwSerializable;
use crate::utils::{ReadExtension, WriteExtension};

pub struct P48 {
	pub chunk: Point<i32, 2>,
	pub sub_packets: Vec<[u8; 16]>
}

impl CwSerializable for P48 {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		let chunk = reader.read_struct::<Point<i32, 2>>()?;

		let sub_packet_count = reader.read_struct::<i32>()?;
		let mut sub_packets = Vec::with_capacity(sub_packet_count as usize);
		for _ in 0..sub_packet_count {
			sub_packets.push(reader.read_struct::<[u8; 16]>()?);
		};

		let instance = Self {
			chunk,
			sub_packets
		};
		Ok(instance)
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		writer.write_struct(&self.chunk)?;
		writer.write_struct(&(self.sub_packets.len() as i32))?;
		for sub_packet in &self.sub_packets {
			writer.write_all(sub_packet)?;
		}
		Ok(())
	}
}