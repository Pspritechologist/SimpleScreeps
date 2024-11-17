use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateMoveTo<S: State> {
	move_state: general_states::StateMove,
	state: S,
	in_move_state: bool,
}

pub enum MoveToStateFlag {
	Moving,
	State,
}

impl<S: State> StateMoveTo<S> {
	pub fn new(move_state: general_states::StateMove, state: S) -> Self {
		Self {
			move_state,
			state,
			in_move_state: true,
		}
	}

	pub fn new_from_ends(state: S, start: impl HasPosition, end: impl HasPosition, range: u8) -> Self {
		Self::new(general_states::StateMove::new_from_ends(start, end, range), state)
	}

	pub fn current_state(&self) -> MoveToStateFlag {
		if self.in_move_state {
			MoveToStateFlag::Moving
		} else {
			MoveToStateFlag::State
		}
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum MoveToError<T> {
	MoveError(general_states::MoveError),
	StateError(T),
}

impl<S: State> State for StateMoveTo<S> {
	type Error = MoveToError<S::Error>;
	type Return = S::Return;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if self.in_move_state {
			match self.move_state.run(creep, _data) {
				Working => return Working,
				Finished(_) => self.in_move_state = false,
				Failed(e) => return Failed(MoveToError::MoveError(e)),
			}
		}

		match self.state.run(creep, _data) {
			Working => Working,
			Finished(r) => Finished(r),
			Failed(e) => Failed(MoveToError::StateError(e)),
		}
	}
}

pub trait StateMoveToExt: State {
	fn move_to(self, move_state: general_states::StateMove) -> StateMoveTo<Self> {
		StateMoveTo::new(move_state, self)
	}

	fn move_to_ends(self, start: impl HasPosition, end: impl HasPosition, range: u8) -> StateMoveTo<Self> {
		StateMoveTo::new_from_ends(self, start, end, range)
	}
}

impl<S: State> StateMoveToExt for S { }

pub trait StateMoveToDefaultExt: StateMoveToExt + Default {
	fn move_to_default(move_state: general_states::StateMove) -> StateMoveTo<Self> {
		StateMoveTo::new(move_state, Default::default())
	}

	fn move_to_default_ends(start: impl HasPosition, end: impl HasPosition, range: u8) -> StateMoveTo<Self> {
		StateMoveTo::new_from_ends(Default::default(), start, end, range)
	}
}

impl<S: StateMoveToExt + Default> StateMoveToDefaultExt for S { }
