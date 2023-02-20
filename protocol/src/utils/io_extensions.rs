use std::mem::size_of;
use std::slice;

use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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