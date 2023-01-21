use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use async_trait::async_trait;
use nalgebra::Point3;
use strum::EnumCount;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::CwSerializable;
use crate::utils::constants::SIZE_BLOCK;
use crate::utils::io_extensions::{ReadStruct, WriteStruct};

pub mod io_extensions;
pub mod flagset;
pub mod constants;

fn level_scaling_factor(level: i32) -> f32 {
	1f32 / (0.05f32 * (level as f32 - 1f32) + 1f32)
}

pub fn power_of(level: i32) -> i32 {
	(101f32 - 100f32 * level_scaling_factor(level)) as i32
}

pub fn maximum_experience_of(level: i32) -> i32 {
	1050 - 1000 * level_scaling_factor(level) as i32
}

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

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayWrapper<Idx: Into<usize> + EnumCount, Element>([Element; Idx::COUNT], PhantomData<Idx>)
	where [(); Idx::COUNT]:;

impl<Idx: Into<usize> + EnumCount, Element> ArrayWrapper<Idx, Element>
	where [(); Idx::COUNT]:
{
	pub fn iter(&self) -> Iter<'_, Element> {
		self.0.iter()
	}
}

impl<Idx: Into<usize> + EnumCount, Element> Index<Idx> for ArrayWrapper<Idx, Element>
	where [(); Idx::COUNT]:
{
	type Output = Element;

	fn index(&self, index: Idx) -> &Self::Output {
		&self.0[index.into()]
	}
}

impl<Idx: Into<usize> + EnumCount, Element> IndexMut<Idx> for ArrayWrapper<Idx, Element>
	where [(); Idx::COUNT]:
{
	fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
		&mut self.0[index.into()]
	}
}