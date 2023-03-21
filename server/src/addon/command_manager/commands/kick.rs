use std::str::SplitWhitespace;
use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Kick;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Kick {
	const LITERAL: &'static str = "kick";
	const ADMIN_ONLY: bool = true;

	async fn execute(&self, server: &Server, _caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let target_query = params.next().ok_or("no target specified")?;
		let target = server
			.find_player(target_query).await
			.ok_or("target not found")?;

		let reason = params
			.collect::<Vec<_>>()
			.join(" ");

		server.kick(&target, reason).await;

		Ok(None)
	}
}