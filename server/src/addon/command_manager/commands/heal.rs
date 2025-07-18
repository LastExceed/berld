use std::str::SplitWhitespace;
use protocol::packet::{Hit, WorldUpdate};
use protocol::packet::hit::Kind::Normal;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Heal;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Heal {
	const LITERAL: &'static str = "heal";
	const ADMIN_ONLY: bool = true;

	async fn execute<'fut>(&'fut self, _server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;
		let character = caller.character.read().await;

		let amount: f32 = params
			.next()
			.map_or(
				Ok(9999.0),
				|str| str
					.parse()
					.map_err(|_| "invalid amount")
			)?;

		let heal = Hit {
			attacker: caller.id,
			target: caller.id,
			damage: -amount,
			position: character.position,
			kind: Normal,
			..Default::default()
		};
		drop(character);

		caller.send_ignoring(&WorldUpdate::from(heal)).await;

		Ok(None)
	}
}