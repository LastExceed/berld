use std::fs;
use std::mem::transmute;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use protocol::packet::ChatMessageFromServer;
use protocol::packet::common::CreatureId;

use crate::server::Server;

//todo: this entire file is horrible, partly because serenity is
//it was apparently created with the assumption that every discord bot exclusively reacts to discord events

pub struct DiscordIntegration {
	tx: Sender<String>,
	rx: std::sync::Mutex<Option<Receiver<String>>>//yikes
}

impl DiscordIntegration {
	const FILE_PATH: &'static str = "discord_bot_token.txt";

	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel(4);

		Self { tx, rx: std::sync::Mutex::new(Some(rx)) }
	}

	pub async fn run(&self, server: &Server) {
		let Ok(token) = fs::read_to_string(Self::FILE_PATH) else {
			fs::write(Self::FILE_PATH, "insert token here").unwrap();
			panic!("{} not found, created dummy file", Self::FILE_PATH);
		};
		let rx = self.rx.lock().unwrap().take().unwrap();

		let server_static: &'static Server = unsafe { transmute(server) }; //todo: scoped task
		tokio::spawn(async move {
			Client::builder(&token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
				.event_handler(Handler { rx: RwLock::new(rx), server: server_static })
				.await//why is this not a function called .build()? and why is it async???
				.expect("failed to create discord client")
				.start().await
				.expect("failed to start discord client")
		});
	}

	pub async fn post(&self, message: impl Into<String>) {
		self.tx.send(message.into()).await.unwrap()
	}
}

struct Handler {
	pub rx: RwLock<Receiver<String>>,
	server: &'static Server
}

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, _: Context, message: Message) {
		if message.author.bot { return; }

		self.server.broadcast(&ChatMessageFromServer {//dont use server.announce() as that would cause an echo
			source: CreatureId(0),
			text: format!("<{}> {}", message.author.name, message.content)
		}, None).await;
	}

	async fn ready(&self, context: Context, _: Ready) {
		let mut rx = self.rx.write().await;
		let channel = ChannelId(1067011357129580667);

		loop {
			let text = rx.recv().await.unwrap();

			channel.say(context.http.clone(), text.clone()).await.expect("failed to send discord message");
		}
	}
}