use std::io::Error;
use crate::packet::*;

pub struct AirshipTraffic {
	pub airships: Vec<Airship>
}

impl CwSerializable for AirshipTraffic {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		let airship_count = reader.read_struct::<i32>()?;
		let mut airships = Vec::with_capacity(airship_count as usize);
		for _ in 0..airship_count {
			airships.push(reader.read_struct::<Airship>()?);
		};
		Ok(Self { airships })
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		writer.write_struct(&(self.airships.len() as i32))?;
		for airship in &self.airships {
			writer.write_struct(airship)?;
		}
		Ok(())
	}
}
impl Packet for AirshipTraffic {
	fn id() -> PacketId {
		PacketId::AirshipTraffic
	}
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

#[repr(i32)]
pub enum State {
	Unknown //todo
}