use data_encoding;
use serde::de;

use std::fmt::Display;
use std::io;
use std::string;

error_chain! {
    errors { 
        Custom(msg: String)
        Unsupported
    }

    foreign_links {
        Decoding(data_encoding::decode::Error);
        Io(io::Error);
        Utf8(string::FromUtf8Error);
    }
}

impl Error {
    /// Generate error to show top-level type cannot be deserialized.
    pub fn top_level(object: &'static str) -> Error {
        ErrorKind::Custom(format!("cannot deserialize {} at the top level.\
                           Try deserializing into a struct.", object)).into()

    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self 
        where T: Display {
            ErrorKind::Custom(msg.to_string()).into()
    }
}