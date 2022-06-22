use nalgebra::Point;

use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct CurrentChunk(Point<i32, 2>);

impl CwSerializable for CurrentChunk {}
impl Packet for CurrentChunk {
	const ID: PacketId = PacketId::CurrentChunk;
}
impl PacketFromClient for CurrentChunk {}