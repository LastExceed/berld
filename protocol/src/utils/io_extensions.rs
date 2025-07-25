use std::{ptr, slice};

use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Packet, packet, ReadCwData, Validate, Validator, WriteCwData};

pub trait ReadArbitrary: AsyncRead + Unpin {
	async fn read_arbitrary<T>(&mut self) -> io::Result<T>
		where [(); size_of::<T>()]:
	{
		let mut buffer = [0_u8; size_of::<T>()];
		self.read_exact(&mut buffer).await?;

		//SAFETY: consumer is expected to validate the struct (terrible, i know)
		Ok(unsafe { buffer.as_ptr().cast::<T>().read() })
		//Ok(unsafe { transmute(buffer)}) //compiler is not smart enough to recognize that matching sizes for input and output are guaranteed
	}
}

pub trait WriteArbitrary: AsyncWrite + Unpin {
	async fn write_arbitrary<T>(&mut self, data: &T) -> io::Result<()> {
		//SAFETY: infallible
		let data_as_bytes = unsafe { //todo: there gotta be an easier way to do this
			slice::from_raw_parts(
				ptr::from_ref(data).cast::<u8>(),
				size_of::<T>()
			)
		};
		self.write_all(data_as_bytes).await
	}
}

impl<Readable: AsyncRead + Unpin> ReadArbitrary for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteArbitrary for Writable {}


pub trait ReadPacket: AsyncRead + Unpin + Sized {
	async fn read_packet<P: Packet>(&mut self) -> io::Result<P>
		where
			[(); size_of::<P>()]:,
			Self: ReadCwData<P>,
	{
		let instance = ReadCwData::<P>::read_cw_data(self).await?;
		Validator::validate(&instance)?;

		Ok(instance)
	}

	async fn read_id(&mut self) -> io::Result<packet::Id> {
		self.read_arbitrary().await
	}
}

pub trait WritePacket<P: Packet>: WriteCwData<P> {
	async fn write_packet(&mut self, packet: &P) -> io::Result<()> {
		self.write_arbitrary(&P::ID).await?;
		self.write_cw_data(packet).await?;
		self.flush().await
	}
}

impl<Readable: AsyncRead + Unpin> ReadPacket for Readable {}

impl<P: Packet, Writable: WriteCwData<P>> WritePacket<P> for Writable {}