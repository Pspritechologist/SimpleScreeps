use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateBuilding {
}

impl StateBuilding {
	pub fn new() -> Self {
		Self { }
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BuildError {
	
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BuildReturn {
	
}

impl State for StateBuilding {
	type Error = BuildError;
	type Return = BuildReturn;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		Working
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateBuilderJob {
	pub job: RoomObjectId,
}

impl StateBuilderJob {
	pub fn new(creep: &Creep, job: RoomObjectId) -> Self {
		Self { job }
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum PotentialState {
	
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateBuilderJobError {
	
}

impl State for StateBuilderJob {
	type Error = StateBuilderJobError;
	type Return = ();

	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		Working
	}
}
