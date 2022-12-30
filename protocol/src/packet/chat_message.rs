use std::mem::size_of;

use async_trait::async_trait;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::packet::*;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

#[async_trait]
impl CwSerializable for ChatMessageFromClient {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		Ok(
			Self {
				text: read_text(readable).await?
			}
		)
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		write_text(writable, &self.text).await
	}
}

#[async_trait]
impl CwSerializable for ChatMessageFromServer {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		Ok(
			Self {
				source: readable.read_struct::<CreatureId>().await?,
				text: read_text(readable).await?
			}
		)
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(&self.source).await?;
		write_text(writable, &self.text).await
	}
}

async fn read_text<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<String> {
	let character_count = readable.read_struct::<i32>().await? as usize;
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

async fn write_text<Writable: AsyncWrite + Unpin + Send>(writable: &mut Writable, string: &str) -> io::Result<()> {
	let bytes = string
		.encode_utf16()
		.flat_map(u16::to_le_bytes)
		.collect::<Vec<u8>>();
	let character_count = (bytes.len() / 2) as i32; //cant use the utf16 iterator as counting it's elements would consume it prematurely
	writable.write_struct(&character_count).await?;
	writable.write_all(&bytes).await
}