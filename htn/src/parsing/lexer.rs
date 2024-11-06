use chumsky::prelude::*;
use super::tokens::{HtnToken::{self, *}, FlowSym::*, Keyword::*, OpSym::*};

pub type Error<'src, T> = Rich<'src, T, Span>;
pub type Extra<'src, T> = extra::Err<Error<'src, T>>;
pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

pub fn htn_lexer<'src>() -> impl Parser<'src, &'src str, Vec<HtnToken<'src>>, Extra<'src, char>> {
	let keyword = choice(( // Keywords.
		just("True").to(True),
		just("False").to(False),
		just("Null").to(Null),
		just("if").to(If),
		just("else").to(Else),
	)).map(Keyword);
	
	let operator = choice(( // Operators.
		just('+').to(Add),
		just('-').to(Sub),
		just('*').to(Mul),
		just('/').to(Div),
		just('%').to(Mod),
		just('&').then(just('&').or_not()).to(And),
		just('|').then(just('|').or_not()).to(Or),
		just('=').then(just('=').or_not()).to(Eq),
		just("!=").to(Neq),
		just('<').to(Lt),
		just('>').to(Gt),
		just("<=").to(Lte),
		just(">=").to(Gte),
		just('!').to(Not),
	)).map(OpSym);
	
	let flow_sym = choice(( // Flow symbols.
		just('(').to(OpenParen),
		just(')').to(CloseParen),
		just('{').to(OpenBrace),
		just('}').to(CloseBrace),
		just('[').to(OpenSquare),
		just(']').to(CloseSquare),
		just(',').to(Comma),
		just('.').to(Dot),
		just(';').to(LineEnd),
	)).map(FlowSym);
	
	let ident = text::ident().map(Ident); // Identifiers.

	let int = text::digits(10).to_slice().from_str().unwrapped().map(Int); // Integers.

	let float = text::digits(10) // Floats.
		.then(just('.').then(text::digits(10).or_not()))
		.to_slice()
		.from_str()
		.unwrapped() // ? This can fail if float overflow.
		.map(Float);

	let str_es = none_of("\"\\") // Escaped strings.
		.or(just('\\').ignore_then(any().to_slice().try_map(handle_escape)))
		.repeated()
		.delimited_by(just('"'), just('"'))
		.to_slice()
		.map(EscStr);

	let str_rw = none_of('\'')
		.or(just('\'').then_ignore(just('\'')))
		.repeated()
		.delimited_by(just('\''), just('\''))
		.to_slice()
		.map(RawStr);
	
	let token = choice((
		keyword,
		flow_sym,
		int,
		float,
		str_rw,
		str_es,
		operator,
		ident,
	));

	let comment = just('#').then(none_of('\n').repeated().then(just('\n').ignored().or(end().ignored())));

	token
		.padded()
		.padded_by(comment.repeated())
		.recover_with(skip_then_retry_until(any().ignored(), end()))
		.repeated()
		.collect()
}

fn handle_escape<'src>(c: &'src str, span: Span) -> Result<char, Error<'src, char>> {
	Ok(match c {
		"b" => '\u{0008}',
		"f" => '\u{000C}',
		"n" => '\n',
		"r" => '\r',
		"t" => '\t',
		"\"" => '\"',
		"\\" => '\\',
		// _ => return Err(Extra::from(span)),
		_ => return Err(Error::custom(span, "invalid escape sequence")),
	})
}
