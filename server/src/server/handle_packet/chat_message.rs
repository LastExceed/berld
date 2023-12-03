use std::sync::atomic::Ordering::Relaxed;

use colour::{cyan, white_ln};

use protocol::packet::ChatMessageFromClient;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<ChatMessageFromClient> for Server {
	async fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) {
		let source_name = source.character.read().await.name.clone();

		cyan!("{}: ", source_name);
		white_ln!("{}", packet.text);

		let is_command = self
			.addons
			.command_manager
			.on_message(
				self,
				Some(source),
				source.admin.load(Relaxed),
				&packet.text,
				'/',
				|message| { source.notify(message) }
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
		self.play_chat_sound().await;
	}
}