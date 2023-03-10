use std::str::SplitWhitespace;
use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Tp;
use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

impl Command for Tp {
	const LITERAL: &'static str = "tp";
	const ADMIN_ONLY: bool = true;

	async fn execute(&self, server: &Server, caller: &Player, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let destination = server
			.find_player(params.next().ok_or("no target specified")?).await
			.ok_or("target not found")?
			.character.read().await
			.position;

		server.teleport(caller, destination).await;

		Ok(())
	}
}