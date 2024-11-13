use general_states::{StateMove, StateWithdraw};
use move_to::StateMoveTo;
use screeps::{ConstructionSite, ErrorCode, ResourceType};

use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateBuilding {
	target: ObjectId<ConstructionSite>,
	retry_count: u8,
}

impl StateBuilding {
	pub fn new(target: ObjectId<ConstructionSite>) -> Self {
		Self::new_with_retries(target, 0)
	}

	pub fn new_with_retries(target: ObjectId<ConstructionSite>, retry_count: u8) -> Self {
		Self { target, retry_count }
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BuildError {
	TargetNotReal,
	Empty,
	NoBodyPart,
	NotInRange,
	SpaceOccupied,
	Unknown,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BuildReturn {
	Constructed,
	InProgress,
}
impl From<BuildReturn> for bool {
	fn from(value: BuildReturn) -> Self { matches!(value, BuildReturn::Constructed) }
}
impl From<bool> for BuildReturn {
	fn from(b: bool) -> Self { if b { BuildReturn::Constructed } else { BuildReturn::InProgress } }
}

impl State for StateBuilding {
	type Error = BuildError;
	type Return = BuildReturn;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		let target = self.target.resolve().ok_or(BuildError::TargetNotReal)?;

		//TODO: A single build action only delivers a small amount of energy- it seems to be 5 per fire.
		//TODO: I don't know how much this increases per build part, but probably by five. I need to try and find a constant
		//TODO: for it and implement this logic to continue firing until empty or built.
		let verge = creep.store().get_used_capacity(Some(ResourceType::Energy)) >= 
			target.progress_total() - target.progress();

		if let Err(e) = creep.build(&target) {
			match e {
				ErrorCode::NotEnough => return Failed(BuildError::Empty),
				ErrorCode::NoBodypart => return Failed(BuildError::NoBodyPart),
				ErrorCode::NotInRange => return Failed(BuildError::NotInRange),
				ErrorCode::InvalidTarget => {
					if self.retry_count > 0 {
						self.retry_count -= 1;
						return Working;
					} else {
						return Failed(BuildError::SpaceOccupied);
					}
				},
				_ => return Failed(BuildError::Unknown),
			}
		}

		Finished(verge.into())
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateBuilderJob {
	target: ObjectId<ConstructionSite>,
	container: StructureId,

	current_state: PotentialState,
}

impl StateBuilderJob {
	pub fn new(creep: &Creep, target: ObjectId<ConstructionSite>, container: StructureId) -> Self {
		let site = target.resolve().unwrap();
		
		let required = site.progress_total() - site.progress();
		let current_state = if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0
			&& creep.store().get_used_capacity(Some(ResourceType::Energy)) < required
		{
			let dest = container.resolve().unwrap();
			let move_state = StateMove::new_from_ends(creep, dest, 1);
			let withdraw_state = StateWithdraw::new(container, ResourceType::Energy, None);
			PotentialState::Collecting(StateMoveTo::new(move_state, withdraw_state))
		} else {
			let move_state = StateMove::new_from_ends(creep, site, 3);
			let build_state = StateBuilding::new(target);
			PotentialState::Building(StateMoveTo::new(move_state, build_state))
		};

		Self { target, container, current_state }
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum PotentialState {
	Building(StateMoveTo<StateBuilding>),
	Collecting(StateMoveTo<StateWithdraw>),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateBuilderJobError {
	BuildingError(<StateMoveTo<StateBuilding> as State>::Error),
	CollectingError(<StateMoveTo<StateWithdraw> as State>::Error),
	TargetNotReal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Constructed;

impl State for StateBuilderJob {
	type Error = StateBuilderJobError;
	type Return = Constructed;

	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.current_state {
			PotentialState::Building(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(BuildReturn::InProgress) => {
						let cont = self.container.resolve().ok_or(StateBuilderJobError::TargetNotReal)?;
						let move_state = StateMove::new_from_ends_lazy(creep, cont, 1);
						let withdraw_state = StateWithdraw::new(self.container, ResourceType::Energy, None);
						self.current_state = PotentialState::Collecting(StateMoveTo::new(move_state, withdraw_state));
						Working
					},
					Finished(BuildReturn::Constructed) => Finished(Constructed),
					Failed(e) => Failed(StateBuilderJobError::BuildingError(e)),
				}
			},
			PotentialState::Collecting(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let site = self.target.resolve().ok_or(StateBuilderJobError::TargetNotReal)?;
						let move_state = StateMove::new_from_ends_lazy(creep, site, 3);
						let build_state = StateBuilding::new(self.target);
						self.current_state = PotentialState::Building(StateMoveTo::new(move_state, build_state));
						Working
					},
					Failed(e) => Failed(StateBuilderJobError::CollectingError(e)),
				}
			},
		}
	}
}
