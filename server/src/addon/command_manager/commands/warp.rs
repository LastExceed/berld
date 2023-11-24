use std::fs;
use std::io::ErrorKind::NotFound;
use std::str::SplitWhitespace;

use tap::Tap;

use protocol::nalgebra::Point3;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Warp;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Warp {
	const FILE_PATH: &'static str = "warps.csv";

	pub fn new() -> Self {//todo: error handling (not that important, we want panics here anyway)
		let file_content = match fs::read_to_string(Self::FILE_PATH) {
			Ok(content) => content,

			Err(error) if error.kind() == NotFound => {
				concat!("spawn;",0x8020800000,';',0x8020800000)
					.tap(|content| fs::write(Self::FILE_PATH, content).unwrap())
					.to_owned()
			}

			Err(error) => panic!("failed to load {} - {}", Self::FILE_PATH, error)
		};

		Self {
			locations: file_content.lines().map(|line| {
				let splits: [&str; 3] = line
					.split(';')
					.collect::<Vec<_>>()
					.try_into()
					.unwrap();

				(
					splits[0].to_owned(),
					Point3::new(
						splits[1].parse().unwrap(),
						splits[2].parse().unwrap(),
						0_i64
					)
				)
			}).collect()
		}
	}
}

impl Command for Warp {
	const LITERAL: &'static str = "warp";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		let location_name = params
			.next()
			.ok_or("no destination specified")?;

		let coordinates = self.locations
			.get(location_name)
			.ok_or("unkown destination")?;

		server.teleport(caller, *coordinates).await;

		Ok(None)
	}
}