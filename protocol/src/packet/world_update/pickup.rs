use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ReadCwData, WriteCwData};
use crate::packet::common::Item;
use crate::packet::world_update::Pickup;
use crate::utils::io_extensions::{ReadArbitrary as _, WriteArbitrary as _};

//custom read/write impl is necessary solely because of formula weirdness :(
impl<Readable: AsyncRead + Unpin> ReadCwData<Pickup> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<Pickup> {
		let pickup = Pickup {
			interactor: self.read_arbitrary().await?,
			//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			item: <Readable as ReadCwData<Item>>::read_cw_data(self).await?,
		};

		Ok(pickup)
	}
}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Pickup> for Writable {
	async fn write_cw_data(&mut self, pickup: &Pickup) -> io::Result<()> {
		self.write_arbitrary(&pickup.interactor).await?;
		self.write_cw_data(&pickup.item).await
	}
}