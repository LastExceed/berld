#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(cstr_from_bytes_until_nul)]
#![allow(const_evaluatable_unchecked)]

use std::io;
use std::io::{Read, Write};
use std::mem::size_of;

pub use nalgebra;
pub use rgb;

use crate::utils::io_extensions::{ReadExtension, WriteExtension};

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

pub trait CwSerializable: Sized {
	fn read_from(readable: &mut impl Read) -> Result<Self, io::Error>
		where [(); size_of::<Self>()]:
	{
		readable.read_struct::<Self>()
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), io::Error>
		where [(); size_of::<Self>()]:
	{
		writable.write_struct(self)
	}
}

pub trait Packet: CwSerializable {
	const ID: packet::Id; //dedicated type ensures this can't be used in any mathematic manner

	fn write_to_with_id(&self, writable: &mut impl Write) -> Result<(), io::Error>
		where [(); size_of::<Self>()]:
	{
		writable.write_struct(&Self::ID)?;
		self.write_to(writable)
	}
}