use std::ops::Div;
use std::collections::HashMap;

use config::{Config, ConfigError};
use dot_vox::SceneNode;
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use protocol::rgb::{RGB8, RGBA8};
use protocol::packet::world_update::Block;
use protocol::nalgebra::{Point2, Point3, Vector3};
use protocol::packet::world_update::block::Kind::*;
use tap::Pipe;

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
			.map(|(_zone, blocks)| blocks)
			.flatten()
			.cloned()
			.collect()
	}
}


const PURE_BLUE: RGB8 = RGB8::new(0, 0, 255);



pub fn parse_model(filename: &str) -> Vec<Block> {
    let vox = dot_vox::load(filename).expect("vox load failed");

	let mut model_offsets = HashMap::new();
	if vox.scenes.len() != 0 {
		walk_scene_graph(&vox.scenes, 0, Vector3::zeros(), &mut model_offsets);
	}

	vox
		.models
		.iter()
		.enumerate()
		.flat_map(|(model_id, model)| {
			model.voxels.iter().map(|voxel| {
				let color = RGBA8::from(<[u8; 4]>::from(vox.palette[voxel.i as usize])).rgb();

				Block {
					position: Point3::new(voxel.x, voxel.y, voxel.z).cast::<i32>() + model_offsets.get(&(model_id as _)).unwrap_or(&Vector3::zeros()),
					color,
					kind: if color == PURE_BLUE { Liquid } else { Solid },
					padding: 0,
				}
			})
			.collect::<Vec<_>>()
		})
		.collect()
}

fn walk_scene_graph(scene_graph: &Vec<SceneNode>, index: usize, mut current_offset: Vector3<i32>, model_offsets: &mut HashMap<u32, Vector3<i32>>) {
    match &scene_graph[index] {
        SceneNode::Transform { attributes: _, frames, child, layer_id: _ } => {
            current_offset += frames[0]
                .attributes
                .get("_t")
                .unwrap_or(&"0 0 0".into())
                .split(' ')
                .map(|s| s.parse().unwrap())
                .collect::<Vec<i32>>()
                .pipe(<[i32; 3]>::try_from)
                .unwrap()
                .pipe(Vector3::<i32>::from);

            walk_scene_graph(scene_graph, *child as usize, current_offset, model_offsets);
        },
        SceneNode::Group { attributes: _, children } => {
            for child in children {
                walk_scene_graph(scene_graph, *child as usize, current_offset, model_offsets);
            }
        },
        SceneNode::Shape { attributes: _, models } => {
            for model in models {
                model_offsets.insert(model.model_id, current_offset);
            }
        },
    }
}