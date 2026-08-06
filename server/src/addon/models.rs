use std::ops::Div as _;
use std::collections::HashMap;
use std::path;
use std::sync::{Arc, OnceLock};

use config::{Config, ConfigError};
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use protocol::rgb::RGB8;
use protocol::packet::WorldUpdate;
use protocol::packet::world_update::Block;
use protocol::nalgebra::{Point2, Vector3};
use protocol::packet::world_update::block::Kind::*;
use protocol::utils::io_extensions::WritePacket;

mod vox;
mod zox;

const BLOCKS_PER_ZONE: i32 = (SIZE_ZONE / SIZE_BLOCK) as i32;

pub struct Models {
	blocks_by_zone: HashMap<Point2<i32>, Vec<Block>>,
	packets_by_zone: OnceLock<HashMap<Point2<i32>, Arc<[u8]>>>
}

impl Models {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let mut blocks_by_zone: HashMap<Point2<i32>, Vec<Block>> = HashMap::new();

		for (filename, pos) in config.get::<HashMap<String, [i64; 3]>>("models")? {
			let pos: Vector3<i64> = pos.into();
			let model_origin = pos
				.div(SIZE_BLOCK)
				.cast::<i32>();

			for mut block in parse_model(&filename) {
				block.position += model_origin;
				blocks_by_zone
					.entry(block.position.xy().map(|scalar| scalar.div_euclid(BLOCKS_PER_ZONE)))
					.or_default()
					.push(block);
			}
		}

		Ok(Self {
			blocks_by_zone,
			packets_by_zone: OnceLock::new()
		})
	}

	pub async fn prepare(&self) {
		let mut packets_by_zone = HashMap::with_capacity(self.blocks_by_zone.len());

		for (zone, blocks) in &self.blocks_by_zone {
			let mut packet = vec![];
			packet
				.write_packet(&WorldUpdate::from(blocks.clone()))
				.await
				.expect("failed to serialize a world update in-memory");

			packets_by_zone.insert(*zone, packet.into());
		}

		assert!(
			self.packets_by_zone.set(packets_by_zone).is_ok(),
			"models prepared twice"
		);
	}

	pub fn packet_for(&self, requested_zone: Point2<i32>) -> Option<Arc<[u8]>> {
		self.packets_by_zone
			.get()?
			.get(&requested_zone)
			.map(Arc::clone)
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