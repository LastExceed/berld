use std::io::{Error, Read, Write};
use std::mem::size_of;
use nalgebra::{Point, Vector3};
use crate::utils::{FlagSet8, ReadExtension, WriteExtension};

pub mod creature_update;
pub mod multi_creature_update;
pub mod server_tick;
pub mod airship_traffic;
pub mod world_update;
pub mod ingame_datetime;
pub mod creature_action;
pub mod hit;
pub mod status_effect;
pub mod projectile;
pub mod chat_message;
pub mod chunk_discovery;
pub mod sector_discovery;
pub mod map_seed;
pub mod player_initialization;
pub mod protocol_version;
pub mod server_full;

pub trait CwSerializable: Sized {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error>
		where [(); size_of::<Self>()]:
	{
		reader.read_struct::<Self>()
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(self)
	}
}

pub trait Packet: CwSerializable {
	const ID: PacketId;

	fn write_to_with_id<T: Write>(&self, writer: &mut T) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(&Self::ID)?;
		self.write_to(writer)
	}
}

pub trait PacketFromServer: Packet {}
pub trait PacketFromClient: Packet {}

//todo: impl for Vec<T> possible?

#[derive(Eq, PartialEq, Debug)]
#[repr(i32)]
pub enum PacketId {
	CreatureUpdate,
	MultiEntityUpdate,
	ServerTick,
	AirshipTraffic,
	WorldUpdate,
	IngameDatetime,
	CreatureAction,
	Hit,
	StatusEffect,
	Projectile,
	ChatMessage,
	ChunkDiscovery,
	SectorDiscovery,
	//Unknown13,
	//Unknown14,
	MapSeed = 15,
	PlayerInitialization,
	ProtocolVersion,
	ServerFull
}

#[repr(C)]
pub struct Item {
	pub type_major: ItemTypeMajor,
	pub type_minor: u8,
	//pad 2
	pub seed: i32,
	pub recipe: ItemTypeMajor,
	//pad 1
	pub minus_modifier: i16,//todo: structure alignment entails this properties' existence, name adopted from cuwo
	pub rarity: Rarity,
	pub material: Material,
	pub flags: FlagSet8<ItemFlag>,
	//pad1
	pub level: i16,
	//pad2
	pub spirits: [Spirit; 32],
	pub spirit_counter: i32
}

#[repr(u8)]
pub enum ItemTypeMajor {
	None,
	Food,
	Formula,
	Weapon,
	Chest,
	Gloves,
	Boots,
	Shoulder,
	Amulet,
	Ring,
	Block,
	Resource,
	Coin,
	PlatinumCoin,
	Leftovers,
	Beak,
	Painting,
	Vase,
	Candle,
	Pet,
	PetFood,
	Quest,
	Unknown,
	Special,
	Lamp,
	ManaCube
}

#[repr(u8)]
pub enum Rarity {
	Normal,
	Uncommon,
	Rare,
	Epic,
	Legendary,
	Mythic
}

#[repr(i8)]
pub enum Material {
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

	Fire = -128,
	Unholy,
	IceSpirit,
	Wind,
}

#[repr(u8)]
pub enum ItemFlag {
	Adapted
}

impl From<ItemFlag> for u8 {
	fn from(it: ItemFlag) -> Self {
		it as Self
	}
}

#[repr(C, align(4))]
pub struct Spirit {
	pub position: Point<i8, 3>,
	pub material: Material,
	pub level: i16,
	//pad2 //todo: struct align suggests that this could be a property, maybe seed/rarity/flags of the spirit?
}