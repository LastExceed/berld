use strum_macros::*;

use crate::packet::common::item::Kind;

impl Kind {
	#[must_use]
	pub const fn is_stackable(&self) -> bool {
		matches!(self,
			Self::Consumable(_) | Self::Resource(_) | Self::Coin | Self::PlatinumCoin | Self::Quest(_)
		)
	}

	#[must_use]
	pub const fn uses_rarity(&self) -> bool {
		matches!(self,
			Self::Weapon(_) | Self::Chest | Self::Gloves | Self::Boots | Self::Shoulder | Self::Amulet |
			Self::Ring | Self::Resource(Resource::Spirit) | Self::Leftovers | Self::Lamp
		)
	}

	#[must_use]
	pub const fn uses_level(&self) -> bool {
		matches!(self,
			Self::Consumable(_) | Self::Weapon(_) | Self::Chest | Self::Gloves | Self::Boots |
			Self::Shoulder | Self::Amulet | Self::Ring | Self::Resource(Resource::Spirit) |
			Self::Leftovers | Self::Pet(_)
		)
	}

	// level is converted to amount on pickup for these items
	#[must_use]
	pub const fn uses_level_as_amount(&self) -> bool {
		!matches!(self, Self::Resource(Resource::Spirit)) && matches!(self,
			Self::Resource(_) | Self::Coin | Self::PlatinumCoin | Self::PetFood(_) | Self::Quest(_) |
			Self::Special(_) | Self::Lamp
		)
	}

	#[must_use]
	pub const fn uses_power(&self) -> bool {
		matches!(self,
			Self::Weapon(_) | Self::Chest | Self::Gloves | Self::Boots | Self::Shoulder | Self::Amulet |
			Self::Ring | Self::Resource(Resource::Spirit)
		)
	}

	#[must_use]
	pub const fn uses_stats(&self) -> bool {
		matches!(self,
			Self::Weapon(_) | Self::Chest | Self::Gloves | Self::Boots | Self::Shoulder | Self::Amulet | Self::Ring
		)
	}
}

#[repr(u8)]
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Consumable {
	#[default]
	Cookie,
	LifePotion,
	CactusPotion,
	ManaPotion,
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
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
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
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Candle {
	#[default]
	Red,
	Green
}

#[repr(u8)]
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
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
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Special {
	#[default]
	HangGlider,
	Boat
}