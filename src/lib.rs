//! Serde support for querystring-style strings
//!
//! Querystrings are not formally defined and loosely take the form of
//! _nested_ urlencoded queries.
//!
//! This library aims for compatability with the syntax of
//! [qs](https://github.com/ljharb/qs) and also of the
//! [`Rack::Utils::parse_nested_query`](http://www.rubydoc.info/github/rack/rack/Rack/Utils#parse_nested_query-class_method)
//! implementation.
//!
//! For users who do *not* require nested URL parameters, it is highly
//! recommended that the `serde_urlencoded` crate is used instead, which
//! will almost certainly perform better for deserializing simple inputs.
//!
//! ## Supported Types
//!
//! At the **top level**, `serde_qs` only supports `struct`, `map`, and `enum`.
//! These are the only top-level structs which can be de/serialized since
//! Querystrings rely on having a (key, value) pair for each field, which
//! necessitates this kind of structure.
//!
//! However, after the top level you should find all supported types can be
//! de/serialized.
//!
//! ## Usage
//!
//! See the examples folder for a more detailed introduction.
//!
//! Serializing/Deserializing is designed to work with maps and structs.
//!
//! ```
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_qs as qs;
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Address {
//!     city: String,
//!     postcode: String,
//! }
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct QueryParams {
//!     id: u8,
//!     name: String,
//!     address: Address,
//!     phone: u32,
//!     user_ids: Vec<u8>,
//! }
//!
//! # fn main() {
//! let params = QueryParams {
//!     id: 42,
//!     name: "Acme".to_string(),
//!     phone: 12345,
//!     address: Address {
//!         city: "Carrot City".to_string(),
//!         postcode: "12345".to_string(),
//!     },
//!     user_ids: vec![1, 2, 3, 4],
//! };
//! let rec_params: QueryParams = qs::from_str("\
//!     name=Acme&id=42&phone=12345&address[postcode]=12345&\
//!     address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&\
//!     user_ids[2]=3&user_ids[3]=4")
//!     .unwrap();
//! assert_eq!(rec_params, params);
//!
//! # }
extern crate data_encoding;
#[macro_use]
extern crate error_chain;
extern crate itoa;
extern crate dtoa;
#[macro_use]
extern crate serde;
extern crate url;

mod de;
mod ser;

#[doc(inline)]
pub use de::{QsDeserializer, from_bytes, from_reader, from_str};
#[doc(inline)]
pub use de::Config;
#[doc(inline)]
pub use ser::{QsSerializer, to_string};
