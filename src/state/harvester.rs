use super::{*, general_states::*};
use screeps::{Creep, ErrorCode, ObjectId, ResourceType, Source };

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateHarvesting {
	source: ObjectId<Source>,
}

impl StateHarvesting {
	pub fn new(source: ObjectId<Source>) -> Self {
		Self { source }
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum HarvestReturn {
	Filled,
	RanOut,
}

impl State for StateHarvesting {
	type Error = GenericStateError;
	type Return = HarvestReturn;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if creep.store().get_free_capacity(None) == 0 {
			return Finished(HarvestReturn::Filled);
		}

		let Some(source) = self.source.resolve() else {
			return Failed(GenericStateError::TargetNotReal);
		};

		if let Err(e) = creep.harvest(&source) {
			match e {
				ErrorCode::NotInRange => return Failed(GenericStateError::OutOfRange),
				ErrorCode::NoBodypart => return Failed(GenericStateError::NoParts),
				ErrorCode::NotEnough => return Finished(HarvestReturn::RanOut),
				_ => return Failed(GenericStateError::Unknown),
			}
		}

		Working
	}
}



#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateHarvesterJob {
	pub job: RoomObjectId,

	current_state: PotentialState,
	target: StructureId,
	source: ObjectId<Source>,
	recurring: bool,
	moving_to_source: bool,
}

impl StateHarvesterJob {
	pub fn new(creep: &Creep, job: RoomObjectId, target: StructureId, source: ObjectId<Source>) -> Self {
		Self {
			recurring: false,
			..Self::new_recurring(creep, job, target, source)
		}
	}

	pub fn new_recurring(creep: &Creep, job: RoomObjectId, target: StructureId, source: ObjectId<Source>) -> Self {
		let moving_to_source = creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_capacity(Some(ResourceType::Energy)) as i32 / 2;
		let dest = if moving_to_source { source.resolve().unwrap().pos() } else { target.resolve().unwrap().pos() };

		Self {
			job,
			current_state: PotentialState::Moving(StateMove::new_from_ends(creep.pos(), dest, 1)),
			target,
			source,
			recurring: true,
			moving_to_source,
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum PotentialState {
	Harvesting(StateHarvesting),
	Transferring(StateTransfer),
	Moving(StateMove),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateHarvesterJobError {
	HarvestingError(<StateHarvesting as State>::Error),
	TransferringError(<StateTransfer as State>::Error),
	MovingError(<StateMove as State>::Error),
	TargetNotReal,
	SourceNotReal,
}

impl State for StateHarvesterJob {
	type Error = StateHarvesterJobError;
	type Return = ();
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.current_state {
			PotentialState::Harvesting(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(target) = self.target.resolve() else {
							return Failed(StateHarvesterJobError::TargetNotReal);
						};
						(self.moving_to_source, self.current_state) = (false, PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), target.pos())));
						self.run(creep, data)
					}
					Failed(e) => Failed(StateHarvesterJobError::HarvestingError(e)),
				}
			}
			PotentialState::Transferring(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(source) = self.source.resolve() else {
							return Failed(StateHarvesterJobError::SourceNotReal);
						};
						
						if self.recurring {
							(self.moving_to_source, self.current_state) = (true, PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), source.pos())));
							Working
						} else {
							Finished(())
						}
					}
					Failed(e) => Failed(StateHarvesterJobError::TransferringError(e)),
				}
			}
			PotentialState::Moving(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						if self.moving_to_source {
							self.current_state = PotentialState::Harvesting(StateHarvesting::new(self.source));
						} else {
							self.current_state = PotentialState::Transferring(StateTransfer::new(self.target, ResourceType::Energy, None));
						}

						// We run again because a completed move state means we've already arrived.
						self.run(creep, data)
					}
					Failed(e) => Failed(StateHarvesterJobError::MovingError(e)),
				}
			}
		}
	}
}
