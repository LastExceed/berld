use nalgebra::Point;
use crate::packet::{CwSerializable, Item, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct CreatureAction {
	pub item: Item,
	pub chunk: Point<i32, 2>,
	pub item_index: i32,
	pub unknown_a: i32,
	pub type_: CreatureActionType
	//pad3
}

impl CwSerializable for CreatureAction {}
impl Packet for CreatureAction {
	const ID: PacketId = PacketId::CreatureAction;
}
impl PacketFromClient for CreatureAction {}

#[repr(u8)]
pub enum CreatureActionType {
	Bomb = 1,
	Talk,
	ObjectInteraction,

	PickUp = 5,
	Drop,

	CallPet = 8
}