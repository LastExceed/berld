use std::env::consts::OS;
use std::time::{Instant, Duration};

use axum::{Router, Json};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::extract::State;
use futures::future::join_all;
use serde::Serialize;
use tap::Pipe;
use tokio::net::TcpListener;

use crate::server::Server;
use crate::server::utils::extend_lifetime;

pub async fn run(server: &Server) {
    let router = Router::new()
        .route("/api/info", get(info))
        .with_state((extend_lifetime(server), Instant::now()));

    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .expect("failed to bind API socket");

    tokio::spawn(async move {
        axum::serve(listener, router)
        .await
        .expect("API error");
    });
}

async fn info(State((server, startup_time)): State<(&Server, Instant)>) -> impl IntoResponse {
    Info {
        players: get_all_player_names(server).await,
        platform: OS.into(),
        mapseed: server.mapseed,
        uptime: startup_time.elapsed(),
        // slots: -1,
        // name: todo!(),
        // discord: todo!(),
    }.pipe(Json)
}

#[derive(Serialize)]
struct Info {
    players: Vec<String>,
    platform: String,
    mapseed: i32,
    uptime: Duration,
    // slots: i32,
    // name: String,
    // discord: String,
}

async fn get_all_player_names(server: &Server) -> Vec<String> {
    server
        .players
        .read()
        .await
        .iter()
        .map(|player| async {
            player.character
                .read()
                .await
                .name
                .clone()
        })
        .pipe(join_all)
        .await
}