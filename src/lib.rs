use log::*;
use screeps::game;
use wasm_bindgen::prelude::*;

mod logging;
mod temp;
mod creep_dispatch;
pub mod utils;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(logging::Warn);
    });

    // Replace the memory object
    // screeps::memory::ROOT;

    debug!("loop starting! CPU: {}", game::cpu::get_used());

    temp::tick();

    info!("done! cpu: {}", game::cpu::get_used())
}

#[macro_export]
macro_rules! handle_err {
	($e:expr) => {
		if let Err(err) = $e {
			log::error!(
                "[{}:{}:{}]: {:?}\n\tsrc = {}", 
                file!(), 
                line!(), 
                column!(), 
                &err,
                {
                    let src = stringify!($e);
                    if src.len() > 45 {
                        format!("{}...", &src[..40])
                    } else {
                        src.to_string()
                    }
                }
            );
		}
	};
}

#[macro_export]
macro_rules! handle_warn {
    ($e:expr) => {
		if let Err(err) = $e {
			log::info!(
                "[{}:{}:{}]: {:?}", 
                file!(), 
                line!(), 
                column!(), 
                &err,
            );
		}
    };
}
