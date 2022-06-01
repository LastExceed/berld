use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct ServerFull {}

impl CwSerializable for ServerFull {}
impl Packet for ServerFull {
	fn id() -> PacketId {
		PacketId::ServerFull
	}
}