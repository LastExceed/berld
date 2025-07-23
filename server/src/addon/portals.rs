use config::{Config, ConfigError};
use protocol::nalgebra::Point3;
use protocol::packet::common::{CreatureId, Hitbox};
use protocol::packet::creature_update::{Affiliation, Appearance};
use protocol::packet::CreatureUpdate;
use tap::Pipe;

use crate::server::player::Player;

pub struct Portals {
    links: Vec<[Portal; 2]>
}

impl Portals {
    pub fn new(config: &Config) -> Result<Self, ConfigError> {
        Self {
            links: config.get("portals")?
        }.pipe(Ok)
    }
    
    pub async fn send_entities(&self, player: &Player) {
        for (index, [portal1, portal2]) in self.links.iter().enumerate() {
            player.send_ignoring(&portal1.create(CreatureId(20000 + index as i64))).await;
            player.send_ignoring(&portal2.create(CreatureId(21000 + index as i64))).await;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct Portal {
    title: String,
    position: Point3<i64>
}

impl Portal {
    fn create(&self, id: CreatureId) -> CreatureUpdate {
        CreatureUpdate {
            id,
            position: Some(self.position),
            affiliation: Some(Affiliation::NPC),
            appearance: Some(Appearance {
                creature_size: Hitbox {
                    width: 1.0,
                    depth: 0.0,
                    height: 0.0
                },
                body_model: 2525, // runestone
                body_size: 1.0,
                ..Default::default()
            }),
            name: Some(self.title.clone()),
            ..Default::default()
        }
    }
}