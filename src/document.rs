use core::fmt;
use std::{
	collections::HashMap,
	error::Error,
	fmt::write,
	io,
	iter::Peekable,
	path::{Path, PathBuf},
	str::{Chars, FromStr},
};

use crate::{options::IncludeMethod, Options};

#[derive(Clone, Debug)]
pub struct Document {
	options: Options,
	template_path: Option<PathBuf>,
	pub(crate) tokens: Vec<Token>,
	variables: HashMap<String, String>,
}

impl Document {
	/// Attempt to read an entire file and parse it as a Document
	pub fn from_file<P: AsRef<Path>>(path: P, options: Options) -> Result<Self, ParseError> {
		let doc = Self {
			options,
			template_path: Some(path.as_ref().to_owned()),
			tokens: vec![],
			variables: HashMap::new(),
		};

		doc.parse_string(std::fs::read_to_string(path.as_ref()).map_err(|ioe| {
			ParseError::ReadError {
				inner: ioe,
				file: path.as_ref().to_owned(),
			}
		})?)
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
	pub fn compile(mut self) -> String {
		let tokens = self.tokens.drain(..).collect();
		self.tokens_to_string(tokens)
	}

	fn tokens_to_string(&self, tokens: Vec<Token>) -> String {
		let mut ret = String::new();

		for token in tokens {
			match token {
				Token::Text(str) => ret.push_str(&str),
				Token::Variable { name } => match self.variables.get(&name) {
					Some(value) => ret.push_str(value),
					None => {
						ret.push('{');
						ret.push_str(&name);
						ret.push('}');
					}
				},
				Token::IfSet {
					variable_name,
					tokens,
				} => match self.variables.get(&variable_name) {
					Some(val) if !val.is_empty() => ret.push_str(&self.tokens_to_string(tokens)),
					_ => (),
				},
				Token::End => (),
			}
		}

		ret
	}

	fn parse_string<S: AsRef<str>>(mut self, raw: S) -> Result<Self, ParseError> {
		let Document {
			options,
			template_path,
			tokens,
			variables,
		} = self.first_pass(raw)?;

		let mut new_tokens = vec![];
		let mut current = None;

		for token in tokens {
			match current {
				None => {
					current = Some(token);
				}
				Some(Token::IfSet { ref mut tokens, .. }) => match token {
					Token::End => {
						new_tokens.push(current.unwrap());
						current = None;
					}
					_ => tokens.push(token),
				},
				Some(tok) => {
					new_tokens.push(tok);
					current = Some(token);
				}
			}
		}

		if let Some(tok) = current {
			new_tokens.push(tok);
		}

		Ok(Self {
			options,
			template_path,
			tokens: new_tokens,
			variables,
		})
	}

	// Does all the parsing and follows includes but does not collapse IfSet
	fn first_pass<S: AsRef<str>>(mut self, raw: S) -> Result<Self, ParseError> {
		let mut current = String::new();
		let mut chars = raw.as_ref().chars().peekable();
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
					// What are we?
					let inside = match chars.peek() {
						Some('%') => {
							// We're a command, take everything until the next '}'
							take_while_chars(&mut chars, |ch| *ch != '}')
						}
						Some(ch) => {
							// We're a variable, no whitespace!
							take_while_chars(&mut chars, |ch| *ch != '}' && !ch.is_whitespace())
						}
						None => {
							current.push('{');
							continue;
						}
					};

					match chars.peek() {
						// Variable is valid!
						Some('}') => {
							if !current.is_empty() {
								self.tokens.push(Token::Text(current.clone()));
								current.clear();
							}

							self.parse_token(inside)?;

							chars.next(); // throw away the }
						}
						// Variable was not valid, we have to recover!
						_ => {
							current.push('{');
							current.push_str(&inside);
						}
					}
				}
				Some(ch) => current.push(ch),
				None => {
					if !current.is_empty() {
						self.tokens.push(Token::Text(current));
					}

					break Ok(self);
				}
			}
		}
	}

	/// Expects unbraced commands. For example the variable `varname` would be
	/// in the document as `{varname}` but should be given as just `varname`.
	fn parse_token<S: AsRef<str>>(&mut self, s: S) -> Result<(), ParseError> {
		let s = s.as_ref();
		match s.chars().next() {
			None => self.tokens.push(Token::Text("{}".into())),
			Some('%') => {
				//Command
				match s.strip_prefix('%').unwrap().split_once(' ') {
					Some((command, arguments)) => self.parse_command(command, arguments)?,
					None => return Err(ParseError::UnknownCommand { command: s.into() }),
				}
			}
			Some(_) => self.tokens.push(Token::Variable { name: s.into() }),
		}

		Ok(())
	}

	fn parse_command(&mut self, command: &str, arguments: &str) -> Result<(), ParseError> {
		match command {
			"include" => {
				if arguments.is_empty() {
					Err(ParseError::CommandArgumentInvalid {
						command: command.into(),
						argument: arguments.into(),
					})
				} else {
					let resolved = self.resolve_include_path(arguments)?;
					let doc = Document::from_file(resolved, self.options.clone())?;
					self.tokens.extend_from_slice(&doc.tokens);
					Ok(())
				}
			}
			"if-set" => {
				if arguments.is_empty() {
					Err(ParseError::CommandArgumentInvalid {
						command: command.into(),
						argument: arguments.into(),
					})
				} else {
					self.tokens.push(Token::IfSet {
						variable_name: arguments.into(),
						tokens: vec![],
					});

					Ok(())
				}
			}
			"end" => {
				self.tokens.push(Token::End);
				Ok(())
			}
			_ => Err(ParseError::UnknownCommand {
				command: command.to_owned(),
			}),
		}
	}

	fn resolve_include_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, ParseError> {
		match self.options.include_method {
			IncludeMethod::Path(ref buf) => {
				let mut buf = buf.clone();

				if buf.is_file() {
					buf.pop();
				}
				buf.push(path);

				buf.canonicalize()
					.map_err(|ioe| ParseError::CanonicalizationError {
						path: buf,
						inner: ioe,
					})
			}
			IncludeMethod::CurrentDirectory => {
				path.as_ref()
					.canonicalize()
					.map_err(|ioe| ParseError::CanonicalizationError {
						path: path.as_ref().to_owned(),
						inner: ioe,
					})
			}
			IncludeMethod::Template => {
				if let Some(ref buf) = self.template_path {
					let mut buf = buf.clone();

					if buf.is_file() {
						buf.pop();
					}
					buf.push(path);

					buf.canonicalize()
						.map_err(|ioe| ParseError::CanonicalizationError {
							path: buf,
							inner: ioe,
						})
				} else {
					Err(ParseError::UnresolvableInclude {
						included_file: path.as_ref().to_owned(),
						include_path: PathBuf::new(),
						from_buffer_template: true,
					})
				}
			}
		}
	}
}

