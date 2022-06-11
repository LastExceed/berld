use nalgebra::{Point, Vector3};
use crate::packet::{CwSerializable, Packet, PacketFromClient, PacketId};

#[repr(C)]
pub struct Projectile {
	pub attacker: u64,
	pub chunk: Point<i32, 2>,
	pub unknown_a: i32,
	//pad4
	pub position: Point<i64, 3>,
	pub unknown_v: [i32; 3],
	pub velocity: Vector3<f32>,
	pub legacy_damage: f32,
	pub unknown_b: f32, //2-4 depending on mana for boomerangs, otherwise 0.5
	pub scale: f32,
	pub mana: f32,
	pub particles: f32,
	pub skill: u8,
	//pad3
	pub type_: ProjectileType,
	pub unknown_c: i32,
	pub unknown_d: f32,
	pub unknown_e: f32
}

impl CwSerializable for Projectile {}
impl Packet for Projectile {
	const ID: PacketId = PacketId::Projectile;
}
impl PacketFromClient for Projectile {}

#[repr(u32)]
pub enum ProjectileType {
	Arrow,
	Magic,
	Boomerang,
	Unknown,
	Boulder
}