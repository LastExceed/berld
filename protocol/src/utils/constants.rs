use crate::packet::common::item::{TypeMajor, TypeMinor};

pub mod item_types;

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct ItemType {
	pub major: TypeMajor,
	pub minor: TypeMinor
}