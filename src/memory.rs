use base64::Engine;
use screeps::{Creep, ObjectId};
use vecmap::VecMap;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MemData {
	pub creep_data: VecMap<ObjectId<Creep>, CreepData>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CreepData {
	// pub current_task: Option<Job>,
	pub current_task: Option<(crate::JobIdentifier, crate::dynamic_stuff::DynState)>,
}

pub fn get_memory() -> MemData {
	// js_sys::Reflect::delete_property(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory")).unwrap();
	// js_sys::Reflect::set(&js_sys::global(), &wasm_bindgen::JsValue::from_str("Memory"), &js_sys::Object::default().into()).unwrap();

	let raw_memory = screeps::raw_memory::get().as_string().unwrap();
	if raw_memory.is_empty() {
		return Default::default();
	}

	let data = match base64::prelude::BASE64_STANDARD_NO_PAD.decode(
		screeps::raw_memory::get().as_string().unwrap()
	) {
		Ok(data) => data,
		Err(err) => {
			log::error!("Failed to decode memory: '{err}'. Dumping memory...");
			log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
			return Default::default();
		}
	};

	// if let Ok(data) = bitcode::deserialize(&data) {
	// 	data
	// } else {
	// 	log::error!("Memory not valid Bitcode! Dumping memory...");
	// 	log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
	// 	MemData::default()
	// }

	if let Ok(data) = rmp_serde::from_slice(&data) {
		data
	} else {
		log::error!("Memory not valid MessagePack! Dumping memory...");
		log::error!("{}", screeps::raw_memory::get().as_string().unwrap());
		MemData::default()
	}

	// if let Ok(data) = serde_json::from_str(&raw_memory) {
	// 	data
	// } else {
	// 	log::error!("Memory not valid JSON! Dumping memory...");
	// 	log::error!("{}", raw_memory);
	// 	MemData::default()
	// }
}

pub fn set_memory(data: &MemData) {
	// let data = match bitcode::serialize(data) {
	// 	Ok(data) => data,
	// 	Err(err) => {
	// 		log::error!("Failed to serialize memory: '{err}'.");
	// 		bitcode::serialize(&MemData::default()).unwrap()
	// 	}
	// };
	
	let data = match rmp_serde::to_vec_named(data) {
		Ok(data) => data,
		Err(err) => {
			log::error!("Failed to serialize memory: '{err}'.");
			rmp_serde::to_vec(&MemData::default()).unwrap()
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
