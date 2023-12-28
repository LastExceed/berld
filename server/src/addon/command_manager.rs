use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::io::ErrorKind::NotFound;
use std::pin::Pin;
use std::str::SplitWhitespace;
use std::sync::atomic::Ordering::Relaxed;

use boolinator::Boolinator;
use tap::Tap;

use crate::addon::command_manager::commands::*;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

mod commands;
mod utils;

type CommandFuture<'fut> = Pin<Box<dyn Future<Output=CommandResult> + Send + 'fut>>;
pub type CommandResult = Result<Option<String>, &'static str>;

pub struct CommandManager {
	commands: HashMap<&'static str, Box<dyn CommandProxy>>,
	admin_password: String
}

impl Default for CommandManager {
	fn default() -> Self {
		let admin_password = match fs::read_to_string(Self::FILE_PATH) {
			Ok(content) => content,

			Err(error) if error.kind() == NotFound => {
				"change-me"
					.tap(|content| fs::write(Self::FILE_PATH, content).unwrap())
					.to_owned()
			}

			Err(error) => panic!("failed to load {} - {}", Self::FILE_PATH, error)
		};

		Self {
			commands: HashMap::new(),
			admin_password
		}.tap_mut(|cm| {
			cm.register(Who);
			cm.register(WhoIp);
			cm.register(Player);
			cm.register(Xp);
			cm.register(Level);
			cm.register(Countdown);
			cm.register(Warp::new());
			cm.register(Gear);
			cm.register(Kick);
			cm.register(Tp);
			cm.register(Test);
			cm.register(Team);
			cm.register(Act);
			cm.register(Heal);
		})
	}
}

impl CommandManager {
	const FILE_PATH: &'static str = "admin_password.txt";

	pub fn register<C: Command + 'static>(&mut self, command: C) {//todo: can the lifetime be relaxed?
		self.commands.insert(C::LITERAL, Box::new(command));
	}

	pub async fn on_message<Fut: Future<Output=()>, Cb: FnOnce(CommandResult) -> Fut>(//todo: figure out lifetimes to optimize this to &str
		&self,
		server: &Server,
		caller: Option<&Player>,
		admin: bool,
		text: &str,
		command_prefix: char,
		callback: Cb
	) -> bool {
		let is_command = text.starts_with(command_prefix);

		if is_command {
			let command_result = self.handle_command(server, caller, admin, text).await;
			callback(command_result).await;
		}

		is_command
	}

	async fn handle_command(&self, server: &Server, caller: Option<&Player>, admin: bool, text: &str) -> CommandResult {
		let mut fragments = text.trim_start_matches('/').split_whitespace();

		let command_literal = fragments
			.next()
			.ok_or("no command specified (type /help for a list)")?;

		match command_literal {
			//implementing these as regular command structs would effectively require inserting a reference to the command map into itself
			"help" => self.on_help(admin),
			"login" => self.attempt_login(caller, &mut fragments),
			_ => {
				let command = self.commands
					.get(command_literal)
					.ok_or("unknown command (type /help for a list)")?;

				if command.get_admin_only() && !admin {
					return Err("no permission");
				}

				command.get_execution_future(server, caller, &mut fragments).await
			}
		}
	}

	fn on_help(&self, admin: bool) -> CommandResult {
		let mut message = String::new();
		message.push_str("help");

		let literals = self.commands
			.iter()
			.filter_map(|(literal, command)| {
				if command.get_admin_only() && !admin {
					return None;
				}

				Some(literal)
			});

		for command_literal in literals {//todo: there's probably a better way to do this
			message.push_str(", ");
			message.push_str(command_literal);
		}

		Ok(Some(message))
	}

	fn attempt_login(&self, caller: Option<&Player>, params: &mut SplitWhitespace<'_>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;

		params
			.next()
			.ok_or("no password specified")?
			.eq(&self.admin_password)
			.ok_or("wrong password")?;

		caller.admin.store(true, Relaxed);

		Ok(Some("login successful".to_owned()))
	}
}

pub trait Command: Send + Sync {//todo: move to commands.rs ?
	const LITERAL: &'static str;
	const ADMIN_ONLY: bool;

	fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> impl Future<Output=CommandResult> + Send + 'fut;//if you see an error here, ignore it -> https://github.com/intellij-rust/intellij-rust/issues/10216
}


//`Command` isn't object safe so we need a proxy
trait CommandProxy: Send + Sync {//todo: Sync bound is only because of discord spaghetti {
	fn get_admin_only(&self) -> bool;

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandFuture<'fut>;
}

impl<T: Command> CommandProxy for T {
	fn get_admin_only(&self) -> bool {
		T::ADMIN_ONLY
	}

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandFuture<'fut> {
		Box::pin(self.execute(server, caller, params))
	}
}