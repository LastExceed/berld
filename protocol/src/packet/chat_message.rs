use std::mem::size_of;

use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::packet::*;
use crate::ReadCwData;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

async fn read_text<Readable: AsyncRead + Unpin>(readable: &mut Readable) -> io::Result<String> {
	let character_count = readable.read_arbitrary::<i32>().await? as usize;
	const U16_SIZE: usize = size_of::<u16>();

	let mut u8s = vec![0u8; character_count * U16_SIZE];

	readable.read_exact(&mut u8s).await?;
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

async fn write_text<Writable: AsyncWrite + Unpin>(writable: &mut Writable, string: &str) -> io::Result<()> {
	let bytes = string
		.encode_utf16()
		.flat_map(u16::to_le_bytes)
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use the utf16 iterator as counting it's elements would consume it prematurely
	writable.write_arbitrary(&character_count).await?;
	writable.write_all(&bytes).await
}


impl<Readable: AsyncRead + Unpin> ReadCwData<ChatMessageFromClient> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<ChatMessageFromClient> {
		Ok(ChatMessageFromClient { text: read_text(self).await? })
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<ChatMessageFromClient> for Writable {
	async fn write_cw_data(&mut self, chat_message: &ChatMessageFromClient) -> io::Result<()> {
		write_text(self, &chat_message.text).await
	}
}


impl<Readable: AsyncRead + Unpin> ReadCwData<ChatMessageFromServer> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<ChatMessageFromServer> {
		Ok(ChatMessageFromServer {
			source: self.read_arbitrary().await?,
			text: read_text(self).await?
		})
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<ChatMessageFromServer> for Writable {
	async fn write_cw_data(&mut self, cw_struct: &ChatMessageFromServer) -> io::Result<()> {
		self.write_arbitrary(&cw_struct.source).await?;
		write_text(self, &cw_struct.text).await
	}
}


impl From<ChatMessageFromServer> for ChatMessageFromClient {
	fn from(value: ChatMessageFromServer) -> Self {
		Self {
			text: value.text,
		}
	}
}

impl ChatMessageFromServer {
	pub fn from_reverse(value: ChatMessageFromClient, source: CreatureId) -> Self {
		Self {
			source,
			text: value.text,
		}
	}
}

impl ChatMessageFromClient {
	pub fn into_reverse(self, source: CreatureId) -> ChatMessageFromServer {
		ChatMessageFromServer::from_reverse(self, source)
	}
}