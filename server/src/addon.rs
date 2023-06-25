use std::intrinsics::transmute;
use std::time::Duration;

use futures::future::join_all;
use tokio::time::sleep;

use protocol::packet::{CreatureUpdate, IngameDatetime, WorldUpdate};
use protocol::packet::creature_update::CreatureFlag;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::MenuSelect;
use protocol::utils::sound_position_of;

use crate::addon::anti_cheat::AntiCheat;
use crate::addon::balancing::AirTimeTracker;
use crate::addon::command_manager::CommandManager;
use crate::addon::discord_integration::DiscordIntegration;
use crate::server::creature::Creature;
use crate::server::Server;

pub mod anti_cheat;
pub mod traffic_filter;
pub mod balancing;
pub mod discord_integration;
pub mod command_manager;

#[derive(Default)]
pub struct Addons {
	pub anti_cheat: AntiCheat,
	pub discord_integration: DiscordIntegration,
	pub air_time_tracker: AirTimeTracker,
	pub command_manager: CommandManager
}

pub fn enable_pvp(creature_update: &mut CreatureUpdate) {
	if let Some(ref mut flags) = creature_update.flags {
		flags.set(CreatureFlag::FriendlyFire, true);
	}
}

pub fn fix_cutoff_animations(creature_update: &mut CreatureUpdate, previous_state: &Creature) {
	if let Some(animation_time) = creature_update.animation_time && animation_time <= previous_state.animation_time {
		creature_update.animation_time = Some(0); //starts all animations from the beginning to prevent cut-off animations, at the cost of some minimal delay
	}
}

pub fn freeze_time(server: &Server) {
	let server_static: &'static Server = unsafe { transmute(server) }; //todo: scoped task
	tokio::spawn(async move {
		loop {
			server_static.broadcast(&IngameDatetime { time: 12 * 60 * 60 * 1000, day: 0 }, None).await;
			sleep(Duration::from_secs(6)).await;
		}
	});
}

impl Server {
	pub async fn play_chat_sound(&self) {
		let players = self.players.read().await;

		//cant use broadcast as sound position is different for each player
		join_all(players.iter().map(|player| async {
			player.send_ignoring(&WorldUpdate::from(Sound {
				position: sound_position_of(player.character.read().await.position),
				kind: MenuSelect,
				pitch: 2.0,
				volume: 0.5,
			})).await;
		})).await;
	}
}