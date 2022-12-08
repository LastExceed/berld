use std::io::Error;

use nalgebra::Point3;

use crate::packet::*;

impl CwSerializable for AirshipTraffic {
	fn read_from(readable: &mut impl Read) -> Result<Self, Error> {
		Ok(Self { airships: Vec::read_from(readable)? })
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), Error> {
		self.airships.write_to(writable)
	}
}

#[repr(C)]
pub struct Airship {
	pub id: i64,
	pub unknown_a: i32,
	pub unknown_b: i32,
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub rotation: f32,
	pub station: Point3<i64>,
	pub path_rotation: f32,
	//pad4
	pub destination: Point3<i64>,
	pub state: State,
	// u8 ?
	// pad3
}

impl CwSerializable for Airship {}

#[repr(i32)]
pub enum State {
	Unknown //todo
}