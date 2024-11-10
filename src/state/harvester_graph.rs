use super::*;
use crate::dbg;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HarvesterStateGraph {
	pub job: RoomObjectId,

	current_state: PotentialState,
	target: StructureId,
	source: ObjectId<Source>,
	recurring: bool,
	moving_to_source: bool,
}

impl HarvesterStateGraph {
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
pub enum HarvesterStateGraphError {
	HarvestingError(<StateHarvesting as State>::Error),
	TransferringError(<StateTransfer as State>::Error),
	MovingError(<StateMove as State>::Error),
	TargetNotReal,
	SourceNotReal,
}

impl State for HarvesterStateGraph {
	type Error = HarvesterStateGraphError;
	type Return = ();
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.current_state {
			PotentialState::Harvesting(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(target) = self.target.resolve() else {
							return Failed(HarvesterStateGraphError::TargetNotReal);
						};
						(self.moving_to_source, self.current_state) = (false, PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), target.pos())));
						self.run(creep, data)
					}
					Failed(e) => Failed(HarvesterStateGraphError::HarvestingError(e)),
				}
			}
			PotentialState::Transferring(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(source) = self.source.resolve() else {
							return Failed(HarvesterStateGraphError::SourceNotReal);
						};
						
						if self.recurring {
							(self.moving_to_source, self.current_state) = (true, PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), source.pos())));
							Working
						} else {
							Finished(())
						}
					}
					Failed(e) => Failed(dbg!(HarvesterStateGraphError::TransferringError(e))),
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
					Failed(e) => Failed(HarvesterStateGraphError::MovingError(e)),
				}
			}
		}
	}
}
