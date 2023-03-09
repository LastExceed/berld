use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind::NotFound;
use std::str::SplitWhitespace;
use std::string::ToString;

use tap::Tap;

use protocol::nalgebra::Point3;

use crate::addon::commands::{Command, CommandResult};
use crate::server::player::Player;
use crate::server::Server;

pub struct Warpgate {
	locations: HashMap<String, Point3<i64>>
}

impl Warpgate {
	const FILE_PATH: &'static str = "warps.csv";

	pub fn new() -> Self {//todo: error handling (not that important, we want panics here anyway)
		let file_content = match fs::read_to_string(Self::FILE_PATH) {
			Ok(content) => content,

			Err(error) if error.kind() == NotFound => {
				concat!("spawn;",0x8020800000,';',0x8020800000)
					.tap(|content| fs::write(Self::FILE_PATH, content).unwrap())
					.to_string()
			}

			Err(error) => panic!("failed to load {} - {}", Self::FILE_PATH, error)
		};
		let locations_iter = file_content.lines().map(|line| {
			let splits: [&str; 3] = line
				.split(';')
				.collect::<Vec<_>>()
				.try_into()
				.unwrap();

			(
				splits[0].to_string(),
				Point3::new(
					splits[1].parse().unwrap(),
					splits[2].parse().unwrap(),
					0i64
				)
			)
		});

		Self {
			locations: HashMap::from_iter(locations_iter)
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