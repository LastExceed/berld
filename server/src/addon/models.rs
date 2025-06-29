use std::ops::Div as _;
use std::collections::HashMap;
use std::path;

use config::{Config, ConfigError};
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use protocol::rgb::RGB8;
use protocol::packet::world_update::Block;
use protocol::nalgebra::{Point2, Vector3};
use protocol::packet::world_update::block::Kind::*;
use tap::Pipe as _;

mod vox;
mod zox;

pub struct Models {
	models: Vec<(Point2<i32>, Vec<Block>)>
}

impl Models {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		Self {
			models: config
				.get::<HashMap<String, [i64; 3]>>("models")?
				.into_iter()
				.map(|(filename, pos)| {
					let pos: Vector3<i64> = pos.into();
					let zone = pos
						.xy()
						.div(SIZE_ZONE)
						.cast::<i32>()
						.into();

					let mut blocks = parse_model(&filename);
					let model_origin = pos
						.div(SIZE_BLOCK)
						.cast::<i32>();

					for block in &mut blocks {
						block.position += model_origin;
					}

					(zone, blocks)
				})
				.collect()
		}.pipe(Ok)
	}

	pub fn blocks_in(&self, requested_zone: Point2<i32>) -> Vec<Block> {
		self.models
			.iter()
			.filter(|(zone, _blocks)| *zone == requested_zone)
			.flat_map(|(_zone, blocks)| blocks)
			.cloned()
			.collect()
	}
}

const PURE_BLUE: RGB8 = RGB8::new(0, 0, 255);

pub fn parse_model(filename: &str) -> Vec<Block> {
	let path = path::Path::new(filename);
	if path.extension().unwrap() == "vox" {
		vox::parse(filename)
	} else {
		zox::parse(path)
	}
		.into_iter()
		.map(|(position, color)| Block {
			position,
			color,
			kind: if color == PURE_BLUE { Liquid } else { Solid },
			padding: 0,
		})
		.collect()
}