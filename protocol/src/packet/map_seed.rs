use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct MapSeed(pub i32);

impl CwSerializable for MapSeed {}
impl Packet for MapSeed {
	const ID: PacketId = PacketId::MapSeed;
}
impl PacketFromServer for MapSeed {}