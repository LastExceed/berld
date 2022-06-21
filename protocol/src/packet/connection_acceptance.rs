use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct ConnectionAcceptance;

impl CwSerializable for ConnectionAcceptance {}
impl Packet for ConnectionAcceptance {
	const ID: PacketId = PacketId::PlayerInitialization;
}
impl PacketFromServer for ConnectionAcceptance {}