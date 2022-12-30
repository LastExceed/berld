use std::mem::size_of;

use async_trait::async_trait;
use nalgebra::Point3;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{CwSerializable, SIZE_BLOCK};
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

pub mod io_extensions;
pub mod flagset;
pub mod constants;

pub fn sound_position_of(position: Point3<i64>) -> Point3<f32> { //todo: move to SoundEffect ?
	position.map(|scalar| scalar as f32 / SIZE_BLOCK as f32)
}

///ideally this would be done with a #[derive()] macro instead,
///but the boilerplate required for that is completely overkill for this use case
#[macro_export]
macro_rules! bulk_impl {
	($trait:ident for $($struct:ty),*) => { //todo: investigate if 'trait' can be restricted to :ty
		$(impl $trait for $struct {})*
	}
}

#[async_trait]
impl<Element: CwSerializable + Send + Sync> CwSerializable for Vec<Element>
	where [(); size_of::<Element>()]:
{
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		let count = readable.read_struct::<i32>().await?;
		let mut vec = Vec::with_capacity(count as usize);
		for _ in 0..count {
			vec.push(Element::read_from(readable).await?); //todo: figure out how to do this functional style (probably create and collect an Iter)
		}
		Ok(vec)
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		writable.write_struct(&(self.len() as i32)).await?;
		for element in self {
			element.write_to(writable).await?;
		}
		Ok(())
	}
}