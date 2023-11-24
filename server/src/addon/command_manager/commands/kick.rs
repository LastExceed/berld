use std::str::SplitWhitespace;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Kick;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Kick {
	const LITERAL: &'static str = "kick";
	const ADMIN_ONLY: bool = true;

	async fn execute<'fut>(&'fut self, server: &'fut Server, _caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
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