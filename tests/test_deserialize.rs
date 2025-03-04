extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Address {
    city: String,
    postcode: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    id: u8,
    name: String,
    address: Address,
    phone: u32,
    user_ids: Vec<u8>,
}

// Compares a map generated by hash_to_map with the map returned by
// qs::from_str. All types are inferred by the compiler.
macro_rules! map_test {
    ($string:expr, $($mapvars:tt)*) => {
        for config in vec![qs::Config::new(5, true), qs::Config::new(5, false)] {
            let testmap: HashMap<_, _> = config.deserialize_str($string).unwrap();
            let expected_map = hash_to_map!(New $($mapvars)*);
            assert_eq!(testmap, expected_map);
        }
    }
}

// Macro used to quickly generate a nested HashMap from a string.
macro_rules! hash_to_map {
    // Base case: a map with no inputs, do nothing.
    ($map:expr, ) => ();
    //{}
    // This parses a single map entry, with a value explicitly an expression.
    ($map:expr, $k:tt[e $v:expr] $($rest:tt)*) => {{
        $map.insert($k.to_owned(), $v.to_owned());
        hash_to_map!($map, $($rest)*);
    }};

    // This parses a single map entry, plus the rest of the values.
    ($map:expr, $k:tt[$v:tt] $($rest:tt)*) => {{
        $map.insert($k.to_owned(), $v.to_owned());
        hash_to_map!($map, $($rest)*);
    }};

    // This parses the first entry as a nested entry, and tail calls the
    // remaining in rest.
    ($map:expr, $k:tt[$($inner:tt)*] $($rest:tt)*) => {{
        let mut inner_map = HashMap::new();
        hash_to_map!(inner_map, $($inner)*);
        $map.insert($k.to_owned(), inner_map);
        hash_to_map!($map, $($rest)*);
    }};

    // Constructs the map and then runs the macro. This infers the types for the
    // hashmap.
    (New $($rest:tt)*) => {{
      let mut map = HashMap::new();
      hash_to_map!(map, $($rest)*);
      map
    }}

}

#[test]
fn deserialize_struct() {
    let params = QueryParams {
        id: 42,
        name: "Acme".to_string(),
        phone: 12345,
        address: Address {
            city: "Carrot City".to_string(),
            postcode: "12345".to_string(),
        },
        user_ids: vec![1, 2, 3, 4],
    };

    for config in [qs::Config::new(5, true), qs::Config::new(5, false)] {
        // standard parameters
        let rec_params: QueryParams = config
            .deserialize_str(
                "\
                 name=Acme&id=42&phone=12345&address[postcode]=12345&\
                 address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&\
                 user_ids[2]=3&user_ids[3]=4",
            )
            .unwrap();
        assert_eq!(rec_params, params);

        // unindexed arrays
        let rec_params: QueryParams = config
            .deserialize_str(
                "\
                 name=Acme&id=42&phone=12345&address[postcode]=12345&\
                 address[city]=Carrot+City&user_ids[]=1&user_ids[]=2&\
                 user_ids[]=3&user_ids[]=4",
            )
            .unwrap();
        assert_eq!(rec_params, params);

        // ordering doesn't matter
        let rec_params: QueryParams = config
            .deserialize_str(
                "\
                 address[city]=Carrot+City&user_ids[]=1&user_ids[]=2&\
                 name=Acme&id=42&phone=12345&address[postcode]=12345&\
                 user_ids[]=3&user_ids[]=4",
            )
            .unwrap();
        assert_eq!(rec_params, params);
    }
}

