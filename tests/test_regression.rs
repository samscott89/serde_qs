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
