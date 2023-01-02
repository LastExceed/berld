#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(cstr_from_bytes_until_nul)]
#![allow(const_evaluatable_unchecked)]
#![feature(async_closure)]

use std::mem::size_of;

use async_trait::async_trait;
pub use nalgebra;
pub use rgb;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::utils::io_extensions::{ReadStruct, WriteStruct};

pub mod packet;
pub mod utils;

pub const SIZE_BLOCK: i64 = 65536;
pub const SIZE_MAPBLOCK: i64 = SIZE_BLOCK * 8;
pub const SIZE_CHUNK: i64 = SIZE_BLOCK * 32;
pub const SIZE_ZONE: i64 = SIZE_CHUNK * 8;
pub const SIZE_REGION: i64 = SIZE_ZONE * 64;
pub const SIZE_WORLD: i64 = SIZE_REGION * 1024;
pub const SIZE_UNIVERSE: i64 = SIZE_WORLD * 256;
//const SIZE_MULTIVERSE: i64 = SIZE_UNIVERSE * 65536; //overflows; it's basically u64::MAX + 1

#[async_trait]
pub trait CwSerializable: Sized {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self>
		where [(); size_of::<Self>()]:
	{
		readable.read_struct::<Self>().await
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(self).await
	}
}

#[async_trait]
pub trait Packet: CwSerializable {
	const ID: packet::Id; //dedicated type ensures this can't be used in any mathematic manner

	async fn write_to_with_id<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> Result<(), io::Error> {
		writable.write_struct(&Self::ID).await?;
		self.write_to(writable).await
	}
}