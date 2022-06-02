use protocol::packet::creature_update::CreatureId;

pub struct CreatureIdPool {
	claimed_ids: Vec<i64>
}

impl CreatureIdPool {
	pub fn new() -> Self {
		Self {
			claimed_ids: Vec::new()
		}
	}

	pub fn claim(&mut self) -> CreatureId {
		let mut x = 0i64;
		while self.claimed_ids.binary_search(&x).is_ok() {
			x += 1;
		}
		self.claimed_ids.push(x);
		CreatureId(x)
	}

	pub fn free(&mut self, id: CreatureId) {
		self.claimed_ids.retain(|it| { *it != id.0 })
	}
}