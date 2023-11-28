use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{ReadCwData, WriteCwData};
use crate::packet::common::Item;
use crate::packet::CreatureAction;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Bomb = 1,
	Talk,
	ObjectInteraction,

	PickUp = 5,
	Drop,

	CallPet = 8
}

//custom read/write impl is necessary solely because of formula weirdness :(
impl<Readable: AsyncRead + Unpin> ReadCwData<CreatureAction> for Readable {
	async fn read_cw_data(&mut self) -> std::io::Result<CreatureAction> {
		let creature_action = CreatureAction {
			item: <Readable as ReadCwData<Item>>::read_cw_data(self).await?,//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			zone: self.read_arbitrary().await?,
			item_index: self.read_i32_le().await?,
			unknown_a: self.read_i32_le().await?,
			kind: self.read_arbitrary().await?,
		};
		self.read_exact(&mut [0u8; 3]).await?;

		Ok(creature_action)
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<CreatureAction> for Writable {
	async fn write_cw_data(&mut self, creature_action: &CreatureAction) -> std::io::Result<()> {
		self.write_cw_data(&creature_action.item).await?;
		self.write_arbitrary(&creature_action.zone).await?;
		self.write_i32_le(creature_action.item_index).await?;
		self.write_i32_le(creature_action.unknown_a).await?;
		self.write_u8(creature_action.kind as u8).await?;
		self.write_all(&[0u8; 3]).await
	}
}