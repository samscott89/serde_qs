#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

/// Helper function for testing serialization with default config
#[track_caller]
fn serialize_test<T: serde::Serialize>(data: &T, expected: &str) {
    let serialized = qs::to_string(data).expect("serialize");
    assert_eq!(serialized, expected);
}

/// Helper function for testing serialization with custom config
#[track_caller]
fn serialize_test_with_config<T: serde::Serialize>(data: &T, expected: &str, config: qs::Config) {
    let serialized = config.serialize_string(data).expect("serialize");
    assert_eq!(serialized, expected);
}

#[track_caller]
fn serialize_test_with_config_err<T: serde::Serialize>(
    data: &T,
    expected_err: &str,
    config: qs::Config,
) {
    let err = config.serialize_string(data).unwrap_err();
    assert!(err.to_string().contains(expected_err), "got: {}", err);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Address {
    city: String,
    street: String,
    postcode: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    id: u8,
    name: String,
    phone: u32,
    address: Address,
    user_ids: Vec<u8>,
}

#[test]
fn serialize_struct() {
    let params = QueryParams {
        id: 42,
        name: "Acme".to_string(),
        phone: 12345,
        address: Address {
            city: "Carrot City".to_string(),
            street: "Special-Street* No. 11".to_string(),
            postcode: "12345".to_string(),
        },
        user_ids: vec![1, 2, 3, 4],
    };

    serialize_test(
        &params,
        "id=42&name=Acme&phone=12345&address[city]=Carrot+City&\
         address[street]=Special-Street*+No.+11&\
         address[postcode]=12345&user_ids[0]=1&user_ids[1]=2&\
         user_ids[2]=3&user_ids[3]=4",
    );
}

#[test]
fn serialize_option() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Option<Vec<u8>>,
    }

    serialize_test(&Query { vec: None }, "vec");
    serialize_test(
        &Query {
            vec: Some(vec![1, 2]),
        },
        "vec[0]=1&vec[1]=2",
    );
}

#[test]
fn serialize_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        A,
        B(bool),
        C { x: u8, y: u8 },
        D(u8, u8),
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        e: TestEnum,
    }

    serialize_test(&Query { e: TestEnum::A }, "e[a]");
    serialize_test(
        &Query {
            e: TestEnum::B(true),
        },
        "e[b]=true",
    );
    serialize_test(
        &Query {
            e: TestEnum::C { x: 2, y: 3 },
        },
        "e[c][x]=2&e[c][y]=3",
    );
    serialize_test(
        &Query {
            e: TestEnum::D(128, 1),
        },
        "e[d][0]=128&e[d][1]=1",
    );
}

#[test]
fn serialize_flatten() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Query {
        a: u8,
        #[serde(flatten)]
        common: CommonParams,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct CommonParams {
        limit: u64,
        offset: u64,
    }

    serialize_test(
        &Query {
            a: 1,
            common: CommonParams {
                limit: 100,
                offset: 50,
            },
        },
        "a=1&limit=100&offset=50",
    );
}

#[test]
fn serialize_map_with_unit_enum_keys() {
    use std::collections::HashMap;

    #[derive(Serialize, Eq, PartialEq, Hash)]
    enum Operator {
        Lt,
        Gt,
    }

    #[derive(Serialize)]
    struct Filter {
        point: HashMap<Operator, u64>,
    }

    let mut map = HashMap::new();
    map.insert(Operator::Gt, 123);
    map.insert(Operator::Lt, 321);
    let test = Filter { point: map };

    let query = qs::to_string(&test).unwrap();

    assert!(query == "point[Lt]=321&point[Gt]=123" || query == "point[Gt]=123&point[Lt]=321");
}

#[test]
fn serialize_bytes() {
    struct Bytes(&'static [u8]);

    #[derive(Serialize)]
    struct Query {
        bytes: Bytes,
    }

    impl serde::Serialize for Bytes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(self.0)
        }
    }
    let query = Query {
        bytes: Bytes(b"hello, world!"),
    };
    serialize_test(&query, "bytes=hello,+world!");

    let form_config = qs::Config::new().use_form_encoding(true);
    serialize_test_with_config(&query, "bytes=hello%2C%20world%21", form_config);
}

