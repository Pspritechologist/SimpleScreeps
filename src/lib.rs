#![feature(try_blocks)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(let_chains)]

#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use screeps::{find::{MY_CREEPS, SOURCES_ACTIVE}, game, Direction, HasId, HasPosition, MaybeHasId, Position, RoomObject, SharedCreepProperties, Structure, StructureController, Terrain};
use state::{RoomObjectId, State, StateResult};
use wasm_bindgen::prelude::*;

pub mod logging;
pub mod utils;
pub mod cmds;
pub mod memory;
pub mod state;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum JobEnum {
	Harvester(state::harvester_graph::HarvesterStateGraph),
	Upgrader(state::upgrader_graph::UpgraderStateGraph),
	Idle(state::StateIdle),
}
impl From<state::harvester_graph::HarvesterStateGraph> for JobEnum {
	fn from(job: state::harvester_graph::HarvesterStateGraph) -> Self {
		JobEnum::Harvester(job)
	}
}
impl From<state::upgrader_graph::UpgraderStateGraph> for JobEnum {
	fn from(job: state::upgrader_graph::UpgraderStateGraph) -> Self {
		JobEnum::Upgrader(job)
	}
}
impl From<state::StateIdle> for JobEnum {
	fn from(job: state::StateIdle) -> Self {
		JobEnum::Idle(job)
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Job {
	job: JobFlag,
	id: RoomObjectId,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum JobFlag {
	Harvest,
	Upgrade,
}

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Trace);
	});

	let total_cpu = screeps::game::cpu::get_used();

	let cpu = screeps::game::cpu::get_used();
	let mut global_memory = memory::get_memory();
	log::trace!("Spent {} CPU on memory access", screeps::game::cpu::get_used() - cpu);

	let spawn = game::spawns().values().next().unwrap();
	let spawn_room = spawn.room();

	let creep_count = game::creeps().keys().count();
	
	// Holds Creeps without jobs, to be arranged after all other Creeps are dispatched.
	let mut creep_queue = Vec::new();

	let room_cpu = screeps::game::cpu::get_used();

	for room in game::rooms().values() {
		let job_cpu = screeps::game::cpu::get_used();

		let sources = room.find(SOURCES_ACTIVE, None);
		let mut room_jobs = Vec::with_capacity(sources.len() * 2);

		if let Some(spawn_room) = &spawn_room && room == *spawn_room && let Some(controller) = spawn_room.controller() {
			for _ in 0..creep_count.div_ceil(2) {
				room_jobs.push(Job {
					job: JobFlag::Upgrade,
					id: controller.id().into_type(),
				});
			}
		}

		let mut terrain = room.get_terrain();
		// let mut tick = false;
		for source in sources {
			let pos: Position = source.pos();
			for dir in Direction::iter() {
				if let Ok(pos) = pos.checked_add_direction(*dir) && terrain.get_xy(pos.xy()) != Terrain::Wall {
					room_jobs.push(Job {
						job: JobFlag::Harvest,
						id: source.id().into_type(),
					});
					// if tick {
						room_jobs.push(Job {
							job: JobFlag::Harvest,
							id: source.id().into_type(),
						});
					// }
					// tick = !tick;
				}
			}
		}

		log::trace!("Spent {} CPU on room jobs", screeps::game::cpu::get_used() - job_cpu);
		let creep_cpu = screeps::game::cpu::get_used();

		for creep in room.find(MY_CREEPS, None) {
			let Some(mut creep_data) = global_memory.creep_data.get_mut(&conto!(creep.try_id())) else {
				global_memory.creep_data.insert(conto!(creep.try_id()), Default::default());
				creep_queue.push(creep);
				continue;
			};
			
			let Some(task) = creep_data.current_task.take() else {
				creep_queue.push(creep);
				continue;
			};
			
			let next_task: JobEnum = match task {
				JobEnum::Harvester(mut graph) => {
					if let Some(i) = room_jobs.iter().position(|id| id.id == graph.job.into_type()) {
						room_jobs.remove(i);
					}

					match graph.run(&creep, &mut creep_data) {
						StateResult::Working => graph.into(),
						StateResult::Finished(_) => {
							log::info!("Creep {} finished Harvest task", creep.name());
							creep_queue.push(creep);
							state::StateIdle::default().into()
						}
						StateResult::Failed(e) => {
							ign!(creep.say("harvest :(", true));
							log::warn!("Creep {} failed to complete Harvest task: {:?}", creep.name(), e);
							creep_queue.push(creep);
							state::StateIdle::default().into()
						}
					}
				},
				JobEnum::Upgrader(mut graph) => {
					if let Some(i) = room_jobs.iter().position(|id| id.id == graph.job.into_type()) {
						room_jobs.remove(i);
					}

					match graph.run(&creep, &mut creep_data) {
						StateResult::Working => graph.into(),
						StateResult::Finished(_) => {
							log::info!("Creep {} finished Upgrade task", creep.name());
							creep_queue.push(creep);
							state::StateIdle::default().into()
						}
						StateResult::Failed(e) => {
							ign!(creep.say("upgrade :(", true));
							log::warn!("Creep {} failed to complete Upgrade task: {:?}", creep.name(), e);
							creep_queue.push(creep);
							state::StateIdle::default().into()
						}
					}
				}
				JobEnum::Idle(_) => {
					creep_queue.push(creep);
					continue;
				}
			};

			creep_data.current_task = Some(next_task);
		}

		log::trace!("Spent {} CPU on creeps", screeps::game::cpu::get_used() - creep_cpu);

		let queue_cpu = screeps::game::cpu::get_used();

		for creep in creep_queue.drain(..) {
			// We ensure each Creep has a data entry above.
			let creep_data = global_memory.creep_data.get_mut(&conto!(creep.try_id())).unwrap();
			let Some(job) = room_jobs.pop() else {
				creep_data.current_task = Some(state::StateIdle::default().into());
				continue;
			};

			// creep_data.current_task = Some(state::harvester_graph::HarvesterStateGraph::new(&creep, job.id.into_type(), AsRef::<Structure>::as_ref(&spawn).id(), job.id.into_type()).into());
			creep_data.current_task = Some(match job.job {
				JobFlag::Harvest => state::harvester_graph::HarvesterStateGraph::new(&creep, job.id.into_type(), AsRef::<Structure>::as_ref(&spawn).id(), job.id.into_type()).into(),
				JobFlag::Upgrade => state::upgrader_graph::UpgraderStateGraph::new(&creep, job.id.into_type(), job.id.into_type(), AsRef::<Structure>::as_ref(&spawn).id()).into(),
			});

			log::info!("Creep {} assigned to job {:?}", creep.name(), job.job);
		}

		let mut used: Vec<_> = screeps::game::creeps().keys().collect();

		if !room_jobs.is_empty() {
			use screeps::Part::*;
			let name = utils::get_new_creep_name(&used);
			ign!(spawn.spawn_creep(&[Move, Move, Work, Carry, Carry], &utils::generate_name()));
			used.push(name);
		}

		log::trace!("Spent {} CPU on creep queue", screeps::game::cpu::get_used() - queue_cpu);

		log::trace!("Room {} used {} CPU", room.name(), screeps::game::cpu::get_used() - room_cpu);
	}

	memory::set_memory(&global_memory);

	screeps::raw_memory::set_active_segments(&[0]);
	screeps::raw_memory::segments().set(0, "".to_string());
	// screeps::raw_memory::segments().keys().for_each(|key| {
	// 	screeps::raw_memory::segments().set(key, );
	// });

	log::debug!("CPU used during tick: {}", screeps::game::cpu::get_used() - total_cpu);
}

// #[derive(tabled::Tabled)]
// struct MemoryDisplay {

// }

// #[derive(tabled::Tabled)]
// struct CreepDisplay {
// 	name: String,
// 	#[tabled(rename = "Task")]
// 	current_task: JobEnum,

// }
