use core::fmt;
use std::{
	collections::HashMap,
	error::Error,
	io,
	iter::Peekable,
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	str::{Chars, FromStr},
};

use crate::{options::IncludeMethod, Options};

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
	options: Options,
	template_path: Option<PathBuf>,
	pub(crate) tokens: Vec<Token>,
	variables: HashMap<String, String>,
	patterns: HashMap<String, Vec<String>>,
}

impl Document {
	/// Attempt to read an entire file and parse it as a Document
	pub fn from_file<P: AsRef<Path>>(path: P, options: Options) -> Result<Self, ParseError> {
		let doc = Self {
			options,
			template_path: Some(path.as_ref().to_owned()),
			tokens: vec![],
			variables: HashMap::new(),
			patterns: HashMap::new(),
		};

		doc.parse_string(Self::read_to_string(path)?)
	}

	fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, ParseError> {
		std::fs::read_to_string(path.as_ref()).map_err(|ioe| ParseError::ReadError {
			inner: ioe,
			file: path.as_ref().to_owned(),
		})
	}

	pub fn from_str<S: AsRef<str>>(s: S, options: Options) -> Result<Self, ParseError> {
		Document {
			options,
			template_path: None,
			tokens: vec![],
			variables: HashMap::new(),
			patterns: HashMap::new(),
		}
		.parse_string(s)
	}

	/// Clear all set variables as if this document was just parsed.
	pub fn clear_variables(&mut self) {
		self.variables.clear();
		self.patterns.clear();
	}

	/// Set a variable with the given key to the given value
	pub fn set<K: Into<String>, V: fmt::Display>(&mut self, key: K, value: V) {
		self.variables.insert(key.into(), format!("{}", value));
	}

	/// Get pattern
	pub fn get_pattern<K: Into<String>>(&self, key: K) -> Option<Pattern> {
		let key = key.into();

		self.tokens.iter().find_map(|tok| {
			if let Token::Pattern {
				pattern_name,
				tokens,
			} = tok
			{
				if *pattern_name == key {
					Some(Pattern {
						name: key.clone(),
						document: Document {
							options: self.options.clone(),
							template_path: self.template_path.clone(),
							tokens: tokens.clone(),
							variables: self.variables.clone(),
							patterns: HashMap::new(),
						},
					})
				} else {
					None
				}
			} else {
				None
			}
		})
	}

