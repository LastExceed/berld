use nalgebra::Point;
use crate::packet::CwSerializable;
use crate::packet::creature_update::Race;

#[repr(C)]
pub struct Mission {
	pub sector: Point<i32, 2>,
	pub unknown_a: i32,
	pub unknown_b: i32,
	pub unknown_c: i32,
	pub id: i32,
	pub kind: i32,
	pub boss: Race,
	pub level: i32,
	pub unknown_d: u8,
	pub state: MissionState,
	//pad2
	pub health_current: i32,
	pub health_maximum: i32,
	pub chunk: Point<i32, 2>
}

impl CwSerializable for Mission {}

#[repr(u8)]
pub enum MissionState {
	TODO
}