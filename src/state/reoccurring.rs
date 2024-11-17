use super::*;

pub trait CheckFunc<S: State>: serde::Serialize + Clone {
	fn check(&mut self, return_val: S::Return, state: &S) -> bool;
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DefaultCheck;

impl<S: State> CheckFunc<S> for DefaultCheck {
	#[inline(always)]
	fn check(&mut self, _return_val: <S as State>::Return, _state: &S) -> bool {
		true
	}
}

impl<S: State, F: FnMut(S::Return, &S) -> bool + Clone + serde::Serialize> CheckFunc<S> for F {
	fn check(&mut self, return_val: S::Return, state: &S) -> bool {
		self(return_val, state)
	}
}

// impl<S: State> CheckFunc<S> for fn(S::Return, &S) -> bool {
// 	fn check(&mut self, return_val: S::Return, state: &S) -> bool {
// 		self(return_val, state)
// 	}
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateReoccurring<S: State, C = DefaultCheck> {
	state: S,
	
	check: C,
	#[serde(skip)]
	last_return: Option<S::Return>,
}

impl<S: State, C: CheckFunc<S>> StateReoccurring<S, C> {
	pub fn new(state: S, check: C) -> Self {
		Self { state, check, last_return: None }
	}

	/// Get the last return value from the state.  
	/// This is only present for the tick 'run' was called on, and will
	/// always return None after being deserialized and before being run.
	pub fn last_return(&self) -> Option<S::Return> {
		self.last_return.as_ref().copied()
	}
}

impl<S: State + Default, C: CheckFunc<S> + Default> Default for StateReoccurring<S, C> {
	fn default() -> Self {
		Self::new(S::default(), C::default())
	}
}

impl<S: State, C: CheckFunc<S>> State for StateReoccurring<S, C> {
	type Error = S::Error;
	type Return = S::Return;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		match self.state.run(creep, _data) {
			Working => Working,
			Finished(r) => {
				if self.check.check(r, &self.state) {
					self.last_return = Some(r);
					log::debug!("Reoccurring state looping");
					Working
				} else {
					log::debug!("Reoccurring state finished");
					Finished(r)
				}
			},
			Failed(e) => Failed(e),
		}
	}
}

pub trait StateReoccurringExt: State {
	fn reoccurring(self) -> StateReoccurring<Self> {
		StateReoccurring::new(self, DefaultCheck)
	}

	fn reoccurring_cond<C>(self, check: C) -> StateReoccurring<Self, C> where Self: State, C: CheckFunc<Self> {
		StateReoccurring::new(self, check)
	}
}

impl<S: State> StateReoccurringExt for S { }

#[macro_export]
macro_rules! reoccurring_check {
	($name:ident, $state:ident, |$v_self:ident, $v_ret:ident, $v_state:ident| $expr:expr) => {
		#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
		pub struct $name;
		impl CheckFunc<$state> for $name {
			fn check($v_self: &mut Self, $v_ret: <$state as State>::Return, $v_state: &$state) -> bool {
				$expr
			}
		}
	};

	($name:ident, $state:ident, |$v_ret:ident, state| $expr:expr) => {
		#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
		pub struct $name;
		impl CheckFunc<$state> for $name {
			fn check(&mut self, $v_ret: <$state as State>::Return, state: &$state) -> bool {
				$expr
			}
		}
	};

	($name:ident, $state:ident, |self, $v_ret:ident| $expr:expr) => {
		#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
		pub struct $name;
		impl CheckFunc<$state> for $name {
			fn check(&mut self, $v_ret: <$state as State>::Return, _state: &$state) -> bool {
				$expr
			}
		}
	};

	($name:ident, $state:ident, |$v_ret:ident| $expr:expr) => {
		#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
		pub struct $name;
		impl CheckFunc<$state> for $name {
			fn check(&mut self, $v_ret: <$state as State>::Return, _state: &$state) -> bool {
				$expr
			}
		}
	};
}
