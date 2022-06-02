use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketFromServer, PacketId};

#[repr(C)]
pub struct ProtocolVersion(pub i32);

impl CwSerializable for ProtocolVersion {}
impl Packet for ProtocolVersion {
	fn id() -> PacketId {
		PacketId::ProtocolVersion
	}
}
impl PacketFromServer for ProtocolVersion {}
impl PacketFromClient for ProtocolVersion {}