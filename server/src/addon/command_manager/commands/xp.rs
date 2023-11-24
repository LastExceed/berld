use std::str::SplitWhitespace;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Xp;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

impl Command for Xp {
	const LITERAL: &'static str = "xp";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, _server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let amount: i32 = params
			.next()
			.ok_or("no amount specified")?
			.parse()
			.map_err(|_| "invalid amount specified")?;

		give_xp(caller, amount).await;

		Ok(None)
	}
}