#[test]
fn qs_test_simple() {
    // test('parse()', function (t) {
    // t.test('parses a simple string', function (st) {
    // st.deepEqual(qs.parse('0=foo'), { 0: 'foo' });
    map_test!("0=foo", 0["foo"]);

    // st.deepEqual(qs.parse('&0=foo'), { 0: 'foo' });
    map_test!("&0=foo", 0["foo"]);

    // st.deepEqual(qs.parse('0=foo&'), { 0: 'foo' });
    map_test!("0=foo&", 0["foo"]);

    // st.deepEqual(qs.parse('foo=c++'), { foo: 'c  ' });
    map_test!("foo=c++", "foo"["c  "]);

    // st.deepEqual(qs.parse('a[>=]=23'), { a: { '>=': '23' } });
    map_test!("a[>=]=23", "a"[">="[23]]);

    // st.deepEqual(qs.parse('a[<=>]==23'), { a: { '<=>': '=23' } });
    map_test!("a[<=>]==23", "a"["<=>"["=23"]]);

    // st.deepEqual(qs.parse('a[==]=23'), { a: { '==': '23' } });
    map_test!("a[==]=23", "a"["=="[23]]);

    // st.deepEqual(qs.parse('foo', { strictNullHandling: true }),
    // { foo: null });
    let none: Option<String> = Option::None;
    map_test!("foo", "foo"[none]);

    // st.deepEqual(qs.parse('foo'), { foo: '' });
    map_test!("foo", "foo"[""]);

    // st.deepEqual(qs.parse('foo='), { foo: '' });
    map_test!("foo=", "foo"[""]);

    // st.deepEqual(qs.parse('foo=bar'), { foo: 'bar' });
    map_test!("foo=bar", "foo"["bar"]);

    // st.deepEqual(qs.parse(' foo = bar = baz '), { ' foo ': ' bar = baz ' });
    map_test!(" foo = bar = baz ", " foo "[" bar = baz "]);

    // st.deepEqual(qs.parse('foo=bar=baz'), { foo: 'bar=baz' });
    map_test!("foo=bar=baz", "foo"["bar=baz"]);

    // st.deepEqual(qs.parse('foo=bar&bar=baz'), { foo: 'bar', bar: 'baz' });
    map_test!("foo=bar&bar=baz", "foo"["bar"] "bar"["baz"]);

    // st.deepEqual(qs.parse('foo=bar&&bar=baz'), { foo: 'bar', bar: 'baz' });
    map_test!("foo=bar&&bar=baz", "foo"["bar"] "bar"["baz"]);

    // st.deepEqual(qs.parse('foo2=bar2&baz2='), { foo2: 'bar2', baz2: '' });
    map_test!("foo2=bar2&baz2=", "foo2"["bar2"] "baz2"[""]);

    // st.deepEqual(qs.parse('foo=bar&baz', { strictNullHandling: true }), {
    // foo: 'bar', baz: null });
    map_test!("foo=bar&baz", "foo"[e Some("bar".to_string())] "baz"[e None]);

    // st.deepEqual(qs.parse('foo=bar&baz'), { foo: 'bar', baz: '' });
    map_test!("foo=bar&baz", "foo"["bar"] "baz"[""]);

    // st.deepEqual(qs.parse('cht=p3&chd=t:60,40&chs=250x100&chl=Hello|World'),
    // {
    //     cht: 'p3',
    //     chd: 't:60,40',
    //     chs: '250x100',
    //     chl: 'Hello|World'
    // });
    map_test!("cht=p3&chd=t:60,40&chs=250x100&chl=Hello|World",
      "cht"["p3"]
      "chd"["t:60,40"]
      "chs"["250x100"]
      "chl"["Hello|World"]
    );
    // st.end();
    // });
}

#[test]
fn no_panic_on_parse_error() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Vec<u32>,
    }

    let params: Result<Query, _> = qs::from_str("vec[]=a&vec[]=2");
    assert!(params.is_err())
}

#[test]
fn qs_nesting() {
    // t.deepEqual(qs.parse('a[b]=c'), { a: { b: 'c' } }, 'parses a single
    // nested string');
    map_test!("a[b]=c", "a"["b"["c"]]);

    // t.deepEqual(qs.parse('a[b][c]=d'), { a: { b: { c: 'd' } } }, 'parses a
    // double nested string');
    map_test!("a[b][c]=d", "a"["b"["c"["d"]]]);
    // t.deepEqual(
    //     qs.parse('a[b][c][d][e][f][g][h]=i'),
    //     { a: { b: { c: { d: { e: { f: { '[g][h]': 'i' } } } } } } },
    //     'defaults to a depth of 5'
    // );
    // This looks like depth 6 to me? Tweaked test to make it 5.
    map_test!(
        "a[b][c][d][e][f][g][h]=i",
        "a"["b"["c"["d"["e"["[f][g][h]"["i"]]]]]]
    );
}

