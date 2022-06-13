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
mod options;

pub use document::Document;
pub use document::Token;
pub use options::Options;

#[cfg(test)]
mod test {
	use crate::options::IncludeMethod;

	use super::*;
	use std::{path::PathBuf, str::FromStr};

	#[test]
	fn compile_all_set() {
		let mut doc = Document::from_str("One: {one} | Two: {two} | Three: {three}").unwrap();
		doc.set("one", "1");
		doc.set("two", "2");
		doc.set("three", "3");

		assert_eq!(&doc.compile(), "One: 1 | Two: 2 | Three: 3");
	}

	#[test]
	fn compile_some_set() {
		let mut doc = Document::from_str("One: {one} | Two: {two} | Three: {three}").unwrap();
		doc.set("one", "1");
		doc.set("three", "3");

		assert_eq!(&doc.compile(), "One: 1 | Two: {two} | Three: 3");
	}

	// Parsing related tests

	#[test]
	fn no_text() {
		let doc = Document::from_str("").unwrap();
		assert_eq!(doc.tokens, vec![]);
	}

	#[test]
	fn only_text() {
		let doc = Document::from_str("Nothing but text").unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Text(String::from("Nothing but text"))]
		);
	}

	#[test]
	fn escaped_bracket() {
		let doc = Document::from_str("escape this: \\{, but not this \\n").unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Text(String::from(
				"escape this: {, but not this \\n"
			))]
		);
	}

	#[test]
	fn only_variable() {
		let doc = Document::from_str("{variable}").unwrap();
		assert_eq!(
			doc.tokens,
			vec![Token::Variable {
				name: String::from("variable")
			}]
		);
	}

	#[test]
	fn sandwhiched_variable() {
		let doc = Document::from_str("Hello {name}, how are you?").unwrap();
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
		let doc = Document::from_str("Hello {name}").unwrap();
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
		let doc = Document::from_str("{name}, hello!").unwrap();
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
		let doc = Document::from_str("The weather is {weather} in {location} today.").unwrap();
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
	fn ifset_variable_set() {
		let mut doc = Document::from_str("{%if-set foo}set!{%end end}").unwrap();
		doc.set("foo", "");

		assert_eq!(doc.compile(), "set!")
	}
}
