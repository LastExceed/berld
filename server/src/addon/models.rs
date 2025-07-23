use std::collections::HashMap;

use config::Config;
use itertools::Itertools;
use protocol::packet::world_update::{Block, WorldObject};
use protocol::nalgebra::Point2;
use protocol::utils::constants::SIZE_BLOCK;
use tap::Pipe;

use self::model::Model;
use self::instance::Instance;

pub mod model;
mod instance;

pub struct Models {
	block_cache: HashMap<Point2<i32>, Vec<Block>>,
	object_cache: Vec<WorldObject>
}

impl Models {
	pub fn new(config: &Config) -> anyhow::Result<Self> {
		let instances: Vec<Instance> = config.get("models")?;
		let models: HashMap<&String, Model> = instances
			.iter()
			.map(|instance| &instance.filename)
			.unique()
			.map(|filename|
				Model::try_parse(filename)
					.map(|model| (filename, model))
			)
			.collect::<anyhow::Result<_>>()?;

		instances
			.iter()
			.map(|Instance { filename, position }| {
				let offset = position
					.map(|scalar| scalar / SIZE_BLOCK)
					.cast::<i32>()
					.coords;

				let Model {
					ref blocks,
					ref world_objects
				} = models[&filename];

    			let positioned_blocks = blocks
					.iter()
					.cloned()
					.update(move |block| block.position += offset);
				
				let positioned_objects = world_objects
					.iter()
					.cloned()
					.update(|world_object| world_object.position += position.coords);

				(positioned_blocks, positioned_objects)
			})
			.unzip::<_, _, Vec<_>, Vec<_>>() // avoiding this prolly requires going full imperative
			.pipe(|(blocks, objects)| Self {
				block_cache: blocks .into_iter().flatten().into_group_map_by(|block | block.position.xy() / 256),
				object_cache: objects.into_iter().flatten().collect(), // todo: assign zone and id
			})
			.pipe(Ok)
	}

	pub fn blocks_in(&self, requested_zone: Point2<i32>) -> Vec<Block> {
		self.block_cache
			[&requested_zone]
			.clone()
	}
}

