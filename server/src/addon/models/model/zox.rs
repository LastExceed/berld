use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use itertools::Itertools;
use protocol::nalgebra::Point4;
use protocol::rgb::RGBA8;
use serde::Deserialize;
use tap::{Conv, Pipe};

use super::ParsedVoxel;

#[expect(clippy::big_endian_bytes, reason ="file format")]
pub fn parse(path: &Path) -> anyhow::Result<Vec<ParsedVoxel>> {
    File::open(path)?
        .pipe(BufReader::new)
        .pipe(serde_json::from_reader::<_, Zox>)?
        .frame1
        .into_iter()
        .map(Point4::from)
        .map(|raw| {
            let pos = raw
                .xzy()
                .cast::<i32>();
            let color = raw
                .w
                .to_be_bytes()
                .conv::<RGBA8>()
                .rgb();
    
            ParsedVoxel(pos, color)
        })
        .collect_vec()
        .pipe(Ok)
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