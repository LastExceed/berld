use crate::packet::creature_update::Animation;
use crate::packet::creature_update::Animation::*;

pub const UNARMED: [Animation; 2] = [
	UnarmedM2Charging,
	UnarmedM2
];

//pub const DUALWIELD: [Animation; 2] = UNARMED;

//pub const SWORD  : [Animation; 2] = DUALWIELD;
//pub const AXE    : [Animation; 2] = DUALWIELD;
//pub const MACE   : [Animation; 2] = DUALWIELD;
//pub const SHIELD : [Animation; 2] = DUALWIELD;
//pub const ARROW  : [Animation; 2] = DUALWIELD;
//pub const QUIVER : [Animation; 2] = DUALWIELD;
//pub const PICKAXE: [Animation; 2] = DUALWIELD;
//pub const TORCH  : [Animation; 2] = DUALWIELD;

pub const DAGGER: [Animation; 1] = [
	DaggersM2
];

pub const FIST: [Animation; 1] = [
	FistsM2
];

pub const LONGSWORD: [Animation; 1] = [
	LongswordM2
];

pub const BOW: [Animation; 2] = [
	BowM2Charging,
	BowM2
];

pub const CROSSBOW: [Animation; 2] = [
	CrossbowM2Charging,
	CrossbowM2
];

pub const BOOMERANG: [Animation; 1] = [
	BoomerangM2Charging
];

pub const STAFF_FIRE: [Animation; 1] = [
	StaffFireM2
];

pub const STAFF_WATER: [Animation; 1] = [
	StaffWaterM2
];

pub const WAND_FIRE: [Animation; 1] = [
	WandFireM2
];

pub const WAND_WATER: [Animation; 1] = [
	WandWaterM2
];

pub const BRACELET_FIRE: [Animation; 1] = [
	BraceletFireM2
];

pub const BRACELET_WATER: [Animation; 1] = [
	BraceletWaterM2
];

pub const GREATWEAPON: [Animation; 3] = [
	GreatweaponM2Charging,
	GreatweaponM2Berserker,//TODO
	GreatweaponM2Guardian
];

pub const SHIELD: [Animation; 2] = [
	ShieldM2Charging,
	ShieldM2
];