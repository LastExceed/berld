use std::io;
use std::io::{Read, Write};
use std::mem::size_of;

use crate::CwSerializable;
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

pub mod io_extensions;
pub mod flagset;

///ideally this would be done with a #[derive()] macro instead,
///but the boilerplate required for that is completely overkill for this use case
#[macro_export]
macro_rules! bulk_impl {
	($trait:ident for $($struct:ty),*) => { //todo: investigate if 'trait' can be restricted to :ty
		$(impl $trait for $struct {})*
	}
}

impl<Element: CwSerializable> CwSerializable for Vec<Element>
	where [(); size_of::<Element>()]:
{
	fn read_from(readable: &mut impl Read) -> Result<Self, io::Error> {
		(0..readable.read_struct::<i32>()?)
			.map(|_| Element::read_from(readable))
			.collect::<Result<Self, io::Error>>()
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), io::Error> {
		writable.write_struct(&(self.len() as i32))?;
		for element in self {
			element.write_to(writable)?;
		}
		Ok(())
	}
}