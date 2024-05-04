use std::str::SplitWhitespace;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Tp;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Tp {
	const LITERAL: &'static str = "tp";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let destination = server
			.find_player(params.next().ok_or("no target specified")?).await
			.ok_or("target not found")?
			.character.read().await
			.position;

		server.teleport(caller, destination).await;

		Ok(None)
	}
}