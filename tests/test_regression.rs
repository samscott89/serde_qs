use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[test]
fn double_encoding_keys() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Human {
        #[serde(rename = "full name")]
        name: String,
    }

    let human = Human {
        name: "John Doe".to_string(),
    };

    let encoded = serde_qs::to_string(&human).unwrap();
    print!("{}", encoded);
    assert_eq!(serde_qs::from_str::<Human>(&encoded).unwrap(), human);
}

#[test]
fn ignored_compound_fields_do_not_fail_index_check() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Test {
        example: HashMap<i32, i32>,
    }

    let decoded: Test = serde_qs::from_str("example[123]=321&unrelated[123]=321").unwrap();
    assert_eq!(decoded.example.get(&123), Some(&321));

    let decoded: Test = serde_qs::from_str("example[123]=321&unrelated[0][a]=1").unwrap();
    assert_eq!(decoded.example.get(&123), Some(&321));
}

#[test]
fn skipped_compound_fields_do_not_fail_index_check() {
    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct Test {
        example: HashMap<i32, i32>,
        #[serde(skip)]
        unrelated: i8,
    }

    let decoded: Test = serde_qs::from_str("example[123]=321&unrelated[123]=321").unwrap();
    assert_eq!(decoded.example.get(&123), Some(&321));
    assert_eq!(decoded.unrelated, 0);
}

/// https://github.com/samscott89/serde_qs/issues/170
///
/// The default (non-form) encoding set did not include `%`, so any `%` in a
/// string value or map key was emitted raw and re-decoded on the way back.
#[test]
fn percent_signs_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Q {
        name: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct W {
        a: std::collections::BTreeMap<String, String>,
    }

    // values containing something that looks like a percent escape
    for value in ["%41", "100%25", "a%3Db", "%5Bx%5D", "%AD0", "%c5", "50%"] {
        let q = Q {
            name: value.to_string(),
        };
        let encoded = serde_qs::to_string(&q).unwrap();
        assert_eq!(
            serde_qs::from_str::<Q>(&encoded).unwrap(),
            q,
            "value {value:?} did not roundtrip (encoded as {encoded:?})"
        );
    }

    assert_eq!(
        serde_qs::to_string(&Q {
            name: "%41".to_string()
        })
        .unwrap(),
        "name=%2541"
    );

    // map keys, too
    let map_key = |key: &str| {
        let mut m = std::collections::BTreeMap::new();
        m.insert(key.to_string(), "1".to_string());
        W { a: m }
    };

    for key in ["K%47", "%5Bx%5D", "[x]"] {
        let w = map_key(key);
        let encoded = serde_qs::to_string(&w).unwrap();
        assert_eq!(
            serde_qs::from_str::<W>(&encoded).unwrap(),
            w,
            "key {key:?} did not roundtrip (encoded as {encoded:?})"
        );
    }

    // distinct keys must not collide
    assert_ne!(
        serde_qs::to_string(&map_key("[x]")).unwrap(),
        serde_qs::to_string(&map_key("%5Bx%5D")).unwrap()
    );
}

/// https://github.com/samscott89/serde_qs/issues/176
///
/// Form-encoding mode eagerly rewrites percent-encoded brackets so that nested
/// keys can still form structure, but it compared the two bytes after `%`
/// against the literal uppercase `5B`/`5D`. Lowercase `%5b`/`%5d` fell through
/// to the catch-all arm, so the key stayed flat and no error was raised.
///
/// The hex digits of a percent-encoding are case-insensitive (RFC 3986 section
/// 6.2.2.1), and this crate's own `char_to_hexdigit` already accepts both
/// cases, so the two paths disagreed.
#[test]
fn form_encoded_brackets_are_case_insensitive() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Outer {
        abc: Inner,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Inner {
        def: String,
    }

    let config = serde_qs::Config::new().use_form_encoding(true);
    let expected = Outer {
        abc: Inner {
            def: "ghi".to_string(),
        },
    };

    for encoded in [
        "abc%5Bdef%5D=ghi",
        "abc%5bdef%5d=ghi",
        // mixed case in a single key
        "abc%5Bdef%5d=ghi",
    ] {
        let decoded: Outer = config
            .deserialize_str(encoded)
            .unwrap_or_else(|e| panic!("{encoded:?} failed to deserialize: {e}"));
        assert_eq!(decoded, expected, "{encoded:?} did not nest correctly");
    }

    // a doubly-encoded bracket must NOT be rewritten into structure, in either
    // case: the rewrite inspects the two bytes after `%` (`25`) and declines,
    // so the key stays flat and keeps the literal `%5B` text
    for (encoded, key) in [
        ("abc%255Bdef%255D=ghi", "abc%5Bdef%5D"),
        ("abc%255bdef%255d=ghi", "abc%5bdef%5d"),
    ] {
        let decoded: HashMap<String, String> = config
            .deserialize_str(encoded)
            .unwrap_or_else(|e| panic!("{encoded:?} failed to deserialize: {e}"));
        assert_eq!(
            decoded.keys().collect::<Vec<_>>(),
            vec![key],
            "{encoded:?} should stay flat"
        );
    }
}
