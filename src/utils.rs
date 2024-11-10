pub mod prelude {
    use screeps::{Creep, ObjectId, Room, RoomObject, Structure, StructureController, StructureSpawn};

	pub type CreepId = ObjectId<Creep>;
	pub type RoomId = ObjectId<Room>;
	pub type StructureId = ObjectId<Structure>;
	pub type ControllerId = ObjectId<StructureController>;
	pub type SpawnId = ObjectId<StructureSpawn>;
	pub type RoomObjectId = ObjectId<RoomObject>;
}

const NAME_DATA: &[u8] = include_bytes!("../data/names.bit");
const QUOTES: &[&str] = &[
	// 10 characters limit.
	"Im a creep",
	"a wirdough",
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
	"Thriller"	,
	"In the end",
	"cabbeg"	,
	"cabag"		,
	"cabbege"	,
];

thread_local! {
	pub static NAMES: Vec<String> = bitcode::decode(NAME_DATA).unwrap();
	// pub static UNIQUE_QUEUE: RefCell<Vec<QueuedUniqueTask>> = const { RefCell::new(Vec::new()) };
}

pub fn get_new_creep_name(used: &[String]) -> String {
	NAMES.with(|names| {
		loop {
			let name = fastrand::choice(names.iter()).unwrap();
			if used.contains(name) {
				continue;
			} else {
				return name.clone(); // ! This is a potential source of infinite looping... Let's hope we don't run out.
			}
		}
	})
}

pub fn generate_name() -> String {
	let used: Vec<_> = screeps::game::creeps().keys().collect();
	get_new_creep_name(&used)
}

// #[macro_export]
// macro_rules! measure_cpu {
// 	($name:expr, $block:expr) => {
// 		let start = screeps::game::cpu::get_used();
// 		$block
// 		let end = screeps::game::cpu::get_used();
// 		log::trace!("{} took {} CPU", $name, end - start);
// 	};
// }

#[macro_export]
macro_rules! conte {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(e) => continue,
		}
	};
}

#[macro_export]
macro_rules! conto {
	($e:expr) => {
		match $e {
			Some(v) => v,
			None => continue,
		}
	};
}

#[macro_export]
macro_rules! ign {
	($e:expr) => {
		let _ = $e;
	};
}

#[macro_export]
macro_rules! dbg {
	($e:expr) => {
		{
			log::debug!("{:?}", $e);
			$e
		}
	};
}

// #[macro_export]
// macro_rules! include_mod {
//     ($(#[$attr:meta])* $vis:vis $modname:ident) => {
//         crate::include_mod!($(#[$attr])* $vis $modname, concat!("/", stringify!($modname), ".rs"));
//     };

//     ($(#[$attr:meta])* $vis:vis $modname:ident, $source:expr) => {
//         #[rustfmt::skip]
//         #[allow(clippy::extra_unused_lifetimes)]
//         #[allow(clippy::needless_lifetimes)]
//         #[allow(clippy::let_unit_value)]
//         #[allow(clippy::just_underscores_and_digits)]
//         $(#[$attr])* $vis mod $modname { include!(concat!(env!("OUT_DIR"), $source)); }
//     };
// }

// include_mod!(pub state_enum);
