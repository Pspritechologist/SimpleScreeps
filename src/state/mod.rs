pub mod harvester_graph;
pub mod upgrader_graph;

use wasm_bindgen::JsValue;
pub use StateResult::*;
pub use crate::utils::prelude::*;

use crate::{ign, memory::CreepData};
use screeps::{game, prelude::*, Creep, ErrorCode, ObjectId, Position, ResourceType, RoomXY, Source, Structure, StructureObject, TypedRoomObject };

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

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum GenericStateError {
	OutOfRange,
	TargetNotReal,
	NoParts,
	Unknown,
}

// pub struct TopLevelStateError(Box<dyn Error>);

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
pub struct StateTransfer {
	target: ObjectId<Structure>,
	resource: ResourceType,
	amount: Option<u32>,
}

impl StateTransfer {
	pub fn new(target: ObjectId<Structure>, resource: ResourceType, amount: Option<u32>) -> Self {
		Self { target, resource, amount }
	}

	pub fn new_object(target: Structure, resource: ResourceType, amount: Option<u32>) -> Self {
		Self { target: target.id(), resource, amount }
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum TransferError {
	OutOfRange,
	TargetNotReal,
	InvalidTarget,
	TargetFull,
	RanOut,
	Empty,
	Unknown,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum TransferReturn {
	Empty,
	Leftover,
	NotEnough,
}

impl State for StateTransfer {
	type Error = TransferError;
	type Return = TransferReturn;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if creep.store().get_used_capacity(Some(self.resource)) == 0 {
			return Failed(TransferError::Empty);
		}

		let Some(target) = self.target.resolve() else {
			return Failed(TransferError::TargetNotReal);
		};
		let target = StructureObject::from(target);
		let Some(target) = target.as_transferable() else {
			return Failed(TransferError::InvalidTarget);
		};

		if let Err(e) = creep.transfer(target, self.resource, self.amount) {
			match e {
				ErrorCode::Full => return Finished(TransferReturn::Leftover),
				ErrorCode::NotEnough => {
					ign!(creep.transfer(target, self.resource, None));
					return Finished(TransferReturn::NotEnough);
				}
				ErrorCode::NotInRange => return Failed(TransferError::OutOfRange),
				ErrorCode::InvalidTarget => return Failed(TransferError::InvalidTarget),
				_ => return Failed(TransferError::Unknown),
			}
		} else {
			return Finished(TransferReturn::Empty);
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateWithdraw {
	target: StructureId,
	resource: ResourceType,
	amount: Option<u32>,
}

impl StateWithdraw {
	pub fn new(target: ObjectId<Structure>, resource: ResourceType, amount: Option<u32>) -> Self {
		Self { target, resource, amount }
	}

	pub fn new_object(target: Structure, resource: ResourceType, amount: Option<u32>) -> Self {
		Self { target: target.id(), resource, amount }
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum WithdrawError {
	OutOfRange,
	TargetNotReal,
	InvalidTarget,
	NotEnoughCapacity,
	Unknown,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum WithdrawReturn {
	Full,
	Partial,
}

impl State for StateWithdraw {
	type Error = WithdrawError;
	type Return = WithdrawReturn;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if let Some(amnt) = self.amount && creep.store().get_free_capacity(Some(self.resource)) < amnt as i32 {
			return Failed(WithdrawError::NotEnoughCapacity);
		}

		let Some(target) = self.target.resolve() else {
			return Failed(WithdrawError::TargetNotReal);
		};
		let target = StructureObject::from(target);
		let Some(target) = target.as_withdrawable() else {
			return Failed(WithdrawError::InvalidTarget);
		};

		if let Err(e) = creep.withdraw(target, self.resource, self.amount) {
			match e {
				ErrorCode::Full => return Failed(WithdrawError::NotEnoughCapacity),
				ErrorCode::NotEnough => {
					ign!(creep.withdraw(target, self.resource, None));
					return Finished(WithdrawReturn::Partial);
				}
				ErrorCode::NotInRange => return Failed(WithdrawError::OutOfRange),
				ErrorCode::InvalidTarget => return Failed(WithdrawError::InvalidTarget),
				_ => return Failed(WithdrawError::Unknown),
			}
		} else {
			return Finished(WithdrawReturn::Full);
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateMove {
	destination: Position,
	path_cache: String,
	/// Zero means exact, 1 means 1 tile off, etc.  
	/// To interact with something at `target` range should be 1.
	range: u8,
	/// How often to recalculate the path, either in ticks or tiles moved.  
	/// If 0, never recalculate. 1 means every tick, 2 every other tick, etc.
	recalc_rate: u8,
	/// Accumulator for recalc_rate.
	recalc_accumulator: u8,
	/// Whether to recalculate the path per tile moved or per tick.
	/// If true, recalc_rate is in tiles, if false, recalc_rate is in ticks.
	recalc_per_tile: bool,
	/// If true, Creeps will only move as close as they need to.
	/// If false, Creeps will note when they moved close enough but continue
	/// to try and move closer until they cannot.  
	/// Creeps will attempt to 'move closer' until hitting their `recalc_rate` from when they get close enough.
	lazy: bool,
	reached_destination: bool,
}

impl StateMove {
	pub fn new_from_ends_close(start: impl HasPosition, end: impl HasPosition) -> Self {
		Self::new_from_ends(start, end, 1)
	}

	pub fn new_from_ends(start: impl HasPosition, end: impl HasPosition, range: u8) -> Self {
		let start: Position = start.pos();
		let end: Position = end.pos();
		let path = find_path(&start, &end);

		Self {
			path_cache: path,
			destination: end,
			range,
			recalc_rate: 4,
			recalc_accumulator: 0,
			recalc_per_tile: false,
			lazy: false,
			reached_destination: false,
		}
	}

	pub fn new_from_ends_lazy(start: impl HasPosition, end: impl HasPosition, range: u8) -> Self {
		Self {
			lazy: true,
			..Self::new_from_ends(start, end, range)
		}
	}
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum MoveError {
	NoMovePart,
	UnownedCreep,
	/// This means the Creep is not at the correct position on their path.  
	/// This usually happens because the Creep collided with something and was unable to move.
	OffPath,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Arrived;

impl State for StateMove {
	type Error = MoveError;
	type Return = Arrived;

	fn run(&mut self, creep: &Creep, _data: &mut CreepData) -> StateResult<Self::Return, Self::Error> {
		if self.reached_destination && !self.lazy && creep.pos().in_range_to(self.destination, 1) {
			return Finished(Arrived);
		}

		if !self.reached_destination && creep.pos().in_range_to(self.destination, self.range as u32) {
			if self.range <= 1 {
				return Finished(Arrived);
			}

			if self.lazy {
				return Finished(Arrived);
			} else {
				self.reached_destination = true;
				self.recalc_accumulator = 0;
			}
		}

		if self.recalc_rate != 0 {
			self.recalc_accumulator += 1;

			if self.recalc_accumulator >= self.recalc_rate {
				// Exit point for non-lazy Creeps.
				if self.reached_destination {
					return Finished(Arrived);
				}

				self.path_cache = find_path(creep, &self.destination);
				self.recalc_accumulator = 0;
			}
		}

		if let Err(e) = creep.move_by_path(&JsValue::from_str(&self.path_cache)) {
			match e {
				ErrorCode::Tired | ErrorCode::Busy => {},
				ErrorCode::NoBodypart => return Failed(MoveError::NoMovePart),
				ErrorCode::NotOwner => return Failed(MoveError::UnownedCreep),
				ErrorCode::NotFound => {
					self.path_cache = find_path(creep, &self.destination);
					match creep.move_by_path(&JsValue::from_str(&self.path_cache)) {
						Ok(_) | Err(ErrorCode::Tired) | Err(ErrorCode::Busy) => {},
						_ => return Failed(MoveError::OffPath),
					}
				},
				_ => unreachable!("Move should never return another error code."),
			}
		} else if self.recalc_rate > 0 && self.recalc_per_tile {
			self.recalc_accumulator += 1;
		}

		Working
	}
}

fn find_path(start: &impl HasPosition, end: &impl HasPosition) -> String {
	match start.pos().find_path_to::<_, _, screeps::pathfinder::SingleRoomCostResult>(&end.pos(), Some(screeps::FindPathOptions::default().serialize(true))) {
		screeps::Path::Serialized(path) => path,
		_ => unreachable!()
	}
}
