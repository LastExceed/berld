use crate::packet::CwSerializable;
use crate::packet::creature_update::CreatureId;

#[repr(C)]
pub struct Kill {
	pub killer: CreatureId,
	pub victim: CreatureId,
	pub unknown: i32,
	pub xp: i32
}

impl CwSerializable for Kill {}