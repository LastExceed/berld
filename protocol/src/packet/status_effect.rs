use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};
use crate::packet::creature_update::CreatureId;

#[repr(C)]
pub struct StatusEffect {
	pub source: CreatureId,
	pub target: CreatureId,
	pub type_: StatusEffectType,
	//pad3
	pub modifier: f32,
	pub duration: i32,
	//pad4
	pub creature_id3: CreatureId
}

impl CwSerializable for StatusEffect {}
impl Packet for StatusEffect {
	fn id() -> PacketId {
		PacketId::StatusEffect
	}
}
impl PacketFromClient for StatusEffect {}

#[repr(u8)]
pub enum StatusEffectType {

	Bulwalk = 1,
	WarFrenzy,
	Camouflage,
	Poison,

	ManaShield = 6,


	FireSpark = 9,
	Intuition,
	Elusiveness,
	Swiftness
}