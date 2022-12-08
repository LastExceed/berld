use std::io::{Error, Read, Write};
use std::mem::size_of;

use crate::packet::*;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

impl CwSerializable for ChatMessageFromClient {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(
			Self {
				text: read_text(reader)?
			}
		)
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		write_text(writer, &self.text)
	}
}

impl CwSerializable for ChatMessageFromServer {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(
			Self {
				source: reader.read_struct::<CreatureId>()?,
				text: read_text(reader)?
			}
		)
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		writer.write_struct(&self.source)?;
		write_text(writer, &self.text)
	}
}

fn read_text(reader: &mut impl Read) -> Result<String, Error> {
	let character_count = reader.read_struct::<i32>()? as usize;
	const U16_SIZE: usize = size_of::<u16>();

	let mut u8s = vec![0u8; character_count * U16_SIZE];

	reader.read_exact(&mut u8s)?;
	let u16s = u8s
		.windows(U16_SIZE)
		.step_by(U16_SIZE)
		.map(|window| {
			u16::from_le_bytes(
				window
					.try_into()
					.unwrap()
			)
		})
		.collect::<Vec<_>>();

	Ok(String::from_utf16_lossy(&u16s))
}

fn write_text(writer: &mut impl Write, string: &str) -> Result<(), Error> {
	let bytes = string
		.encode_utf16()
		.flat_map(u16::to_le_bytes)
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use the utf16 iterator as counting it's elements would consume it prematurely
	writer.write_struct(&character_count)?;
	writer.write_all(&bytes)
}