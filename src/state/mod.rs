pub mod general_states;
pub mod harvester;
pub mod upgrader;
pub mod builder;

pub use StateResult::*;
pub use crate::utils::prelude::*;

use crate::{ign, memory::CreepData};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StateResult<S, E> {
	Working,
	Finished(S),
	Failed(E),
}
impl<S, E> From<Result<S, E>> for StateResult<S, E> {
	fn from(r: Result<S, E>) -> Self {
		match r {
			Ok(s) => Finished(s),
			Err(e) => Failed(e),
		}
	}
}
impl<S, E> Into<Option<Result<S, E>>> for StateResult<S, E> {
	fn into(self) -> Option<Result<S, E>> {
		match self {
			Working => None,
			Finished(s) => Some(Ok(s)),
			Failed(e) => Some(Err(e)),
		}
	}
}
impl<S, E> Into<StateResult<S, E>> for Option<Result<S, E>> {
	fn into(self) -> StateResult<S, E> {
		match self {
			Some(Ok(s)) => Finished(s),
			Some(Err(e)) => Failed(e),
			None => Working,
		}
	}
}

pub trait State {
	type Error;
	type Return;
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct StateIdle(u32);

impl State for StateIdle {
	type Error = !;
	type Return = !;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if self.0 >= game::time() {
			ign!(creep.say("Idling...", true));
			ign!(creep.move_direction(screeps::Direction::Top.multi_rot(fastrand::i8(..))));
		}

		Working
	}
}
