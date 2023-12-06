use nalgebra::Scalar;
use num_traits::Zero;

pub trait Area {
	type Coordinate: Scalar + Zero;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Zone;
impl Area for Zone {
	type Coordinate = i32;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Region;
impl Area for Region {
	type Coordinate = i32;
}