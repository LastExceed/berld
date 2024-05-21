use std::ops::{Div, Mul};
use std::str::SplitWhitespace;

use protocol::{nalgebra::{Point3, Vector3}, packet::{status_effect, StatusEffect}};
use protocol::packet::WorldUpdate;
use protocol::packet::common::Hitbox;
use protocol::packet::world_update::{Block, WorldObject};
use protocol::packet::world_update::block::Kind::*;
use protocol::packet::world_update::world_object::Kind::{Crate, FireTrap};
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE};
use strum::IntoEnumIterator;
use protocol::rgb::{RGBA8, RGB8};
use protocol::packet::world_update::sound;

use crate::addon::{command_manager::{Command, CommandResult}, play_sound_at_player};
use crate::addon::command_manager::commands::Test;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Test {
	const LITERAL: &'static str = "t";
	const ADMIN_ONLY: bool = true;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;
		let character = caller.character.read().await;

		match params.next() {
			Some("check") => checkerboard(server, &character).await,
			Some("zg") => zone_grid(server, &character).await,
			Some("obj") => world_object(caller, &character).await,
			Some("objs") => objs(caller, &character).await,
			Some("block") => place_block(caller, &character).await,
			Some("ba") => place_blocks::<true>(caller, &character).await,
			Some("bs") => place_blocks::<false>(caller, &character).await,
			Some("s") => play_sound(caller, params).await,
			Some("model") => model(server, &character).await,
			Some("shield") => shield(caller).await,
			Some(_) => { return Err("unknown sub-command") }
			None => { return Err("too few arguments") },
		}

		Ok(None)
	}
}

async fn play_sound(caller: &Player, params: &mut SplitWhitespace<'_>) {
	let nth = params.next().unwrap().parse().unwrap();
	let pitch = params.next().map_or(1.0, |it| it.parse().unwrap());
	let kind = sound::Kind::iter().nth(nth).unwrap();
	play_sound_at_player(caller, kind, pitch, 1.0).await;

}

async fn place_block(caller: &Player, character: &Creature) {
	let block = Block {
		position: character.position.map(|scalar| (scalar / SIZE_BLOCK) as _),
		color: [0,0,0].into(),
		kind: Solid,
		padding: 0,
	};

	caller.send_ignoring(&WorldUpdate::from(block)).await;
}

async fn place_blocks<const B: bool>(caller: &Player, character: &Creature) {
	let pos = character.position.map(|scalar| (scalar / SIZE_BLOCK) as _);

	let mut blocks = vec![];
	for dx in 0..8 {
		for dy in 0..8 {
			for dz in 0..1000 {
				let block = Block {
					position: pos + Vector3::new(dx, dy, -dz),
					color: [0,0,0].into(),
					kind: if B { Air } else { Solid },
					padding: 0,
				};

				blocks.push(block);
			}
		}
	}

	caller.send_ignoring(&WorldUpdate::from(blocks)).await;
}

async fn world_object(caller: &Player, character: &Creature) {
	let object = WorldObject {
		zone: character.position.xy().map(|scalar| (scalar / SIZE_ZONE) as _),
		id: 0,
		position: character.position,
		orientation: 0,
		size: Hitbox {
			width: 5.0,
			height: 5.0,
			depth: 5.0
		},
		is_closed: false,
		transform_time: 0,
		unknown_b: 0,
		kind: Crate,

		unknown_a: 0,
		interactor: caller.id,
	};

	caller.send_ignoring(&WorldUpdate::from(object)).await;
}

pub async fn zone_grid(server: &Server, character: &Creature) {
	let start = character
		.position
		.div(SIZE_ZONE)
		.sub(Vector3::new(3,3,0))
		.mul(SIZE_ZONE)
		.div(SIZE_BLOCK)
		.cast::<i32>();

		let blocks: Vec<Block> =
			(0..7).flat_map(|zone_x| {
				(0..7).flat_map(move |zone_y| {
					(0..128).flat_map(move |block_d| {
						[
							[block_d, 0, 0],
							[block_d, 127, 0],
							[block_d, block_d, 0],
							[block_d, 127 - block_d, 0],
							[0, block_d, 0],
							[127, block_d, 0]
						]
							.map(Vector3::from)
							.map(|block_offset| Block {
								position: start + Vector3::new(zone_x, zone_y, 1) * 256 + block_offset * 2,
								color: [0,0,0].into(),
								kind: Kind::Solid,
								padding: 0,
							})
					})
				})
			})
			.collect();

		server.broadcast(&WorldUpdate::from(blocks), None).await;
}

