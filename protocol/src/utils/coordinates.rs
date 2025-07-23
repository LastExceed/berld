use nalgebra::Point;

struct Coords<S: Scale + 'static>(Point<S::Scalar, { S::DIMENSION }>)
    where [(); S::DIMENSION]:;
    
impl<S: Scale> Coords<S>
    where [(); S::DIMENSION]:
{
    
}

trait Scale {
    type Scalar: nalgebra::Scalar;
    const DIMENSION: usize;
    const MAGNITUDE: i64;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Unit;
impl Scale for Unit {
    type Scalar = i64;
    const DIMENSION: usize = 3;
    const MAGNITUDE: i64 = 1;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Block;
impl Scale for Block {
    type Scalar = i64;
    const DIMENSION: usize = 3;
    const MAGNITUDE: i64 = Unit::MAGNITUDE * 0x10000;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MapBlock;
impl Scale for MapBlock {
    type Scalar = i64;
    const DIMENSION: usize = 3;
    const MAGNITUDE: i64 = Block::MAGNITUDE * 8;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Chunk;
impl Scale for Chunk {
    type Scalar = i64;
    const DIMENSION: usize = 2;
    const MAGNITUDE: i64 = Block::MAGNITUDE * 32;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Zone;
impl Scale for Zone {
    type Scalar = i64;
    const DIMENSION: usize = 2;
    const MAGNITUDE: i64 = Chunk::MAGNITUDE * 8;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Region;
impl Scale for Region {
    type Scalar = i64;
    const DIMENSION: usize = 2;
    const MAGNITUDE: i64 = Zone::MAGNITUDE * 64;
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct World;
impl Scale for World {
    type Scalar = i64;
    const DIMENSION: usize = 2;
    const MAGNITUDE: i64 = Region::MAGNITUDE * 1024;
}