use nalgebra::Point;
use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct ChunkDiscovery {
	pub chunk: Point<i32, 2>
}

impl CwSerializable for ChunkDiscovery {}
impl Packet for ChunkDiscovery {
	const ID: PacketId = PacketId::ChunkDiscovery;
}
impl PacketFromClient for ChunkDiscovery {}