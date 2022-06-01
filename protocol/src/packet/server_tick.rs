use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct ServerTick {}

impl CwSerializable for ServerTick {}
impl Packet for ServerTick {
	fn id() -> PacketId {
		PacketId::ServerTick
	}
}