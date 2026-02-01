use serde::de;

use std::fmt::{self, Display};
use std::io;
use std::num;
use std::str;
use std::string;

/// Error type for `serde_qs`.
#[derive(Debug)]
pub enum Error {
    /// Custom string-based error
    Custom(String),

    /// Maximum serialization depth exceeded
    MaxSerializationDepthExceeded(usize),

    /// Parse error at a specified position in the query string
    Parse(String, usize),

    /// Unsupported type that `serde_qs` can't serialize into a query string
    Unsupported,

    /// Error processing UTF-8 for a `String`
    FromUtf8(string::FromUtf8Error),

    /// I/O error
    Io(io::Error),

    /// Error parsing a number
    ParseInt(num::ParseIntError),

    /// Error processing UTF-8 for a `str`
    Utf8(str::Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "{}", msg),
            Error::MaxSerializationDepthExceeded(depth) => {
                write!(f, "Maximum serialization depth `{depth}` exceeded")
            }
            Error::Parse(msg, pos) => {
                write!(f, "parsing failed with error: '{msg}' at position: {pos}")
            }
            Error::Unsupported => write!(f, "unsupported type for serialization"),
            Error::FromUtf8(e) => write!(f, "{e}"),
            Error::Io(e) => write!(f, "{e}"),
            Error::ParseInt(e) => write!(f, "{e}"),
            Error::Utf8(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::FromUtf8(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::ParseInt(e) => Some(e),
            Error::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(e: string::FromUtf8Error) -> Self {
        Error::FromUtf8(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl Error {
    /// Generate error to show top-level type cannot be deserialized.
    pub fn top_level(object: &'static str) -> Self {
        Error::Custom(format!(
            "cannot deserialize {object} at the top level.\
             Try deserializing into a struct.",
        ))
    }

    /// Generate a parsing error message with position.
    pub fn parse_err<T>(msg: T, position: usize) -> Self
    where
        T: Display,
    {
        Error::Parse(msg.to_string(), position)
    }

    #[cfg(feature = "debug_parsed")]
    pub fn custom<T, D: std::fmt::Debug>(msg: T, parsed: &D) -> Self
    where
        T: Display,
    {
        Error::Custom(format!("{msg}\nParsed:\n{parsed:#?}"))
    }

    #[cfg(not(feature = "debug_parsed"))]
    pub fn custom<T, D: std::fmt::Debug>(msg: T, _parsed: &D) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
