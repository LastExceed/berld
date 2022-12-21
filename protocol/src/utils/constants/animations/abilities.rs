use crate::packet::creature_update::Animation;
use crate::packet::creature_update::Animation::*;

pub const WARRIOR: [Animation; 2] = [
	Smash,
	Cyclone
];

pub const RANGER: [Animation; 1] = [
	Kick
];

pub const FIRE_MAGE: [Animation; 2] = [
	Teleport,
	FireExplosionShort
];

pub const WATER_MAGE: [Animation; 2] = [
	Teleport,
	HealingStream
];

pub const ASSASSIN: [Animation; 2] = [
	Intercept,
	Stealth
];

pub const NINJA: [Animation; 3] = [
	Intercept,
	Stealth,
	Shuriken
];