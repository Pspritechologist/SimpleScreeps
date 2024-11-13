#![feature(try_blocks)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(let_chains)]

#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

pub mod logging;
pub mod utils;
pub mod cmds;
pub mod memory;
pub mod state;
pub mod dynamic_stuff;

use std::ops::Div;

use screeps::ConstructionSite;
use wasm_bindgen::prelude::*;
use utils::prelude::*;
use dynamic_stuff::DynState;
use state::{RoomObjectId, StateResult};

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

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

		let sources = room.find(screeps::find::SOURCES_ACTIVE, None);
		let mut room_jobs = Vec::with_capacity(sources.len() * 2);

		if let Some(spawn_room) = &spawn_room && room == *spawn_room && let Some(controller) = spawn_room.controller() {
			for _ in 0..(creep_count as f32).div(2.).ceil() as u32 {
				room_jobs.push(Job::Upgrade(controller.clone()));
			}
		}

		let sites = room.find(screeps::find::MY_CONSTRUCTION_SITES, None);
		for site in sites {
			room_jobs.push(Job::Construct(site));
		}

		let mut terrain = room.get_terrain();
		// let mut tick = false;
		for source in sources {
			let pos: Position = source.pos();
			for dir in screeps::Direction::iter() {
				if let Ok(pos) = pos.checked_add_direction(*dir) && terrain.get_xy(pos.xy()) != screeps::Terrain::Wall {
					room_jobs.push(Job::Harvest(source.clone(), spawn.clone().into()));
					room_jobs.push(Job::Harvest(source.clone(), spawn.clone().into()));
				}
			}
		}

		log::trace!("Spent {} CPU on room jobs", screeps::game::cpu::get_used() - job_cpu);
		let creep_cpu = screeps::game::cpu::get_used();

		for creep in room.find(screeps::find::MY_CREEPS, None) {
			let Some(mut creep_data) = global_memory.creep_data.get_mut(&conto!(creep.try_id())) else {
				global_memory.creep_data.insert(conto!(creep.try_id()), Default::default());
				creep_queue.push(creep);
				continue;
			};
			
			let Some((job, mut state)) = creep_data.current_task.take() else {
				creep_queue.push(creep);
				continue;
			};

			if job.job == JobDiscriminants::Idle {
				creep_queue.push(creep);
				continue;
			}

			if let Some(i) = room_jobs.iter().position(|id| job.job ==  id.into()) {
				room_jobs.remove(i);
			}

			let next_task = match state.state.run(&creep, &mut creep_data) {
				StateResult::Working => {
					(job, state)
				}
				StateResult::Finished(r) => {
					ign!(creep.say(":)", true));
					log::info!("Creep {} finished task {:?} - {r:?}", creep.name(), state.flag);
					creep_queue.push(creep);
					new_idle()
				}
				StateResult::Failed(e) => {
					ign!(creep.say(":(", true));
					log::warn!("Creep {} failed to complete task: {:?} - {e:?}", creep.name(), state.flag);
					creep_queue.push(creep);
					new_idle()
				}
			};

			creep_data.current_task = Some(next_task);
		}

		log::trace!("Spent {} CPU on creeps", screeps::game::cpu::get_used() - creep_cpu);

		let queue_cpu = screeps::game::cpu::get_used();

		fastrand::shuffle(&mut room_jobs);
		for creep in creep_queue.drain(..) {
			// We ensure each Creep has a data entry above.
			let creep_data = global_memory.creep_data.get_mut(&conto!(creep.try_id())).unwrap();
			let Some(job) = room_jobs.pop() else {
				creep_data.current_task = Some(new_idle());
				continue;
			};

			log::info!("Creep {} assigned to job {:?}", creep.name(), JobDiscriminants::from(&job));

			// creep_data.current_task = Some(state::harvester_graph::HarvesterStateGraph::new(&creep, job.id.into_type(), AsRef::<Structure>::as_ref(&spawn).id(), job.id.into_type()).into());
			creep_data.current_task = Some(match job {
				Job::Idle => unreachable!(),
				Job::Harvest(ref source, ref target) => {
					let state = state::harvester::StateHarvesterJob::new(&creep, target.id(), source.id());
					(job.into(), DynState::new(state, dynamic_stuff::StateFlag::HarvesterJob))
				}
				Job::Upgrade(ref controller) => {
					let state = state::upgrader::StateUpgraderJob::new(&creep, controller.id(), spawn.id().into_type());
					(job.into(), DynState::new(state, dynamic_stuff::StateFlag::UpgraderJob))
				}
				Job::Construct(ref site) => {
					let state = state::builder::StateBuilderJob::new(&creep, site.try_id().expect("Construction site doesn't have an ID"), spawn.id().into_type());
					(job.into(), DynState::new(state, dynamic_stuff::StateFlag::BuilderJob))
				}
			});
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

	let cpu = screeps::game::cpu::get_used();
	memory::set_memory(&global_memory);
	log::trace!("Spent {} CPU on memory save", screeps::game::cpu::get_used() - cpu);

	log::debug!("CPU used during tick: {}", screeps::game::cpu::get_used() - total_cpu);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JobIdentifier {
	job: JobDiscriminants,
	id: RoomObjectId,
}

impl From<Job> for JobIdentifier {
	fn from(job: Job) -> Self {
		Self {
			id: match job {
				Job::Idle => RoomObjectId::from_packed(0),
				Job::Harvest(ref source, _) => source.id().into_type(),
				Job::Upgrade(ref controller) => controller.id().into_type(),
				Job::Construct(ref site) => site.try_id().expect("Construction site doesn't have an ID").into_type(),
			},
			job: job.into(),
		}
	}
}

fn new_idle() -> (JobIdentifier, DynState) {
	(JobIdentifier {
		job: JobDiscriminants::Idle,
		id: RoomObjectId::from_packed(0),
	},
	DynState::new(state::StateIdle::default(), dynamic_stuff::StateFlag::Idle))
}

#[derive(strum::EnumDiscriminants, Clone, Debug)]
#[strum_discriminants(derive(serde::Serialize, serde::Deserialize))]
enum Job {
	Harvest(Source, Structure),
	Upgrade(StructureController),
	Construct(ConstructionSite),
	//? Is kinda used as a fallback tag? It's not real.
	#[allow(dead_code)] Idle,
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
