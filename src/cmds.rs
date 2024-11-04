use base64::Engine;
use screeps::RoomName;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::temp::MemData;


#[wasm_bindgen]
pub fn cmd_write_contr(room: String, msg: String) -> Result<String, String> {
	let Ok(room) = RoomName::new(&room) else {
		return Err("Invalid room name".to_string());
	};

	crate::UNIQUE_QUEUE.with(|queue| {
		queue.borrow_mut().push(crate::QueuedUniqueTask::SignMessage(msg, room));
	});

	Ok("Queued command".to_string())
}

#[wasm_bindgen]
pub fn cmd_wipe_memory() {
	screeps::raw_memory::set(&base64::prelude::BASE64_STANDARD_NO_PAD.encode(
		bitcode::serialize(&MemData::default()).unwrap()
	).into());
}
