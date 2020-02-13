use std::error::Error as ErrorTrait;
use std::io::Error as IOError;
use std::fmt;

/// Errors that can be encountered when calling functions on a [`Document`]
///
/// [`Document`]: struct.Document.html
#[derive(Debug)]
pub enum Error {
    /// Container for an [`io::Error`]
    ///
    /// [`io::Error`]: https://doc.rust-lang.org/std/io/struct.Error.html
    IOError(IOError),
    /// The pattern could not be found
    BadPattern(String),
}

impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        match self {
            Error::IOError(ioe) => Some(ioe),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(ioe) => ioe.fmt(f),
            Error::BadPattern(name) => write!(f, "The pattern {} does not exist", name)
        }
    }
}

impl From<IOError> for Error {
    fn from(ioe: IOError) -> Self {
        Error::IOError(ioe)
    }
}
