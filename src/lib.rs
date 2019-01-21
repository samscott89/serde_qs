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
//! Note that integer keys are reserved for array indices. That is, a string of
//! the form `a[0]=1&a[1]=3` will deserialize to the ordered sequence `a =
//! [1,3]`.
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
//! ```
//!
//! ## Strict vs Non-Strict modes
//!
//! `serde_qs` supports two operating modes, which can be specified using
//! [`Config`](struct.Config.html), and is all about how `serde_qs` handles square brackets.
//!
//! Techncially, square brackets should be encoded in URLs as `%5B` and `%5D`.
//! However, they are often used in their raw format to specify querystrings
//! such as `a[b]=123`.
//!
//! In strict mode, `serde_qs` will only tolerate unencoded square brackets
//! to denote nested keys. So `a[b]=123` will decode as `{"a": {"b": 123}}`.
//! This means that encoded square brackets can actually be part of the key.
//! `a[b%5Bc%5D]=123` becomes `{"a": {"b[c]": 123}}`.
//!
//! However, since some implementations will automatically encode everything
//! in the URL, we also have a non-strict mode. This means that `serde_qs`
//! will assume that any encoded square brackets in the string were meant to
//! be taken as nested keys. From the example before, `a[b%5Bc%5D]=123` will 
//! now become `{"a": {"b": {"c": 123 }}}`.
//!
//! Non-strict mode can be useful when, as said before, some middleware
//! automatically encodes the brackets. But care must be taken to avoid
//! using keys with square brackets in them, or unexpected things can
//! happen.
//!
//! 

#![allow(
)]
#![deny(
    const_err,
    dead_code,
    deprecated,
    exceeding_bitshifts,
    improper_ctypes,
    missing_docs,
    mutable_transmutes,
    no_mangle_const_items,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    plugin_as_library,
    stable_features,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unknown_crate_types,
    unknown_lints,
    unreachable_code,
    unsafe_code,
    unstable_features,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_comparisons,
    unused_extern_crates,
    unused_features,
    unused_imports,
    unused_import_braces,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_qualifications,
    unused_results,
    unused_unsafe,
    unused_variables,
    variant_size_differences,
    while_true,
)]

extern crate data_encoding;
#[macro_use]
extern crate error_chain;
extern crate percent_encoding;
#[macro_use]
extern crate serde;

mod de;
mod error;
mod ser;


#[doc(inline)]
pub use de::{from_bytes, from_str};
#[doc(inline)]
pub use de::Config;
pub use error::Error;
#[doc(inline)]
pub use ser::{QsSerializer, to_string, to_writer};
