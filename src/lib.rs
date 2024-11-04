#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use std::cell::RefCell;

use log::*;
use screeps::game;
use unique_tasks::QueuedUniqueTask;
use wasm_bindgen::prelude::*;

mod logging;
mod temp;
mod creep_dispatch;
pub mod utils;
pub mod cmds;
pub mod unique_tasks;

const NAME_DATA: &[u8] = include_bytes!("../data/names.bit");
const QUOTES: &[&str] = &[
	// 10 characters limit.
	"Im a creep",
	"A loserrr"	,
	"uwu"		,
	"owo"		,
	"awa"		,
	"nya"		,
	"uwu"		,
	"owo"		,
	"awa"		,
	"nya"		,
	"uwu"		,
	"owo"		,
	"uwu"		,
	"owo"		,
	"uwu"		,
	"owo"		,
	"Death eggs",
	"are wet"	,
	"bungo"		,
	"bongo"		,
	"bingo"		,
	"Hello!"	,
	"Hi there!"	,
	"Howdy!"	,
	"Greetings!",
	"Howdy doo!",
	"Hey!"		,
	"Listen!"	,
	"Watch out!",
	"Behind you",
	"Boo!"		,
	"Dark days"	,
	"Roll bluff",
	"Hyaah!"	,
	"Your mom!"	,
	"Nyeh hehe!",
];

thread_local! {
	pub static NAMES: Vec<String> = bitcode::decode(NAME_DATA).unwrap();
	pub static UNIQUE_QUEUE: RefCell<Vec<QueuedUniqueTask>> = const { RefCell::new(Vec::new()) };
}

pub fn get_new_creep_name(used: &[String]) -> String {
	crate::NAMES.with(|names| {
		loop {
			let name = fastrand::choice(names.iter()).unwrap();
			if used.contains(name) {
				continue;
			} else {
				return name.clone();
			}
		}
	})
}

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Info);
	});

	debug!("loop starting! CPU: {}", game::cpu::get_used());

	temp::tick();

	debug!("done! cpu: {}", game::cpu::get_used());
	// info!("");
}
