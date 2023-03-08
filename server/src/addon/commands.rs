use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

type CommandFuture<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;

pub struct CommandManager {
	commands: HashMap<&'static str, Box<dyn ObjectSafeCommand>>
}

impl CommandManager {
	pub fn new() -> Self {
		Self {
			commands: HashMap::from([
				("xp", Box::new(Xp) as Box<dyn ObjectSafeCommand>)
			])
		}
	}

	pub async fn on_chat_message(&self, server: &Server, caller: &Player, text: &str) -> bool {
		if !text.starts_with('/') {
			return false;
		}

		text[1..]
			.split_whitespace()
			.next()
			.and_then(|frag| self.commands.get(frag))
			.map(async move |command| command.get_execution_future(server, caller).await);

		true
	}
}

pub trait Command {
	const ADMIN_ONLY: bool;

	fn execute<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> impl Future<Output=()> + Send + 'fut;//if you see an error here, ignore it -> https://github.com/intellij-rust/intellij-rust/issues/10216
}

//neither associated constants nor async functions are object safe, so we need a proxy for both
trait ObjectSafeCommand: Send + Sync {//todo: Sync bound is only because of discord spaghetti {
	fn get_admin_only(&self) -> bool;

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> CommandFuture<'fut>;
}

impl<T: Command + Send + Sync> ObjectSafeCommand for T {
	fn get_admin_only(&self) -> bool {
		T::ADMIN_ONLY
	}

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> CommandFuture<'fut> {
		Box::pin(self.execute(server, caller))
	}
}


struct Xp;

impl Command for Xp {
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, _server: &Server, caller: &Player) {
		give_xp(caller, 42).await;
	}
}