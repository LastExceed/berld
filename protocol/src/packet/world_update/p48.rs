use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ReadCwData, WriteCwData};
use crate::packet::world_update::P48;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

impl<Readable: AsyncRead + Unpin> ReadCwData<P48> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<P48> {
		Ok(P48 {
			zone: self.read_arbitrary().await?,
			//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			sub_packets: ReadCwData::<Vec<P48sub>>::read_cw_data(self).await?//self.read_cw_struct().await?
		})
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<P48> for Writable {
	async fn write_cw_data(&mut self, p48: &P48) -> io::Result<()> {
		self.write_arbitrary(&p48.zone).await?;
		self.write_cw_data(&p48.sub_packets).await
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct P48sub(pub [u8; 16]);

impl<Readable: AsyncRead + Unpin> ReadCwData<P48sub> for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<P48sub> for Writable {}