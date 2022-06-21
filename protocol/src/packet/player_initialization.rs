use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct PlayerInitialization;

impl CwSerializable for PlayerInitialization {}
impl Packet for PlayerInitialization {
	const ID: PacketId = PacketId::PlayerInitialization;
}
impl PacketFromServer for PlayerInitialization {}