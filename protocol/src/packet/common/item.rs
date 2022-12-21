use nalgebra::Point3;

pub mod type_;

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub enum TypeMajor {
	#[default]
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

#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub struct TypeMinor(u8);

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy, Default)]
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
#[derive(Clone, PartialEq, Eq, Copy, Default)]
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
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum ItemFlag {
	Adapted
}

impl From<ItemFlag> for u8 {
	fn from(it: ItemFlag) -> Self {
		it as Self
	}
}

#[repr(C, align(4))]
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Spirit {
	pub position: Point3<i8>,
	pub material: Material,
	pub level: i16,
	//pad2 //todo: struct align suggests that this could be a property, maybe seed/rarity/flags of the spirit?
}

//todo: consider moving this to utils (currently here so inner value of TypeMinor can stay private)
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct ItemType {
	pub major: TypeMajor,
	pub minor: TypeMinor
}