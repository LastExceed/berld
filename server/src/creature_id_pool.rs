use protocol::packet::common::CreatureId;

pub struct CreatureIdPool {
	claimed_ids: Vec<i64>
}

impl CreatureIdPool {
	pub fn new() -> Self {
		Self {
			claimed_ids: vec![]
		}
	}

	pub fn claim(&mut self) -> CreatureId {
		let mut x = 990i64;
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