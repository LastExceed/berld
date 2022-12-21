use std::marker::PhantomData;

//todo: there gotta be a better way to do this

#[derive(Debug, PartialEq, Eq, Clone)]
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


#[derive(Debug, PartialEq, Eq, Clone)]
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


#[derive(Debug, PartialEq, Eq, Clone)]
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