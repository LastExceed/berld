use crate::packet::common::item::{TypeMajor, TypeMinor};
use crate::packet::creature_update::{CombatClassMajor, CombatClassMinor};

pub mod item_types;
pub mod combat_classes;
pub mod animations;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct ItemType {
	pub major: TypeMajor,
	pub minor: TypeMinor
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct CombatClass {
	pub major: CombatClassMajor,
	pub minor: CombatClassMinor
}