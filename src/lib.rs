#![feature(try_blocks)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(let_chains)]
#![feature(trait_alias)]

// #![allow(clippy::collapsible_if)]
// #![allow(clippy::collapsible_else_if)]

pub(crate) mod logging;
pub(crate) mod utils;
pub(crate) mod cmds;
pub(crate) mod memory;
pub(crate) mod state;
pub(crate) mod dynamic_stuff;
pub mod quotes;

use wasm_bindgen::prelude::*;
use utils::prelude::*;
use state::{seppuku::StateSeppuku, StateResult};
use dynamic_stuff::DynState;
use screeps::ConstructionSite;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Info);
	});

	let total_cpu = screeps::game::cpu::get_used();

	let cpu = screeps::game::cpu::get_used();
	let mut global_memory = memory::get_memory();
	log::trace!("Spent {} CPU on memory access", screeps::game::cpu::get_used() - cpu);

	let spawn = game::spawns().values().next().unwrap();
	let spawn_room = spawn.room();

	let mut creep_count = game::creeps().keys().count();
	
	// Holds Creeps without jobs, to be arranged after all other Creeps are dispatched.
	let mut creep_queue = Vec::new();

	let room_cpu = screeps::game::cpu::get_used();

	for room in game::rooms().values() {
		let job_cpu = screeps::game::cpu::get_used();

		let sources = room.find(screeps::find::SOURCES_ACTIVE, None);
		let mut room_jobs = Vec::with_capacity(sources.len() * 2);

		// This is primarily based on the number of 'constant' jobs such as upgrading and harvesting.
		// This number acts as a baseline for other things.
		// It is slightly inflated by things such as redundant harvesters and a large
		// number of upgraders. This allows for allocation of builders and the like without
		// factoring them into overall population. At the end of job allocation, this number
		// is used to determine whether or not to spawn additional Creeps.
		let mut desired_pop = 0;
		
		// Construction jobs.
		let sites = room.find(screeps::find::MY_CONSTRUCTION_SITES, None);
		for site in sites {
			// This value was determined by extensive testing and heavy
			// deliberation over multiple months by a panel of experts.
			let requsted_creeps = (site.progress_total() - site.progress()).div_ceil(500);
			(0..requsted_creeps).for_each(|i| {
				room_jobs.push(JobInstance {
					id: site.try_id().expect("Construction site doesn't have an ID").into_type(),
					egg: JobEgg::Construct(site.clone()),
					priority: 170 - i.min(160) as u8 * 13,
				});

				desired_pop += 1;
			});
		}

		// Harvester jobs.
		let mut harvester_jobs = 0u32;
		let mut terrain = room.get_terrain();
		let spawn_near_full = spawn.store().get_free_capacity(None) <= 150;
		for source in sources {
			let pos: Position = source.pos();
			let mut valid_dir_count = 0u8;
			for dir in screeps::Direction::iter() {
				if let Ok(pos) = pos.checked_add_direction(*dir) && terrain.get_xy(pos.xy()) != screeps::Terrain::Wall {
					let instance = JobInstance {
						id: source.id().into_type(),
						egg: JobEgg::Harvest(source.clone(), spawn.clone().into()),
						priority: 250 - valid_dir_count * 30,
					};
					let second_instance = JobInstance {
						priority: 120 - valid_dir_count * 30,
						..instance.clone()
					};
					room_jobs.push(instance);
					if spawn_near_full {
						room_jobs.push(second_instance);
					}

					harvester_jobs += 1;
					valid_dir_count += 1;

					desired_pop += 2;
				}
			}
		}

		// Upgrader jobs.
		if let Some(spawn_room) = &spawn_room && room == *spawn_room && let Some(controller) = spawn_room.controller() {
			for i in 0..harvester_jobs.div_ceil(2).min(20) {
				room_jobs.push(JobInstance {
					id: controller.id().into_type(),
					egg: JobEgg::Upgrade(controller.clone()),
					priority: 200 - i.min(200) as u8 * 13,
				});

				desired_pop += 1;
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

			if job.job == JobFlag::Idle {
				creep_queue.push(creep);
				continue;
			}

			if job.job == JobFlag::Seppuku {
				log::debug!("Creep {} is seppukuing", creep.name());
				creep_count -= 1;
				continue;
			}

			if let Some(i) = room_jobs.iter().position(|id| job.id ==  id.id.into_type()) {
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

		room_jobs.sort_unstable_by_key(|j| j.priority);
		// fastrand::shuffle(&mut room_jobs);
		for creep in creep_queue.drain(..) {
			// We ensure each Creep has a data entry above.
			let creep_data = global_memory.creep_data.get_mut(&conto!(creep.try_id())).unwrap();

			let job = if let Some(to_live) = creep.ticks_to_live() && to_live < 120 {
				log::debug!("Assigning creep {} to seppuku", creep.name());
				JobInstance {
					egg: JobEgg::Seppuku,
					id: creep.try_id().unwrap().into_type(),
					priority: 255,
				}
			} else if let Some(job) = room_jobs.pop() {
				job
			} else {
				creep_data.current_task = Some(new_idle());
				continue;
			};

			log::info!("Creep {} assigned to job {:?}", creep.name(), JobFlag::from(&job.egg));

			// creep_data.current_task = Some(state::harvester_graph::HarvesterStateGraph::new(&creep, job.id.into_type(), AsRef::<Structure>::as_ref(&spawn).id(), job.id.into_type()).into());
			creep_data.current_task = Some(match job.egg {
				JobEgg::Idle => unreachable!(),
				JobEgg::Seppuku => {
					(job.egg.into(), DynState::new(StateSeppuku::new(&creep, &spawn), dynamic_stuff::StateFlag::Seppuku))
				}
				JobEgg::Harvest(ref source, ref target) => {
					let state = state::harvester::StateHarvesterJob::new(&creep, target.id(), source.id());
					(job.egg.into(), DynState::new(state, dynamic_stuff::StateFlag::HarvesterJob))
				}
				JobEgg::Upgrade(ref controller) => {
					let state = state::upgrader::StateUpgraderJob::new(&creep, controller.id(), spawn.id().into_type());
					(job.egg.into(), DynState::new(state, dynamic_stuff::StateFlag::UpgraderJob))
				}
				JobEgg::Construct(ref site) => {
					let state = state::builder::StateBuilderJob::new(&creep, site.try_id().expect("Construction site doesn't have an ID"), spawn.id().into_type());
					(job.egg.into(), DynState::new(state, dynamic_stuff::StateFlag::BuilderJob))
				}
			});
		}

		let mut used: Vec<_> = screeps::game::creeps().keys().collect();

		if creep_count < desired_pop {
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
pub(crate) struct JobIdentifier {
	job: JobFlag,
	id: RoomObjectId,
}

impl From<JobEgg> for JobIdentifier {
	fn from(job: JobEgg) -> Self {
		Self {
			id: match job {
				JobEgg::Idle => RoomObjectId::from_packed(0),
				JobEgg::Seppuku => RoomObjectId::from_packed(0),
				JobEgg::Harvest(ref source, _) => source.id().into_type(),
				JobEgg::Upgrade(ref controller) => controller.id().into_type(),
				JobEgg::Construct(ref site) => site.try_id().expect("Construction site doesn't have an ID").into_type(),
			},
			job: job.into(),
		}
	}
}

fn new_idle() -> (JobIdentifier, DynState) {
	(JobIdentifier {
		job: JobFlag::Idle,
		id: RoomObjectId::from_packed(0),
	},
	DynState::new(state::StateIdle::default(), dynamic_stuff::StateFlag::Idle))
}

#[derive(strum::EnumDiscriminants, Clone, Debug)]
#[strum_discriminants(name(JobFlag), derive(serde::Serialize, serde::Deserialize))]
enum JobEgg {
	Harvest(Source, Structure),
	Upgrade(StructureController),
	Construct(ConstructionSite),
	Seppuku,
	//? Is kinda used as a fallback tag? It's not real.
	#[allow(dead_code)] Idle,
}

#[derive(Clone, Debug)]
struct JobInstance {
	id: RoomObjectId,
	egg: JobEgg,
	priority: u8,
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
