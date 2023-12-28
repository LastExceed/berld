#![expect(incomplete_features, reason = "generic_const_exprs is incomplete, but works for our purposes")]
#![feature(generic_const_exprs)]
#![feature(async_closure)]
#![feature(future_join)]
#![feature(let_chains)]
#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(lint_reasons)]

#![allow(unreachable_pub)] //this isn't a lib, so adding `(crate)` to every `pub` is just pointless noise

use colour::magenta_ln;

use server::Server;

mod server;
mod addon;

#[tokio::main]
async fn main() {
	magenta_ln!("===== Berld =====");
	Server::default().run().await;
}