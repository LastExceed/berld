use nalgebra::Point;
use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct ChunkDiscovery {
	pub chunk: Point<i32, 2>
}

impl CwSerializable for ChunkDiscovery {}
impl Packet for ChunkDiscovery {
	fn id() -> PacketId {
		PacketId::ChunkDiscovery
	}
}