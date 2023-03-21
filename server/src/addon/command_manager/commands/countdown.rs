use std::mem::transmute;
use std::str::SplitWhitespace;
use std::time::Duration;

use tokio::time::sleep;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Countdown;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Countdown {
	const LITERAL: &'static str = "countdown";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, _caller: Option<&'fut Player>, _params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let server_static: &'static Server = unsafe { transmute(server) };//todo: scoped task

		tokio::spawn(async move {
			let mut count = 3;

			loop {
				server_static.announce(char::from_digit(count, 10).unwrap()).await;
				sleep(Duration::from_secs(1)).await;

				count -= 1;
				if count == 0 { break };
			}
			server_static.announce("go!").await;
		});

		Ok(None)
	}
}