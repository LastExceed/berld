use std::io::{Error, Read, Write};
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
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


//todo: there gotta be a better way to do this
pub struct FlagSet8<F: Into<u8>>(u8, PhantomData<F>);
impl<F: Into<u8>> FlagSet8<F> {
	pub fn new() -> Self {
		Self(0, PhantomData::<F>::default())
	}
	pub fn get(&self, flag: F) -> bool {
		self.0 & (1 << flag.into()) > 0
	}
	pub fn set(&mut self, flag: F, value: bool) {
		let index = flag.into();
		self.0 = (self.0 & !(1 << index)) | ((value as u8) << index);
	}
}
impl<F: Into<u8>> Default for FlagSet8<F> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct FlagSet16<F: Into<u16>>(u16, PhantomData<F>);
impl<F: Into<u16>> FlagSet16<F> {
	pub fn new() -> Self {
		Self(0, PhantomData::<F>::default())
	}
	pub fn get(&self, flag: F) -> bool {
		self.0 & (1 << flag.into()) > 0
	}
	pub fn set(&mut self, flag: F, value: bool) {
		let index = flag.into();
		self.0 = (self.0 & !(1 << index)) | ((value as u16) << index);
	}
}
impl<F: Into<u16>> Default for FlagSet16<F> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct FlagSet32<F: Into<u32>>(u32, PhantomData<F>);
impl<F: Into<u32>> FlagSet32<F> {
	pub fn new() -> Self {
		Self(0, PhantomData::<F>::default())
	}
	pub fn get(&self, flag: F) -> bool {
		self.0 & (1 << flag.into()) > 0
	}
	pub fn set(&mut self, flag: F, value: bool) {
		let index = flag.into();
		self.0 = (self.0 & !(1 << index)) | ((value as u32) << index);
	}
}
impl<F: Into<u32>> Default for FlagSet32<F> {
	fn default() -> Self {
		Self::new()
	}
}