//! ### Syntax
//! Variables are alphanumeric strings (underscores, too) surrounded by braces. Here's an `{example}`.
//!
//! You can prevent `{word}` from being seen as a variable by escaping the opening brace. Like `\{this}`.
//!
//! ## Example
//! If you have this document in something like `template.bpl`
//! ```text
//! Dear {name},
//!
//! Some generic email text here!
//!
//! Sincerely,
//! Some Company
//! ```
//!
//! You can fill it out for the names `Ferris` and `Rusty` like so
//! ```rust
//! use bempline::{Document, Options};
//!
//! fn main() {
//! 	let doc = Document::from_file("test/template.bpl", Options::default()).unwrap();
//! 	let names = vec!["Ferris", "Rusty"];
//!
//! 	for name in names {
//! 		let mut cloned = doc.clone();
//! 		cloned.set("name", name);
//!
//! 		println!("{}", cloned.compile());
//! 	}
//! }
//! ```

mod document;
pub mod options;

pub use document::Document;
pub use document::ParseError;
pub use document::Token;
pub use options::Options;

#[macro_export]
macro_rules! variables {
	($template:expr, $variable:ident) => {
		$template.set(stringify!($variable), $variable);
	};

	($template:expr, $variable:ident, $($variables:ident),+) => {
		variables!($template, $variable);
		variables!($template, $($variables),+)
	}
}

#[macro_export]
macro_rules! set {
	($template:expr, $key:ident, $($arg:tt)*) => {
		$template.set(stringify!($key), std::fmt::format(format_args!($($arg)*)));
	};
}

#[cfg(test)]
mod test {
	use crate::options::IncludeMethod;

	use super::*;
	use std::path::PathBuf;

	#[test]
	fn compile_all_set() {
		let mut doc = Document::from_str(
			"One: {one} | Two: {two} | Three: {three}",
			Options::default(),
		)
		.unwrap();
		doc.set("one", "1");
		doc.set("two", "2");
		doc.set("three", "3");

		assert_eq!(&doc.compile(), "One: 1 | Two: 2 | Three: 3");
	}

	#[test]
	fn compile_some_set() {
		let mut doc = Document::from_str(
			"One: {one} | Two: {two} | Three: {three}",
			Options::default(),
		)
		.unwrap();
		doc.set("one", "1");
		doc.set("three", "3");

		assert_eq!(&doc.compile(), "One: 1 | Two: {two} | Three: 3");
	}

	// Parsing related tests

	#[test]
	fn no_text() {
		let doc = Document::from_str("", Options::default()).unwrap();
		assert_eq!(doc.tokens, vec![]);
	}

