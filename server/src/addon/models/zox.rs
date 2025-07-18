use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use protocol::nalgebra::{Point3, Point4};
use protocol::rgb::{RGB8, RGBA8};
use serde::Deserialize;
use tap::{Conv, Pipe};

#[expect(clippy::big_endian_bytes, reason ="file format")]
pub fn parse(path: &Path) -> Vec<(Point3<i32>, RGB8)> {
    let zox = File::open(path)
        .unwrap()
        .pipe(BufReader::new)
        .pipe(serde_json::from_reader::<_, Zox>)
        .unwrap();
    zox
        .frame1
        .into_iter()
        .map(Point4::from)
        .map(|raw| {
            let pos = raw.xzy().cast();
            let color = raw.w
                .to_be_bytes()
                .conv::<RGBA8>()
                .rgb();
    
            (pos, color)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Zox {
    creator: String,
    width: u32,
    height: u32,
    depth: u32,
    version: u32,
    frames: u32,
    frame1: Vec<[u32; 4]>
}