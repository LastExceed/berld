use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct ServerFull {}

impl CwSerializable for ServerFull {}
impl Packet for ServerFull {
	fn id() -> PacketId {
		PacketId::ServerFull
	}
}
impl PacketFromServer for ServerFull {}