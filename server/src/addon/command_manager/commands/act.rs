use std::str::SplitWhitespace;
use std::time::Duration;
use protocol::nalgebra::Point2;
use protocol::packet::WorldUpdate;
use protocol::packet::common::Hitbox;
use protocol::packet::world_update::world_object::Kind;
use protocol::packet::world_update::WorldObject;
use protocol::utils::constants::SIZE_ZONE;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Act;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Act {
	const LITERAL: &'static str = "act";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, _server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;
		let character_guard = caller.character.read().await;

		let action: &str = params
			.next()
			.ok_or("no amount specified")?;

		let kind = match action {
			"sit" => Kind::Bench,
			"sleep" => Kind::SleepingMat,
			_ => {return Err("Unknown action")}
		};
		let zone =  Point2::new(
			(character_guard.position[0] / SIZE_ZONE) as i32,
			(character_guard.position[1] / SIZE_ZONE) as i32
		);

		let world_object = WorldObject{
			zone,
			id: 0,
			unknown_a: 0,
			kind,
			position: character_guard.position,
			orientation: 0,
			size: Hitbox::default(),
			is_closed: false,
			transform_time: 0,
			unknown_b: 0,
			interactor: caller.id.0,
		};

		caller.send_ignoring(&WorldUpdate{
			world_objects: vec![world_object.clone()],
			..Default::default()
		}).await;

		tokio::time::sleep(Duration::from_millis(10)).await; // We need this or the packet is not affecting the player.

		caller.send_ignoring(&WorldUpdate{
			world_objects: vec![WorldObject {
				interactor: 0,
				..world_object
			}],
			..Default::default()
		}).await;

		Ok(None)
	}
}