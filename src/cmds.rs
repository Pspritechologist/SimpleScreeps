use base64::Engine;
use wasm_bindgen::prelude::wasm_bindgen;


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
	screeps::raw_memory::set(&base64::prelude::BASE64_STANDARD_NO_PAD.encode(
		bitcode::serialize(&crate::memory::MemData::default()).unwrap()
	).into());
}

#[wasm_bindgen]
pub fn cmd_set_log_level(level: String) -> Result<(), String> {
	let level = match level.to_lowercase().as_str() {
		"trace" => log::LevelFilter::Trace,
		"debug" => log::LevelFilter::Debug,
		"info" => log::LevelFilter::Info,
		"warn" => log::LevelFilter::Warn,
		"error" => log::LevelFilter::Error,
		_ => return Err("Invalid log level".to_string()),
	};

	log::set_max_level(level);

	Ok(())
}
