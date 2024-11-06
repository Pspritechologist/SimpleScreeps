pub mod lexer;
pub mod tokens;

use super::htn_vm::{EndState, Num};
use chumsky::input::ValueInput;
use chumsky::{prelude::*, Parser};
use lexer::{Extra, Span};
use tokens::{HtnToken, VmValue};

pub trait ParseIn<'src> = ValueInput<'src, Token = HtnToken<'src>, Span = Span>;
pub trait HtnParser<'src, I: ParseIn<'src>, O> = Parser<'src, I, O, Extra<'src, I::Token>> + Clone;

#[derive(Debug, Clone)]
pub enum HtnInstr {
	Assign(String, ParseValue),
	CallTask(String),
	If(ParseValue, Vec<HtnInstr>, Option<Vec<HtnInstr>>),
	Exit(EndState),
	Value(ParseValue),
	// Log(log::Level, ParseValue),
	Noop,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BinaryOp {
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
pub enum UnaryOp {
	Not,
	Neg,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ParseValue {
	Literal(VmValue),
	Variable(String),
	Index(Box<ParseValue>, Box<ParseValue>),
	Call(Box<ParseValue>, Vec<ParseValue>),
	Access(Box<ParseValue>, String),
	Expression(Box<ParseValue>, BinaryOp, Box<ParseValue>),
	Unary(UnaryOp, Box<ParseValue>),
}

impl std::fmt::Display for ParseValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			ParseValue::Literal(value) => write!(f, "{value}"),
			ParseValue::Variable(name) => write!(f, "{name}"),
			ParseValue::Index(lhs, rhs) => write!(f, "{lhs}[{rhs}]"),
			ParseValue::Call(func, args) => write!(f, "{func}({})", args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ")),
			ParseValue::Access(lhs, rhs) => write!(f, "{lhs}.{rhs}"),
			ParseValue::Expression(lhs, op, rhs) => write!(f, "{{{lhs} {op:?} {rhs}}}"),
			ParseValue::Unary(op, value) => write!(f, "{{{op:?} {value}}}"),
        }
    }
}

fn expr<'src, I: ParseIn<'src>>(lhs: &impl HtnParser<'src, I, ParseValue>, rhs: &impl HtnParser<'src, I, ParseValue>, op_parser: impl HtnParser<'src, I, BinaryOp>) -> impl HtnParser<'src, I, ParseValue> {
	lhs.clone()
		.then(op_parser)
		.then(rhs.clone())
		.map(|((lhs, op), rhs)| ParseValue::Expression(Box::new(lhs), op, Box::new(rhs)))
}

pub fn htn_parser<'src, I: ParseIn<'src>>() -> impl HtnParser<'src, I, Vec<HtnInstr>> {
	use tokens::{HtnToken::*, FlowSym::*, Keyword::*, OpSym::*};

	let literal = select! {
		Keyword(True) => VmValue::Bool(true),
		Keyword(False) => VmValue::Bool(false),
		Keyword(Null) => VmValue::Null,
		EscStr(s) => VmValue::String(unescape::unescape(s).unwrap()),
		RawStr(s) => VmValue::String(s.to_string()),
		Int(i) => VmValue::Number(Num::Int(i)),
		Float(f) => VmValue::Number(Num::Float(f)),
	};

	let ident = select! { Ident(i) => i.to_string() };

	let value = recursive(|value| {
		let lit = literal.map(ParseValue::Literal);
		let variable = ident.map(ParseValue::Variable);
		let wrapped = value.clone().delimited_by(just(FlowSym(OpenParen)), just(FlowSym(CloseParen)));

		let atom = choice((
			lit,
			variable,
			wrapped,
		));

		let tu = select! { OpSym(Not) => UnaryOp::Not, OpSym(Sub) => UnaryOp::Neg }
			.then(atom.clone())
			.map(|(op, value)| ParseValue::Unary(op, Box::new(value)))
			.or(atom.clone())
			.boxed();

		enum Suffix {
			Index(ParseValue),
			Call(Vec<ParseValue>),
			Access(String),
		}

		let suffix = choice((
			value.clone()
				.separated_by(just(FlowSym(Comma)))
				.allow_trailing()
				.collect()
				.delimited_by(just(FlowSym(OpenParen)), just(FlowSym(CloseParen)))
				.map(Suffix::Call),
			value.clone()
				.delimited_by(just(FlowSym(OpenSquare)), just(FlowSym(CloseSquare)))
				.map(Suffix::Index),
			just(FlowSym(Dot))
				.ignore_then(ident)
				.map(Suffix::Access),
		));

		let ts = tu.clone()
			.foldl(suffix.repeated(), |expr, suffix| {
				match suffix {
					Suffix::Index(index) => ParseValue::Index(Box::new(expr), Box::new(index)),
					Suffix::Call(args) => ParseValue::Call(Box::new(expr), args),
					Suffix::Access(field) => ParseValue::Access(Box::new(expr), field),
				}
			})
			.or(tu.clone())
			.boxed();

		let op_t1 = select! {
			OpSym(Mul) => BinaryOp::Mul,
			OpSym(Div) => BinaryOp::Div,
			OpSym(Mod) => BinaryOp::Mod,
		};

		let t1 = recursive(|t1| expr(&ts, &t1, op_t1).or(ts.clone()));

		let op_t2 = select! {
			OpSym(Add) => BinaryOp::Add,
			OpSym(Sub) => BinaryOp::Sub,
		};

		let t2 = recursive(|t2| expr(&t1, &t2, op_t2).or(t1.clone()));

		let op_t3 = select! {
			OpSym(Lt) => BinaryOp::Lt,
			OpSym(Gt) => BinaryOp::Gt,
			OpSym(Lte) => BinaryOp::Lte,
			OpSym(Gte) => BinaryOp::Gte,
			OpSym(Eq) => BinaryOp::Eq,
			OpSym(Neq) => BinaryOp::Neq,
		};

		let t3 = recursive(|t3| expr(&t2, &t3, op_t3).or(t2.clone()));

		let op_t4 = select! {
			OpSym(And) => BinaryOp::And,
		};

		let t4 = recursive(|t4| expr(&t3, &t4, op_t4).or(t3.clone()));

		let op_t5 = select! {
			OpSym(Or) => BinaryOp::Or,
		};

		let t5 = recursive(|t5| expr(&t4, &t5, op_t5).or(t4.clone()));

		t5
	});

	let assignment = ident
		.then_ignore(just(OpSym(Eq)))
		.then(value.clone())
		.map(|(name, value)| HtnInstr::Assign(name, value));

	

	value.map(HtnInstr::Value).separated_by(just(FlowSym(LineEnd))).allow_trailing().collect()
}
