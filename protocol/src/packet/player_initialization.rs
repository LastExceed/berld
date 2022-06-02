use crate::packet::creature_update::CreatureId;
use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};

#[repr(C, packed(4))]
pub struct PlayerInitialization {
	pub unknown: i32,
	pub assigned_id: CreatureId,
	pub borked_creature_data: [u8; 0x1168]
}

impl Default for PlayerInitialization {
	fn default() -> Self {
		Self {
			unknown: 0,
			assigned_id: Default::default(),
			borked_creature_data: [0u8; 0x1168]
		}
	}
}

impl CwSerializable for PlayerInitialization {}
impl Packet for PlayerInitialization {
	fn id() -> PacketId {
		PacketId::PlayerInitialization
	}
}
impl PacketFromServer for PlayerInitialization {}