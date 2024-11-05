#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use screeps::{prelude::*, find, game, Room, StructureSpawn};
use vecmap::VecMap;
use wasm_bindgen::prelude::*;

pub mod logging;
pub mod utils;
pub mod cmds;
pub mod htn_vm;
pub mod memory;
pub mod parser;

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
	let global_memory = memory::get_memory();
	log::trace!("Spent {} CPU on memory access", screeps::game::cpu::get_used() - cpu);

	let spawns = game::spawns().values();
	let owned_rooms: VecMap<Room, Vec<StructureSpawn>> = spawns.fold(VecMap::new(), |mut acc, spawn| {
		let room = spawn.room().expect("Spawner has no Room!");
		acc.entry(room).or_default().push(spawn);
		acc
	});

	let creeps = game::creeps_jsstring();

	for (room, spawns) in owned_rooms {
		log::trace!("Handling room: {} with spawns: {}", room.name(), spawns.iter().map(|s| s.name()).collect::<Vec<_>>().join(", "));

		let sources = room.find(find::SOURCES_ACTIVE, None);
		let controller = room.controller();
		let structures = room.find(find::MY_STRUCTURES, None);

		let energy_containers = structures.iter().filter_map(|s| s.as_transferable());

		for cont in energy_containers {
			let creep_obj: &js_sys::Object = creeps.values().next().unwrap().dyn_ref().unwrap();

			// creep_obj
		}

		let sites = room.find(find::MY_CONSTRUCTION_SITES, None);


	}

	log::debug!("CPU used during tick: {}", screeps::game::cpu::get_used() - total_cpu);
}
