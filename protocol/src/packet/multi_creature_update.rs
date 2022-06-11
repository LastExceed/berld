use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct MultiCreatureUpdate {
	//todo
}

impl CwSerializable for MultiCreatureUpdate {}
impl Packet for MultiCreatureUpdate {
	const ID: PacketId = PacketId::MultiEntityUpdate;
}
impl PacketFromServer for MultiCreatureUpdate {}