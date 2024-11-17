use std::str::FromStr;

use js_sys::JsString;
use screeps::Part;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::utils::{self, prelude::*};


// #[wasm_bindgen]
// pub fn cmd_write_contr(room: String, msg: String) -> Result<String, String> {
// 	let Ok(room) = RoomName::new(&room) else {
// 		return Err("Invalid room name".to_string());
// 	};

// 	crate::UNIQUE_QUEUE.with(|queue| {
// 		queue.borrow_mut().push(crate::QueuedUniqueTask::SignMessage(msg, room));
// 	});

// 	Ok("Queued command".to_string())
// }

#[wasm_bindgen]
pub fn cmd_wipe_memory() {
	screeps::raw_memory::set(&JsString::from_str("").unwrap());
}

#[wasm_bindgen]
pub fn cmd_set_log_level(level: String) -> Result<(), String> {
	let level = match level.to_lowercase().as_str() {
		"trace" => log::LevelFilter::Trace,
		"debug" => log::LevelFilter::Debug,
		"info" => log::LevelFilter::Info,
		"warn" => log::LevelFilter::Warn,
		"error" => log::LevelFilter::Error,
		"off" => log::LevelFilter::Off,
		_ => return Err("Invalid log level".to_string()),
	};

	log::set_max_level(level);

	Ok(())
}

#[wasm_bindgen]
pub fn cmd_spawn(spawn: JsString, body: Option<Vec<JsValue>>, name: Option<String>) -> Result<(), String> {
	let Some(spawn) = screeps::game::spawns_jsstring().get(spawn.clone()) else {
		return Err(format!("Spawn '{spawn}' not found"));
	};

	let body = if let Some(body) = body {
		&body.iter().map(Part::from_js_value).flatten().collect::<Vec<_>>()[..]
	} else {
		&[Part::Work, Part::Move, Part::Move, Part::Carry]
	};


	match spawn.spawn_creep(body, &name.as_ref().cloned().unwrap_or_else(utils::generate_name)) {
		Ok(_) => Ok(()),
		Err(err) => match err {
			screeps::ErrorCode::Busy => Err("Spawn is busy".to_string()),
			screeps::ErrorCode::NameExists => Err(format!("A Creep with name '{}' already exists", name.unwrap())),
			screeps::ErrorCode::NotEnough => Err("Not enough resources".to_string()),
			err => Err(format!("Unknown error: {err:?}")),
		}
	}
}

#[wasm_bindgen]
pub fn cmd_get_state(creep: JsString) -> Result<String, JsString> {
	let Some(creep) = screeps::game::creeps_jsstring().get(creep.clone()) else {
		return Err(format!("Creep '{creep}' not found").into());
	};

	let state = crate::memory::get_memory()
		.creep_data.remove(&creep.try_id().ok_or(JsString::from_str("Creep has no ID").unwrap())?)
		.ok_or(JsString::from_str("No entry for Creep").unwrap())?
		.current_task
		.ok_or(JsString::from_str("Creep has no current task").unwrap())?
		.1
		.state
		;

	Ok(format!("{:?}", state))
}
