use crate::packet::{CwSerializable, Packet, PacketId};

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