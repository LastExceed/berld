use strum_macros::EnumIter;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
pub enum Candle {
	#[default]
	Red,
	Green
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
pub enum Special {
	#[default]
	HangGlider,
	Boat
}