use async_trait::async_trait;
use nalgebra::Point2;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::CwSerializable;
use crate::packet::world_update::P48;
use crate::utils::io_extensions::{ReadStruct, WriteStruct};

#[async_trait]
impl CwSerializable for P48 {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		Ok(Self {
			zone: readable.read_struct::<Point2<i32>>().await?,
			sub_packets: Vec::read_from(readable).await?
		})
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(&self.zone).await?;
		self.sub_packets.write_to(writable).await
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct P48sub(pub [u8; 16]);

impl CwSerializable for P48sub {}