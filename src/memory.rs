use base64::Engine;
use screeps::{Creep, ObjectId};
use vecmap::VecMap;

use crate::state_machine::State;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MemData {
	pub states: VecMap<ObjectId<Creep>, State>,
}

pub fn get_memory() -> MemData {
	js_sys::Reflect::delete_property(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory")).unwrap();
	js_sys::Reflect::set(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory"), &js_sys::Object::default().into()).unwrap();

	let data = base64::prelude::BASE64_STANDARD_NO_PAD.decode(
		screeps::raw_memory::get().as_string().unwrap()
	).unwrap_or_else(|err| {
		log::error!("Failed to decode memory: '{err}'. Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		bitcode::serialize(&MemData::default()).unwrap()
	});

	if let Ok(data) = bitcode::deserialize(&data) {
		data
	} else {
		log::error!("Memory not valid Bitcode! Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		MemData::default()
	}
}
