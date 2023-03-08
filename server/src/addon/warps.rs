use std::collections::HashMap;
use std::str::SplitWhitespace;

use protocol::nalgebra::Point3;

use crate::addon::commands::{Command, CommandResult};
use crate::server::player::Player;
use crate::server::Server;

pub struct Warpgate {
	locations: HashMap<&'static str, Point3<i64>>
}

impl Warpgate {
	pub fn new() -> Self {
		//todo: load locations from file
		Self {
			locations: HashMap::new()
		}
	}
}

impl Command for Warpgate {
	const LITERAL: &'static str = "warp";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let location_name =
			params
				.next()
				.ok_or("no destination specified")?;

		let coordinates =
			self.locations
				.get(location_name)
				.ok_or("unkown destination")?;

		server.teleport(caller, *coordinates).await;

		Ok(())
	}
}