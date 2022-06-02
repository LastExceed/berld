use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C)]
pub struct IngameDateTime {
	pub day: i32,
	pub time: i32
}

impl CwSerializable for IngameDateTime {}
impl Packet for IngameDateTime {
	fn id() -> PacketId {
		PacketId::IngameDatetime
	}
}
impl PacketFromServer for IngameDateTime {}