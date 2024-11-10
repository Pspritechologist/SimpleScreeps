use base64::Engine;
use screeps::{Creep, ObjectId};
use vecmap::VecMap;
use crate::state::*;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MemData {
	pub creep_data: VecMap<ObjectId<Creep>, CreepData>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CreepData {
	// pub current_task: Option<Job>,
	pub current_task: Option<crate::JobEnum>,
}

pub fn get_memory() -> MemData {
	// js_sys::Reflect::delete_property(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory")).unwrap();
	// js_sys::Reflect::set(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory"), &js_sys::Object::default().into()).unwrap();

	let raw_memory = screeps::raw_memory::get().as_string().unwrap();
	if raw_memory.is_empty() {
		return Default::default();
	}

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

	// match serde_json::from_str(&raw_memory) {
	// 	Ok(data) => data,
	// 	Err(err) => {
	// 		log::error!("Failed to deserialize memory: '{err}'. Dumping...");
	// 		log::error!("{}", raw_memory);
	// 		MemData::default()
	// 	}
	// }
}

pub fn set_memory(data: &MemData) {
	let data = match bitcode::serialize(data) {
		Ok(data) => data,
		Err(err) => {
			log::error!("Failed to serialize memory: '{err}'.");
			bitcode::serialize(&MemData::default()).unwrap()
		}
	};
	let data = base64::prelude::BASE64_STANDARD_NO_PAD.encode(&data);

	// let data = match serde_json::to_string(&data) {
	// 	Ok(data) => data,
	// 	Err(err) => {
	// 		log::error!("Failed to serialize memory: '{err}'.");
	// 		String::new()
	// 	}
	// };

	screeps::raw_memory::set(&data.into());
}
