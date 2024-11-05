fn main() {
	setup_logging(log::LevelFilter::Trace);

	log::warn!("Hello, world!");

	match screepies::parser::Parser::parse(include_str!("htn/root.htn")) {
		Ok(tree) => {
			log::info!("Parsed tree: {:#?}", tree);
		}
		Err(err) => {
			log::error!("Failed to parse tree: {:?}", err);
		}
	}
}

fn setup_logging(verbosity: log::LevelFilter) {
    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| {
            out.finish(format_args!(
                "({}) {}: {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(fern::Dispatch::new().chain(std::io::stdout()))
        .apply()
        .expect("expected setup_logging to only ever be called once per instance");
}
