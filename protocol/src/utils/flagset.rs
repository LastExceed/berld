use std::marker::PhantomData;

use num_traits::PrimInt;

//todo: use size_of<flag> to infer inner, maybe [u8] ?
#[derive(Debug, PartialEq, Eq, Hash, Clone)] //todo: default?
pub struct FlagSet<Inner: PrimInt, Flag: Into<usize>>(Inner, PhantomData<Flag>);

impl<Inner: PrimInt, Flag: Into<usize>> FlagSet<Inner, Flag> {
	pub fn get(&self, flag: Flag) -> bool {
		self.0 & (Inner::from(1).unwrap() << flag.into()) > Inner::from(0).unwrap()
	}
	pub fn set(&mut self, flag: Flag, value: bool) {
		let index = flag.into();
		self.0 = (self.0 & !(Inner::from(1).unwrap() << index)) | ((Inner::from(value as usize).unwrap()) << index);
	}
}

impl<Inner: PrimInt, Flag: Into<usize>> Default for FlagSet<Inner, Flag> {
	fn default() -> Self {
		Self(Inner::from(0).unwrap(), PhantomData)
	}
}