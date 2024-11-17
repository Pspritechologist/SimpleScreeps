pub mod general_states;
pub mod harvester;
pub mod upgrader;
pub mod builder;
pub mod move_to;
pub mod reoccurring;
pub mod seppuku;
pub mod and_then;
pub mod ignore_then;

use std::ops::Try;

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

use std::ops::ControlFlow;

impl<R, E> std::ops::Try for StateResult<R, E> {
	type Output = Option<R>;
	type Residual = StateResult<!, E>;
	fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
		match self {
			Working => ControlFlow::Continue(None),
			Finished(r) => ControlFlow::Continue(Some(r)),
			Failed(e) => ControlFlow::Break(Failed(e)),
		}
	}
	fn from_output(output: Self::Output) -> Self {
		match output {
			Some(r) => Finished(r),
			None => Working,
		}	
	}
}

impl<R, E> std::ops::FromResidual<StateResult<!, E>> for StateResult<R, E> {
	fn from_residual(residual: <Self as Try>::Residual) -> Self {
		match residual {
			Failed(e) => Failed(e),
			Working => Working,
		}
	}
}

impl<R, E> std::ops::FromResidual<Option<!>> for StateResult<R, E> {
	fn from_residual(residual: Option<!>) -> Self {
		match residual {
			None => Working,
		}
	}
}

impl<R, E> std::ops::FromResidual<Result<std::convert::Infallible, E>> for StateResult<R, E> {
	fn from_residual(residual: Result<std::convert::Infallible, E>) -> Self {
		match residual {
			Err(e) => Failed(e),
		}
	}
}

impl<R, E> std::ops::FromResidual<!> for StateResult<R, E> {
	fn from_residual(_: !) -> Self {
		Working
	}
}

pub trait State: Clone + serde::Serialize {
	type Error: Copy;
	type Return: Copy;
	fn run(&mut self, creep: &Creep, data: &mut CreepData) -> StateResult<Self::Return, Self::Error>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct StateIdle(u32);

impl State for StateIdle {
	type Error = !;
	type Return = !;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		// if self.0 <= game::time() {
			ign!(creep.say("Idling...", true));
			ign!(creep.move_direction(screeps::Direction::Top.multi_rot(fastrand::i8(..))));

			// self.0 = game::time() + fastrand::u32(5..20);
		// }

		Working
	}
}
