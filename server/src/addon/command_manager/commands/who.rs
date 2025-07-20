use std::str::SplitWhitespace;

use futures::future::join_all;
use tap::Pipe;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::{Who, WhoIp};
use crate::server::player::Player;
use crate::server::Server;

impl Command for Who {
	const LITERAL: &'static str = "who";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, _caller: Option<&'fut Player>, _params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		server
			.players
			.read()
			.await
			.iter()
			.map(async |player| format!(
				"#{} {}",
				player.id.0,
				&player.character.read().await.name
			))
			.pipe(join_all)
			.await
			.pipe(|names|
				if names.is_empty() {
					"(nobody here)".to_owned()
				} else {
					names.join(", ")
				}
			)
			.pipe(Some)
			.pipe(Ok)
	}
}

impl Command for WhoIp {
	const LITERAL: &'static str = "who_ip";
	const ADMIN_ONLY: bool = true;

	async fn execute<'fut>(&'fut self, server: &'fut Server, _caller: Option<&'fut Player>, _params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		server
			.players
			.read()
			.await
			.iter()
			.map(async |player| format!(
				"#{} {} {}",
				player.id.0,
				&player.character.read().await.name,
				player.address
			))
			.pipe(join_all)
			.await
			.pipe(|names|
				if names.is_empty() {
					"(nobody here)".to_owned()
				} else {
					names.join(", ")
				}
			)
			.pipe(Some)
			.pipe(Ok)
	}
}