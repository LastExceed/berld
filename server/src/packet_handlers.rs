use std::io;

use parking_lot::lock_api::RawRwLockDowngrade;

use protocol::nalgebra::Vector3;
use protocol::packet::*;
use protocol::packet::creature_action::CreatureActionType;
use protocol::packet::world_update::Pickup;
use protocol::SIZE_BLOCK;

use crate::player::Player;
use crate::pvp::enable_pvp;
use crate::Server;
use crate::traffic_filter::filter;

impl Server {
	pub fn on_creature_update(&self, source: &Player, mut packet: CreatureUpdate) -> Result<(), io::Error> {
		enable_pvp(&mut packet);

		let mut character = source.creature.write();
		let snapshot = character.clone();
		character.update(&packet);
		unsafe { source.creature.raw().downgrade(); }//todo: not sure

		if filter(&mut packet, &snapshot, &character) {
			self.broadcast(&packet, Some(source));
		}

		Ok(())
	}

	pub fn on_creature_action(&self, source: &Player, packet: CreatureAction) -> Result<(), io::Error> {
		match packet.type_ {
			CreatureActionType::Bomb => {
				source.notify("bombs are disabled".to_owned());

				//the player consumed a bomb, so we need to reimburse it
				source.send(&WorldUpdate {
					pickups: vec![Pickup {
						interactor: source.creature.read().id,
						item: packet.item
					}],
					..Default::default()
				});
			}
			CreatureActionType::Talk => {
				source.notify("quests coming soon(tm)".to_owned());
			}
			CreatureActionType::ObjectInteraction => {
				source.notify("object interactions are disabled".to_owned());
			}
			CreatureActionType::PickUp => {
				if let Some(item) = self.remove_drop(packet.zone, packet.item_index as usize) {
					source.send(&WorldUpdate {
						pickups: vec![Pickup {
							interactor: source.creature.read().id,
							item
						}],
						..Default::default()
					});
				}
			}
			CreatureActionType::Drop => {
				let creature_guard = source.creature.read();

				self.add_drop(
					packet.item,
					creature_guard.position - Vector3::new(0, 0, SIZE_BLOCK),
					creature_guard.rotation.yaw
				);
			}
			CreatureActionType::CallPet => {
				//source.notify("pets are disabled".to_owned());
			}
		}

		Ok(())
	}

	pub fn on_hit(&self, source: &Player, packet: Hit) -> Result<(), io::Error> {
		if packet.target == packet.attacker && packet.damage.is_sign_negative() {
			return Ok(()) //self-heal is already applied client-side (which is a bug)
		}

		self.broadcast(&WorldUpdate { //todo: broadcast necessary?
			hits: vec![packet],
			..Default::default()
		}, Some(source));

		Ok(())
	}

	pub fn on_status_effect(&self, source: &Player, packet: StatusEffect) -> Result<(), io::Error> {
		self.broadcast(
			&WorldUpdate {
				status_effects: vec![packet],
				..Default::default()
			},
			Some(source)
		);

		Ok(())
	}

	pub fn on_projectile(&self, source: &Player, packet: Projectile) -> Result<(), io::Error> {
		self.broadcast(
			&WorldUpdate {
				projectiles: vec![packet],
				..Default::default()
			},
			Some(source)
		);

		Ok(())
	}

	pub fn on_chat_message(&self, source: &Player, packet: ChatMessageFromClient) -> Result<(), io::Error> {
		self.broadcast(
			&ChatMessageFromServer {
				source: source.creature.read().id,
				text: packet.text
			},
			None
		);

		Ok(())
	}

	pub fn on_zone_discovery(&self, _source: &Player, _packet: ZoneDiscovery) -> Result<(), io::Error> {
		Ok(())
	}

	pub fn on_region_discovery(&self, _source: &Player, _packet: RegionDiscovery) -> Result<(), io::Error> {
		Ok(())
	}
}