use nalgebra::{Point2, Point3};
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ReadCwData, WriteCwData};
use crate::packet::Item;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

//todo: implementation is extremely similar to P48 and AirshipTraffic
impl<Readable: AsyncRead + Unpin> ReadCwData<(Point2<i32>, Vec<Drop>)> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<(Point2<i32>, Vec<Drop>)> {
		//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
		Ok((self.read_arbitrary().await?, ReadCwData::<Vec<Drop>>::read_cw_data(self).await?))//self.read_cw_struct().await?))
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<(Point2<i32>, Vec<Drop>)> for Writable {
	async fn write_cw_data(&mut self, zone_drops: &(Point2<i32>, Vec<Drop>)) -> io::Result<()> {
		self.write_arbitrary(&zone_drops.0).await?;
		self.write_cw_data(&zone_drops.1).await
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

impl<Readable: AsyncRead + Unpin> ReadCwData<Drop> for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Drop> for Writable {}