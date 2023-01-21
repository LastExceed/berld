use std::ops::{Index, IndexMut};
use std::slice::Iter;

use crate::packet::common::Item;
use crate::packet::creature_update::Equipment;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Slot {
	Unknown,
	Neck,
	Chest,
	Feet,
	Hands,
	Shoulder,
	LeftWeapon,
	RightWeapon,
	LeftRing,
	RightRing,
	Lamp,
	Special,
	Pet,
}

impl Equipment {
	pub fn iter(&self) -> Iter<'_, <Self as Index<Slot>>::Output> {
		self.0.iter()
	}
}

impl Index<Slot> for Equipment {
	type Output = Item;

	fn index(&self, index: Slot) -> &Self::Output {
		&self.0[index as usize]
	}
}

impl IndexMut<Slot> for Equipment {
	fn index_mut(&mut self, index: Slot) -> &mut Self::Output {
		&mut self.0[index as usize]
	}
}