use std::collections::HashMap;

use dot_vox::SceneNode;
use protocol::rgb::{RGB8, RGBA8};
use protocol::nalgebra::{Point3, Vector3};
use tap::{Conv, Pipe};

pub fn parse(filename: &str) -> Vec<(Point3<i32>, RGB8)> {
    let vox = dot_vox::load(filename).expect("vox load failed");

	let mut model_offsets = HashMap::new();
	if !vox.scenes.is_empty() {
		walk_scene_graph(&vox.scenes, 0, Vector3::zeros(), &mut model_offsets);
	}

	vox
		.models
		.iter()
		.enumerate()
		.flat_map(|(model_id, model)| {
            let model_offsets_ref = &model_offsets;
            let palette_ref = &vox.palette;
            
			model
                .voxels
                .iter()
                .map(move |voxel| {
                    let pos = Point3
                        ::new(voxel.x, voxel.y, voxel.z)
                        .cast::<i32>()
                        + model_offsets_ref
                            .get(&(model_id as _))
                            .copied()
                            .unwrap_or_else(Vector3::zeros);
                    
                    let color = palette_ref
                        [voxel.i as usize]
                        .conv::<[u8; 4]>()
                        .conv::<RGBA8>()
                        .rgb();
                    
                    (pos, color)
                })
		})
		.collect()
}

fn walk_scene_graph(scene_graph: &Vec<SceneNode>, index: usize, mut current_offset: Vector3<i32>, model_offsets: &mut HashMap<u32, Vector3<i32>>) {
    match &scene_graph[index] {
		#[expect(clippy::or_fun_call, reason = "false positive")]
        SceneNode::Transform { frames, child, .. } => {
            current_offset += frames[0]
                .attributes
                .get("_t")
                .unwrap_or(&"0 0 0".into())
                .split(' ')
				.map(str::parse)
				.map(Result::unwrap)
                .collect::<Vec<i32>>()
                .pipe(<[i32; 3]>::try_from)
                .unwrap()
                .conv::<Vector3::<i32>>();

            walk_scene_graph(scene_graph, *child as usize, current_offset, model_offsets);
        },
        SceneNode::Group { children, .. } => {
            for child in children {
                walk_scene_graph(scene_graph, *child as usize, current_offset, model_offsets);
            }
        },
        SceneNode::Shape { models, .. } => {
            for model in models {
                model_offsets.insert(model.model_id, current_offset);
            }
        },
    }
}