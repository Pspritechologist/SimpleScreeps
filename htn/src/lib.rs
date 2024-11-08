#![feature(trait_alias)]

pub mod parsing;
pub mod htn_vm;
pub mod prelude {
	pub use crate::htn_vm::{HtnVm, Operation, Num, EndState, Error};
	pub use crate::parsing::htn_parser;
	pub use crate::parsing::lexer::htn_lexer;
	pub use chumsky::Parser;
	pub mod embed_requirements {
		pub use crate::parsing::{BinaryOp, HtnInstr, ParseValue, UnaryOp};
		pub use crate::parsing::tokens::*;
	}
}
