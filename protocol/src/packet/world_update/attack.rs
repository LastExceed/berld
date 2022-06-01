use crate::packet::CwSerializable;

#[repr(C)]
pub struct Attack {
	pub target: i64,
	pub attacker: i64,
	pub damage: f32,
	//pad4
}

impl CwSerializable for Attack {}