#[test]
fn optional_seq() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Option<Vec<u8>>,
    }

    let params = "";
    let query = Query { vec: None };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);

    let params = "vec=";
    let query = Query { vec: None };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);

    let params = "vec[0]=1&vec[1]=2";
    let query = Query {
        vec: Some(vec![1, 2]),
    };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);
}

#[test]
fn optional_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        address: Option<Address>,
    }

    let params = "";
    let query = Query { address: None };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);

    let params = "address=";
    let query = Query { address: None };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);

    let params = "address[city]=Carrot+City&address[postcode]=12345";
    let query = Query {
        address: Some(Address {
            city: "Carrot City".to_string(),
            postcode: "12345".to_string(),
        }),
    };
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(rec_params, query);
}

#[test]
fn deserialize_enum_untagged() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum E {
        B(bool),
        S(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        e: E,
    }

    let params = "e=true";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            e: E::S("true".to_string())
        }
    );
}

#[test]
fn deserialize_enum_adjacently() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(tag = "type", content = "val")]
    enum E {
        B(bool),
        S(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(tag = "type", content = "val")]
    enum V {
        V1 { x: u8, y: u16 },
        V2(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        e: E,
        v: Option<V>,
    }

    let params = "e[type]=B&e[val]=true&v[type]=V1&v[val][x]=12&v[val][y]=300";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            e: E::B(true),
            v: Some(V::V1 { x: 12, y: 300 }),
        }
    );

    let params = "e[type]=S&e[val]=other";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            e: E::S("other".to_string()),
            v: None,
        }
    );
}

#[test]
fn deserialize_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct NewU8(u8);

    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        B,
        S(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    enum V {
        V1 { x: u8, y: u16 },
        V2(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        e: E,
        v: Option<V>,
        u: NewU8,
    }

    let params = "e=B&v[V1][x]=12&v[V1][y]=300&u=12";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            e: E::B,
            v: Some(V::V1 { x: 12, y: 300 }),
            u: NewU8(12),
        }
    );

    let params = "e[S]=other&u=1";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            e: E::S("other".to_string()),
            v: None,
            u: NewU8(1),
        }
    );

    let params = "B=";
    let rec_params: E = qs::from_str(params).unwrap();
    assert_eq!(rec_params, E::B);

    let params = "S=Hello+World";
    let rec_params: E = qs::from_str(params).unwrap();
    assert_eq!(rec_params, E::S("Hello World".to_string()));
}

#[test]
fn seq_of_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        a: u8,
        b: u8,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        elements: Vec<Test>,
    }

    let params = "elements[0][a]=1&elements[0][b]=3&elements[1][a]=2&elements[1][b]=4";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            elements: vec![Test { a: 1, b: 3 }, Test { a: 2, b: 4 }]
        }
    );
}

#[should_panic]
#[test]
fn unsupported_seq_of_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        a: u8,
        b: u8,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        elements: Vec<Test>,
    }

    let params = "elements[][a]=1&elements[][b]=3&elements[][a]=2&elements[][b]=4";
    let rec_params: Query = qs::from_str(params).unwrap();
    assert_eq!(
        rec_params,
        Query {
            elements: vec![Test { a: 1, b: 3 }, Test { a: 2, b: 4 }]
        }
    );
}

#[test]
fn correct_decoding() {
    map_test!("foo=%24", "foo"["$"]);

    map_test!("foo=%26", "foo"["&"]);
}

#[test]
fn returns_errors() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Vec<u32>,
    }

    let params: Result<Query, _> = qs::from_str("vec[[]=1&vec[]=2");
    assert!(params.is_err());
    println!("{}", params.unwrap_err());

    let params: Result<Query, _> = qs::from_str("vec[\x00[]=1&vec[]=2");
    assert!(params.is_err());
    println!("{}", params.unwrap_err());

    let params: Result<Query, _> = qs::from_str("vec[0]=1&vec[0]=2");
    assert!(params.is_err());
    println!("{}", params.unwrap_err());
}

