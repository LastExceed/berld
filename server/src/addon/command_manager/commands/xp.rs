use std::str::SplitWhitespace;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Xp;
use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

impl Command for Xp {
	const LITERAL: &'static str = "xp";
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, _server: &Server, caller: &Player, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let amount: i32 = params
			.next()
			.ok_or("no amount specified")?
			.parse()
			.map_err(|_| "invalid amount specified")?;

		give_xp(caller, amount).await;

		Ok(())
	}
}