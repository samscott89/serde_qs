# Serde Querystrings [![badge-ci]][badge-ci-link] [![Latest Version]][crates.io] [![Documentation]][docs-rs]

[badge-ci]: https://github.com/samscott89/serde_qs/workflows/Rust%20CI%20checks/badge.svg
[badge-ci-link]: https://github.com/samscott89/serde_qs/actions?query=workflow%3A%22Rust+CI+checks%22+branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/serde_qs.svg
[crates.io]: https://crates.io/crates/serde_qs
[Documentation]: https://docs.rs/serde_qs/badge.svg
[docs-rs]: https://docs.rs/serde_qs/

This crate is a Rust library for serialising to and deserialising from
querystrings using [`serde`][Serde]. This crate is designed to extend [`serde_urlencoded`][urlencoded]
when using nested parameters, similar to those used by [qs][qs] for Node, and
commonly used by Ruby on Rails via [Rack][Rack].

The core of the library was inspired by [`serde_urlencoded`][urlencoded].
In order to support _deserializing_ abitrarily nested structs encoded in arbitrary orders, we
perform two passes over the input string. This adds a non-trivial amount
of memory and compute, approximately a 50% overhead compared to `serde_urlencoded`.
However, in absolute terms, deserialization is on the order of single-digit microseconds.

Similarly, serialization needs to buffer keys in case there are nested values,
resulting in about 50% overhead.

For detailed benchmark documentation, see [`benches/README.md`](benches/README.md).

[rust-url]: https://github.com/servo/rust-url
[Serde]: https://github.com/serde-rs/serde
[urlencoded]: https://github.com/nox/serde_urlencoded
[qs]: https://www.npmjs.com/package/qs
[Rack]: https://www.rubydoc.info/gems/rack/3.1.15/Rack/Utils#parse_nested_query-class_method

# Installation

> [!IMPORTANT]
> We are currently in the process of stabilizing a major v1 release of this crate.
> If you are evaluating this crate, consider using the release candidate
> over the stable 0.x release.
>
> See [this issue](https://github.com/samscott89/serde_qs/issues/134) and the
> [release notes](./CHANGELOG.md) for more information.

This crate works with Cargo and can be found on
[crates.io] with a `Cargo.toml` like:

```toml
[dependencies]
serde_qs = "0.15"
```

Minimum supported Rust version is 1.82

For older versions of Rust, `serde_qs` versions `<= 0.11` support Rust 1.36.

[crates.io]: https://crates.io/crates/serde_qs

## License

serde_qs is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in serde_qs by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
