use base64::Engine;
use macros::ratios;
use screeps::{prelude::*, ConstructionSite, Creep, Direction, ObjectId, Position, RoomName, Structure, StructureContainer, StructureController, StructureObject, StructureSpawn};
use screeps::{HasPosition, Part, Source, SpawnOptions};
use vecmap::VecMap;

use crate::creep_dispatch::DispatchContext;
use crate::utils::{chance, IterEnum};

use super::{handle_err, handle_warn};

type CreepId = ObjectId<Creep>;
type SourceId = ObjectId<Source>;
type ConstructId = ObjectId<ConstructionSite>;

pub type MemData = VecMap<CreepId, CreepData>;

pub fn tick() {
	let data = base64::prelude::BASE64_STANDARD_NO_PAD.decode(
		screeps::raw_memory::get().as_string().unwrap()
	).unwrap_or_else(|err| {
		log::error!("Failed to decode memory: '{err}'. Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		bitcode::serialize(&MemData::new()).unwrap()
	});

	let mut global_memory = if let Ok(data) = bitcode::deserialize(&data) {
		data
	} else {
		log::error!("Memory not valid Bitcode! Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		MemData::new()
	};

	js_sys::Reflect::delete_property(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory")).unwrap();
	js_sys::Reflect::set(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory"), &js_sys::Object::default().into()).unwrap();

	let spawn = screeps::game::spawns().values().next().expect("No spawns!");

	let creeps = screeps::game::creeps();
	let count = creeps.keys().count();

	if count < 12 || super::UNIQUE_QUEUE.with(|queue| queue.borrow().len()) > 0 {
		handle_warn!(spawn.spawn_creep_with_options(&[ Part::Move, Part::Move, Part::Carry, Part::Work ], &super::get_new_creep_name(&creeps.keys().collect::<Vec<_>>()), &SpawnOptions::default()));
	}

	let mut dispatch = DispatchContext::new(count);

	for creep in creeps.values() {
		if chance(1) { handle_err!(creep.say(fastrand::choice(crate::QUOTES).unwrap(), true)) }

		let Some(id) = creep.try_id() else { continue };

		let data = if global_memory.contains_key(&id) {
			global_memory.get_mut(&id).unwrap()
		} else {
			if let Some(task) = super::UNIQUE_QUEUE.with(|queue| queue.borrow_mut().pop()) {
				log::info!("Assigning unique task to creep: {:?}", task);

				let (task_data, _) = crate::unique_tasks::handle_task(task);
				global_memory.insert(id, task_data);
				
				global_memory.get_mut(&id).unwrap()
			} else {
				global_memory.insert(id, CreepData {
					duties: vec![ Duty::Harvest, Duty::Build, Duty::Repair, Duty::Upgrade ],
					current_task: None,
					unique_task: None,
					idle_pos: spawn.pos().checked_add_direction(Direction::Top.multi_rot(fastrand::i8(..))).expect("OUT OF BOUNDS"),
					target: None,
					spawn: Some(spawn.clone()),
				});

				global_memory.get_mut(&id).unwrap()
			}
		};

		dispatch.dispatch_creep(&creep, data);
	}

	// Log unfulfilled duties.
	// for (duty, count) in dispatch.get_fulfilment().iter() {
	// 	if *count == 0 { continue }
	// 	log::warn!("Duty {:?} was unfulfilled: {}", duty, count);
	// }

	screeps::raw_memory::set(&base64::prelude::BASE64_STANDARD_NO_PAD.encode(bitcode::serialize(&global_memory).unwrap()).into());
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreepData {
	pub duties: Vec<Duty>,
	pub current_task: Option<Duty>,
	pub unique_task: Option<crate::unique_tasks::UniqueTask>,
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
		Ok(id.and_then(|id| id.resolve()))
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Target {
	Source(SourceId),
	ConstructionSite(ConstructId),
	Controller(ObjectId<StructureController>),
	EnergyStorage(ObjectId<Structure>),
	Position(Position),
	Room(RoomName),
}

// impl Target {
// 	fn position<T>(&self) -> Option<T> {
// 		match self {
// 			Self::Source(id) => id.resolve(),
// 			Self::ConstructionSite(id) => id.resolve(),
// 		}
// 	}
// }

#[ratios]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, macros::IterEnum, serde::Serialize, serde::Deserialize)]
pub enum Duty {
	#[ratio(0)]
	Repair,
	#[ratio(0)]
	Build,
	#[ratio(4)]
	Harvest,
	#[ratio(7)]
	Upgrade,
}

impl Duty {
	pub fn get_ratios() -> VecMap<Self, usize> {
		Self::variants().iter().map(
			|duty| (*duty, duty.get_ratio())
		).collect()
	}
}
