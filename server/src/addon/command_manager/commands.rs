use std::collections::HashMap;

use protocol::nalgebra::Point3;

mod xp;
mod warp;
mod level;
mod countdown;
mod who;
mod gear;

pub struct Who;

pub struct Xp;
pub struct Level;

pub struct Countdown;

pub struct Warp {
	locations: HashMap<String, Point3<i64>>
}

pub struct Gear;