use std::mem::size_of;
use std::slice;

use async_trait::async_trait;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[async_trait]
pub trait ReadExtension: AsyncRead + Unpin {
	async fn read_struct<T>(&mut self) -> io::Result<T>
		where [(); size_of::<T>()]:
	{
		let mut buffer = [0u8; size_of::<T>()];
		self.read_exact(&mut buffer).await?;

		//Ok(unsafe { transmute::<[u8; size_of::<T>()], T>(buffer)}) //compiler is not smart enough to recognize that matching sizes for input and output are guaranteed
		Ok(unsafe { (buffer.as_ptr().cast::<T>()).read() })
	}
}

#[async_trait]
pub trait WriteExtension: AsyncWrite + Unpin {
	async fn write_struct<T: Sync>(&mut self, data: &T) -> io::Result<()> {
		let data_as_bytes = unsafe { slice::from_raw_parts((data as *const T).cast::<u8>(), size_of::<T>()) };
		self.write_all(data_as_bytes).await
	}
}

impl<Readable: AsyncRead + Unpin> ReadExtension for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteExtension for Writable {}