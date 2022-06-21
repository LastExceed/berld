use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct ConnectionRejection {}

impl CwSerializable for ConnectionRejection {}
impl Packet for ConnectionRejection {
	const ID: PacketId = PacketId::ServerFull;
}
impl PacketFromServer for ConnectionRejection {}