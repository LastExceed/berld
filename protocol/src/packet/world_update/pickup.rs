use crate::packet::CwSerializable;
use crate::packet::creature_update::CreatureId;
use crate::packet::Item;

#[repr(C)]
pub struct Pickup {
	pub interactor: CreatureId,
	pub item: Item
}

impl CwSerializable for Pickup {}