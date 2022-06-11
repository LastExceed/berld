use nalgebra::Point;
use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct SectorDiscovery {
	pub sector: Point<i32, 2>
}

impl CwSerializable for SectorDiscovery {}
impl Packet for SectorDiscovery {
	const ID: PacketId = PacketId::SectorDiscovery;
}
impl PacketFromClient for SectorDiscovery {}