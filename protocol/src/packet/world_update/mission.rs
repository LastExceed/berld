#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Objective {
	RemoveMission,//TODO: surrogate value
	DefeatTheRaceInLocation,
	BesiegeNameDenIn,
	BesiegeDenInLocation,
	BesiegeNameDenIm,
	DefeatTheRulerInLocation,
	Blank,
	BesiegeDieRacesInDenLändernVon,
	BesiegeDieRacesImLocation,
	BesiegeDieRacesInLocation,
	SpüreDieRacesInHöhlenAufUndBekämpfeSie,
	SucheDieFlüsseImGanzenLandAbUndBekämpfeDieRaces,
	BesiegeDieBandeImLocation,
	BesiegeNameDenAnführerDerBanditenImLocation
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MissionState {
	Ready,
	InProgress,
	Finished
}