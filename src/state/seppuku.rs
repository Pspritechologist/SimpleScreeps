use super::*;
use and_then::StateAndThen;
use general_states::{StateMove, StateSinging, StateTransfer};
use ignore_then::{StateIgnoreThen, StateIgnoreThenExt};
use move_to::{StateMoveTo, StateMoveToExt};
use screeps::ErrorCode;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct StateSuicide;

impl StateSuicide {
	pub fn new() -> Self {
		Default::default()
	}
}

impl State for StateSuicide {
	type Error = !;
	type Return = ();
	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if matches!(creep.suicide(), Err(ErrorCode::Busy)) {
			log::error!("Tried to kill Creep {} while they were being spawned", creep.name());
		}

		Finished(())
	}
}

/// An honourable way to go.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateSeppuku(SeppukuType);

impl StateSeppuku {
	pub fn new(creep: &Creep, spawn: &StructureSpawn) -> Self {
		let dest = screeps::game::flags().get("Funeral".to_string()).map(|f| f.pos()).unwrap_or_else(|| creep.pos());
		StateSeppuku(StateTransfer::new(spawn.id().into_type(), screeps::ResourceType::Energy, None)
			.move_to_ends(&creep, &spawn, 1)
			.ignore_then(StateMove::new_from_ends(creep, dest, 1).ignore_then(Default::default())))
	}
}

impl State for StateSeppuku {
	type Error = <SeppukuType as State>::Error;
	type Return = <SeppukuType as State>::Return;

	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		self.0.run(creep, data)
	}
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FuneralRites(usize);
impl general_states::Lyrics for FuneralRites {
	fn next_line(&mut self) -> Option<&str> {
		if let Some(l) = &crate::quotes::FUNERAL_RITES.get(self.0) {
			self.0 += 1;
			Some(l)
		} else {
			None
		}
	}
}

type SeppukuType = StateIgnoreThen<StateMoveTo<StateTransfer>, StateIgnoreThen<StateMove, StateAndThen<StateSinging<FuneralRites>, StateSuicide>>>;
