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
//! You can fill it out for the named `Ferris` and `Rusty` like so
//! ```rust
//! use bempline::Document;
//!
//! fn main() {
//! 	let doc = Document::from_file("test/template.bpl").unwrap();
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

use std::{
	collections::HashMap,
	io,
	iter::Peekable,
	path::Path,
	str::{Chars, FromStr},
};

#[derive(Clone, Debug)]
pub struct Document {
	tokens: Vec<Token>,
	variables: HashMap<String, String>,
}

impl Document {
	/// Attempt to read an entire file and parse it as a Document
	pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
		std::fs::read_to_string(path).map(|str| str.parse().unwrap())
	}

	/// Clear all set variables as if this document was just parsed.
	pub fn clear_variables(&mut self) {
		self.variables.clear();
	}

	/// Set a variable with the given key to the given value
	pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
		self.variables.insert(key.into(), value.into());
	}

	/// Compile the document into a string. If you set a value for a variable,
	/// it will be replaced. If you have not, the declaration is passed through.
	/// IE: If you have {variable} and do not set a value, it'll come through
	/// with the braces and all.
	pub fn compile(self) -> String {
		let mut ret = String::new();

		for token in self.tokens {
			match token {
				Token::Text(str) => ret.push_str(&str),
				Token::Variable(key) => match self.variables.get(&key) {
					Some(value) => ret.push_str(value),
					None => {
						ret.push('{');
						ret.push_str(&key);
						ret.push('}');
					}
				},
			}
		}

		ret
	}
}

impl FromStr for Document {
	type Err = ();

	fn from_str(raw: &str) -> Result<Self, Self::Err> {
		let mut tokens = vec![];

		let mut current = String::new();
		let mut chars = raw.chars().peekable();
		loop {
			match chars.next() {
				// Escapes
				Some('\\') => match chars.next() {
					// Only esccape the opening brace
					Some('{') => current.push('{'),
					// Keep \ if { is not next
					Some(ch) => {
						current.push('\\');
						current.push(ch);
					}
					// leave it up to the other None handler
					None => (),
				},
				Some('{') => {
					let key = take_while_chars(&mut chars, |ch| ch.is_alphanumeric() || *ch == '_');

					match chars.peek() {
						// Variable is valid!
						Some('}') => {
							if !current.is_empty() {
								tokens.push(Token::Text(current.clone()));
								current.clear();
							}

							tokens.push(Token::Variable(key));
							chars.next(); // throw away the }
						}
						// Variable was not valid, we have to recover
						_ => {
							current.push('{');
							current.push_str(&key);
						}
					}
				}
				Some(ch) => current.push(ch),
				None => {
					if !current.is_empty() {
						tokens.push(Token::Text(current));
					}
					break;
				}
			}
		}

		Ok(Self {
			tokens,
			variables: HashMap::new(),
		})
	}
}

fn take_while_chars(iter: &mut Peekable<Chars>, func: impl Fn(&char) -> bool) -> String {
	let mut ret = String::new();

	loop {
		match iter.peek() {
			Some(ch) if func(ch) => ret.push(iter.next().unwrap()),
			_ => break,
		}
	}

	ret
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
	Text(String),
	Variable(String),
}

#[cfg(test)]
mod test {
	use super::*;

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
		assert_eq!(doc.tokens, vec![Token::Variable(String::from("variable"))]);
	}

	#[test]
	fn sandwhiched_variable() {
		let doc = Document::from_str("Hello {name}, how are you?").unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Text(String::from("Hello ")),
				Token::Variable(String::from("name")),
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
				Token::Variable(String::from("name"))
			]
		);
	}

	#[test]
	fn starts_variable() {
		let doc = Document::from_str("{name}, hello!").unwrap();
		assert_eq!(
			doc.tokens,
			vec![
				Token::Variable(String::from("name")),
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
				Token::Variable(String::from("weather")),
				Token::Text(String::from(" in ")),
				Token::Variable(String::from("location")),
				Token::Text(String::from(" today."))
			]
		);
	}
}