#[test]
fn strict_mode() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Test {
        a: u8,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Query {
        vec: Vec<Test>,
    }

    let strict_config = qs::Config::default();

    let params: Result<Query, _> = strict_config.deserialize_str("vec%5B0%5D%5Ba%5D=1&vec[1][a]=2");
    assert!(params.is_err());
    println!("{}", params.unwrap_err());

    let loose_config = qs::Config::new(5, false);

    let params: Result<Query, _> = loose_config.deserialize_str("vec%5B0%5D%5Ba%5D=1&vec[1][a]=2");
    assert_eq!(
        params.unwrap(),
        Query {
            vec: vec![Test { a: 1 }, Test { a: 2 }]
        }
    );

    let params: Result<Query, _> =
        loose_config.deserialize_str("vec[%5B0%5D%5Ba%5D]=1&vec[1][a]=2");
    assert_eq!(
        params.unwrap(),
        Query {
            vec: vec![Test { a: 1 }, Test { a: 2 }]
        }
    );

    let params: Result<Query, _> = loose_config.deserialize_str("vec[%5B0%5D%5Ba%5D=1&vec[1][a]=2");
    assert_eq!(
        params.unwrap(),
        Query {
            vec: vec![Test { a: 1 }, Test { a: 2 }]
        }
    );

    let params: Result<Query, _> = loose_config.deserialize_str("vec%5B0%5D%5Ba%5D]=1&vec[1][a]=2");
    assert_eq!(
        params.unwrap(),
        Query {
            vec: vec![Test { a: 1 }, Test { a: 2 }]
        }
    );

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct OddTest {
        #[serde(rename = "[but&why=?]")]
        a: u8,
    }

    let params = OddTest { a: 12 };
    let enc_params = qs::to_string(&params).unwrap();
    println!("Encoded as: {}", enc_params);
    let rec_params: Result<OddTest, _> = strict_config.deserialize_str(&enc_params);
    assert_eq!(rec_params.unwrap(), params);

    // Non-strict decoding cannot necessarily handle these weird scenerios.
    let rec_params: Result<OddTest, _> = loose_config.deserialize_str(&enc_params);
    assert!(rec_params.is_err());
    println!("{}", rec_params.unwrap_err());

    // Test that we don't panic
    let malformed_params: Result<Query, _> = loose_config.deserialize_str("%");
    assert!(malformed_params.is_err());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query2 {
        vec: Vec<u32>,
    }
    let repeated_key: Result<Query2, _> = strict_config.deserialize_str("vec%5B%5D=1&vec%5B%5D=2");
    assert!(repeated_key.is_err());
    println!("{}", repeated_key.unwrap_err());

    let params: Query2 = loose_config
        .deserialize_str("vec%5B%5D=1&vec%5B%5D=2")
        .unwrap();
    assert_eq!(params.vec, vec![1, 2]);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringQueryParam {
        field: String,
    }

    // Ensure strict mode produces an error for invalid UTF-8 percent encoded characters.
    let invalid_utf8: Result<StringQueryParam, _> = strict_config.deserialize_str("field=%E9");
    assert!(invalid_utf8.is_err());

    // Ensure loose mode invalid UTF-8 percent encoded characters become � U+FFFD.
    let valid_utf8: StringQueryParam = loose_config.deserialize_str("field=%E9").unwrap();
    assert_eq!(valid_utf8.field, "�");
}

#[test]
fn square_brackets_in_values() {
    map_test!("foo=%5BHello%5D", "foo"["[Hello]"]);
}

#[test]
#[ignore]
fn deserialize_flatten() {
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
        remaining: bool,
    }

    let params = "a=1&limit=100&offset=50&remaining=true";
    let query = Query {
        a: 1,
        common: CommonParams {
            limit: 100,
            offset: 50,
            remaining: true,
        },
    };
    let rec_query: Result<Query, _> = qs::from_str(params);
    assert_eq!(rec_query.unwrap(), query);
}

#[test]
fn deserialize_flatten_workaround() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Query {
        a: u8,
        #[serde(flatten)]
        common: CommonParams,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct CommonParams {
        #[serde(deserialize_with = "from_str")]
        limit: u64,
        #[serde(deserialize_with = "from_str")]
        offset: u64,
        #[serde(deserialize_with = "from_str")]
        remaining: bool,
    }

    let params = "a=1&limit=100&offset=50&remaining=true";
    let query = Query {
        a: 1,
        common: CommonParams {
            limit: 100,
            offset: 50,
            remaining: true,
        },
    };
    let rec_query: Result<Query, _> = qs::from_str(params);
    assert_eq!(rec_query.unwrap(), query);
}

