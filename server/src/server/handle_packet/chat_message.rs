use std::sync::atomic::Ordering::Relaxed;

use colour::{cyan, white_ln};

use protocol::packet::ChatMessageFromClient;
use protocol::packet::world_update::sound::Kind::*;
use crate::addon::command_manager::CommandResult;
use crate::addon::{play_sound_for_everyone, play_sound_at_player};

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<ChatMessageFromClient> for Server {
	async fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) {
		let source_name = source.character.read().await.name.clone();

		cyan!("{}: ", source_name);
		white_ln!("{}", packet.text);

		let callback = |command_result| { command_callback(command_result, source) };

		let is_command = self
			.addons
			.command_manager
			.on_message(
				self,
				Some(source), //TODO: use a custom enum so `admin` doesnt have to be passed here
				source.admin.load(Relaxed),
				&packet.text,
				'/',
				callback
			).await;
		if is_command { return; }

		//TODO: move to addon
		self.addons
			.discord_integration
			.post(
				&format!(
					"**{}:** {}",
					source_name,
					packet.text
				),
				false
			).await;

		self.broadcast(&packet.into_reverse(source.id), None).await;
		play_sound_for_everyone(self, MenuSelect, 2.0, 0.5).await;
	}
}

async fn command_callback(result: CommandResult, source: &Player) {
	match result {
		Ok(response_option) => {
			play_sound_at_player(source, CraftProc, 1.0, 1.0).await;

			let Some(response) = response_option
				else { return };

			source.notify(response).await; //send message
		}
		Err(error) => {
			play_sound_at_player(source, MenuSelect, 0.5, 1.0).await;
			source.notify(error).await; //send error
		}
	}
}