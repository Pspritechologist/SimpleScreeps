#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use screeps::{prelude::*, find, game, Room, StructureSpawn};
use vecmap::VecMap;
use wasm_bindgen::prelude::*;

pub mod logging;
pub mod utils;
pub mod cmds;
pub mod memory;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

macro_rules! conte {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(e) => continue,
		}
	};
}

macro_rules! conto {
	($e:expr) => {
		match $e {
			Some(v) => v,
			None => continue,
		}
	};
}

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Debug);
	});

	let total_cpu = screeps::game::cpu::get_used();

	let cpu = screeps::game::cpu::get_used();
	let mut global_memory = memory::get_memory();
	log::trace!("Spent {} CPU on memory access", screeps::game::cpu::get_used() - cpu);

	let creeps = game::creeps();

	for creep in creeps.values() {
		let vm = global_memory.creep_data.entry(conto!(creep.try_id())).or_insert_with(get_vm);

		log::debug!("Got a VM with the following blackboard:");
		log::debug!("{:?}", vm.blackboard);
		let creep_obj: &wasm_bindgen::JsValue = creep.as_ref();

		// Set up tick blackboard entries
		vm.blackboard.insert("self".to_string(), creep_obj.clone().dyn_into().unwrap());
		vm.blackboard.insert("log".to_string(), js_sys::Reflect::get(&js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("console")).unwrap(), &JsValue::from_str("log")).unwrap().into());
		vm.blackboard.insert("game".to_string(), js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("Game")).unwrap().into());

		match vm.execute() {
			Ok(state) => log::info!("Creep {} finished with state {:?}", creep.name(), state),
			Err(e) => log::error!("Creep {} errored while executing: {:?}", creep.name(), e),
		}

		// Remove tick blackboard entries
		vm.blackboard.remove("self");
		vm.blackboard.remove("log");
		vm.blackboard.remove("game");

		log::debug!("Finished Creep with blackboard: {:?}", vm.blackboard);
	}

	memory::set_memory(global_memory);

	log::debug!("CPU used during tick: {}", screeps::game::cpu::get_used() - total_cpu);
}
