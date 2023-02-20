use nalgebra::{Point2, Point3};
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::CwSerializable;
use crate::packet::Item;
use crate::utils::io_extensions::{ReadStruct, WriteStruct};

//todo: implementation is extremely similar to P48
impl CwSerializable for (Point2<i32>, Vec<Drop>) {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		Ok((readable.read_struct::<Point2<i32>>().await?, Vec::read_from(readable).await?))
	}
	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(&self.0).await?;
		self.1.write_to(writable).await
	}
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Drop {
	pub item: Item,
	pub position: Point3<i64>,
	pub rotation: f32,
	pub scale: f32,
	pub unknown_a: u8,
	//pad3
	pub droptime: i32,
	pub unknown_b: i32,
	//pad4 //i32 according to cuwo
}

impl CwSerializable for Drop {}