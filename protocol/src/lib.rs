#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(cstr_from_bytes_until_nul)]
#![allow(const_evaluatable_unchecked)]

pub use nalgebra;
pub use rgb;

pub mod packet;
pub mod utils;

pub const SIZE_BLOCK: i64 = 65536;
pub const SIZE_MAPBLOCK: i64 = SIZE_BLOCK * 8;
pub const SIZE_CHUNK: i64 = SIZE_BLOCK * 32;
pub const SIZE_ZONE: i64 = SIZE_CHUNK * 8;
pub const SIZE_REGION: i64 = SIZE_ZONE * 64;
pub const SIZE_WORLD: i64 = SIZE_REGION * 1024;
pub const SIZE_UNIVERSE: i64 = SIZE_WORLD * 256;
//const SIZE_MULTIVERSE: i64 = SIZE_UNIVERSE * 65536; //overflows; it's basically u64::MAX + 1