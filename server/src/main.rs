#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(let_else)]

use server::Server;

mod server;
mod creature_id_pool;
mod player;
mod traffic_filter;
mod pvp;
mod creature;
mod packet_handlers;

fn main() {
	println!("go");
	Server::new().run();
}