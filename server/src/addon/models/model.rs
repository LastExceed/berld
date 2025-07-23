use std::path;

use anyhow::anyhow;
use itertools::Itertools;
use protocol::rgb::RGB8;
use protocol::packet::world_update::{Block, WorldObject};
use protocol::nalgebra::Point3;
use protocol::packet::world_update::block::Kind::*;
use tap::Pipe;

mod vox;
mod zox;


pub struct Model {
    pub blocks: Vec<Block>,
    pub world_objects: Vec<WorldObject>
}

impl Model {
    pub fn try_parse(filename: &str) -> anyhow::Result<Self> {
        let path = path::Path::new(filename);

        Self {
            blocks:
                if path.extension().ok_or_else(||anyhow!("no file extension"))? == "vox" {
                    vox::parse(filename)?
                } else {
                    zox::parse(path)?
                }
                .into_iter()
                .map(Block::from)
                .collect_vec(),

            world_objects: vec![]
        }.pipe(Ok)
    }   
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct ParsedVoxel(Point3<i32>, RGB8);

// todo: how does this not violate the orphan rule?
impl From<ParsedVoxel> for Block {
    fn from(ParsedVoxel(position, color): ParsedVoxel) -> Self {
        Self {
            position,
            color,
            kind: if color == PURE_BLUE { Liquid } else { Solid },
            padding: 0,
        }
    }
}

const PURE_BLUE: RGB8 = RGB8::new(0, 0, 255);