#[allow(unused_imports)]//import is used in doc comments
use crate::common::Race;

///all names (including the enum itself) are data mined
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Objective {
	///TODO: surrogate value
	RemoveMission,

	///"Defeat the {[Race]} in {LOCATION}"
	Monster,

	///"Besiege {NAME} den/die in"
	///
	///(german, translates to "Defeat {NAME} the in")
	VillageRareBoss,

	///"Besiege den/die in {LOCATION}"
	///
	///(german, translates to "Defeat the in {LOCATION}")
	VillageMonster,

	///"Besiege {NAME} den/die im"
	///
	///(german, translates to "Defeat {NAME} the in the")
	RareBoss,

	///"Defeat the ruler in {LOCATION}"
	Dungeon,

	Blank,

	///"Besiege die {[Race]}s in den Ländern von"
	///
	///(german, translates to "Defeat the {[Race]}s in the lands of")
	SceneryInvasion,

	///"Besiege die {[Race]}s im {LOCATION}"
	///
	///(german, translates to "Defeat the {[Race]}s in the {LOCATION}")
	Invasion,

	///"Besiege die {[Race]}s in {LOCATION}"
	///
	///(german, translates to "Defeat the {[Race]}s in {LOCATION}")
	VillageInvasion,

	///"Spüre die {[Race]}s in Höhlen auf und bekämpfe sie"
	///
	///(german, translates to "Track down the {[Race]}s in caves and fight them")
	CaveInvasion,

	///"Suche die Flüsse im ganzen Land ab und bekämpfe die {[Race]}s"
	///
	///(german, translates to "Search the rivers in the whole land and fight the {[Race]}s")
	RiverInvasion,

	///"Besiege die Bande im {LOCATION}"
	///
	///(german, translates to "Defeat the Gang in the {LOCATION}")
	Gang,

	///"Besiege {NAME} den/die Anführer der Banditen im {LOCATION}"
	///
	///(german, translates to "Defeat {NAME} the leader of the bandits in the {LOCATION}")
	GangBoss,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MissionState {
	Ready,
	InProgress,
	Finished
}