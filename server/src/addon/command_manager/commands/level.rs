use std::str::SplitWhitespace;

use protocol::utils::maximum_experience_of;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Level;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

impl Command for Level {
	const LITERAL: &'static str = "level";
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, _server: &Server, caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let target_level: i32 = params
			.next()
			.ok_or("no amount specified")?
			.parse()
			.map_err(|_| "invalid amount specified")?;

		if target_level > 500 {//todo: const
			return Err("max level is 500");
		}

		let character = caller.character.read().await;
		if target_level <= character.level {
			return Err("cannot downlevel");
		}

		let xp = (character.level..target_level)
			.map(maximum_experience_of)
			.sum::<i32>()
			- character.experience;

		give_xp(caller, xp).await;

		Ok(None)
	}
}