use nalgebra::Point;
use crate::packet::CwSerializable;

#[repr(C)]
pub struct SoundEffect {
	pub position: Point<f32, 3>,
	pub sound: Sound,
	pub pitch: f32,
	pub volume: f32
}

impl CwSerializable for SoundEffect {}

#[repr(i32)]
pub enum Sound {
	TODO
}