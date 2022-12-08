use std::io::{Error, Read, Write};
use std::mem::size_of;
use std::slice::from_raw_parts;

pub trait ReadExtension: Read {
	fn read_struct<Data>(&mut self) -> Result<Data, Error>
		where [(); size_of::<Data>()]:
	{
		let mut buffer = [0u8; size_of::<Data>()];
		self.read_exact(&mut buffer)?;

		//Ok(unsafe { transmute::<[u8; size_of::<Data>()], T>(buffer)})
		Ok(unsafe { (buffer.as_ptr().cast::<Data>()).read() })
	}
}

pub trait WriteExtension: Write {
	fn write_struct<Data>(&mut self, data: &Data) -> Result<(), Error>
		where [(); size_of::<Data>()]:
	{
		self.write_all(unsafe { from_raw_parts((data as *const Data).cast::<u8>(), size_of::<Data>()) })
	}
}

impl<Readable: Read> ReadExtension for Readable {}
impl<Writable: Write> WriteExtension for Writable {}