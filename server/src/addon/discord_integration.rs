use std::fs;
use std::mem::transmute;

use twilight_gateway::Shard;
use twilight_http::Client;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::gateway::event::Event::MessageCreate;
use twilight_model::id::Id;

use protocol::packet::ChatMessageFromServer;
use protocol::packet::common::CreatureId;

use crate::server::Server;

pub struct DiscordIntegration {
	http: Client,
	token: String
}

impl DiscordIntegration {
	const FILE_PATH: &'static str = "discord_bot_token.txt";

	pub fn new() -> Self {
		let Ok(token) = fs::read_to_string(Self::FILE_PATH) else {
			fs::write(Self::FILE_PATH, "insert token here").unwrap();
			panic!("{} not found, created dummy file", Self::FILE_PATH);
		};

		Self {
			http: Client::new(token.clone()),
			token,
		}
	}

	pub fn run(&self, server: &Server) {
		let mut shard = Shard::new(
			ShardId::ONE,
			self.token.clone(),
			Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT
		);

		let server_static: &'static Server = unsafe { transmute(server) }; //todo: scoped task
		tokio::spawn(async move {
			loop {
				match shard.next_event().await {
					Ok(MessageCreate(message)) if message.channel_id == Id::new(1067011357129580667) && !message.author.bot => {
						let is_command = server_static.addons.command_manager.on_message(
							server_static,
							None,
							&message.content,
							'.',
							|response| async move { server_static.addons.discord_integration.post(&response).await }//todo: oof
						).await;

						if is_command {
							continue;
						}

						server_static.broadcast(&ChatMessageFromServer {//dont use server.announce() as that would cause an echo
							source: CreatureId(0),
							text: format!("<{}> {}", message.author.name, message.content)
						}, None).await;
					},

					Ok(_) => (),

					Err(error) => {
						println!("{:?}", error);//todo: proper error handling
						if error.is_fatal() {
							panic!("fatal error in event loop of discord integration");
						}
						continue;
					}
				};
			}
		});
	}

	pub async fn post(&self, message: &str) {
		self.http.create_message(Id::new(1067011357129580667))
			.content(message)
			.expect("setting content failed")
			.await
			.expect("create message failed");
	}
}