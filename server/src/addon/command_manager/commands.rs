use std::collections::HashMap;

use protocol::nalgebra::Point3;

mod xp;
mod warp;
mod level;

pub struct Xp;
pub struct Level;

pub struct Warp {
	locations: HashMap<String, Point3<i64>>
}