	#[test]
	fn only_text() {
		let doc = Document::from_str("Nothing but text", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Text(String::from("Nothing but text"))]
		);
	}

	#[test]
	fn escaped_bracket() {
		let doc =
			Document::from_str("escape this: \\{, but not this \\n", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Text(String::from(
				"escape this: {, but not this \\n"
			))]
		);
	}

	#[test]
	fn only_variable() {
		let doc = Document::from_str("{variable}", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Variable {
				name: String::from("variable")
			}]
		);
	}

	#[test]
	fn sandwhiched_variable() {
		let doc = Document::from_str("Hello {name}, how are you?", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text(String::from("Hello ")),
				Token::Variable {
					name: String::from("name")
				},
				Token::Text(String::from(", how are you?"))
			]
		);
	}

	#[test]
	fn ends_variable() {
		let doc = Document::from_str("Hello {name}", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text(String::from("Hello ")),
				Token::Variable {
					name: String::from("name")
				}
			]
		);
	}

	#[test]
	fn starts_variable() {
		let doc = Document::from_str("{name}, hello!", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Variable {
					name: String::from("name")
				},
				Token::Text(String::from(", hello!"))
			]
		);
	}

	#[test]
	fn multivariable() {
		let doc = Document::from_str(
			"The weather is {weather} in {location} today.",
			Options::default(),
		)
		.unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text(String::from("The weather is ")),
				Token::Variable {
					name: String::from("weather")
				},
				Token::Text(String::from(" in ")),
				Token::Variable {
					name: String::from("location")
				},
				Token::Text(String::from(" today."))
			]
		);
	}

	#[test]
	fn include_test() {
		let doc = Document::from_file("test/include_test.bpl", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text("Before the include!\n".into()),
				Token::Text("The included file! With a ".into()),
				Token::Variable {
					name: "variable".into()
				},
				Token::Text("!".into()),
				Token::Text("\naand after~".into())
			]
		)
	}

	#[test]
	fn include_method_path_test() {
		let doc = Document::from_file(
			"test/include_some.bpl",
			Options::default().include_path(IncludeMethod::Path(PathBuf::from("test/subdir"))),
		)
		.unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text("Testing IncludeMethod::Path here...\n".into()),
				Token::Text("I'm in a subdir :D\n".into()),
				Token::Variable {
					name: "variable".into()
				},
				Token::Text("!".into())
			]
		)
	}

	#[test]
	fn complex_include() {
		let doc =
			Document::from_file("test/pattern_include_ifset_base.bpl", Options::default()).unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Pattern {
				pattern_name: String::from("name"),
				tokens: vec![Token::IfSet {
					variable_name: String::from("variable"),
					tokens: vec![Token::Variable {
						name: String::from("variable")
					}],
					else_tokens: None
				}]
			}]
		)
	}

	#[test]
	fn ifset_variable_set() {
		let mut doc = Document::from_str("{%if-set foo}set!{%end}", Options::default()).unwrap();
		doc.set("foo", "bar");

		assert_eq!(doc.compile(), "set!")
	}

	#[test]
	fn ifset_variable_set_empty_string() {
		let mut doc = Document::from_str(
			"{%if-set foo}set!{%end}{%if-set bar}barset!{%end}",
			Options::default(),
		)
		.unwrap();
		doc.set("foo", "");
		doc.set("bar", "set!");

		assert_eq!(doc.compile(), "barset!")
	}

	#[test]
	fn iftest_else() {
		let doc = Document::from_str(
			"{%if-set donotset}wasset{%else}notset{%end}",
			Options::default(),
		)
		.unwrap();

		assert_eq!(doc.compile(), "notset");
	}

	#[test]
	fn pattern_parse() {
		let doc = Document::from_str("{%pattern name}blah{variable}lah{%end}", Options::default())
			.unwrap();

		assert_eq!(
			doc.get_pattern("name").unwrap().tokens,
			vec![
				Token::Text(String::from("blah")),
				Token::Variable {
					name: String::from("variable")
				},
				Token::Text(String::from("lah"))
			]
		)
	}

	#[test]
	fn pattern_fill() {
		let mut doc =
			Document::from_str("{%pattern name}-{variable}-{%end}", Options::default()).unwrap();

		let mut pat = doc.get_pattern("name").unwrap();

		let mut name = pat.clone();
		name.set("variable", "one");
		pat.set("variable", "two");

		doc.set_pattern(name);
		doc.set_pattern(pat);

		assert_eq!(doc.compile(), String::from("-one--two-"))
	}

	#[test]
	fn nested_scoped_commands() {
		let doc = Document::from_str(
			"{%pattern name}{%if-set var}{%end}{%end}",
			Options::default(),
		)
		.unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Pattern {
				pattern_name: String::from("name"),
				tokens: vec![Token::IfSet {
					variable_name: String::from("var"),
					tokens: vec![],
					else_tokens: None
				}]
			}]
		)
	}

	#[test]
	fn variables_macro() {
		let mut doc = Document::from_str("{foo} and {bar}", Options::default()).unwrap();
		let foo = "one";
		let bar = "two";

		variables!(doc, foo, bar);

		assert_eq!(doc.compile(), String::from("one and two"))
	}

	#[test]
	fn set_macro() {
		let mut doc = Document::from_str("{foo}", Options::default()).unwrap();

		set!(doc, foo, "{:X}", 17);

		assert_eq!(doc.compile(), String::from("11"))
	}

	#[test]
	fn wrapping_include() {
		let expected = "<html><head>Foo<title>Test!</title></head></html>";
		let mut doc = Document::from_file("test/wrapped_include.bpl", Options::default()).unwrap();
		doc.set("var_in", "Foo");

		assert_eq!(doc.compile(), expected)
	}

	#[test]
	fn set_command() {
		let expected = "foo";
		let doc = Document::from_file("test/set-command.bpl", Options::default()).unwrap();

		assert_eq!(doc.compile(), expected)
	}

	#[test]
	fn wrapping_set() {
		let expected = "<html><head>foobar</head></html>";
		let doc = Document::from_file("test/wrapping_set.bpl", Options::default()).unwrap();

		assert_eq!(doc.compile(), expected)
	}
}
