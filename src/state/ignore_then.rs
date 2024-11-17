use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateIgnoreThen<A: State, B: State> {
	state_a: A,
	state_b: B,
	in_state_a: bool,
}

impl<A: State, B: State> State for StateIgnoreThen<A, B> {
	type Error = B::Error;
	type Return = B::Return;

	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if self.in_state_a {
			match self.state_a.run(creep, data) {
				StateResult::Working => {},
				_ => self.in_state_a = false,
			}

			return Working;
		}

		self.state_b.run(creep, data)
	}
}

impl<A: State, B: State> StateIgnoreThen<A, B> {
	pub fn new(state_a: A, state_b: B) -> Self {
		Self {
			state_a,
			state_b,
			in_state_a: true,
		}
	}
}

pub trait StateIgnoreThenExt: State {
	fn ignore_then<B: State>(self, state: B) -> StateIgnoreThen<Self, B> where Self: Sized {
		StateIgnoreThen::new(self, state)
	}

	fn then_ignore<A: State>(self, state: A) -> StateIgnoreThen<A, Self> where Self: Sized {
		StateIgnoreThen::new(state, self)
	}
}

impl<A: State> StateIgnoreThenExt for A { }
