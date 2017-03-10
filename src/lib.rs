//! Serde support for querystring-style strings
//!
//! Querystrings are not formally defined and loosely take the form of
//! _nested_ urlencoded queries.
//!
//! This library aims for compatability with the syntax of
//! [qs](https://github.com/ljharb/qs) and also of the [Rack::Utils::parse_neste
//! d_query](http://www.rubydoc.info/github/rack/rack/Rack/Utils
//! #parse_nested_query-class_method) implementation.
//!
//! For users who do *not* require nested URL parameters, it is highly
//! recommended that the `serde_urlencoded` crate is used instead, which 
//! will almost certainly perform better for deserializing simple inputs.
//! 
//! The serialization implementation of this library is adapted from
//! `serde_urlencoded`.

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
