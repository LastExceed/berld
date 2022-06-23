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

	let mut u8s = vec![0u8; character_count * 2];
	reader.read_exact(&mut u8s)?;

	let window_size = size_of::<u16>();
	let u16s = u8s
		.windows(window_size)
		.step_by(window_size)
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
		.flat_map(|u16character| u16character.to_le_bytes())
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use use the utf16 iterator as counting it's elements would consume it prematurely
	writer.write_struct(&character_count)?;
	writer.write_all(&bytes)
}