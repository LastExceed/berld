use std::mem::size_of;
use std::io::{Error, Read, Write};
use crate::packet::creature_update::CreatureId;
use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketFromServer, PacketId};
use crate::utils::{ReadExtension, WriteExtension};

pub struct ChatMessageFromClient {
	pub text: String
}

impl CwSerializable for ChatMessageFromClient {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		let instance = Self {
			text: read_text(reader)?
		};
		Ok(instance)
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		write_text(writer, &self.text)
	}
}
impl Packet for ChatMessageFromClient {
	const ID: PacketId = PacketId::ChatMessage;
}
impl PacketFromClient for ChatMessageFromClient {}

pub struct ChatMessageFromServer {
	pub source: CreatureId,
	pub text: String
}

impl CwSerializable for ChatMessageFromServer {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		let instance = Self {
			source: reader.read_struct::<CreatureId>()?,
			text: read_text(reader)?
		};
		Ok(instance)
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		writer.write_struct(&self.source)?;
		write_text(writer, &self.text)
	}
}
impl Packet for ChatMessageFromServer {
	const ID: PacketId = PacketId::ChatMessage;
}
impl PacketFromServer for ChatMessageFromServer {}


fn read_text<T: Read>(reader: &mut T) -> Result<String, Error> {
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

fn write_text<T: Write>(writer: &mut T, string: &str) -> Result<(), Error> {
	let bytes = string
		.encode_utf16()
		.flat_map(|it: u16| { it.to_le_bytes() })
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use use the utf16 iterator as counting it's elements would consume it prematurely
	writer.write_struct(&character_count)?;
	writer.write_all(&bytes)
}