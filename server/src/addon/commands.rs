use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::SplitWhitespace;

use tap::Tap;

use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

type CommandFuture<'a> = Pin<Box<dyn Future<Output=CommandResult> + Send + 'a>>;
pub type CommandResult = Result<(), &'static str>;


pub struct CommandManager {
	commands: HashMap<&'static str, Box<dyn ObjectSafeCommand>>
}


impl CommandManager {
	pub fn new() -> Self {
		Self {
			commands: HashMap::new()
		}.tap_mut(|x|x.register(Xp))
	}

	pub fn register<C: Command + 'static>(&mut self, command: C) {//todo: can the lifetime be relaxed?
		self.commands.insert(C::LITERAL, Box::new(command));
	}

	pub async fn on_chat_message(&self, server: &Server, caller: &Player, text: &str) -> bool {
		if !text.starts_with('/') {
			return false;
		}

//		let mut fragments: Vec<_> = text[1..].split_whitespace()
//			.map(|fragment| fragment.to_string()).collect(); //todo: figure out how to pass fragments by refer
		let mut fragments = text[1..].split_whitespace();

		let result = fragments
			.next()
			.ok_or("no command specified")
			.and_then(|frag| {
				self.commands
					.get(frag)
					.ok_or("unknown command")
			});
//			.and_then(|command| command.get_execution_future(server, caller).await)
//			.map_err(async move |error| caller.notify(error).await)

		let result = match result {//unfortunately mapping functions dont support async/await, so we need to fallback to `match`
			Ok(command) => command.get_execution_future(server, caller, &mut fragments).await,
			Err(e) => Err(e)
		};

		if let Err(error) = result {
			caller.notify(error).await;
		}

		true
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