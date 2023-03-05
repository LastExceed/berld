use crate::packet::creature_update::Animation;
use crate::packet::creature_update::Animation::*;

pub mod abilities;
pub mod m1;
pub mod m2;

pub const GENERAL: [Animation; 8] = [
	Idle,
	Drink,
	Eat,
	PetFoodPresent,
	Sit,
	Sleep,
	Riding,
	Sail
];