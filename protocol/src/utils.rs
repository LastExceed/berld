use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use nalgebra::Point3;
use strum::EnumCount;
use tokio::io;

use crate::{ReadCwData, WriteCwData};
use crate::utils::constants::SIZE_BLOCK;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

pub mod io_extensions;
pub mod flagset;
pub mod constants;

#[must_use]
fn something(level: f32) -> f32 {
	1.0 / (0.05 * (level - 1.0) + 1.0)
}

#[must_use]
pub fn level_scaling_factor(level: f32) -> f32 {
	2.0_f32.powf((1.0 - something(level)) * 3.0)
}

#[must_use]
pub fn rarity_scaling_factor(rarity: u8) -> f32 {
	2.0_f32.powf(rarity as f32 * 0.25)
}

#[must_use]
pub fn power_of(level: i32) -> i32 {
	(101.0 - 100.0 * something(level as f32)) as i32
}

#[must_use]
pub fn maximum_experience_of(level: i32) -> i32 {
	(1050.0 - 1000.0 * something(level as f32)) as i32
}

#[must_use]
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

impl<Element, Readable: ReadCwData<Element>> ReadCwData<Vec<Element>> for Readable
	where [(); size_of::<Element>()]:
{
	//todo: relax to iterable
	async fn read_cw_data(&mut self) -> io::Result<Vec<Element>>
		where [(); size_of::<Element>()]:
	{
		let count = self.read_arbitrary::<i32>().await?;
		let mut vec = Vec::with_capacity(count as usize);
		for _ in 0..count {
			vec.push(self.read_cw_data().await?); //todo: figure out how to do this functional style (probably create and collect an Iter)
		}
		Ok(vec)
	}
}

impl<Element, Writable: WriteCwData<Element>> WriteCwData<Vec<Element>> for Writable {//todo: relax to iterable
	async fn write_cw_data(&mut self, elements: &Vec<Element>) -> io::Result<()> {
		self.write_arbitrary(&(elements.len() as i32)).await?;
		for element in elements {
			self.write_cw_data(element).await?;
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

impl<Idx: Into<usize> + EnumCount, Element> From<[Element; Idx::COUNT]> for ArrayWrapper<Idx, Element>
	where [(); Idx::COUNT]:
{
	fn from(value: [Element; Idx::COUNT]) -> Self {
		Self(value, PhantomData)
	}
}