use nalgebra::{Point, Vector3};
use crate::packet::CwSerializable;

#[repr(C)]
pub struct Particle {
	pub position: Point<i64, 3>,
	pub velocity: Vector3<f32>,
	pub color: [f32; 3],//todo: type
	pub alpha: f32,
	pub size: f32,
	pub count: i32,
	pub type_: ParticleType,
	pub spread: f32,
	//pad4
}

impl CwSerializable for Particle {}

#[repr(i32)]
pub enum ParticleType {
	TODO
}