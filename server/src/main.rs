#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use server::Server;

mod server;
mod creature_id_pool;
mod player;
mod traffic_filter;
mod pvp;
mod creature;
mod handle_packet;
mod anti_cheat;

fn main() {
	println!("go");
	Server::new().run();
}