use crate::addon::anti_cheat::AntiCheatData;

#[derive(Debug, Default)]
pub struct AddonData {
	pub team: Option<i32>,
	pub anti_cheat_data: AntiCheatData
}