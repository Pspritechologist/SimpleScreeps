use super::{*, general_states::*};
use screeps::{ErrorCode, ResourceType};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateUpgraderJob {
	current_state: PotentialState,
	target: ControllerId,
	container: StructureId,
	recurring: bool,
	moving_to_container: bool,
}

impl StateUpgraderJob {
	pub fn new(creep: &Creep, target: ControllerId, container: StructureId) -> Self {
		Self {
			recurring: false,
			..Self::new_recurring(creep, target, container)
		}
	}

	pub fn new_recurring(creep: &Creep, target: ControllerId, container: StructureId) -> Self {
		let moving_to_container = creep.store().get_used_capacity(Some(ResourceType::Energy)) < creep.store().get_capacity(Some(ResourceType::Energy)) / 2;
		let dest = if moving_to_container { container.resolve().unwrap().pos() } else { target.resolve().unwrap().pos() };

		Self {
			current_state: PotentialState::Moving(StateMove::new_from_ends(creep.pos(), dest, 1)),
			target,
			container,
			recurring: true,
			moving_to_container,
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum PotentialState {
	Withdrawing(StateWithdraw),
	Upgrading,
	Moving(StateMove),
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateUpgraderJobError {
	WithdrawingError(<StateWithdraw as State>::Error),
	MovingError(<StateMove as State>::Error),
	TargetNotReal,
	NoBodyPart,
	ControllerBlocked,
	Unknown,
}

impl State for StateUpgraderJob {
	type Error = StateUpgraderJobError;
	type Return = ();
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.current_state {
			PotentialState::Withdrawing(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						let Some(target) = self.target.resolve() else {
							return Failed(StateUpgraderJobError::TargetNotReal);
						};
						(self.moving_to_container, self.current_state) = (false, PotentialState::Moving(StateMove::new_from_ends(creep.pos(), target.pos(), 3)));
						Working
					}
					Failed(e) => Failed(StateUpgraderJobError::WithdrawingError(e)),
				}
			}
			PotentialState::Upgrading => {
				let Some(target) = self.target.resolve() else {
					return Failed(StateUpgraderJobError::TargetNotReal);
				};
				match creep.upgrade_controller(&target) {
					Ok(_) => Working,
					Err(e) => {
						match e {
							ErrorCode::NotInRange => {
								// Switch back to moving.
								let dest = target.pos();
								self.current_state = PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), dest));
								self.run(creep, data)
							}
							ErrorCode::NotEnough => {
								if self.recurring {
									let Some(dest) = self.container.resolve() else {
										return Failed(StateUpgraderJobError::TargetNotReal);
									};
									(self.moving_to_container, self.current_state) = (true, PotentialState::Moving(StateMove::new_from_ends_close(creep.pos(), dest.pos())));
									self.run(creep, data)
								} else {
									Finished(())
								}
							}
							ErrorCode::InvalidTarget => Failed(StateUpgraderJobError::ControllerBlocked),
							ErrorCode::NoBodypart => Failed(StateUpgraderJobError::NoBodyPart),
							_ => Failed(StateUpgraderJobError::Unknown),
						}
					}
				}
			}
			PotentialState::Moving(ref mut state) => {
				match state.run(creep, data) {
					Working => Working,
					Finished(_) => {
						if self.moving_to_container {
							self.current_state = PotentialState::Withdrawing(StateWithdraw::new(self.container, ResourceType::Energy, None));
						} else {
							self.current_state = PotentialState::Upgrading;
						}

						// We run again because a completed move state means we've already arrived.
						self.run(creep, data)
					}
					Failed(e) => Failed(StateUpgraderJobError::MovingError(e)),
				}
			}
		}
	}
}