impl FromStr for Document {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Document {
			options: Options::default(),
			template_path: None,
			tokens: vec![],
			variables: HashMap::new(),
		}
		.parse_string(s)
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
pub enum Token {
	Text(String),
	Variable {
		name: String,
	},
	IfSet {
		variable_name: String,
		tokens: Vec<Token>,
	},
	End,
}

#[derive(Debug)]
pub enum ParseError {
	ReadError {
		file: PathBuf,
		inner: io::Error,
	},
	CanonicalizationError {
		path: PathBuf,
		inner: io::Error,
	},
	UnknownCommand {
		command: String,
	},
	CommandArgumentInvalid {
		command: String,
		argument: String,
	},
	UnresolvableInclude {
		included_file: PathBuf,
		include_path: PathBuf,
		from_buffer_template: bool,
	},
}

impl Error for ParseError {}
impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ParseError::ReadError { file, inner } => {
				write!(
					f,
					"There was a problem reading '{}': {}",
					file.to_string_lossy(),
					inner
				)
			}
			ParseError::CanonicalizationError { path, inner } => {
				write!(
					f,
					"Could not canonixalize the path '{}': {}",
					path.to_string_lossy(),
					inner
				)
			}
			ParseError::UnknownCommand { command } => {
				write!(f, "'{}' is not a valid command", command)
			}
			ParseError::CommandArgumentInvalid { command, argument } => {
				write!(
					f,
					"'{}' is not a valid argument for the command {}",
					argument, command
				)
			}
			ParseError::UnresolvableInclude {
				included_file,
				include_path,
				from_buffer_template,
			} => {
				if *from_buffer_template {
					write!(f, "Could not find the included template '{}' because the IncludeMethod is Template and a buffer was parsed", included_file.to_string_lossy())
				} else {
					write!(
						f,
						"Could not find the included template '{}' while looking in '{}'",
						included_file.to_string_lossy(),
						include_path.to_string_lossy()
					)
				}
			}
		}
	}
}
