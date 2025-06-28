use nalgebra::Point3;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};

use crate::{ReadCwData, WriteCwData};
use crate::packet::Item;
use crate::utils::io_extensions::{ReadArbitrary as _, WriteArbitrary as _};

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct GroundItem {
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

//custom read/write impl is necessary solely because of formula weirdness :(
impl<Readable: AsyncRead + Unpin> ReadCwData<GroundItem> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<GroundItem> {
		let drop = GroundItem {
			//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			item: <Readable as ReadCwData<Item>>::read_cw_data(self).await?,
			position: self.read_arbitrary().await?,
			rotation: self.read_f32_le().await?,
			scale: self.read_f32_le().await?,
			unknown_a: {
				let unknown_a = self.read_u8().await?;
				self.read_exact(&mut [0_u8; 3]).await?; //pad3
				unknown_a
			},
			droptime: self.read_i32_le().await?,
			unknown_b: self.read_i32_le().await?,
		};
		self.read_exact(&mut [0_u8; 4]).await?; //pad4

		Ok(drop)
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<GroundItem> for Writable {
	async fn write_cw_data(&mut self, drop: &GroundItem) -> io::Result<()> {
		self.write_cw_data(&drop.item).await?;
		self.write_arbitrary(&drop.position).await?;
		self.write_f32_le(drop.rotation).await?;
		self.write_f32_le(drop.scale).await?;
		self.write_u8(drop.unknown_a).await?;
		self.write_all(&[0_u8; 3]).await?;
		self.write_i32_le(drop.droptime).await?;
		self.write_i32_le(drop.unknown_b).await?;
		self.write_all(&[0_u8; 4]).await
	}
}