use nalgebra::Point;
use crate::packet::CwSerializable;

#[repr(C)]
pub struct WorldObject {
	pub chunk: Point<i32, 2>,
	pub id: i32,
	pub unknown_a: i32,
	pub type_: WorldObjectType,
	//pad4
	pub position: Point<i64, 3>,
	pub orientation: i8,
	//pad3
	pub size: [f32; 3], //todo: type
	pub is_closed: bool,
	//pad3
	pub transform_time: i32,
	pub unknown_b: i32,
	//pad4
	pub interactor: i64
}

impl CwSerializable for WorldObject {}

#[repr(i32)]
pub enum WorldObjectType {
	TODO
}