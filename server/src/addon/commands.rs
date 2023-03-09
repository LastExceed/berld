use std::collections::{HashMap, HashSet};
use std::fs;
use std::future::Future;
use std::io::ErrorKind::NotFound;
use std::pin::Pin;
use std::str::SplitWhitespace;

use boolinator::Boolinator;
use tap::Tap;
use tokio::sync::RwLock;

use protocol::packet::ChatMessageFromClient;
use protocol::packet::common::CreatureId;

use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

type CommandFuture<'a> = Pin<Box<dyn Future<Output=CommandResult> + Send + 'a>>;
pub type CommandResult = Result<(), &'static str>;


pub struct CommandManager {
	commands: HashMap<&'static str, Box<dyn ObjectSafeCommand>>,
	admins: RwLock<HashSet<CreatureId>>,
	admin_password: String
}


impl CommandManager {
	const PREFIX: char = '/';
	const FILE_PATH: &'static str = "admin_password.txt";

	pub fn new() -> Self {
		let admin_password = match fs::read_to_string(Self::FILE_PATH) {
			Ok(content) => content,

			Err(error) if error.kind() == NotFound => {
				"change-me"
					.tap(|content| fs::write(Self::FILE_PATH, content).unwrap())
					.to_string()
			}

			Err(error) => panic!("failed to load {} - {}", Self::FILE_PATH, error)
		};

		Self {
			commands: HashMap::new(),
			admins: RwLock::new(HashSet::new()),
			admin_password
		}.tap_mut(|x|x.register(Xp))
	}

	pub fn register<C: Command + 'static>(&mut self, command: C) {//todo: can the lifetime be relaxed?
		self.commands.insert(C::LITERAL, Box::new(command));
	}

	pub async fn on_chat_message(&self, server: &Server, caller: &Player, packet: &ChatMessageFromClient) -> bool {
		let is_command = packet.text.starts_with(Self::PREFIX);

		if is_command {
			let result = self.handle_command(server, caller, &packet.text).await;

			if let Err(error) = result {
				caller.notify(error).await;
			}
		}

		is_command
	}

	pub async fn on_leave(&self, player: &Player) {
		self.admins
			.write()
			.await
			.remove(&player.id);
	}

	async fn handle_command(&self, server: &Server, caller: &Player, text: &str) -> CommandResult {
		let mut fragments = text[1..].split_whitespace();

		let command_literal = fragments
			.next()
			.ok_or("no command specified")?;

		match command_literal {
			//implementing these as regular command structs would effectively require inserting a reference to the command map into itself
			"help" => self.on_help(caller).await,
			"login" => self.attempt_login(caller, &mut fragments).await,
			_ => {
				let command = self.commands
					.get(command_literal)
					.ok_or("unknown command")?;

				if command.get_admin_only() && !self.admins.read().await.contains(&caller.id) {
					return Err("no permission");
				}

				command.get_execution_future(server, caller, &mut fragments).await
			}
		}
	}

	async fn on_help(&self, caller: &Player) -> CommandResult {
		let mut message = String::new();
		message.push(Self::PREFIX);
		message.push_str("help");

		let caller_is_admin = self.admins.read().await.contains(&caller.id);
		let literals = self.commands
			.iter()
			.filter_map(|(literal, command)| {
				if command.get_admin_only() && !caller_is_admin {
					return None;
				}

				Some(literal)
			});

		for command_literal in literals {
			message.push(' ');
			message.push(Self::PREFIX);
			message.push_str(command_literal);
		}

		caller.notify(message).await;

		Ok(())
	}

	async fn attempt_login(&self, caller: &Player, params: &mut SplitWhitespace<'_>) -> CommandResult {
		params
			.next()
			.ok_or("no password specified")?
			.eq(&self.admin_password)
			.ok_or("wrong password")?;

		self.admins.write().await.insert(caller.id);
		caller.notify("login successful").await;

		Ok(())
	}
}

pub trait Command: Send + Sync {
	const LITERAL: &'static str;
	const ADMIN_ONLY: bool;

	fn execute<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player, params: &'fut mut SplitWhitespace<'fut>) -> impl Future<Output=CommandResult> + Send + 'fut;//if you see an error here, ignore it -> https://github.com/intellij-rust/intellij-rust/issues/10216
}

//neither associated constants nor async functions are object safe, so we need a proxy for both
trait ObjectSafeCommand: Send + Sync {//todo: Sync bound is only because of discord spaghetti {
	fn get_admin_only(&self) -> bool;

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player, params: &'fut mut SplitWhitespace<'fut>) -> CommandFuture<'fut>;
}

impl<T: Command> ObjectSafeCommand for T {
	fn get_admin_only(&self) -> bool {
		T::ADMIN_ONLY
	}

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player, params: &'fut mut SplitWhitespace<'fut>) -> CommandFuture<'fut> {
		Box::pin(self.execute(server, caller, params))
	}
}


struct Xp;

impl Command for Xp {
	const LITERAL: &'static str = "xp";
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, _server: &Server, caller: &Player, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let amount: i32 = params
			.next()
			.ok_or("no amount specified")?
			.parse()
			.map_err(|_| "invalid amount specified")?;

		give_xp(caller, amount).await;

		Ok(())
	}
}