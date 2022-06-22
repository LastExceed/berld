#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(cstr_from_bytes_until_nul)]
#![allow(const_evaluatable_unchecked)]

use std::io::{Error, Read, Write};
use std::mem::size_of;

pub use nalgebra;

use crate::io_extensions::{ReadExtension, WriteExtension};
use crate::packet::common::PacketId;

pub mod packet;
pub mod io_extensions;
pub mod flagset;

pub trait CwSerializable: Sized {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error>
		where [(); size_of::<Self>()]:
	{
		reader.read_struct::<Self>()
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(self)
	}
}

impl<Element: CwSerializable> CwSerializable for Vec<Element>
	where [(); size_of::<Element>()]:
{
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		(0..reader.read_struct::<i32>()?)
			.map(|_| Element::read_from(reader))
			.collect::<Result<Self, Error>>()
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		writer.write_struct(&(self.len() as i32))?;
		for element in self {
			writer.write_struct::<Element>(element)?;
		}
		Ok(())
	}
}

pub trait Packet: CwSerializable {
	const ID: PacketId;

	fn write_to_with_id(&self, writer: &mut impl Write) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(&Self::ID)?;
		self.write_to(writer)
	}
}

pub trait PacketFromServer: Packet {}
pub trait PacketFromClient: Packet {}