pub async fn checkerboard(server: &Server, character: &Creature) {
	let start = Point3::new(
		character.position.x
			.div(SIZE_ZONE)
			.mul(SIZE_ZONE)
			.div(SIZE_BLOCK) as i32,
		character.position.y
			.div(SIZE_ZONE)
			.mul(SIZE_ZONE)
			.div(SIZE_BLOCK) as i32,
		character.position.z.div(SIZE_BLOCK) as i32,
	);

	let mut blocks = Vec::with_capacity(100);

	for dx in 0..300 {
		for dy in 0..300 {
			let block_alt = (dx + dy) % 2 == 1;
			let mapblock_alt = ((dx / 8) + (dy / 8)) % 2 == 1;
			let chunk_alt = ((dx / 32) + (dy / 32)) % 2 == 1;
			let zone_alt = ((dx / 256) + (dy / 256)) % 2 == 1;

			#[expect(clippy::collapsible_else_if, reason = "TODO")]
			let color =
				if zone_alt {
					if chunk_alt {
						if mapblock_alt {
							if block_alt { [0, 192, 192] } else { [0, 255, 255] }
						} else {
							if block_alt { [0, 192, 0] } else { [0, 255, 0] }
						}
					} else {
						if mapblock_alt {
							if block_alt { [0, 96, 192] } else { [0, 128, 255] }
						} else {
							if block_alt { [0, 0, 128] } else { [0, 0, 255] }
						}
					}
				} else {
					if chunk_alt {
						if mapblock_alt {
							if block_alt { [192, 0, 96] } else { [255, 0, 128] }
						} else {
							if block_alt { [96, 0, 96] } else { [255, 0, 255] }
						}
					} else {
						if mapblock_alt {
							if block_alt { [192, 96, 0] } else { [255, 128, 0] }
						} else {
							if block_alt { [128, 0, 0] } else { [255, 0, 0] }
						}
					}
				}.into();

			let block = Block {
				position: start + Vector3::new(dx, dy, 0),
				kind: Solid,
				color,
				padding: 0
			};

			blocks.push(block);
		}
	}

	server.broadcast(&WorldUpdate::from(blocks), None).await;
}

async fn objs(caller: &Player, character: &Creature) {
	let world_objects: Vec<_> = (0_i64..100)
		.map(|i| WorldObject {
			zone: character.position.xy().map(|scalar| (scalar / SIZE_ZONE) as _),
			id: i as _,
			unknown_a: i as _,
			kind: FireTrap,
			position: character.position + Vector3::new((i % 10) * 4 * SIZE_BLOCK, (i / 10) * 4 * SIZE_BLOCK, -SIZE_BLOCK),
			orientation: i as _,
			size: Hitbox {
				width: 2.0,
				depth: 2.0,
				height: 2.0,
			},
			is_closed: true,
			transform_time: 1,
			unknown_b: i as _,
			interactor: caller.id,
		})
		.collect();

	caller.send_ignoring(&WorldUpdate::from(world_objects)).await;
}

//let player_block_position = character.position.map(|scalar| (scalar / SIZE_BLOCK) as i32) - Point3::default();
async fn model(server: &Server, character: &Creature) {//fulcnix/FD_A_2B_minifed
	let vox = dot_vox::load("arena2.vox").expect("vox load failed");

	let mut blocks: Vec<_> = vox.models.iter().flat_map(|model| {
		model.voxels.iter().map(|voxel| {
			let color = RGBA8::from(<[u8; 4]>::from(vox.palette[voxel.i as usize])).rgb();

			Block {
				position: Point3::new(voxel.x, voxel.y, voxel.z).cast(),
				color,
				kind: if color == PURE_BLUE { Liquid } else { Solid },
				padding: 0,
			}
		})
	}).collect();

	for block in &mut blocks {
		block.position += (character.position / SIZE_BLOCK).cast().coords;
	}

	server.broadcast(&WorldUpdate::from(blocks), None).await;
}

async fn shield(caller: &Player) {
	let se = StatusEffect {
		source: caller.id,
		target: caller.id,
		kind: status_effect::Kind::ManaShield,
		modifier: 999.0,
		duration: 1,
		creature_id3: caller.id,
	};

	caller.send_ignoring(&WorldUpdate::from(se)).await;
}

const PURE_BLUE: RGB8 = RGB8::new(0, 0, 255);