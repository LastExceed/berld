use std::collections::HashMap;

use protocol::nalgebra::Point3;

mod xp;
mod warp;

pub struct Xp;

pub struct Warp {
	locations: HashMap<String, Point3<i64>>
}