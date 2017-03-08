//! Serde support for querystring-style strings

#![warn(unused_extern_crates)]

extern crate fnv;
extern crate itoa;
extern crate dtoa;
#[macro_use]
extern crate serde;
extern crate url;

#[macro_use]
extern crate serde_derive;


pub mod de;
pub mod ser;

#[doc(inline)]
pub use de::{Deserializer, from_bytes, from_reader, from_str};
#[doc(inline)]
pub use ser::{Serializer, to_string};
