#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Consumable {
	Cookie,
	LifePotion,
	CactusPotion,
	GinsengSoup,
	SnowBerryMash,
	MushroomSpit,
	BOMB,
	PineappleSlice,
	PumpkinMuffin
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Weapon {
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Resource {
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Candle {
	Red,
	Green
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quest {
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Special {
	HangGlider,
	Boat
}