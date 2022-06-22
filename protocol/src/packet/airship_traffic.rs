use std::io::Error;

use crate::packet::*;

pub struct AirshipTraffic {
	pub airships: Vec<Airship>
}

impl CwSerializable for AirshipTraffic {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		Ok(Self { airships: Vec::read_from(reader)? })
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		self.airships.write_to(writer)
	}
}
impl Packet for AirshipTraffic {
	const ID: PacketId = PacketId::AirshipTraffic;
}
impl PacketFromClient for AirshipTraffic {}

#[repr(C)]
pub struct Airship {
	pub id: i64,
	pub unknown_a: i32,
	pub unknown_b: i32,
	pub position: Point<i64, 3>,
	pub velocity: Vector3<f32>,
	pub rotation: f32,
	pub station: Point<i64, 3>,
	pub path_rotation: f32,
	//pad4
	pub destination: Point<i64, 3>,
	pub state: State,
	// u8 ?
	// pad3
}

impl CwSerializable for Airship {}

#[repr(i32)]
pub enum State {
	Unknown //todo
}