	pub fn set_pattern(&mut self, Pattern { name, document }: Pattern) {
		match self.patterns.get_mut(&name) {
			Some(pats) => pats.push(document.compile()),
			None => {
				self.patterns.insert(name, vec![document.compile()]);
			}
		}
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
					else_tokens,
				} => match (self.variables.get(&variable_name), else_tokens) {
					(Some(val), _) if !val.is_empty() => {
						ret.push_str(&self.tokens_to_string(tokens))
					}
					(_, Some(else_tokens)) => ret.push_str(&self.tokens_to_string(else_tokens)),
					_ => (),
				},
				Token::Pattern { pattern_name, .. } => {
					if let Some(pat) = self.patterns.get(&pattern_name) {
						for compiled_pattern in pat {
							ret.push_str(compiled_pattern);
						}
					}
				}
				Token::WrapInclude { .. } => (),
				Token::WrappedContent => (),
				Token::Else => (),
				Token::End => (),
			}
		}

		ret
	}

	fn do_command_structuring(
		mut command: Token,
		iter: &mut impl Iterator<Item = Token>,
	) -> Result<Token, ParseError> {
		loop {
			let token = match iter.next() {
				Some(Token::End) => return Ok(command),
				Some(tok) if tok.is_command() => Self::do_command_structuring(tok, iter)?,
				Some(tok) => tok,
				None => return Err(ParseError::UnclosedCommand),
			};

			match command {
				Token::IfSet {
					ref mut tokens,
					ref mut else_tokens,
					..
				} => match token {
					Token::Else => {
						*else_tokens = Some(vec![]);
					}
					_ => match else_tokens {
						None => tokens.push(token),
						Some(tok) => tok.push(token),
					},
				},
				Token::Pattern { ref mut tokens, .. } => tokens.push(token),
				Token::WrapInclude { ref mut tokens, .. } => tokens.push(token),
				Token::Text(_)
				| Token::Variable { .. }
				| Token::WrappedContent
				| Token::Else
				| Token::End => {
					panic!("Should not be able to get here!")
				}
			}
		}
	}

	fn parse_string<S: AsRef<str>>(mut self, raw: S) -> Result<Self, ParseError> {
		self.first_pass(raw)?;

		let Document {
			options,
			template_path,
			tokens,
			variables,
			patterns,
		} = self;

		let mut iter = tokens.into_iter();
		let mut doc_tokens = vec![];

		loop {
			match iter.next() {
				Some(Token::WrapInclude { document, tokens }) => {
					let wrap = Token::WrapInclude { document, tokens };
					let wrap = Self::do_command_structuring(wrap, &mut iter)?;

					let (doc, mut toks) = if let Token::WrapInclude { document, tokens } = wrap {
						(document.tokens.into_iter(), tokens)
					} else {
						unreachable!()
					};

					for tok in doc {
						if let Token::WrappedContent = tok {
							doc_tokens.extend(toks.drain(..));
						} else {
							doc_tokens.push(tok);
						}
					}
				}
				Some(tok) if tok.is_command() => {
					doc_tokens.push(Self::do_command_structuring(tok, &mut iter)?)
				}
				Some(tok) => doc_tokens.push(tok),
				None => break,
			}
		}

		Ok(Self {
			options,
			template_path,
			tokens: doc_tokens,
			variables,
			patterns,
		})
	}

	// Does all the parsing and follows includes but does not collapse IfSet or Pattern
	fn first_pass<S: AsRef<str>>(&mut self, raw: S) -> Result<(), ParseError> {
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
						Some(_ch) => {
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

					break Ok(());
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
				let stripped_and_trimmed = s.strip_prefix('%').unwrap().trim();
				//Command
				match stripped_and_trimmed.split_once(' ') {
					Some((command, arguments)) => self.parse_command(command, Some(arguments))?,
					None => self.parse_command(stripped_and_trimmed, None)?,
				}
			}
			Some(_) => self.tokens.push(Token::Variable { name: s.into() }),
		}

		Ok(())
	}

	fn parse_command(&mut self, command: &str, arguments: Option<&str>) -> Result<(), ParseError> {
		let invalid_arguments = || {
			Err(ParseError::CommandArgumentInvalid {
				command: command.into(),
				argument: arguments.unwrap_or_default().to_string(),
			})
		};

		match command {
			"else" => {
				self.tokens.push(Token::Else);
				return Ok(());
			}
			"end" => {
				self.tokens.push(Token::End);
				return Ok(());
			}
			"wrapped-content" => {
				self.tokens.push(Token::WrappedContent);
				return Ok(());
			}
			_ => (),
		}

		let arguments = match arguments {
			None => return invalid_arguments(),
			Some(args) if args.is_empty() => return invalid_arguments(),
			Some(args) => args,
		};

		// Reaching here means we have arguments and they are not an empty string
		match command {
			"set" => match arguments.split_once(" ") {
				None => return invalid_arguments(),
				Some((name, value)) => {
					self.variables.insert(name.to_owned(), value.to_owned());
					Ok(())
				}
			},
			"include" => {
				let resolved = self.resolve_include_path(arguments)?;
				let string = Self::read_to_string(resolved)?;
				self.first_pass(string)?;
				Ok(())
			}
			"if-set" => {
				self.tokens.push(Token::IfSet {
					variable_name: arguments.into(),
					tokens: vec![],
					else_tokens: None,
				});

				Ok(())
			}
			"pattern" => {
				self.tokens.push(Token::Pattern {
					pattern_name: arguments.into(),
					tokens: vec![],
				});

				Ok(())
			}
			"wrap-include" => {
				let resolved = self.resolve_include_path(arguments)?;
				let string = Self::read_to_string(resolved)?;
				let doc = Document::from_str(&string, self.options.clone())?;

				self.tokens.push(Token::WrapInclude {
					document: doc,
					tokens: vec![],
				});

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
		Self::from_str(s, Options::default())
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
		else_tokens: Option<Vec<Token>>,
	},
	Pattern {
		pattern_name: String,
		tokens: Vec<Token>,
	},
	WrapInclude {
		document: Document,
		tokens: Vec<Token>,
	},
	WrappedContent,
	Else,
	End,
}

impl Token {
	pub fn is_command(&self) -> bool {
		match self {
			Token::Text(_) => false,
			Token::Variable { .. } => false,
			Token::IfSet { .. } => true,
			Token::Pattern { .. } => true,
			Token::WrapInclude { .. } => true,
			Token::WrappedContent => false,
			Token::Else => false,
			Token::End => false,
		}
	}
}

/// A loose container over a [Document]. This struct is just the [Document]
/// inside the pattern and the name itself.
#[derive(Clone, Debug)]
pub struct Pattern {
	name: String,
	pub document: Document,
}

impl Pattern {
	pub fn new(name: String, document: Document) -> Self {
		Self { name, document }
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	/// Changes the name of this pattern which determines where it is placed in
	/// the [Document] when [Document::set_pattern] is called.
	pub fn set_name(&mut self, name: String) {
		self.name = name;
	}
}

impl Deref for Pattern {
	type Target = Document;

	fn deref(&self) -> &Self::Target {
		&self.document
	}
}

impl DerefMut for Pattern {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.document
	}
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
	UnclosedCommand,
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
			//FIXME: gen- this isn't cute, write a real error
			Self::UnclosedCommand => write!(f, "No end in sight.."),
		}
	}
}
