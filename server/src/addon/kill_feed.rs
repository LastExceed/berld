use std::time::{Duration, Instant};

use protocol::packet::CreatureUpdate;

use crate::server::{player::Player, Server};

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) {
    if !packet.health.is_some_and(|value| value <= 0.0) {
        return;
    }

    let victim = source.character.read().await.name.clone();

    let message = source
        .addon_data
        .write()
        .await
        .last_attacker
        .take()
        .filter(|(timestamp, _name)| timestamp.elapsed() < Duration::from_secs(1))
        .map(|(_timestamp, name)| format!("{name} killed {victim}"))
        .unwrap_or(format!("{victim} died"));

    server.announce(message).await;
}

pub async fn set_last_attacker(target: &Player, attacker_name: String) {
    target
        .addon_data
        .write()
        .await
        .last_attacker = Some((Instant::now(), attacker_name));
}