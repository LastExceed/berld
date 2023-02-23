use std::mem::size_of;
use std::slice;

use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Packet, packet, ReadCwData, WriteCwData};

pub trait ReadStruct: AsyncRead + Unpin {
	async fn read_struct<T>(&mut self) -> io::Result<T>
		where [(); size_of::<T>()]:
	{
		let mut buffer = [0u8; size_of::<T>()];
		self.read_exact(&mut buffer).await?;

		//Ok(unsafe { transmute::<[u8; size_of::<T>()], T>(buffer)}) //compiler is not smart enough to recognize that matching sizes for input and output are guaranteed
		Ok(unsafe { (buffer.as_ptr().cast::<T>()).read() })
	}
}

pub trait WriteStruct: AsyncWrite + Unpin {
	async fn write_struct<T>(&mut self, data: &T) -> io::Result<()> {
		let data_as_bytes = unsafe { slice::from_raw_parts((data as *const T).cast::<u8>(), size_of::<T>()) };
		self.write_all(data_as_bytes).await
	}
}

impl<Readable: AsyncRead + Unpin> ReadStruct for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteStruct for Writable {}


pub trait ReadPacket: AsyncRead + Unpin + Sized {//todo: move to io extensions
	async fn read_packet<P: Packet>(&mut self) -> io::Result<P>
		where
			[(); size_of::<P>()]:,
			Self: ReadCwData<P>,
	{
		ReadCwData::<P>::read_cw_data(self).await
	}

	async fn read_id(&mut self) -> io::Result<packet::Id> {
		self.read_struct().await
	}
}

pub trait WritePacket<P: Packet>: WriteCwData<P> {//todo: move to io extensions
	async fn write_packet(&mut self, packet: &P) -> io::Result<()> {
		self.write_struct(&P::ID).await?;
		self.write_cw_data(packet).await?;
		self.flush().await
	}
}

impl<Readable: AsyncRead + Unpin> ReadPacket for Readable {}

impl<P: Packet, Writable: WriteCwData<P>> WritePacket<P> for Writable {}