use std::collections::HashMap;

use protocol::nalgebra::Point3;

mod xp;
mod warp;
mod level;
mod countdown;
mod who;
mod gear;
mod kick;
mod player;
mod tp;
mod test;

pub struct Who;
pub struct Player;

pub struct Xp;
pub struct Level;

pub struct Countdown;

pub struct Warp {
	locations: HashMap<String, Point3<i64>>
}

pub struct Gear;

pub struct Kick;

pub struct Tp;

pub struct Test;