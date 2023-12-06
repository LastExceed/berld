use std::str::SplitWhitespace;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Team;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::addon::pvp;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Team {
	const LITERAL: &'static str = "team";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let Some(param) = params.next()
			else {
				return Ok(Some(get_info_message(caller).await));
			};

		let team_id =
			if param.to_lowercase() == "leave" {
				None
			} else {
				Some(param.parse().map_err(|_| "invalid team id")?)
			};

		pvp::change_team(server, caller, team_id).await;

		Ok(None)
	}
}

async fn get_info_message(player: &Player) -> String {
	let current_team = player
		.addon_data
		.read()
		.await
		.team
		.map_or("None".into(), |id| id.to_string());

	format!("-------------
use /team [ID] to create & join a team
to leave your team, use /team leave
current team: {current_team}
-------------")
}