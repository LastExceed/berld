use std::f64::consts::PI;

use config::{Config, ConfigError};
use rand::random;
use serde::de::DeserializeOwned;
use tap::Tap;

use protocol::nalgebra::Point3;
use protocol::packet::common::Hitbox;
use protocol::packet::creature_update::{Appearance, AppearanceFlag};
use protocol::utils::constants::SIZE_BLOCK;
use protocol::utils::flagset::FlagSet;

const YAW_OFFSET: f64 = 90.0;
pub const RENDER_DISTANCE_NAME: i64 = SIZE_BLOCK * 60;
pub const RENDER_DISTANCE_CREATURE: i64 = RENDER_DISTANCE_NAME * 3;

pub fn creatures_circular(center: Point3<i64>, radius: i64, count: usize, i: usize) -> (Point3<i64>, f32) {
    let angle = 2.0 * PI * (i as f64) / (count as f64);
    let x = (center.x as f64) + (radius as f64) * angle.cos();
    let y = (center.y as f64) + (radius as f64) * angle.sin();

    let dx = center.x as f64 - x;
    let dy = center.y as f64 - y;
    let yaw = (dy.atan2(dx) * 180.0 / PI - YAW_OFFSET) as f32;

    (Point3::new(x.round() as i64, y.round() as i64, center.z), yaw)
}

pub fn is_in_zone(position: Point3<i64>, center: Point3<i64>, radius: i64) -> bool {
    let dx = (position.x - center.x) as i128;
    let dy = (position.y - center.y) as i128;
    let dz = (position.z - center.z) as i128;
    let radius = i128::from(radius);

    dx * dx + dy * dy + dz * dz <= radius * radius
}

pub fn appearance_invisible() -> Appearance {
    Appearance {
        flags: FlagSet::default().tap_mut(|fs| {
            fs.set(AppearanceFlag::Unknown7, true);
            fs.set(AppearanceFlag::Immovable, true)
        }),
        creature_size: Hitbox { width: 1.0, depth: 1.0, height: 1.0 },
        head_model: -1,
        hair_model: -1,
        hand_model: -1,
        foot_model: -1,
        body_model: -1,
        tail_model: -1,
        shoulder2model: -1,
        wing_model: -1,
        body_size: 1.0,
        ..Default::default()
    }
}

pub fn pick_from<T: Copy>(items: &[T]) -> T {
    let index = ((random::<f32>() * items.len() as f32) as usize).min(items.len() - 1);
    items[index]
}

pub fn config_fallback<T: DeserializeOwned>(config: &Config, key: &str, default: T) -> Result<T, ConfigError> {
    match config.get(key) {
        Ok(value)                     => Ok(value),
        Err(ConfigError::NotFound(_)) => Ok(default),
        Err(err)                      => Err(err)
    }
}

pub fn config_optional<T: DeserializeOwned>(config: &Config, key: &str) -> Result<Option<T>, ConfigError> {
    match config.get(key) {
        Ok(value)                     => Ok(Some(value)),
        Err(ConfigError::NotFound(_)) => Ok(None),
        Err(err)                      => Err(err)
    }
}