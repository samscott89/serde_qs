# Changelog

## [1.0.0](https://github.com/samscott89/serde_qs/compare/v1.0.0-rc.4...v1.0.0) - 2026-01-09

### Other

- fix typos and improve clarity in documentation comments ([#157](https://github.com/samscott89/serde_qs/pull/157))

## [1.0.0-rc.4](https://github.com/samscott89/serde_qs/compare/v1.0.0-rc.3...v1.0.0-rc.4) - 2025-12-14

### Other

- Bump warp ([#156](https://github.com/samscott89/serde_qs/pull/156))
- Update deserialization logic + add principles. ([#147](https://github.com/samscott89/serde_qs/pull/147))
- Bump MSRV + add profiling examples ([#154](https://github.com/samscott89/serde_qs/pull/154))

## [1.0.0-rc.3](https://github.com/samscott89/serde_qs/compare/v1.0.0-rc.2...v1.0.0-rc.3) - 2025-05-27

### Other

- Error when hitting serialization max depth ([#144](https://github.com/samscott89/serde_qs/pull/144))
- Deserialize empty value as bool=true ([#142](https://github.com/samscott89/serde_qs/pull/142))

## [1.0.0-rc.2](https://github.com/samscott89/serde_qs/compare/v1.0.0-rc.1...v1.0.0-rc.2) - 2025-05-27

### Other

- Cleanup of web utils remove redundant `OptionalQsQuery` (since `Option<T>` now works)
  and add `QsForm` ([#140](https://github.com/samscott89/serde_qs/pull/140))
- More tests / minor change to field encoding ([#139](https://github.com/samscott89/serde_qs/pull/139))

## [1.0.0-rc.1](https://github.com/samscott89/serde_qs/compare/v1.0.0-rc.0...v1.0.0-rc.1) - 2025-05-26

### Other

- Fix maps with integer keys. ([#138](https://github.com/samscott89/serde_qs/pull/138))
- Support explicit serialization formatting for arrays. ([#137](https://github.com/samscott89/serde_qs/pull/137))
- v1 changelog (and more tests) ([#135](https://github.com/samscott89/serde_qs/pull/135))

## [1.0.0-rc.0](https://github.com/samscott89/serde_qs/compare/v0.15.0...v1.0.0-rc.0) - 2025-05-26

This release constitutes an full, incremental rewrite of v0.15 (the core of which was written about [8 years ago](https://github.com/samscott89/serde_qs/commit/6e71ba43eb6bd62f2b567224e387333016bd3a5c#diff-a9463680bdf3fa7278b52b437bfbe9072e20023a015621ed23bcb589f6ccd4b5)).

## Changes

The rewrite:
- addresses numerous existing bugs and feature requests
- expands support for many more types
- ensures most types roundtrip (ser -> de) correctly, with an extensive test suite to check
- implements numerous performance optimizations, resulting in ~3x speedup and times generally in the sub-microsecond range.
- simplifies a ton of the internal code, removing unnecessary de/serialization abstractions

The goal is to release this as a stable release shortly. Please leave any feedback here: https://github.com/samscott89/serde_qs/issues/134

## Breaking changes / migration guide

### Removal of `strict_mode`

Previously, we had a configuration option called `strict_mode` which would do a couple of things:
- Require brackets to be encoded as `[`, `]` instead of `%5B`, `%5D`
- Perform UTF8 validation (when `strict_mode` is false, will use `from_string_lossy`).

In V1, these are replaced by:
- The `use_form_encoding` configuration option. Defaults to `false` which means that square brackets
  will be serialized/deserialized as `[`, `]`. This is suitable to be used when (a) you control both
  sides of the request flow and can control this, or (b) when parsing from querystrings. When
  set to `true`, square brackets will always be percent-encoded.
- We _always_ perform UTF8 validation of strings. If there are invalid UTF8 characters in the input,
  your options are either to (a) deserialize to a `Vec<u8>` first and to the conversion yourself,
  or (b) ignored fields do _not_ perform validation and can be safely ignored (the original motivation
  for the feature).

Alongside this change, the configuration interface has changed:

```rust
// OLD
let config = serde_qs::Config::new(5, true); // defaults; (max_depth, strict_mode)

// NEW / V1
let config = serde_qs::Config::new().use_form_encoding(false).max_depth(5); // defaults
```

In addition, the configuration is now relevant for serialization too. Previously `serde_qs`
could not output percent-encoded square brackets. To support this, the `Config` struct now
also has serialization methods:

```rust
let config = serde_qs::Config::new().use_form_encoding(true);
let res = config.serialize_string(&data);
// ...
```

### Serialization of "empty" values

Previously, serialization did not differentiate between "null" values like `None` and
"empty" values like `""` or `vec![]`. This prevented values like `Some("")` from
being able to roundtrip successfully.

To address this, `serde_qs` now serializes unit values like `x: None` as `"x"` and `x: Some("")` as `"x="`.

Similarly, unit values (like unit enum variants) serialize a little differently. Previously `x: Foo::Bar`
would serialize as `x=Bar`. This now serializes as `x[Bar]` (which is more symmetric with other enum variants).

Both of these changes are backwards compatible on the server side -- serde v0.15 will succesfully deserialize
all of the above the same as previously.

This will result in longer query strings, since null values are now explicitly encoded rather than skipped.
To reduce the size of querystrings, users can still use attributes like `#[serde(skip_serializing_if = "Option::is_none")]`
and `#[serde(default)]`. Previously these were _required_ to prevent errors.

### Fewer characters percent-encoding

We now based our encoding scheme on the [WHATWG](https://url.spec.whatwg.org/#query-percent-encode-set) specification.
The query percent-encoding set is much smaller than previously, which means fewer characters need to be encoded.
The `use_form_encoding` option uses the `application/x-www-form-urlencoded` encoding set which is very broad.

**If there are any use cases where neither is suitable, please raise [here](https://github.com/samscott89/serde_qs/issues/134)**.

### Snapshots

We now have snapshot tests as part of the roundtrip de/seriailzation tests. Although most of the cases are net new
and previously unsupported, [here are some examples](https://github.com/samscott89/serde_qs/commit/5737179913a57928d6bb30fcf94083921b420e5f) of the diff from running snapshots from 0.15 to v1.

### Repeated keys

We no longer error on repeated keys. Instead, there is the following behaviour:

- Repeated keys are collected into a `Vec` of values
- If the deserializer expects a primitive value, we'll take the **last** value
- If the deserializer expects a sequence, we'll deserialize all values

### Max depth change

Previously, `max_depth` was confusingly "off by one". Where flat keys were considered depth `1`, and
a single level of nested required `max_depth = 2`. This is now fixed.

## New Features

### Support for de/serializing more types

Most types are now supported, even at the top level. Including primitive types like `String`, `u8` which get encoded
as `=foo` and `=123`, as well as vecs (`0=foo&1=bar`), and enums.

The primary type that `serde_qs` cannot handle are untagged enums, or internally tagged enums, with types
that need to parse from a `String`. This is due to a `serde` limitation in which the values are buffered into
a string, and we lose the knowledge of what type the serializer is expecting.

### Array formatting helpers

`serde_qs::helpers` contains some helper modules for formatting arrays as a single string,
all usable with the `#[serde(with = "...")]` attribute.

### Deserialization of implicit lists

As part of changing how we handle repeated keys, lists can be deserialized implicitly, e.g.
from `v=1&v=2&v=3` to `vec![1, 2, 3]`.

### Global configuration option

We've introduced a cargo feature `default_to_form_encoding` which changes the default `Config` to `use_form_encoding: true`.
This can be helpful for end applications that want to default to this encoding, or scenarios where `serde_qs` is embedded 
in a library without a way to change the configuration.

## [0.15.0](https://github.com/samscott89/serde_qs/compare/v0.14.0...v0.15.0) - 2025-04-22

### Other

- Support preserving order of parameters when serializing to a Map. ([#106](https://github.com/samscott89/serde_qs/pull/106))
- Fix clippy. ([#129](https://github.com/samscott89/serde_qs/pull/129))
- reorder struct fields to avoid serde buffering ([#128](https://github.com/samscott89/serde_qs/pull/128))

## [0.14.0](https://github.com/samscott89/serde_qs/compare/v0.13.0...v0.14.0) - 2025-03-04

### Other

- Add release plz CI ([#125](https://github.com/samscott89/serde_qs/pull/125))
- Update CI config ([#124](https://github.com/samscott89/serde_qs/pull/124))
- update axum to v0.8 ([#118](https://github.com/samscott89/serde_qs/pull/118))
- :multiple_bound_locations ([#103](https://github.com/samscott89/serde_qs/pull/103))
- Add axum::OptionalQsQuery ([#102](https://github.com/samscott89/serde_qs/pull/102))
- generate docs for axum support as well ([#100](https://github.com/samscott89/serde_qs/pull/100))
- Update README.md

## Version 0.13.0

- Bump `axum` support to 0.7
- Remove support for `actix-web 2.0`
- Add support for extracting form data in actix via `QsForm`
