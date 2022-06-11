use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct ServerTick {}

impl CwSerializable for ServerTick {}
impl Packet for ServerTick {
	const ID: PacketId = PacketId::ServerTick;
}
impl PacketFromServer for ServerTick {}