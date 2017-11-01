use data_encoding;
use serde::de;

use std::fmt::Display;
use std::io;
use std::num;
use std::str;
use std::string;

error_chain! {
    errors { 
        Custom(msg: String)
        Parse(msg: String, pos: (usize, usize)) {
            description("parsing failure")
            display("parsing failed with error: '{}' at position: {:?}", msg, pos)
        }
        Unsupported
    }

    foreign_links {
        Decoding(data_encoding::decode::Error);
        FromUtf8(string::FromUtf8Error);
        Io(io::Error);
        ParseInt(num::ParseIntError);
        Utf8(str::Utf8Error);
    }
}

impl Error {
    /// Generate error to show top-level type cannot be deserialized.
    pub fn top_level(object: &'static str) -> Self {
        ErrorKind::Custom(format!("cannot deserialize {} at the top level.\
                           Try deserializing into a struct.", object)).into()

    }

    /// Generate a parsing error message with position.
    pub fn parse_err<T>(msg: T, position: (usize, usize)) -> Self
        where T: Display {
        ErrorKind::Parse(msg.to_string(), position).into()
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self 
        where T: Display {
            ErrorKind::Custom(msg.to_string()).into()
    }
}