use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
	pub unknown_include: ErrorLevel,
	pub unset_varaible: ErrorLevel,
	pub include_method: IncludeMethod,
}

impl Options {
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets whether or not an unresolvable include is an error or not.
	///
	/// ### Default
	/// By default this is set `true` and if an include cannot be found in the
	/// path, the document will fail to parse.
	pub fn unknown_includer<E: Into<ErrorLevel>>(mut self, error_level: E) -> Self {
		self.unknown_include = error_level.into();
		self
	}

	/// Sets whether or not an unfilled variable is an error or not
	///
	/// ### Default
	/// By default this is set `false`. Unset variables are emitted as text when
	/// the document is compiled.
	pub fn unset_varaible<E: Into<ErrorLevel>>(mut self, error_level: E) -> Self {
		self.unset_varaible = error_level.into();
		self
	}

	/// Sets the path where included templates are searched for. See [IncludeMethod]
	/// for more information.
	///
	/// ### Default
	/// By default this is set to [IncludeMethod::Template].
	pub fn include_path(mut self, include_path: IncludeMethod) -> Self {
		self.include_method = include_path;
		self
	}
}

impl Default for Options {
	fn default() -> Self {
		Self {
			unknown_include: ErrorLevel::Error,
			unset_varaible: ErrorLevel::NoError,
			include_method: IncludeMethod::Template,
		}
	}
}

/// The root from which relative includes are resolved from during [Document::compile].
///
/// **CurrentDirectory** will try to resolve include paths according from the current
/// working directory.
///
/// **Template** will try to resolve include paths from the location of the template file.
/// If this is the method set when trying to parse a buffer- when using [Document::from_str]
/// for example- every include will be considered unknown as there is no template path to
/// attempt to resolve from.
///
/// **Path** will try to resolve include paths from the owned `PathBuf`.
#[derive(Clone, Debug, PartialEq)]
pub enum IncludeMethod {
	/// Relative paths are resolved from the current working directory
	CurrentDirectory,
	/// Relative paths are resolved from the location of the template
	Template,
	/// Relative paths are resolved from this path
	Path(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorLevel {
	Error,
	Warning,
	NoError,
}

impl From<bool> for ErrorLevel {
	fn from(b: bool) -> Self {
		if b {
			ErrorLevel::Error
		} else {
			ErrorLevel::NoError
		}
	}
}
