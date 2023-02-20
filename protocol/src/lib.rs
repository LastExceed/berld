#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(cstr_from_bytes_until_nul)]
#![allow(const_evaluatable_unchecked)]
#![feature(async_closure)]
#![feature(async_fn_in_trait)]

use std::mem::size_of;

pub use nalgebra;
pub use rgb;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::utils::io_extensions::{ReadStruct, WriteStruct};

pub mod packet;
pub mod utils;

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

pub trait Packet: CwSerializable {
	const ID: packet::Id; //dedicated type ensures this can't be used in any mathematic manner

	async fn write_to_with_id<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(&Self::ID).await?;
		self.write_to(writable).await
	}
}