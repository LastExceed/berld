use nalgebra::Point3;
use strum_macros::EnumIter;

use crate::packet::common::{CreatureId, Hitbox};

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Inner {
	pub kind: Kind,
	//pad4
	pub position: Point3<i64>,
	pub orientation: i8,//i32 according to cuwo
	//pad3
	pub size: Hitbox,
	pub is_closed: bool,
	//pad3
	pub transform_time: i32,
	pub unknown_b: i32,
	//pad4 //cuwo says 64bit padding??
	pub interactor: CreatureId
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Kind {
	Statue,
	Door,
	BigDoor,
	Window,
	CastleWindow,
	Gate,
	FireTrap,
	SpikeTrap,
	StompTrap,
	Lever,
	Chest,
	ChestTop02,
	Table1,
	Table2,
	Table3,
	Stool1,
	Stool2,
	Stool3,
	Bench,
	Bed,
	BedTable,
	MarketStand1,
	MarketStand2,
	MarketStand3,
	Barrel,
	Crate,
	OpenCrate,
	Sack,
	Shelter,
	Cupboard,
	Desktop,
	Counter,
	Shelf1,
	Shelf2,
	Shelf3,
	CastleShelf1,
	CastleShelf2,
	CastleShelf3,
	StoneShelf1,
	StoneShelf2,
	StoneShelf3,
	SandstoneShelf1,
	SandstoneShelf2,
	SandstoneShelf3,
	Corpse,
	RuneStone,
	Artifact,
	FlowerBox1,
	FlowerBox2,
	FlowerBox3,
	StreetLight,
	FireStreetLight,
	Fence1,
	Fence2,
	Fence3,
	Fence4,
	Vase1,
	Vase2,
	Vase3,
	Vase4,
	Vase5,
	Vase6,
	Vase7,
	Vase8,
	Vase9,
	Campfire,
	Tent,
	BeachUmbrella,
	BeachTowel,
	SleepingMat,
	Unknown70,
	Furnace,
	Anvil,
	SpinningWheel,
	Loom,
	SawBench,
	Workbench,
	CustomizationBench
}