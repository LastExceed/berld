use protocol::packet::common::CreatureId;

#[derive(Default)]
pub struct CreatureIdPool {
	claimed_ids: Vec<i64>
}

impl CreatureIdPool {
	pub fn claim(&mut self) -> CreatureId {
		let mut x = 0_i64;
		while self.claimed_ids.contains(&x) {
			x += 1;
		}
		self.claimed_ids.push(x);
		CreatureId(x)
	}

	pub fn free(&mut self, id: CreatureId) {
		self.claimed_ids.swap_remove(
			self.claimed_ids
				.iter()
				.position(|other| *other == id.0)
				.expect("attempted to free a non-existing CreatureId")
		);
	}
}