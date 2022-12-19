use std::ops::Range;

use boolinator::Boolinator;
use protocol::packet::common::EulerAngles;
use protocol::packet::CreatureUpdate;

use crate::creature::Creature;

pub fn inspect_creature_update(packet: &CreatureUpdate, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//position
	packet.rotation.map_or_ok(inspect_rotation)?;

	Ok(())
}

fn inspect_rotation(rotation: &EulerAngles) -> Result<(), &'static str> {
	rotation.pitch//normally -180..180, but over-/underflows while attacking
		.is_finite()
		.ok_or("rotation.yaw wasn't finite")?;
	rotation.roll
		.require_in(&(-90f32..90f32), "rotation.roll")?;
	rotation.yaw
		.is_finite()
		.ok_or("rotation.yaw wasn't finite")
}

trait RequireIn: PartialEq + Sized {
	fn require_in<'a>(&self, container: &impl Contains<Self>, property_name: &'a str) -> Result<(), &'a str> {
		container
			.contains(self)
			.ok_or(property_name)//format!("{} was {} instead of {}", property_name, self, container).as_str()
	}
}

impl<T: PartialEq> RequireIn for T {}



trait Contains<T: PartialEq> {
	fn contains(&self, x: &T) -> bool;
}

impl<T: PartialEq> Contains<T> for [T] {
	fn contains(&self, x: &T) -> bool {
		self.contains(x)
	}
}

impl<T: PartialEq> Contains<T> for Range<T> {
	fn contains(&self, x: &T) -> bool {
		self.contains(x)
	}
}



trait PresentIn: PartialEq + Sized {
	fn present_in(&self, container: &impl Contains<Self>) -> bool {
		container.contains(self)
	}
}

impl<T: PartialEq> PresentIn for T {}



trait MapOrOk<T> {
	fn map_or_ok<E>(&self, f: impl FnOnce(&T) -> Result<(), E>) -> Result<(), E>;
}

impl<T> MapOrOk<T> for Option<T> {
	fn map_or_ok<E>(&self, f: impl FnOnce(&T) -> Result<(), E>) -> Result<(), E> {
		self.map_or(f, Ok(()))
	}
}