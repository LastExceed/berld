use nalgebra::Point;

use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct CurrentBiome(Point<i32, 2>);

impl CwSerializable for CurrentBiome {}
impl Packet for CurrentBiome {
	const ID: PacketId = PacketId::CurrentBiome;
}
impl PacketFromClient for CurrentBiome {}