use std::str::SplitWhitespace;
use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Tp;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Tp {
	const LITERAL: &'static str = "tp";
	const ADMIN_ONLY: bool = true;

	async fn execute(&self, server: &Server, caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
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