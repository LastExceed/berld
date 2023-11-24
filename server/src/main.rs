#![expect(incomplete_features, reason = "generic_const_exprs is incomplete, but works for our purposes")]
#![feature(generic_const_exprs)]
#![feature(async_closure)]
#![feature(future_join)]
#![feature(let_chains)]
#![feature(iter_collect_into)]
#![feature(lint_reasons)]

use colour::magenta_ln;

use server::Server;

mod server;
mod addon;

#[tokio::main]
async fn main() {
	magenta_ln!("===== Berld =====");
	Server::default().run().await;
}