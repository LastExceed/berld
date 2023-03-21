use std::str::SplitWhitespace;
use protocol::utils::constants::combat_classes::*;
use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Player as PlayerCommand;
use crate::server::player::Player;
use crate::server::Server;

impl Command for PlayerCommand {
	const LITERAL: &'static str = "player";
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, server: &Server, _caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let player = server
			.find_player(params.next().ok_or("no target specified")?).await
			.ok_or("target not found")?;
		let character = player.character.read().await;

		//todo: impl display?
		let class = match character.combat_class() {
			BERSERKER  => "berserker",
			GUARDIAN   => "guardian",
			SNIPER     => "sniper",
			SCOUT      => "scout",
			FIRE_MAGE  => "fire mage",
			WATER_MAGE => "water mage",
			ASSASSIN   => "assassin",
			NINJA      => "ninja",
			_          => "unknown"
		};

		let display = format!(
"---
name: {} (#{})
class: {} ({:?} -> {:?})
health: {}/{}
mana: {}/{} ({} charged)
---",
			character.name, player.id.0,
			class, character.occupation, character.specialization,
			character.health as i32, character.maximum_health() as i32,
			(character.mana * 100.0) as i32, 100, (character.mana_charge * 100.0) as i32
		);

		Ok(Some(display))
	}
}