#[test]
fn serialize_map_keys() {
    // Issue: https://github.com/samscott89/serde_qs/issues/45

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct MapParams {
        attrs: std::collections::BTreeMap<String, String>,
    }

    let data = MapParams {
        attrs: vec![
            ("key 1!".to_owned(), "val 1".to_owned()),
            ("key 2!".to_owned(), "val 2".to_owned()),
        ]
        .into_iter()
        .collect(),
    };
    serialize_test(&data, "attrs[key+1!]=val+1&attrs[key+2!]=val+2");
}

#[test]
fn test_serializer() {
    use serde::Serialize;
    #[derive(Serialize, Debug, Clone)]
    struct Query {
        a: Vec<u8>,
        b: &'static str,
    }

    serialize_test(
        &Query {
            a: vec![0, 1],
            b: "b",
        },
        "a[0]=0&a[1]=1&b=b",
    );

    serialize_test(
        &Query {
            a: vec![3, 2],
            b: "a",
        },
        "a[0]=3&a[1]=2&b=a",
    );
}

#[test]
fn test_serializer_unit() {
    use serde::Serialize;
    #[derive(Serialize)]
    struct A;
    #[derive(Serialize)]
    struct B {
        t: (),
    }

    // allow this clippy lints cause I like how explicit the test is
    #[allow(clippy::let_unit_value)]
    let unit = ();
    serialize_test(&unit, "=");

    serialize_test(&A, "=");
    serialize_test(&B { t: () }, "t=");
}

#[test]
fn formencoding() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct NestedData {
        a: u8,
        b: String,
    }

    #[derive(Serialize)]
    struct Query {
        data: Vec<NestedData>,
    }

    let query = Query {
        data: vec![
            NestedData {
                a: 1,
                b: "test!".to_string(),
            },
            NestedData {
                a: 2,
                b: "example:.".to_string(),
            },
        ],
    };

    let form_config = qs::Config::new().use_form_encoding(true);
    serialize_test_with_config(
        &query,
        "data%5B0%5D%5Ba%5D=1&data%5B0%5D%5Bb%5D=test%21&data%5B1%5D%5Ba%5D=2&data%5B1%5D%5Bb%5D=example%3A.",
        form_config,
    );
}

#[test]
fn max_depth() {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Nested {
        a: u8,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        b: Vec<u8>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        c: Vec<Vec<u8>>,
    }

    let nested_none = Nested {
        a: 1,
        b: vec![],
        c: vec![],
    };
    let nested_one = Nested {
        a: 1,
        b: vec![2, 3],
        c: vec![],
    };
    let nested_two = Nested {
        a: 1,
        b: vec![2, 3],
        c: vec![vec![4]],
    };

    let config_zero = qs::Config::new().max_depth(0);
    let config_one = qs::Config::new().max_depth(1);
    let config_two = qs::Config::new().max_depth(2);
    serialize_test_with_config(&nested_none, "a=1", config_zero);
    serialize_test_with_config(&nested_none, "a=1", config_one);
    serialize_test_with_config(&nested_none, "a=1", config_two);

    serialize_test_with_config_err(&nested_one, "Maximum serialization depth", config_zero);
    serialize_test_with_config(&nested_one, "a=1&b[0]=2&b[1]=3", config_one);
    serialize_test_with_config(&nested_one, "a=1&b[0]=2&b[1]=3", config_two);

    serialize_test_with_config_err(&nested_two, "Maximum serialization depth", config_zero);
    serialize_test_with_config_err(&nested_two, "Maximum serialization depth", config_one);
    serialize_test_with_config(&nested_two, "a=1&b[0]=2&b[1]=3&c[0][0]=4", config_two);
}
