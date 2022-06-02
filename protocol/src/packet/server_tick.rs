use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct ServerTick {}

impl CwSerializable for ServerTick {}
impl Packet for ServerTick {
	fn id() -> PacketId {
		PacketId::ServerTick
	}
}
impl PacketFromServer for ServerTick {}