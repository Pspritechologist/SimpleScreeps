use crate::htn_vm::{EndState, Num, Operation, VmValue};

pub type ParserResult<'p, T> = Result<(&'p str, T), ((usize, usize), &'static str)>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Instruction {
	Assign(String, ParseValue),
	CallTask(String),
	If(ParseValue, Vec<Instruction>, Option<Vec<Instruction>>),
	Exit(EndState),
	Value(Box<ParseValue>),
	Log(#[serde(with = "serde_level")] log::Level, ParseValue),
}

mod serde_level {
	use log::Level;
	use serde::{de, Deserialize, Deserializer, Serializer};

	pub fn serialize<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer,
	{
		serializer.serialize_str(match level {
			Level::Error => "error",
			Level::Warn => "warn",
			Level::Info => "info",
			Level::Debug => "debug",
			Level::Trace => "trace",
		})
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
	where D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		match s.as_str() {
			"error" => Ok(Level::Error),
			"warn" => Ok(Level::Warn),
			"info" => Ok(Level::Info),
			"debug" => Ok(Level::Debug),
			"trace" => Ok(Level::Trace),
			_ => Err(de::Error::custom("invalid log level")),
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Operator {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	And,
	Or,
	Eq,
	Neq,
	Lt,
	Gt,
	Lte,
	Gte,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ParseValue {
	Literal(VmValue),
	Variable(String),
	CallMethod(String, String, Vec<ParseValue>),
	Expression(Box<ParseValue>, Operator, Box<ParseValue>),
}

impl std::str::FromStr for Operator {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"+" => Self::Add,
			"-" => Self::Sub,
			"*" => Self::Mul,
			"/" => Self::Div,
			"%" => Self::Mod,
			"&&" => Self::And,
			"||" => Self::Or,
			"==" => Self::Eq,
			"!=" => Self::Neq,
			"<" => Self::Lt,
			">" => Self::Gt,
			"<=" => Self::Lte,
			">=" => Self::Gte,
			_ => return Err(()),
		})
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Parser<'p> {
	input: &'p str,
}

impl<'p> Parser<'p> {
	pub fn parse(input: &'p str) -> Result<Vec<Instruction>, ((usize, usize), &'static str)> {
		let parser = Self { input };
		let (input, instrs) = parser.parse_instructions(input)?;
		
		Ok(instrs)
	}

	pub fn parse_comment(&self, mut input: &'p str) -> ParserResult<'p, ()> {
		if input.starts_with('#') {
			input = input.trim_start_matches('#');
		} else {
			return Err(((self.get_err_index(input), 1), "Expected comment"));
		}

		let Some(end) = input.find('\n') else {
			return Ok((&input[input.len()..], ()));
		};

		Ok((input[end..].trim_start(), ()))
	}

	pub fn parse_instructions(&self, mut input: &'p str) -> ParserResult<'p, Vec<Instruction>> {
		let mut instrs = Vec::new();
		let mut instr;
		loop {
			while let Ok((rem, _)) = self.parse_comment(input) { input = rem; }

			(input, instr) = if let Ok(r) = self.parse_instruction(input) { r } else { break };
			instrs.push(instr);

			if input.is_empty() {
				break;
			}
		}

		Ok((input, instrs))
	}

	pub fn parse_block(&self, input: &'p str) -> ParserResult<'p, Vec<Instruction>> {
		let Some(input) = input.strip_prefix('{').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected opening `{`"));
		};

		let (input, instrs) = self.parse_instructions(input)?;

		let Some(input) = input.strip_prefix('}').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected closing `}`"));
		};

		Ok((input, instrs))
	}

	pub fn parse_instruction(&self, input: &'p str) -> ParserResult<'p, Instruction> {

		// if let Ok((input, instr)) = self.parse_call_task(input) {
		// 	return Ok((input, instr));
		// }

		// if let Ok((input, instr)) = self.parse_log(input) {
		// 	return Ok((input, instr));
		// }

		if let Ok((input, instr)) = self.parse_exit(input) {
			return Ok((input, instr));
		}

		if let Ok((input, instr)) = self.parse_if(input) {
			return Ok((input, instr));
		}

		if let Ok((input, instr)) = self.parse_assign(input) {
			return Ok((input, instr));
		}

		if let Ok((input, instr)) = self.parse_value(input) {
			return Ok((input, Instruction::Value(Box::new(instr))));
		}

		Err(((self.get_err_index(input), 1), "Invalid instruction"))
	}

	// pub fn parse_log(&self, input: &'p str) -> ParserResult<'p, Instruction> {
	// 	let Some(input) = input.strip_prefix("log_").map(str::trim_start) else {
	// 		return Err(((self.get_err_index(input), 1), "Expected `log`"));
	// 	};
	
	// 	// Check for 'info', 
	// }

	pub fn parse_assign(&self, input: &'p str) -> ParserResult<'p, Instruction> {
		let (input, ident) = self.parse_identifier(input)?;
		
		let Some(input) = input.strip_prefix('=').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected `=` for assignment"));
		};

		let (input, value) = self.parse_value(input)?;

		Ok((input, Instruction::Assign(ident, value)))
	}

	pub fn parse_exit(&self, input: &'p str) -> ParserResult<'p, Instruction> {
		let Some(input) = input.strip_prefix("exit").map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected `exit`"));
		};
		
		let (input, state) = self.parse_end_state(input)?;

		Ok((input, Instruction::Exit(state)))
	}

	pub fn parse_end_state(&self, input: &'p str) -> ParserResult<'p, EndState> {
		if let Some(input) = input.strip_prefix('S').map(str::trim_start) {
			return Ok((input, EndState::Success));
		} else if let Some(input) = input.strip_prefix('F').map(str::trim_start) {
			return Ok((input, EndState::Failure));
		} else if let Some(input) = input.strip_prefix('R').map(str::trim_start) {
			return Ok((input, EndState::Running));
		} else if let Some(input) = input.strip_prefix("Success").map(str::trim_start) {
			return Ok((input, EndState::Success));
		} else if let Some(input) = input.strip_prefix("Failure").map(str::trim_start) {
			return Ok((input, EndState::Failure));
		} else if let Some(input) = input.strip_prefix("Running").map(str::trim_start) {
			return Ok((input, EndState::Running));
		}
		
		Err(((self.get_err_index(input), 1), "Expected valid end state: Success, Failure, or Running"))
	}
	
	pub fn parse_value(&self, input: &'p str) -> ParserResult<'p, ParseValue> {
		if let Ok((input, res)) = self.parse_expression(input) {
			return Ok((input, res));
		}

		if let Ok((input, res)) = self.parse_constant(input) {
			return Ok((input, ParseValue::Literal(res)));
		}

		if let Ok((input, res)) = self.parse_method_call(input) {
			return Ok((input, res));
		}

		if let Ok((input, res)) = self.parse_identifier(input) {
			return Ok((input, ParseValue::Variable(res)));
		}
		
		Err(((self.get_err_index(input), 1), "Expected value"))
	}

	pub fn parse_expression(&self, input: &'p str) -> ParserResult<'p, ParseValue> {
		let Some(input) = input.strip_prefix('(').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected opening `(` for expression"));
		};

		let (input, left) = self.parse_value(input)?;

		let (input, op) = self.parse_operator(input)?;

		let (input, right) = self.parse_value(input)?;

		let Some(input) = input.strip_prefix(')').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected closing `)` for expression"));
		};
		
		Ok((input, ParseValue::Expression(Box::new(left), op, Box::new(right))))
	}

	pub fn parse_operator(&self, input: &'p str) -> ParserResult<'p, Operator> {
		let op_span = &input[..input.find(|c: char| !c.is_ascii_punctuation()).unwrap_or(0)];

		let Ok(op) = op_span.parse() else {
			return Err(((self.get_err_index(input), op_span.len()), "Invalid operator"));
		};

		Ok((input[op_span.len()..].trim_start(), op))
	}

	pub fn parse_method_call(&self, input: &'p str) -> ParserResult<'p, ParseValue> {
		let (input, target) = self.parse_identifier(input)?;

		let Some(input) = input.strip_prefix('.').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Expected `.` for method call"));
		};

		let (input, method) = self.parse_identifier(input)?;

		let Some(mut input) = input.strip_prefix('(').map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), "Missing `(` for method call"));
		};
		
		let mut args = Vec::new();
		let mut arg;
		// Get a comma separated list of values
		while !input.starts_with(')') {
			(input, arg) = self.parse_value(input)?;
			args.push(arg);

			if input.starts_with(',') {
				input = input[1..].trim_start();
			}
		}


		Ok((input[1..].trim_start(), ParseValue::CallMethod(target, method, args)))
	}

	pub fn parse_if(&self, input: &'p str) -> ParserResult<'p, Instruction> {
		let Some(input) = input.strip_prefix("if").map(str::trim_start) else {
			return Err(((self.get_err_index(input), 1), ""));
		};

		let (input, condition) = self.parse_value(input)?;

		// Parse the body of the if statement
		let (input, body) = self.parse_block(input)?;
		
		// Parse the else block if it exists
		let (input, else_body) = if let Some(input) = input.strip_prefix("else").map(str::trim_start) {
			self.parse_block(input).map(|(input, body)| (input, Some(body)))?
		} else {
			(input, None)
		};

		Ok((input, Instruction::If(condition, body, else_body)))
	}

	pub fn parse_identifier(&self, input: &'p str) -> ParserResult<'p, String> {
		let ident_span = &input[..input.find(|c: char| c != '_' && c != '-' && !c.is_alphabetic()).unwrap_or(0)];

		if ident_span.is_empty() {
			return Err(((self.get_err_index(input), 1), "Expected valid identifier"));
		}

		Ok((input[ident_span.len()..].trim_start(), ident_span.to_string()))
	}

	pub fn parse_constant(&self, input: &'p str) -> ParserResult<'p, VmValue> {
		if let Ok((input, res)) = self.parse_string(input) {
			return Ok((input, VmValue::String(res)));
		}

		if let Ok((input, res)) = self.parse_number(input) {
			return Ok((input, VmValue::Number(res)));
		}

		// Check for Null, True, False
		if let Some(rem) = input.strip_prefix("Null").map(str::trim_start) {
			return Ok((rem, VmValue::Null));
		}

		if let Some(rem) = input.strip_prefix("True").map(str::trim_start) {
			return Ok((rem, VmValue::Bool(true)));
		}

		if let Some(input) = input.strip_prefix("False").map(str::trim_start) {
			return Ok((input, VmValue::Bool(false)));
		}

		Err(((self.get_err_index(input), 1), "Expected constant value"))
	}

	pub fn parse_string(&self, input: &'p str) -> ParserResult<'p, String> {
		// Check for the opening single quote
		let Some(input) = input.strip_prefix('\'') else {
			return Err(((self.get_err_index(input), 1), "Expected starting `'`"));
		};

		let end_index = input.find('\'').ok_or_else(|| ((self.get_err_index(input), 1), "Expected ending `'`"))?;
		let len = input[..end_index].to_string();

		Ok((input[end_index + 1..].trim_start(), len))
	}

	pub fn parse_number(&self, input: &'p str) -> ParserResult<'p, Num> {
		if let Ok((input, num)) = self.parse_float(input) {
			return Ok((input, Num::Float(num)));
		}

		if let Ok((input, num)) = self.parse_int(input) {
			return Ok((input, Num::Int(num)));
		}

		Err(((self.get_err_index(input), 1), "Expected a number"))
	}

	pub fn parse_int(&self, input: &'p str) -> ParserResult<'p, i32> {
		// An int is a sequence of digits
		let num_span = &input[..input.find(|c: char| !c.is_ascii_digit()).unwrap_or(0)];

		let Ok(num) = num_span.parse() else {
			return Err(((self.get_err_index(input), num_span.len()), "Expected an integer"));
		};

		Ok((input[num_span.len()..].trim_start(), num))
	}

	fn parse_float(&self, input: &'p str) -> ParserResult<'p, f32> {
		// A float is zero or more numbers behind or after a required decimal point.
		// There must be at least one number before or after the decimal point.

		Err(((self.get_err_index(input), 1), "Expected a float"))
	}

	pub fn get_err_index(&self, input: &str) -> usize {
		self.input.len() - input.trim_start().len()
	}
}
