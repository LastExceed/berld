use nalgebra::{Point, Vector3};
use crate::packet::creature_update::CreatureId;
use crate::packet::{CwSerializable, Packet, PacketId};

#[repr(C)]
pub struct Hit {
	pub attacker: CreatureId,
	pub target: CreatureId,
	pub damage: f32,
	pub critical: bool,
	//pad3
	pub stuntime: i32,
	//pad3
	pub position: Point<i64, 3>,
	pub direction: Vector3<f32>,
	pub is_yellow: bool,
	pub type_: HitType,
	pub flash: bool,
	//pad1
}

impl CwSerializable for Hit {}
impl Packet for Hit {
	fn id() -> PacketId {
		PacketId::Hit
	}
}

pub enum HitType {
	Default,
	Block,

	Miss = 3,
	Dodge,
	Absorb,
	Invisible
}