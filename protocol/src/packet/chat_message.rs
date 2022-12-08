use std::io::{Error, Read, Write};
use std::mem::size_of;

use crate::packet::*;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

impl CwSerializable for ChatMessageFromClient {
	fn read_from(readable: &mut impl Read) -> Result<Self, Error> {
		Ok(
			Self {
				text: read_text(readable)?
			}
		)
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), Error> {
		write_text(writable, &self.text)
	}
}

impl CwSerializable for ChatMessageFromServer {
	fn read_from(readable: &mut impl Read) -> Result<Self, Error> {
		Ok(
			Self {
				source: readable.read_struct::<CreatureId>()?,
				text: read_text(readable)?
			}
		)
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), Error> {
		writable.write_struct(&self.source)?;
		write_text(writable, &self.text)
	}
}

fn read_text(readable: &mut impl Read) -> Result<String, Error> {
	let character_count = readable.read_struct::<i32>()? as usize;
	const U16_SIZE: usize = size_of::<u16>();

	let mut u8s = vec![0u8; character_count * U16_SIZE];

	readable.read_exact(&mut u8s)?;
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

fn write_text(writable: &mut impl Write, string: &str) -> Result<(), Error> {
	let bytes = string
		.encode_utf16()
		.flat_map(u16::to_le_bytes)
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use the utf16 iterator as counting it's elements would consume it prematurely
	writable.write_struct(&character_count)?;
	writable.write_all(&bytes)
}