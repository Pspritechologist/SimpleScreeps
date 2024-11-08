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
	Exit,
	// Success,
	// Failure,
	// Running,
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
	NullC,
	Dbg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum FlowSym {
	OpenParen,
	CloseParen,
	OpenBrace,
	CloseBrace,
	OpenSquare,
	CloseSquare,
	Colon,
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
			Keyword::Exit => write!(f, "exit"),
			// Keyword::Success => write!(f, "Success"),
			// Keyword::Failure => write!(f, "Failure"),
			// Keyword::Running => write!(f, "Running"),
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
			OpSym::NullC => write!(f, "??"),
			OpSym::Dbg => write!(f, "$"),
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
			FlowSym::Colon => write!(f, ":"),
			FlowSym::Comma => write!(f, ","),
			FlowSym::Dot => write!(f, "."),
			FlowSym::LineEnd => write!(f, ";"),
		}
	}
}

use self_rust_tokenize::SelfRustTokenize;
use vecmap::VecMap;

//TODO! These should not be here..
use crate::htn_vm::Num;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, self_rust_tokenize::SelfRustTokenize)]
pub enum VmValue {
	Null,
	Bool(bool),
	Number(Num),
	String(String),
}

#[derive(Clone, Debug)]
pub struct VecMapWrapper<K, V>(VecMap<K, V>);
impl<K, V> std::ops::Deref for VecMapWrapper<K, V> {
	type Target = vecmap::VecMap<K, V>;
	fn deref(&self) -> &Self::Target { &self.0 }
}
impl<K, V> std::ops::DerefMut for VecMapWrapper<K, V> {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<K, V> AsRef<vecmap::VecMap<K, V>> for VecMapWrapper<K, V> {
	fn as_ref(&self) -> &vecmap::VecMap<K, V> { &self.0 }
}
impl<K, V> AsMut<vecmap::VecMap<K, V>> for VecMapWrapper<K, V> {
	fn as_mut(&mut self) -> &mut vecmap::VecMap<K, V> { &mut self.0 }
}
impl<K, V> From<vecmap::VecMap<K, V>> for VecMapWrapper<K, V> {
	fn from(value: vecmap::VecMap<K, V>) -> Self { Self(value) }
}
impl<K, V> From<VecMapWrapper<K, V>> for vecmap::VecMap<K, V> {
	fn from(value: VecMapWrapper<K, V>) -> Self { value.0 }
}
impl<K: Eq, V> serde::Serialize for VecMapWrapper<K, V> where K: serde::Serialize, V: serde::Serialize {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		self.0.serialize(serializer)
	}
}
impl<'de, K: Eq, V> serde::Deserialize<'de> for VecMapWrapper<K, V> where K: serde::Deserialize<'de>, V: serde::Deserialize<'de> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
		Ok(Self(serde::Deserialize::deserialize(deserializer)?))
	}
}
impl<K: SelfRustTokenize, V: SelfRustTokenize> ::self_rust_tokenize::SelfRustTokenize for VecMapWrapper<K, V> {
    fn append_to_token_stream(
        &self,
        token_stream: &mut ::self_rust_tokenize::helpers::TokenStream,
    ) {
		let mut inner_token_stream = ::self_rust_tokenize::helpers::TokenStream::default();
		for (idx, (key, value)) in self.iter().enumerate() {
			key.append_to_token_stream(&mut inner_token_stream);
			self_rust_tokenize::helpers::TokenStreamExt::append(&mut inner_token_stream, ::self_rust_tokenize::helpers::proc_macro2::Punct::new(':', ::self_rust_tokenize::helpers::proc_macro2::Spacing::Joint));
			value.append_to_token_stream(&mut inner_token_stream);
			if idx != self.len() - 1 {
				self_rust_tokenize::helpers::TokenStreamExt::append(&mut inner_token_stream, self_rust_tokenize::helpers::proc_macro2::Punct::new(',', ::self_rust_tokenize::helpers::proc_macro2::Spacing::Alone));
			}
		}
		self_rust_tokenize::helpers::TokenStreamExt::append(token_stream, ::self_rust_tokenize::helpers::proc_macro2::Group::new(::self_rust_tokenize::helpers::proc_macro2::Delimiter::Brace, inner_token_stream));
    }
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
		}
	}
}
