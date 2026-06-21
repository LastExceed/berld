use strum_macros::*;

use crate::packet::common::item::Kind;

impl Kind {
	pub const fn is_stackable(&self) -> bool {
		matches!(self,
			Kind::Consumable(_) | Kind::Resource(_) | Kind::Coin | Kind::PlatinumCoin | Kind::Quest(_)
		)
	}

	pub const fn uses_rarity(&self) -> bool {
		matches!(self,
			Kind::Weapon(_) | Kind::Chest | Kind::Gloves | Kind::Boots | Kind::Shoulder | Kind::Amulet |
			Kind::Ring | Kind::Resource(Resource::Spirit) | Kind::Leftovers | Kind::Lamp
		)
	}

	pub const fn uses_level(&self) -> bool {
		matches!(self,
			Kind::Consumable(_) | Kind::Weapon(_) | Kind::Chest | Kind::Gloves | Kind::Boots |
			Kind::Shoulder | Kind::Amulet | Kind::Ring | Kind::Resource(Resource::Spirit) |
			Kind::Coin | Kind::PlatinumCoin | Kind::Leftovers | Kind::Pet(_)
		)
	}

	pub const fn uses_seed(&self) -> bool {
		matches!(self,
			Kind::Weapon(_) | Kind::Chest | Kind::Gloves | Kind::Boots | Kind::Shoulder | Kind::Amulet |
			Kind::Ring | Kind::Painting | Kind::Vase | Kind::Candle(_) | Kind::Quest(_)
		)
	}
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Consumable {
	#[default]
	Cookie,
	LifePotion,
	CactusPotion,
	GinsengSoup,
	SnowBerryMash,
	MushroomSpit,
	Bomb,
	PineappleSlice,
	PumpkinMuffin
}

#[repr(u8)]
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Weapon {
	#[default]
	Sword,
	Axe,
	Mace,
	Dagger,
	Fist,
	Longsword,
	Bow,
	Crossbow,
	Boomerang,
	Arrow,
	Staff,
	Wand,
	Bracelet,
	Shield,
	Quiver,
	Greatsword,
	Greataxe,
	Greatmace,
	Pitchfork,
	Pickaxe,
	Torch
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Resource {
	#[default]
	Nugget,
	Log,
	Feather,
	Horn,
	Claw,
	Fiber,
	Cobweb,
	Hair,
	Crystal,
	Yarn,
	Cube,
	Capsule,
	Flask,
	Orb,
	Spirit,
	Mushroom,
	Pumpkin,
	Pineapple,
	Radishslice,
	Shimmermushroom,
	Ginsengroot,
	Onionslice,
	Heartflower,
	Pricklypear,
	Iceflower,
	Soulflower,
	Waterflask,
	Snowberry
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Candle {
	#[default]
	Red,
	Green
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Quest {
	#[default]
	AmuletYellow,
	AmuletBlue,
	JewelCase,
	Key,
	Medicine,
	Antivenom,
	Bandaid,
	Crutch,
	Bandage,
	Salve
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Special {
	#[default]
	HangGlider,
	Boat
}