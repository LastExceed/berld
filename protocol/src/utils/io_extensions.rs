use std::io::{Error, Read, Write};
use std::mem::size_of;
use std::slice::from_raw_parts;

pub trait ReadExtension: Read {
	fn read_struct<T>(&mut self) -> Result<T, Error>
		where [(); size_of::<T>()]:
	{
		let mut buffer = [0u8; size_of::<T>()];
		self.read_exact(&mut buffer)?;

		//Ok(unsafe { transmute::<[u8; size_of::<T>()], T>(buffer)})
		Ok(unsafe { (buffer.as_ptr().cast::<T>()).read() })
	}
}

pub trait WriteExtension: Write {
	fn write_struct<T>(&mut self, data: &T) -> Result<(), Error>
		where [(); size_of::<T>()]:
	{
		self.write_all(unsafe { from_raw_parts((data as *const T).cast::<u8>(), size_of::<T>()) })
	}
}

impl<T> ReadExtension for T where T: Read {}
impl<T> WriteExtension for T where T: Write {}