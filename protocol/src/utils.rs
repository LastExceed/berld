use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use array_init::array_init;
use nalgebra::Point3;
use strum::EnumCount;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWriteExt as _};

use crate::{ReadCwData, WriteCwData};
use crate::utils::constants::SIZE_BLOCK;
use crate::utils::io_extensions::{ReadArbitrary as _, WriteArbitrary as _};

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

///ideally this would be done with a `#[derive()]` macro instead,
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
		let count = self.read_u32_le().await?;
		let mut vec = Vec::with_capacity(count as usize);
		for _ in 0..count {
			vec.push(self.read_cw_data().await?); //todo: figure out how to do this functional style (probably create and collect an Iter)
		}
		Ok(vec)
	}
}

//todo: relax to iterable
impl<Element, Writable: WriteCwData<Element>> WriteCwData<Vec<Element>> for Writable {
	async fn write_cw_data(&mut self, elements: &Vec<Element>) -> io::Result<()> {
		self.write_i32_le(elements.len() as _).await?;
		for element in elements {
			self.write_cw_data(element).await?;
		}
		Ok(())
	}
}

impl<Key: Eq + Hash, Value, Readable: AsyncRead + Unpin + ReadCwData<Value>> ReadCwData<HashMap<Key, Value>> for Readable
	where
		[(); size_of::<Key>()]:,
		[(); size_of::<Value>()]:
{
	async fn read_cw_data(&mut self) -> io::Result<HashMap<Key, Value>> {
		let mut map = HashMap::new();
		let n_keys = self.read_u32_le().await?;
		for _ in 0..n_keys {
			let zone = self.read_arbitrary().await?;
			let ground_items = self.read_cw_data().await?;
			map.insert(zone, ground_items);
		}
		Ok(map)
	}
}

impl<Key, Value, Writable: WriteCwData<Value>> WriteCwData<HashMap<Key, Value>> for Writable {
	async fn write_cw_data(&mut self, map: &HashMap<Key, Value>) -> io::Result<()> {
		self.write_i32_le(map.len() as _).await?;
		for (key, value) in map {
			self.write_arbitrary(key).await?;
			self.write_cw_data(value).await?;
		}
		Ok(())
	}
}

pub trait ArrayWrapperIndex: EnumCount + Into<usize> {
	type Item: Default;
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArrayWrapper<Idx: ArrayWrapperIndex>([Idx::Item; Idx::COUNT])
	where [(); Idx::COUNT]:;

impl<Idx: ArrayWrapperIndex> ArrayWrapper<Idx>
	where [(); Idx::COUNT]:
{
	#[expect(clippy::iter_without_into_iter, reason = "TODO")]
	pub fn iter(&self) -> Iter<'_, Idx::Item> {
		self.0.iter()
	}
}

//deriving this would require `Idx` to impl `Default` as well
impl<Idx: ArrayWrapperIndex> Default for ArrayWrapper<Idx>
	where [(); Idx::COUNT]:
{
    fn default() -> Self {
        Self(array_init(|_| Default::default()))
    }
}

impl<Idx: ArrayWrapperIndex> Index<Idx> for ArrayWrapper<Idx>
	where [(); Idx::COUNT]:
{
	type Output = Idx::Item;

	fn index(&self, index: Idx) -> &Self::Output {
		&self.0[index.into()]
	}
}

impl<Idx: ArrayWrapperIndex> IndexMut<Idx> for ArrayWrapper<Idx>
	where [(); Idx::COUNT]:
{
	fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
		&mut self.0[index.into()]
	}
}

impl<Idx: ArrayWrapperIndex> From<[Idx::Item; Idx::COUNT]> for ArrayWrapper<Idx>
	where [(); Idx::COUNT]:
{
	fn from(value: [Idx::Item; Idx::COUNT]) -> Self {
		Self(value)
	}
}