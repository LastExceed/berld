use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct ProtocolVersion(pub i32);

impl CwSerializable for ProtocolVersion {}
impl Packet for ProtocolVersion {
	fn id() -> PacketId {
		PacketId::ProtocolVersion
	}
}