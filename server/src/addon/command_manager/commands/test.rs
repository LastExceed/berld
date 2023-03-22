use std::str::SplitWhitespace;

use strum::IntoEnumIterator;

use protocol::packet::world_update::{Sound, sound};
use protocol::packet::WorldUpdate;
use protocol::utils::sound_position_of;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Test;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Test {
	const LITERAL: &'static str = "t";
	const ADMIN_ONLY: bool = true;

	async fn execute(&self, _server: &Server, caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let sound = Sound {
			position: sound_position_of(caller.character.read().await.position),
			kind: sound::Kind::iter()
				.nth(
					params
						.next()
						.ok_or("no param")?
						.parse()
						.map_err(|_| "parse failed")?
				)
				.ok_or("out of bounds")?,
			pitch: params
				.next()
				.map(|input| input.parse().unwrap())
				.unwrap_or(1.0),
			volume: 1.0,
		};
		caller.send_ignoring(&WorldUpdate::from(sound)).await;

		Ok(None)
	}
}