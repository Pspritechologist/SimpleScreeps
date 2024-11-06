use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum HtnToken<'src> {
	Ident(&'src str),
	Int(i32),
	Float(f32),
	RawStr(&'src str),
	EscStr(&'src str),
	Keyword(Keyword),
	OpSym(OpSym),
	FlowSym(FlowSym),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Keyword {
	True,
	False,
	Null,
	If,
	Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum OpSym {
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
	Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum FlowSym {
	OpenParen,
	CloseParen,
	OpenBrace,
	CloseBrace,
	OpenSquare,
	CloseSquare,
	Comma,
	Dot,
	LineEnd,
}

impl Display for HtnToken<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			HtnToken::Ident(ident) => write!(f, "{ident}"),
			HtnToken::Int(int) => write!(f, "{int}"),
			HtnToken::Float(float) => write!(f, "{float}"),
			HtnToken::RawStr(raw_str) => write!(f, "'{raw_str}'"),
			HtnToken::EscStr(esc_str) => write!(f, "\"{esc_str}\""),
			HtnToken::Keyword(keyword) => write!(f, "{keyword}"),
			HtnToken::OpSym(operator) => write!(f, "{operator}"),
			HtnToken::FlowSym(flow_sym) => write!(f, "{flow_sym}"),
		}
	}
}

impl Display for Keyword {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Keyword::True => write!(f, "True"),
			Keyword::False => write!(f, "False"),
			Keyword::Null => write!(f, "Null"),
			Keyword::If => write!(f, "if"),
			Keyword::Else => write!(f, "else"),
		}
	}
}

impl Display for OpSym {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			OpSym::Add => write!(f, "+"),
			OpSym::Sub => write!(f, "-"),
			OpSym::Mul => write!(f, "*"),
			OpSym::Div => write!(f, "/"),
			OpSym::Mod => write!(f, "%"),
			OpSym::And => write!(f, "&"),
			OpSym::Or => write!(f, "|"),
			OpSym::Eq => write!(f, "=="),
			OpSym::Neq => write!(f, "!="),
			OpSym::Lt => write!(f, "<"),
			OpSym::Gt => write!(f, ">"),
			OpSym::Lte => write!(f, "<="),
			OpSym::Gte => write!(f, ">="),
			OpSym::Not => write!(f, "!"),
		}
	}
}

impl Display for FlowSym {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			FlowSym::OpenParen => write!(f, "("),
			FlowSym::CloseParen => write!(f, ")"),
			FlowSym::OpenBrace => write!(f, "{{"),
			FlowSym::CloseBrace => write!(f, "}}"),
			FlowSym::OpenSquare => write!(f, "["),
			FlowSym::CloseSquare => write!(f, "]"),
			FlowSym::Comma => write!(f, ","),
			FlowSym::Dot => write!(f, "."),
			FlowSym::LineEnd => write!(f, ";"),
		}
	}
}

//TODO! These should not be here...#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
use crate::htn_vm::Num;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VmValue {
	Null,
	Bool(bool),
	Number(Num),
	String(String),
	List(Vec<VmValue>),
	Map(vecmap::VecMap<String, VmValue>),
}

impl std::fmt::Display for VmValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			VmValue::Null => write!(f, "null"),
			VmValue::Bool(value) => write!(f, "{value}"),
			VmValue::Number(value) => match value {
				Num::Float(value) => write!(f, "{value}"),
				Num::Int(value) => write!(f, "{value}"),
			},
			VmValue::String(value) => write!(f, "{value}"),
			VmValue::List(value) => {
				write!(f, "[")?;
				for (i, value) in value.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "{value}")?;
				}
				write!(f, "]")
			}
			VmValue::Map(value) => {
				write!(f, "{{")?;
				for (i, (key, value)) in value.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "{key}: {value}")?;
				}
				write!(f, "}}")
			}
		}
	}
}
