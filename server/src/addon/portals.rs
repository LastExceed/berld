use config::{Config, ConfigError};
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::common::{CreatureId, Hitbox, Race};
use protocol::packet::creature_update::{Affiliation, Appearance, AppearanceFlag};
use protocol::packet::CreatureUpdate;
use protocol::utils::flagset::FlagSet;
use tap::{Pipe, Tap};

use crate::server::player::Player;
use crate::SERVER;

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
        for (n, portal) in self.links.iter().flatten().enumerate() {
            player.send_ignoring(&portal.create(n as _)).await;
        }
    }
    
    pub async fn on_interaction(&self, player: &Player, zone_data_index: Point3<i32>) {
        if zone_data_index.xy() != Point2::new(-1, -1) {
            return;
        }

        let Some(link) = self.links.get(zone_data_index.z as usize / 2)
        else { return };
        
        let dest = link[(zone_data_index.z as usize + 1) % 2].position;
        
        SERVER.teleport(player, dest).await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct Portal {
    title: String,
    position: Point3<i64>
}

impl Portal {
    fn create(&self, n: i32) -> CreatureUpdate {
        CreatureUpdate {
            id: CreatureId(20000 + n as i64),
            position: Some(self.position),
            affiliation: Some(Affiliation::NPC),
            race: Some(Race::Bandit), // invisible
            appearance: Some(Appearance {
                flags: FlagSet::default().tap_mut(|x| x.set(AppearanceFlag::Immovable, true)),
                creature_size: Hitbox {
                    width: 1.0,
                    depth: 0.0,
                    height: 0.0
                },
                head_model: 2525, // runestone
                hair_model: -1,
                head_size: 2.0,
                ..Default::default()
            }),
            zone_data_index: Some([-1,-1,n].into()),
            name: Some(self.title.clone()),
            ..Default::default()
        }
    }
}