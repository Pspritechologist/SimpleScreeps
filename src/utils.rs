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
