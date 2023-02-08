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
	pub unknown_a: i32, //u8 + 3pad according to cuwo
	pub unknown_b: i32, //maybe padding
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub rotation: f32,
	pub station: Point3<i64>,
	pub path_rotation: f32,
	pub unknown_c: i32,//maybe padding
	pub destination: Point3<i64>,
	pub state: State,
	pub unknown_d: i32 //u8 + 3pad according to cuwo
}

impl CwSerializable for Airship {}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State {//from cuwo
	GoToStart,
	Landing,
	Takeoff,
	GoToDestination
}