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
