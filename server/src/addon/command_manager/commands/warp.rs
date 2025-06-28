use std::str::SplitWhitespace;

use config::{ConfigError, Config};
use tap::Pipe as _;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Warp;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Warp {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let instance = Self {
			locations: config.get("warps")?
		};

		Ok(instance)
	}
}

impl Command for Warp {
	const LITERAL: &'static str = "warp";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let Some(destination) = params.next()
			else {
				return self
					.locations
					.keys()
					.map(String::as_str)
					.intersperse(", ")
					.collect::<String>()
					.pipe(|names| format!("---\navailable locations:\n{names}\n---"))
					.pipe(Some)
					.pipe(Ok)
			};

		let coordinates = self.locations
			.get(destination)
			.ok_or("unkown destination (type /warp for a list)")?;

		server.teleport(caller, *coordinates).await;

		Ok(None)
	}
}