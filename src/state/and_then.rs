use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateAndThen<A: State, B: State> {
	state_a: A,
	state_b: B,
	in_state_a: bool,
}

pub enum AndThenStateFlag {
	A,
	B,
}

impl<A: State, B: State> StateAndThen<A, B> {
	pub fn new(state_a: A, state_b: B) -> Self {
		Self {
			state_a,
			state_b,
			in_state_a: true,
		}
	}

	pub fn current_state(&self) -> AndThenStateFlag {
		if self.in_state_a {
			AndThenStateFlag::A
		} else {
			AndThenStateFlag::B
		}
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum AndThenError<A, B> {
	StateAError(A),
	StateBError(B),
}

impl<A: State, B: State> State for StateAndThen<A, B> {
	type Error = AndThenError<A::Error, B::Error>;
	type Return = B::Return;

	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if self.in_state_a {
			match self.state_a.run(creep, data) {
				StateResult::Working => return Working,
				StateResult::Finished(_) => self.in_state_a = false,
				StateResult::Failed(e) => return StateResult::Failed(AndThenError::StateAError(e)),
			}
		}

		match self.state_b.run(creep, data) {
			StateResult::Working => StateResult::Working,
			StateResult::Finished(r) => StateResult::Finished(r),
			StateResult::Failed(e) => StateResult::Failed(AndThenError::StateBError(e)),
		}
	}
}

impl<A: State + Default, B: State + Default> Default for StateAndThen<A, B> {
	fn default() -> Self {
		Self::new(A::default(), B::default())
	}
}

pub trait StateAndThenExt: State {
	fn and_then<B: State>(self, state_b: B) -> StateAndThen<Self, B> {
		StateAndThen::new(self, state_b)
	}
}

impl<S: State> StateAndThenExt for S { }
