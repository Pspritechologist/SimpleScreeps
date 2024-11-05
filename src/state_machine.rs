use serde::{Deserialize, Serialize};
pub use StateState::*;

use screeps::{ObjectId, Source, StructureController, TransferableObject};
use crate::memory::MemData;

#[derive(Debug, Deserialize, Serialize)]
pub enum State {
	Harvesting(Harvesting),
	Upgrading(Upgrading),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Harvesting {
	source: ObjectId<Source>,
	dest: ObjectId<TransferableObject>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Upgrading {
	controller: ObjectId<StructureController>,
	sources: Vec<ObjectId<Source>>,
}

impl IState for State {
	fn on_enter(&mut self, data: &mut MemData) -> StateState {
		use State::*;
		match self {
			Harvesting(h) => h.on_enter(data),
			Upgrading(u) => u.on_enter(data),
		}
	}

	fn on_exit(&mut self, data: &mut MemData) {
		use State::*;
		match self {
			Harvesting(h) => h.on_exit(data),
			Upgrading(u) => u.on_exit(data),
		}
	}

	fn tick(&mut self, data: &mut MemData) -> StateState {
		use State::*;
		match self {
			Harvesting(h) => h.tick(data),
			Upgrading(u) => u.tick(data),
		}
	}
}

pub trait IState: Serialize + Deserialize<'static> {
	fn on_enter(&mut self, _data: &mut MemData) -> StateState { Working }
	fn on_exit(&mut self, _data: &mut MemData) { }
	fn tick(&mut self, data: &mut MemData) -> StateState;
}

pub enum Job {
	Harvest(Source),
	Upgrade(StructureController),
}

pub enum StateState {
	Working,
	Failed(Option<String>),
	Completed,
}

impl IState for Harvesting {
	fn on_enter(&mut self, _data: &mut MemData) -> StateState {
		Working
	}

	fn tick(&mut self, _data: &mut MemData) -> StateState {
		Working
	}

	fn on_exit(&mut self, _data: &mut MemData) {
		
	}
}

impl IState for Upgrading {
	fn on_enter(&mut self, _data: &mut MemData) -> StateState {
		Working
	}

	fn tick(&mut self, _data: &mut MemData) -> StateState {
		Working
	}

	fn on_exit(&mut self, _data: &mut MemData) {
		
	}
}
