use htn::{parsing::{htn_parser, lexer::htn_lexer, HtnInstr}, Parser};

fn main() {
	setup_logging(log::LevelFilter::Trace);

    let src = std::fs::read_to_string("src/script.test").unwrap();

    let result = htn_lexer().parse(&src);
    match result.output() {
        Some(tokens) => {
            if result.has_errors() {
                println!("Lexing Errors:");
                for err in result.errors() {
                    println!("\t{err:?}");
                }
                println!();
            }

            let result = htn_parser().parse(&**tokens);
            match result.output() {
                Some(instr) => {
                    if result.has_errors() {
                        println!("Parsing Errors:");
                        for err in result.errors() {
                            println!("\t{err:?}");
                        }
                        println!();
                    }

                    let values = instr.iter().map(|instr| {
                        match instr {
                            HtnInstr::Value(value) => value,
                            _ => unreachable!(),
                        }
                    });
                    
                    println!("Instructions:");
                    for value in values {
                        println!("\t{value}");
                    }
                },
                None => {
                    println!("Error parsing instructions:");
                    for err in result.errors() {
                        println!("\t{err:?}");
                    }
                }
            }
        },
        None => {
            println!("Error parsing tokens:");
            for err in result.errors() {
                println!("\t{err:?}");
            }
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
