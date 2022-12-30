use async_trait::async_trait;
use nalgebra::Point3;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::*;

#[async_trait]
impl CwSerializable for AirshipTraffic {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		Ok(Self { airships: Vec::read_from(readable).await? })
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		self.airships.write_to(writable).await
	}
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Airship {
	pub id: i64,
	pub unknown_a: i32,
	pub unknown_b: i32,
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub rotation: f32,
	pub station: Point3<i64>,
	pub path_rotation: f32,
	//pad4
	pub destination: Point3<i64>,
	pub state: State,
	// u8 ?
	// pad3
}

impl CwSerializable for Airship {}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State {
	Unknown //todo
}