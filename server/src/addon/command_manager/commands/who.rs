use std::str::SplitWhitespace;

use futures::future::join_all;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Who;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Who {
	const LITERAL: &'static str = "who";
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, server: &Server, caller: &Player, _params: &mut SplitWhitespace<'_>) -> CommandResult {
		let message = join_all(
			server.players
				.read().await
				.iter().map(|player| async {
					format!(
						"#{} {}",
						player.id.0,
						&player.character
							.read().await
							.name
					)
				})
		).await.join(", ");

		caller.notify(message).await;

		Ok(())
	}
}