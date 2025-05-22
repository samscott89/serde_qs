# Serde Querystrings [![badge-ci]][badge-ci-link] [![Latest Version]][crates.io] [![Documentation]][docs-rs] 


[badge-ci]: https://github.com/samscott89/serde_qs/workflows/Rust%20CI%20checks/badge.svg
[badge-ci-link]: https://github.com/samscott89/serde_qs/actions?query=workflow%3A%22Rust+CI+checks%22+branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/serde_qs.svg
[crates.io]: https://crates.io/crates/serde\_qs
[Documentation]: https://docs.rs/serde_qs/badge.svg
[docs-rs]: https://docs.rs/serde_qs/

This crate is a Rust library for serialising to and deserialising from
querystrings. This crate is designed to extend [`serde_urlencoded`][urlencoded]
when using nested parameters, similar to those used by [qs][qs] for Node, and
commonly used by Ruby on Rails via [Rack][Rack].

The core of the library was inspired by [`serde_urlencoded`][urlencoded].
In order to support abitrarily nested structs encoded in arbitrary orders, we
perform two passes over the input string. This likely adds a non-trivial amount
of memory and compute. Due to this `serde_urlencoded` should be preferred
over this crate whenever non-nested query parameters are sufficient. The crate is built
upon [Serde], a high performance generic serialization framework and [rust-url],
a URL parser for Rust.

[rust-url]: https://github.com/servo/rust-url
[Serde]: https://github.com/serde-rs/serde
[urlencoded]: https://github.com/nox/serde_urlencoded
[qs]: https://www.npmjs.com/package/qs
[Rack]: http://www.rubydoc.info/github/rack/rack/Rack/Utils#parse_nested_query-class_method

Installation
============

This crate works with Cargo and can be found on
[crates.io] with a `Cargo.toml` like:

```toml
[dependencies]
serde_qs = "0.13"
```

Minimum supported Rust version is 1.66.

For older versions of Rust, `serde_qs` versions `<= 0.11` support Rust 1.36.

[crates.io]: https://crates.io/crates/serde_qs

## Performance

This crate includes comprehensive benchmarks to help you understand performance characteristics and make informed decisions about when to use `serde_qs` vs alternatives like `serde_urlencoded`.

### Benchmark Results Summary

For simple flat structures, `serde_urlencoded` is approximately **3x faster**:
- **Simple struct**: `serde_urlencoded` ~134ns vs `serde_qs` ~425ns (serialize)
- **HashMap**: `serde_urlencoded` ~175ns vs `serde_qs` ~536ns (serialize)

However, `serde_qs` provides unique capabilities for nested structures:
- **Nested structs**: ~2.1μs (serialize), ~1.8μs (deserialize)
- **Deep nesting (4 levels)**: ~3.7μs (serialize), ~3.6μs (deserialize)
- **Large collections (100 items)**: ~21μs (serialize/deserialize)

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench serde_qs_benchmarks

# Run specific categories
cargo bench --bench serde_qs_benchmarks -- serialize_simple
cargo bench --bench serde_qs_benchmarks -- comparison

# View detailed results
open target/criterion/report/index.html
```

For detailed benchmark documentation, see [`benches/README.md`](benches/README.md).

### Performance Recommendations

- **Use `serde_urlencoded`** for simple flat structures where performance is critical
- **Use `serde_qs`** when you need nested object support, arrays, or complex query parameter structures
- **Consider the trade-off**: ~3x performance cost for nested structure capabilities

## License

serde_qs is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in serde_qs by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