use serde::de::Error;

fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(s).map_err(|_| D::Error::custom("could not parse string"))
}

#[test]
fn deserialize_plus() {
    #[derive(Deserialize)]
    struct Test {
        email: String,
    }

    let test: Test = serde_qs::from_str("email=a%2Bb%40c.com").unwrap();
    assert_eq!(test.email, "a+b@c.com");
}

#[test]
fn deserialize_map_with_unit_enum_keys() {
    #[derive(Deserialize, Eq, PartialEq, Hash)]
    enum Operator {
        Lt,
        Gt,
    }

    #[derive(Deserialize)]
    struct Filter {
        point: HashMap<Operator, u64>,
    }

    let test: Filter = serde_qs::from_str("point[Gt]=123&point[Lt]=321").unwrap();

    assert_eq!(test.point[&Operator::Gt], 123);
    assert_eq!(test.point[&Operator::Lt], 321);
}

#[test]
fn deserialize_map_with_int_keys() {
    #[derive(Debug, Deserialize)]
    struct Mapping {
        mapping: HashMap<u64, u64>,
    }

    let test: Mapping = serde_qs::from_str("mapping[1]=2&mapping[3]=4").unwrap();

    assert_eq!(test.mapping.get(&1).cloned(), Some(2));
    assert_eq!(test.mapping.get(&3).cloned(), Some(4));
    assert_eq!(test.mapping.get(&2).cloned(), None);

    serde_qs::from_str::<Mapping>("mapping[1]=2&mapping[1]=4")
        .expect_err("should error with repeated key");
}

#[test]
fn deserialize_map_with_uuid_keys() {
    #[derive(Debug, Deserialize)]
    struct Mapping {
        mapping: HashMap<String, u64>,
    }

    let test: Mapping = serde_qs::from_str(
        "mapping[5b53d2c1-3745-47e3-b421-76c05c5c7523]=1&mapping[00000000-0000-0000-0000-000000000000]=2&mapping[a4b2e25c-e80c-4e2a-958c-35f2f5151f46]=3&mapping[ffffffff-ffff-ffff-ffff-ffffffffffff]=4"
    ).unwrap();

    assert_eq!(
        test.mapping
            .get("5b53d2c1-3745-47e3-b421-76c05c5c7523")
            .cloned(),
        Some(1)
    );
    assert_eq!(
        test.mapping
            .get("00000000-0000-0000-0000-000000000000")
            .cloned(),
        Some(2)
    );
    assert_eq!(
        test.mapping
            .get("a4b2e25c-e80c-4e2a-958c-35f2f5151f46")
            .cloned(),
        Some(3)
    );
    assert_eq!(
        test.mapping
            .get("ffffffff-ffff-ffff-ffff-ffffffffffff")
            .cloned(),
        Some(4)
    );
}

#[test]
fn deserialize_unit_types() {
    // allow these clippy lints cause I like how explicit the test is
    #![allow(clippy::let_unit_value)]
    #![allow(clippy::unit_cmp)]

    #[derive(Debug, Deserialize, PartialEq)]
    struct A;
    #[derive(Debug, Deserialize, PartialEq)]
    struct B<'a> {
        t: (),
        a: &'a str,
    }

    let test: () = serde_qs::from_str("").unwrap();
    assert_eq!(test, ());

    let test: A = serde_qs::from_str("").unwrap();
    assert_eq!(test, A);

    let test: B = serde_qs::from_str("a=test&t=").unwrap();
    assert_eq!(test, B { t: (), a: "test" });

    let test: B = serde_qs::from_str("t=&a=test").unwrap();
    assert_eq!(test, B { t: (), a: "test" });
}

#[test]
fn serialization_roundtrip() {
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Data {
        #[serde(default)]
        values: Vec<String>,
    }

    let data = Data { values: Vec::new() };
    let serialized = serde_qs::to_string(&data).unwrap();

    dbg!(&serialized);
    let deserialized = serde_qs::from_str::<Data>(&serialized).unwrap();
    assert_eq!(deserialized, data);
}
