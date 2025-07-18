use std::time::{Duration, Instant};

use crate::{addon::anti_cheat, server::creature::Creature};

use super::{EnsureAtMost, Player, PlayerData};

#[expect(clippy::cast_sign_loss, reason = "checked at runtime")]
pub(super) async fn check_for_timewarp(previous_state: &Creature, updated_state: &Creature, player: &Player) -> anti_cheat::Result {
	let ac_data = &mut player.addon_data.write().await.anti_cheat_data;

	let was_dead = previous_state.health == 0.0;
	let is_dead = updated_state.health == 0.0;

	if was_dead && is_dead {
		//clock freezes while dead
		return Ok(());
		//todo: you can legitimately hit sb while dead by shooting projectiles right before death
		//return updated_state.combo_timeout.ensure_exact(&previous_state.combo_timeout, "combo_timeout");
	}

	let init = ac_data.combo_epoch.is_none();//todo: move to ac data init?
	let respawn = was_dead && !is_dead; //timeout resets to 0 on respawn
	let hit = updated_state.combo_timeout <= previous_state.combo_timeout; //equal incase of seed change lag

	if hit || init || respawn {
        ac_data.reassign_epoch(updated_state.combo_timeout);
		return Ok(());
	}

    if ac_data.last_lag_spike.is_some_and(|timestamp| timestamp.elapsed() < Duration::from_secs(2)) {
        return Ok(());
    }

    if ac_data.last_lag_spike.is_some() {
        ac_data.reassign_epoch(updated_state.combo_timeout);
        ac_data.last_lag_spike = None;
        return Ok(());
    }

    let combo_epoch = ac_data.combo_epoch.as_mut().unwrap();

    let reported = Duration::from_millis(updated_state.combo_timeout as _);
	let elapsed = combo_epoch.elapsed();
    let delta = reported.as_nanos() as i128 - elapsed.as_nanos() as i128;

    if delta.abs() > Duration::from_millis(500).as_nanos() as _ {
        ac_data.last_lag_spike = Some(Instant::now());
        return Ok(());
    }

    ac_data.shift_epoch(delta as _);

    (ac_data.total_shift_nanos.abs() / 1_000_000)
        .ensure_at_most(2000, "timewarp.clockdesync")?;

	Ok(())
}

impl PlayerData {
    #[expect(clippy::cast_sign_loss, reason = "checked at runtime")]
    fn reassign_epoch(&mut self, reported_timeout: i32) {
        self.combo_epoch = Some(Instant::now() - Duration::from_millis(reported_timeout as _));
    }

    #[expect(clippy::cast_sign_loss, reason = "checked at runtime")]
	fn shift_epoch(&mut self, delta: i64) {
		let epoch = self
			.combo_epoch
			.as_mut()
			.expect("this should have been checked at this point");

		//necessary because duration cannot be negative
		if delta.is_positive() {
			*epoch -= Duration::from_nanos(delta as _);
		} else {
			*epoch += Duration::from_nanos(-delta as _);
		}

		self.total_shift_nanos += delta;
        self.decay();
	}

    fn decay(&mut self) {
        let interval = self
            .last_checked
            .get_or_insert_with(Instant::now)
            .elapsed()
            .as_secs_f64();

        self.total_shift_nanos = (self.total_shift_nanos as f64 * 0.9_f64.powf(interval)) as _;

        self.last_checked = Some(Instant::now());
    }
}