use nalgebra::Point;
use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct SectorDiscovery {
	pub sector: Point<i32, 2>
}

impl CwSerializable for SectorDiscovery {}
impl Packet for SectorDiscovery {
	fn id() -> PacketId {
		PacketId::SectorDiscovery
	}
}