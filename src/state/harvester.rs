use crate::dynamic_stuff::DynState;

use super::{*, general_states::*};
use move_to::{StateMoveTo, StateMoveToExt};
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
	current_state: PotentialState,
	target: StructureId,
	source: ObjectId<Source>,
}

impl StateHarvesterJob {
	pub fn new(creep: &Creep, target: StructureId, source: ObjectId<Source>) -> Self {
		let moving_to_source = creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_capacity(Some(ResourceType::Energy)) as i32 / 2;
		let dest = if moving_to_source { source.resolve().unwrap().pos() } else { target.resolve().unwrap().pos() };
		let current_state = if moving_to_source {
			PotentialState::Harvesting(StateHarvesting::new(source).move_to_ends(creep, dest, 1))
		} else {
			PotentialState::Transferring(StateTransfer::new(target, ResourceType::Energy, None).move_to_ends(creep, dest, 1))
		};

		Self {
			current_state,
			target,
			source,
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum PotentialState {
	Harvesting(StateMoveTo<StateHarvesting>),
	Transferring(StateMoveTo<StateTransfer>),
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateHarvesterJobError {
	HarvestingError(<StateMoveTo<StateHarvesting> as State>::Error),
	TransferringError(<StateMoveTo<StateTransfer> as State>::Error),
	TargetNotReal,
	SourceNotReal,
}

impl State for StateHarvesterJob {
	type Error = StateHarvesterJobError;
	type Return = <StateTransfer as State>::Return;
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.current_state {
			PotentialState::Harvesting(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(target) = self.target.resolve() else {
							return Failed(StateHarvesterJobError::TargetNotReal);
						};
						self.current_state = PotentialState::Transferring(StateTransfer::new(target.id(), ResourceType::Energy, None).move_to_ends(creep.pos(), target.pos(), 1));
						self.run(creep, data)
					}
					Failed(e) => Failed(StateHarvesterJobError::HarvestingError(e)),
				}
			}
			PotentialState::Transferring(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(TransferReturn::Leftover(amnt)) => {
						// Hacky
						if amnt < 80 {
							return Finished(TransferReturn::Leftover(amnt));
						}

						let Some(controller) = (try {
							creep.room()?.controller()?
						}) else {
							return Finished(TransferReturn::Leftover(amnt));
						};

						let mut upgrade_state = super::upgrader::StateUpgraderJob::new(creep, controller.id(), self.target);

						if let Failed(e) = upgrade_state.run(creep, data) {
							log::warn!("Failed to upgrade controller while harvesting: {:?}", e);
							return Finished(TransferReturn::Leftover(amnt));
						}

						data.current_task = Some((
							crate::JobIdentifier { id: controller.id().into_type(), job: crate::JobFlag::Upgrade },
							DynState::new(upgrade_state, crate::dynamic_stuff::StateFlag::UpgraderJob)
						));

						Working
					},
					Finished(f) => Finished(f),
					Failed(move_to::MoveToError::<TransferError>::StateError(TransferError::TargetFull)) => {
						// Hacky
						let Some(controller) = (try {
							creep.room()?.controller()?
						}) else {
							return Finished(TransferReturn::Leftover(0));
						};

						let mut upgrade_state = super::upgrader::StateUpgraderJob::new(creep, controller.id(), self.target);

						if let Failed(e) = upgrade_state.run(creep, data) {
							log::warn!("Failed to upgrade controller while harvesting: {:?}", e);
							return Finished(TransferReturn::Leftover(0));
						}

						data.current_task = Some((
							crate::JobIdentifier { id: controller.id().into_type(), job: crate::JobFlag::Upgrade },
							DynState::new(upgrade_state, crate::dynamic_stuff::StateFlag::UpgraderJob)
						));

						Working
					},
					Failed(e) => Failed(StateHarvesterJobError::TransferringError(e)),
				}
			}
		}
	}
}
