use crate::packet::creature_update::Animation;
use crate::packet::creature_update::Animation::*;

pub const UNARMED: [Animation; 2] = [
	UnarmedM1a,
	UnarmedM1b
];

pub const DUALWIELD: [Animation; 2] = [
	DualWieldM1a,
	DualWieldM1b
];
//pub const SWORD  : [Animation; 2] = DUALWIELD;
//pub const AXE    : [Animation; 2] = DUALWIELD;
//pub const MACE   : [Animation; 2] = DUALWIELD;
//pub const SHIELD : [Animation; 2] = DUALWIELD;
//pub const ARROW  : [Animation; 2] = DUALWIELD;
//pub const QUIVER : [Animation; 2] = DUALWIELD;
//pub const PICKAXE: [Animation; 2] = DUALWIELD;
//pub const TORCH  : [Animation; 2] = DUALWIELD;

pub const DAGGER: [Animation; 2] = [
	DaggerM1a,
	DaggerM1b
];

//pub const FIST: [Animation; 2] = UNARMED;

pub const LONGSWORD: [Animation; 2] = [
	LongswordM1a,
	LongswordM1b
];

pub const BOW: [Animation; 1] = [
	ShootArrow
];

pub const CROSSBOW: [Animation; 1] = [
	ShootArrow
];

pub const BOOMERANG: [Animation; 1] = [
	BoomerangThrow
];

pub const STAFF_FIRE: [Animation; 1] = [
	StaffFireM1
];

pub const STAFF_WATER: [Animation; 1] = [
	StaffWaterM1
];

pub const WAND_FIRE: [Animation; 1] = [
	WandFireM1
];

pub const WAND_WATER: [Animation; 1] = [
	WandWaterM1
];

pub const BRACELET_FIRE: [Animation; 2] = [
	BraceletsFireM1a,
	BraceletsFireM1b
];

pub const BRACELET_WATER: [Animation; 2] = [
	BraceletsWaterM1a,
	BraceletsWaterM1b
];

pub const GREATWEAPON: [Animation; 3] = [
	GreatweaponM1a,
	GreatweaponM1b,
	GreatweaponM1c
];

pub const SHIELD: [Animation; 2] = [
	ShieldM1a,
	ShieldM1b
];