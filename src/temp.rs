use std::cell::RefCell;
use base64::Engine;
use screeps::{prelude::*, ConstructionSite, Creep, Direction, ObjectId, Position, StructureSpawn};
use screeps::{find, HasPosition, Part, ResourceType, SharedCreepProperties, Source, SpawnOptions};
use vecmap::VecMap;

use crate::creep_dispatch::DispatchContext;
use crate::utils::IterEnum;

use super::{handle_err, handle_warn};

type CreepId = ObjectId<Creep>;
type SourceId = ObjectId<Source>;
type ConstructId = ObjectId<ConstructionSite>;

type MemData = VecMap<CreepId, CreepData>;

thread_local! {
	static NAME_COUNT: RefCell<u128> = const { RefCell::new(0) };
	pub static CREEP_COUNT: RefCell<usize> = const { RefCell::new(0) };
}

fn get_new_creep_name() -> String {
	NAME_COUNT.with(|count| {
		let mut count = count.borrow_mut();
		let name = count.to_string();
		*count += 1;
		name
	})
}

pub fn tick() {
	let data = base64::prelude::BASE64_STANDARD.decode(
		screeps::raw_memory::get().as_string().unwrap()
	).unwrap_or_else(|err| {
		log::error!("Failed to decode memory: '{err}'. Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		bitcode::serialize(&MemData::new()).unwrap()
	});

	let mut data = if let Ok(data) = bitcode::deserialize(&data) {
		data
	} else {
		log::error!("Memory not valid Bitcode! Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		MemData::new()
	};

	let spawn = screeps::game::spawns().values().next().expect("No spawns!");

	let creeps = screeps::game::creeps();
	CREEP_COUNT.replace(creeps.keys().count());
	let count = CREEP_COUNT.with(|count| *count.borrow());

	if count < 4 {
		handle_warn!(spawn.spawn_creep_with_options(&[ Part::Move, Part::Move, Part::Carry, Part::Work ], &get_new_creep_name(), &SpawnOptions::default()));
	}

	let mut dispatch = DispatchContext::new(count);

	for creep in creeps.values() {
		if fastrand::u8(..=50) == 50 { handle_err!(creep.say("uwu", true)) }

		let data = if data.contains_key(&creep.try_id().unwrap()) {
			data.get_mut(&creep.try_id().unwrap()).unwrap()
		} else {
			data.insert(creep.try_id().unwrap(), CreepData {
				duties: vec![ Duty::Harvest, Duty::Build, Duty::Repair ],
				idle_pos: spawn.pos().checked_add_direction(Direction::Top.multi_rot(fastrand::i8(..))).expect("OUT OF BOUNDS"),
				target: None,
				spawn: Some(spawn.clone()),
			});

			data.get_mut(&creep.try_id().unwrap()).unwrap()
		};

		dispatch.dispatch_creep(creep, data);
	}

	screeps::raw_memory::set(&base64::prelude::BASE64_STANDARD.encode(bitcode::serialize(&data).unwrap()).into());
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreepData {
	pub duties: Vec<Duty>,
	pub target: Option<Target>,
	pub idle_pos: Position,
	#[serde(with = "deser_struct")]
	pub spawn: Option<StructureSpawn>,
}

mod deser_struct {
	use super::*;
	use serde::{Serialize, Deserialize};

	pub fn serialize<S>(data: &Option<StructureSpawn>, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		data.as_ref().map(|data| data.id()).serialize(serializer)
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<StructureSpawn>, D::Error> where D: serde::Deserializer<'de> {
		let id = Option::<ObjectId<StructureSpawn>>::deserialize(deserializer)?;
		Ok(id.map(|id| id.resolve().unwrap()))
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Target {
	Source(SourceId),
	// ConstructionSite(ConstructId),
}

// impl Target {
// 	fn position<T>(&self) -> Option<T> {
// 		match self {
// 			Self::Source(id) => id.resolve(),
// 			Self::ConstructionSite(id) => id.resolve(),
// 		}
// 	}
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, macros::IterEnum, serde::Serialize, serde::Deserialize)]
pub enum Duty {
	Repair,
	Build,
	Harvest,
}

impl Duty {
	pub fn get_ratio(&self) -> usize {
		match self {
			Self::Build => 2,
			Self::Harvest => 10,
			Self::Repair => 5,
		}
	}

	pub fn get_ratios() -> VecMap<Self, usize> {
		Self::variants().iter().map(
			|duty| (*duty, duty.get_ratio())
		).collect()
	}
}
