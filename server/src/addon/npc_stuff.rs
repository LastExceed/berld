use protocol::nalgebra::Point3;
use protocol::packet::common::*;
use protocol::packet::creature_update::*;
use protocol::packet::CreatureUpdate;
use protocol::utils::flagset::FlagSet;
use tap::Tap;

use crate::server::player::Player;

pub struct NpcStuff {
    static_npcs: Vec<CreatureUpdate>
}

impl NpcStuff {
    pub fn new() -> Self {
        Self {
            static_npcs: static_npcs()
        }
    }

    pub async fn load_npcs(&self, player: &Player) {
        for packet in &self.static_npcs {
            player.send_ignoring(packet).await;
        }
    }
}

#[expect(clippy::too_many_lines, reason = "todo")]
fn static_npcs() -> Vec<CreatureUpdate> {
    let appearance_template = Appearance {
        flags: FlagSet::default().tap_mut(|fs| {
            fs.set(AppearanceFlag::Trainer, true);
            fs.set(AppearanceFlag::Immovable, true);
        }),
        hair_color: Default::default(),
        creature_size: Hitbox {
            width: 0.96000004,
            depth: 0.96000004,
            height: 2.16
        },
        head_model    :  9,
        hair_model    : -1,
        hand_model    :  7,
        foot_model    :228,
        body_model    :  0,
        tail_model    : -1,
        shoulder2model: -1,
        wing_model    : -1,
        head_size     : 1.0,
        body_size     : 1.0,
        hand_size     : 1.0,
        foot_size     : 1.0,
        body_offset   : Point3::new(0.0, 0.0,  -5.0),
        head_offset   : Point3::new(0.0, 0.0,   6.0),
        hand_offset   : Point3::new(6.0, 0.0,   0.0),
        foot_offset   : Point3::new(3.0, 1.0, -10.5),
        ..Default::default()
    };
    
    #[expect(clippy::large_stack_arrays, reason = "only happens once on startup")]
    [
        CreatureUpdate {
            appearance: Some(Appearance {
                head_model:  8,
                body_model:  1,
                ..appearance_template.clone()
            }),
            occupation: Some(Occupation::Rogue),
            name: Some("Warrior\nTrainer".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(Appearance {
                head_model:  10,
                body_model:  2,
                ..appearance_template.clone()
            }),
            occupation: Some(Occupation::Ranger),
            name: Some("Ranger\nTrainer".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(Appearance {
                head_model:  5,
                body_model:  6,
                head_offset   : [0.0,0.0,9.0].into(),
                ..appearance_template.clone()
            }),
            occupation: Some(Occupation::Mage),
            name: Some("Mage\nTrainer".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(Appearance {
                head_model:  11,
                body_model:  1,
                ..appearance_template.clone()
            }),
            occupation: Some(Occupation::Rogue),
            name: Some("Rogue\nTrainer".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(appearance_template.clone()),
            occupation: Some(Occupation::Identifier),
            name: Some("Identifier".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(appearance_template.clone()),
            occupation: Some(Occupation::Adapter),
            name: Some("Adapter".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(appearance_template.clone()),
            occupation: Some(Occupation::ArmorShopkeep),
            home_zone: Some([32804, 32803, 7].into()),
            name: Some("Armor\nShop".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(appearance_template.clone()),
            occupation: Some(Occupation::GeneralShopkeep),
            home_zone: Some([32804, 32803, 8].into()),
            name: Some("General\nShop".into()),
            ..Default::default()
        },
        CreatureUpdate {
            appearance: Some(appearance_template),
            occupation: Some(Occupation::WeaponShopkeep),
            home_zone: Some([32804, 32803, 9].into()),
            name: Some("Weapon\nShop".into()),
            ..Default::default()
        }
    ]
        .into_iter()
        .enumerate()
        .map(|(index, mut packet)| {
            packet.id = CreatureId(10000 + index as i64); // todo: claim from pool
            packet.race = Some(Race::Bandit); // invisible
            packet.rotation = Some(EulerAngles { pitch: 0.0, roll: 0.0, yaw: 180.0 });
            packet.position = Some([
                550365916436 + 0x30000*(index as i64),
                550369345631,
                5903482
            ].into());
            packet
        })
        .collect()
}