use std::mem::{size_of, transmute};

use async_trait::async_trait;
use nalgebra::Point3;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::CwSerializable;
use crate::packet::common::{Item, Race};
use crate::packet::common::item::kind::*;

pub mod kind;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Kind {
	#[default]
	Void,
	Consumable(Consumable),
	Formula,
	Weapon(Weapon),
	Chest,
	Gloves,
	Boots,
	Shoulder,
	Amulet,
	Ring,
	Block,
	Resource(Resource),
	Coin,
	PlatinumCoin,
	Leftovers,
	Beak,
	Painting,
	Vase,
	Candle(Candle),
	Pet(Race),
	PetFood(Race),
	Quest(Quest),
	Unknown,
	Special(Special),
	Lamp,
	ManaCube
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Rarity {
	#[default]
	Normal,
	Uncommon,
	Rare,
	Epic,
	Legendary,
	Mythic
}

#[repr(i8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Material {
	#[default]
	None,
	Iron,
	Wood,


	Obsidian = 5,
	Unknown,
	Bone,


	Copper = 10,
	Gold,
	Silver,
	Emerald,
	Sapphire,
	Ruby,
	Diamond,
	Sandstone,
	Saurian,
	Parrot,
	Mammoth,
	Plant,
	Ice,
	Licht,
	Glass,
	Silk,
	Linen,
	Cotton,

	Fire = i8::MIN,
	Unholy,
	IceSpirit,
	Wind,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemFlag {
	Adapted
}

impl From<ItemFlag> for u8 {
	fn from(it: ItemFlag) -> Self {
		it as Self
	}
}

#[repr(C, align(4))]
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Spirit {
	pub position: Point3<i8>,
	pub material: Material,
	pub level: i16,
	//pad2 //todo: struct align suggests that this could be a property, maybe seed/rarity/flags of the spirit?
}

#[async_trait]
impl CwSerializable for Item {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self>
		where [(); size_of::<Self>()]:
	{
		let mut buffer = [0u8; size_of::<Self>()];
		readable.read_exact(&mut buffer).await?;

		//for formulas and leftovers, the resulting item combines the major portion of [recipe] with the minor portion of [kind]
		//this makes type safe item kind handling impossible, as the minor portion of formulas and leftovers can be that of any item::Kind
		//to sidestep this problem, we copy over the minor portion from [kind] to [recipe]
		buffer[9] = buffer[1]; //todo: verify that the overwritten byte was 0
		//this unfortunately overwrites the minor portion of [recipe], which is actually persistent memory ingame
		//but since there is no known usecase it might just be the result of copy optimizations
		//if it ever turns out to be something after all we can still move it into an ephemeral padding

		Ok(unsafe { transmute(buffer) })
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		let mut buffer = unsafe { transmute::<_, [u8; size_of::<Self>()]>(self.clone()) };

		//see above
		if [2, 14].contains(&buffer[0]) { //todo: extract numbers from enum
			buffer[1] = buffer[9];
		}
		buffer[9] = 0;

		writable.write_all(&buffer).await
	}
}