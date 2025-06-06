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
        let config = qs::Config::new();
        let testmap: HashMap<_, _> = config.deserialize_str($string).unwrap();
        let expected_map = hash_to_map!(New $($mapvars)*);
        assert_eq!(testmap, expected_map);
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

#[track_caller]
fn deserialize_test<'a, T>(params: &'a str, expected: &T)
where
    T: serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    deserialize_test_with_config(params, expected, qs::Config::default());
}

#[track_caller]
fn deserialize_test_err<'a, T>(params: &'a str, expected_err: &str)
where
    T: serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    deserialize_test_err_with_config::<T>(params, expected_err, qs::Config::default());
}

#[track_caller]
fn deserialize_test_with_config<'a, T>(params: &'a str, expected: &T, config: qs::Config)
where
    T: serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    let rec_params: T = config.deserialize_str(params).unwrap();
    pretty_assertions::assert_eq!(&rec_params, expected);
}

#[track_caller]
fn deserialize_test_err_with_config<'a, T>(params: &'a str, expected_err: &str, config: qs::Config)
where
    T: serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    let err = config.deserialize_str::<T>(params).unwrap_err();
    assert!(err.to_string().contains(expected_err), "\ngot: {}", err);
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

    // standard parameters
    deserialize_test(
        "name=Acme&id=42&phone=12345&address[postcode]=12345&\
         address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&\
         user_ids[2]=3&user_ids[3]=4",
        &params,
    );

    // unindexed arrays
    deserialize_test(
        "name=Acme&id=42&phone=12345&address[postcode]=12345&\
         address[city]=Carrot+City&user_ids[]=1&user_ids[]=2&\
         user_ids[]=3&user_ids[]=4",
        &params,
    );

    // ordering doesn't matter
    deserialize_test(
        "address[city]=Carrot+City&user_ids[]=1&user_ids[]=2&\
         name=Acme&id=42&phone=12345&address[postcode]=12345&\
         user_ids[]=3&user_ids[]=4",
        &params,
    );
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

    deserialize_test_err::<Query>("vec[]=a&vec[]=2", "invalid digit found in string");
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
    map_test!(
        "a[b][c][d][e][f][g][h]=i",
        "a"["b"["c"["d"["e"["f"["[g][h]"["i"]]]]]]]
    );
}

#[test]
fn optional_seq() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Option<Vec<u8>>,
    }

    deserialize_test("", &Query { vec: None });
    deserialize_test("vec", &Query { vec: None });
    deserialize_test("vec=", &Query { vec: Some(vec![]) });
    deserialize_test(
        "vec[0]=1&vec[1]=2",
        &Query {
            vec: Some(vec![1, 2]),
        },
    );
}

#[test]
fn seq_of_optionals() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        vec: Vec<Option<u8>>,
    }

    deserialize_test("vec", &Query { vec: vec![] });
    deserialize_test("vec[]", &Query { vec: vec![None] });
    deserialize_test(
        "vec[0]=1&vec[1]=2",
        &Query {
            vec: vec![Some(1), Some(2)],
        },
    );
    deserialize_test(
        "vec[]&vec[]=2",
        &Query {
            vec: vec![None, Some(2)],
        },
    );
}

#[test]
fn optional_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        address: Option<Address>,
    }

    deserialize_test("", &Query { address: None });
    deserialize_test("address", &Query { address: None });
    // `address=` implies we have a "null" value which cannot be deserialized
    // into an address
    deserialize_test_err::<Query>("address=", "missing field `city`");
    deserialize_test(
        "address[city]=Carrot+City&address[postcode]=12345",
        &Query {
            address: Some(Address {
                city: "Carrot City".to_string(),
                postcode: "12345".to_string(),
            }),
        },
    );
}

#[test]
fn nested_optionals() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        maybe_maybe: Option<Option<u8>>,
    }

    deserialize_test("", &Query { maybe_maybe: None });
    deserialize_test("maybe_maybe", &Query { maybe_maybe: None });
    deserialize_test(
        "maybe_maybe=",
        &Query {
            maybe_maybe: Some(None),
        },
    );
    deserialize_test(
        "maybe_maybe=1",
        &Query {
            maybe_maybe: Some(Some(1)),
        },
    );
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

    deserialize_test(
        "e=true",
        &Query {
            e: E::S("true".to_string()),
        },
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

    deserialize_test(
        "e[type]=B&e[val]=true&v[type]=V1&v[val][x]=12&v[val][y]=300",
        &Query {
            e: E::B(true),
            v: Some(V::V1 { x: 12, y: 300 }),
        },
    );

    deserialize_test(
        "e[type]=S&e[val]=other",
        &Query {
            e: E::S("other".to_string()),
            v: None,
        },
    );
}

#[test]
fn deserialize_enum_adjacently_out_of_order() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(tag = "Z", content = "A")]
    enum E {
        B(bool),
        S(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(tag = "Z", content = "A")]
    enum V {
        V1 { x: u8, y: u16 },
        V2(String),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        e: E,
        v: Option<V>,
    }

    deserialize_test(
        "e[Z]=B&e[A]=true&v[Z]=V1&v[A][x]=12&v[A][y]=300",
        &Query {
            e: E::B(true),
            v: Some(V::V1 { x: 12, y: 300 }),
        },
    );

    deserialize_test(
        "e[Z]=S&e[A]=other",
        &Query {
            e: E::S("other".to_string()),
            v: None,
        },
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

    deserialize_test(
        "e[B]&v[V1][x]=12&v[V1][y]=300&u=12",
        &Query {
            e: E::B,
            v: Some(V::V1 { x: 12, y: 300 }),
            u: NewU8(12),
        },
    );

    deserialize_test(
        "e[S]=other&u=1",
        &Query {
            e: E::S("other".to_string()),
            v: None,
            u: NewU8(1),
        },
    );

    deserialize_test("B=", &E::B);
    deserialize_test("S=Hello+World", &E::S("Hello World".to_string()));
}

#[test]
fn deserialize_enum_untagged_top_level() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum E {
        B { b: String },
        S { s: String },
    }

    deserialize_test(
        "s=true",
        &E::S {
            s: "true".to_string(),
        },
    );
    deserialize_test(
        "b=test",
        &E::B {
            b: "test".to_string(),
        },
    );
}

#[test]
fn deserialize_enum_top_level() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        A,
        B(u8),
        C { x: u8 },
    }

    deserialize_test("A", &E::A);
    deserialize_test("B=123", &E::B(123));
    deserialize_test("C[x]=234", &E::C { x: 234 });
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

    deserialize_test(
        "elements[0][a]=1&elements[0][b]=3&elements[1][a]=2&elements[1][b]=4",
        &Query {
            elements: vec![Test { a: 1, b: 3 }, Test { a: 2, b: 4 }],
        },
    );
}

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

    deserialize_test_err::<Query>(
        "elements[][a]=1&elements[][b]=3&elements[][a]=2&elements[][b]=4",
        "unsupported: unable to parse nested maps of unindexed sequences",
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

    deserialize_test_err::<Query>("vec[[]=1&vec[]=2", "invalid input: the key `vec` appears in the input as both a sequence and a map (with keys \"[\")");
    deserialize_test_err::<Query>("vec[\x00[]=1&vec[]=2", "invalid input: the key `vec` appears in the input as both a sequence and a map (with keys \"\x00[\")");
}

#[test]
fn querystring_decoding() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Test {
        a: u8,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Query {
        vec: Vec<Test>,
    }

    let config = qs::Config::new();

    // with querystring encoding, the brackets are considered part of the key
    // so this errors with unknown field
    deserialize_test_err_with_config::<Query>(
        "vec%5B0%5D%5Ba%5D=1",
        "unknown field `vec[0][a]`, expected `vec`",
        config,
    );

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct OddTest {
        #[serde(rename = "[but&why=?]")]
        a: u8,
    }

    let params = OddTest { a: 12 };
    let enc_params = qs::to_string(&params).unwrap();
    deserialize_test_with_config(&enc_params, &params, config);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query2 {
        vec: Vec<u32>,
    }
    // with querystring encoding, the brackets are considered part of the key
    // so this errors with missing field (since `vec` is not present)
    deserialize_test_err_with_config::<Query2>(
        "vec%5B%5D=1&vec%5B%5D=2",
        "missing field `vec`",
        config,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringQueryParam {
        field: String,
    }

    // Ensure invalid UTF-8 percent encoded characters produce an error.
    deserialize_test_err_with_config::<StringQueryParam>("field=%E9", "incomplete utf-8", config);
}

#[test]
fn formencoded_decoding() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Test {
        a: u8,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Query {
        vec: Vec<Test>,
    }

    let config = qs::Config::new().use_form_encoding(true);

    let expected = Query {
        vec: vec![Test { a: 1 }, Test { a: 2 }],
    };

    deserialize_test_with_config("vec%5B0%5D%5Ba%5D=1&vec[1][a]=2", &expected, config);

    deserialize_test_with_config("vec[0%5D%5Ba]=1&vec[1][a]=2", &expected, config);

    deserialize_test_with_config("vec[0%5D%5Ba%5D=1&vec[1][a]=2", &expected, config);

    deserialize_test_with_config("vec%5B0%5D%5Ba]=1&vec[1][a]=2", &expected, config);

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct OddTest {
        #[serde(rename = "[but&why=?]")]
        a: u8,
    }

    let params = OddTest { a: 12 };
    let enc_params = qs::to_string(&params).unwrap();
    println!("Encoded as: {}", enc_params);

    // Form encoding cannot necessarily handle these weird scenarios.
    let rec_params: Result<OddTest, _> = config.deserialize_str(&enc_params);
    assert!(rec_params.is_err());
    println!("{}", rec_params.unwrap_err());

    // Test that we don't panic
    // this simply fails to deserialize since the lone `%` doesn't get decoded and
    // just becomes a key
    deserialize_test_err_with_config::<Query>("%", "unknown field `%`, expected `vec`", config);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query2 {
        vec: Vec<u32>,
    }

    deserialize_test_with_config(
        "vec%5B%5D=1&vec%5B%5D=2",
        &Query2 { vec: vec![1, 2] },
        config,
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringQueryParam {
        field: String,
    }
}

#[test]
fn square_brackets_in_values() {
    map_test!("foo=%5BHello%5D", "foo"["[Hello]"]);
}

#[test]
fn deserialize_flatten_bug() {
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

    // see: https://github.com/serde-rs/serde/issues/1183
    // this is a limitation in serde which prevents us from knowing
    // what type the parameters are and we default to string
    // when we don't know
    deserialize_test_err::<Query>(
        "a=1&limit=100&offset=50&remaining=true",
        "invalid type: string \"100\", expected u64",
    );
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

    deserialize_test(
        "a=1&limit=100&offset=50&remaining=true",
        &Query {
            a: 1,
            common: CommonParams {
                limit: 100,
                offset: 50,
                remaining: true,
            },
        },
    );
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
    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        email: String,
    }

    deserialize_test(
        "email=a%2Bb%40c.com",
        &Test {
            email: "a+b@c.com".to_string(),
        },
    );
}

#[test]
fn deserialize_map_with_unit_enum_keys() {
    #[derive(Deserialize, Eq, PartialEq, Hash, Debug)]
    enum Operator {
        Lt,
        Gt,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Filter {
        point: HashMap<Operator, u64>,
    }

    let expected = Filter {
        point: HashMap::from([(Operator::Gt, 123), (Operator::Lt, 321)]),
    };
    deserialize_test("point[Gt]=123&point[Lt]=321", &expected);
}

#[cfg(feature = "indexmap")]
#[test]
fn deserialize_map_with_unit_enum_keys_preserves_order() {
    use indexmap::IndexMap;

    #[derive(Deserialize, Eq, PartialEq, Hash, Debug)]
    enum Key {
        Name,
        Age,
    }

    #[derive(Deserialize, Eq, PartialEq, Hash, Debug)]
    enum Order {
        Asc,
        Desc,
    }

    #[derive(Deserialize)]
    struct Sort {
        sort: IndexMap<Key, Order>,
    }

    let test1: Sort = serde_qs::from_str("sort[Name]=Asc&sort[Age]=Desc").unwrap();
    let values1 = test1.sort.into_iter().collect::<Vec<_>>();

    assert_eq!(
        values1,
        vec![(Key::Name, Order::Asc), (Key::Age, Order::Desc)]
    );

    let test2: Sort = serde_qs::from_str("sort[Age]=Desc&sort[Name]=Asc").unwrap();
    let values2 = test2.sort.into_iter().collect::<Vec<_>>();

    assert_eq!(
        values2,
        vec![(Key::Age, Order::Desc), (Key::Name, Order::Asc)]
    );
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

    let test = serde_qs::from_str::<Mapping>("mapping[1]=2&mapping[1]=4").unwrap();
    assert_eq!(test.mapping.get(&1).cloned(), Some(4));
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

    deserialize_test("", &());
    deserialize_test("", &A);
    deserialize_test("a=test&t", &B { t: (), a: "test" });
    deserialize_test("t&a=test", &B { t: (), a: "test" });
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

#[test]
fn deserialize_repeat_keys() {
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Data {
        vec: Vec<usize>,
        repeated: usize,
        implicit: Vec<usize>,
    }

    let expected = Data {
        vec: vec![1, 2],
        repeated: 4,
        implicit: vec![5],
    };

    let deserialized =
        serde_qs::from_str::<Data>("vec[0]=0&vec[0]=1&vec[1]=2&repeated=3&repeated=4&implicit=5")
            .unwrap();
    assert_eq!(deserialized, expected);

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct NestedData {
        data: Vec<Data>,
    }

    let deserialized = serde_qs::from_str::<NestedData>(
        "data[0][vec][0]=0&data[0][vec][0]=1&data[0][vec][1]=2&data[0][repeated]=3&\
         data[0][repeated]=4&data[0][implicit]=5",
    )
    .unwrap();
    assert_eq!(
        deserialized,
        NestedData {
            data: vec![expected]
        }
    );
}

#[test]
fn depth_one() {
    #[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
    #[serde(default, deny_unknown_fields)]
    struct Form {
        id: i64,
        name: String,
        vec: Vec<String>,
    }

    let default_config = serde_qs::Config::new()
        .max_depth(1)
        .use_form_encoding(false);
    let form_config = serde_qs::Config::new().max_depth(1).use_form_encoding(true);

    //  works correct
    deserialize_test_with_config(
        "id=2",
        &Form {
            id: 2,
            ..Default::default()
        },
        default_config,
    );

    deserialize_test_with_config(
        "name=test",
        &Form {
            name: "test".to_string(),
            ..Default::default()
        },
        default_config,
    );

    deserialize_test_with_config(
        "id=3&name=&vec%5B0%5D=Vector",
        &Form {
            id: 3,
            name: "".to_string(),
            vec: vec!["Vector".to_string()],
        },
        form_config,
    );

    deserialize_test_with_config(
        "vec[0]=Vector",
        &Form {
            id: 0,
            name: "".to_string(),
            vec: vec!["Vector".to_string()],
        },
        default_config,
    );

    deserialize_test_with_config(
        "vec%5B0%5D=Vector",
        &Form {
            id: 0,
            name: "".to_string(),
            vec: vec!["Vector".to_string()],
        },
        form_config,
    );

    deserialize_test_with_config(
        "name=&vec%5B0%5D=Vector",
        &Form {
            id: 0,
            name: "".to_string(),
            vec: vec!["Vector".to_string()],
        },
        form_config,
    );

    deserialize_test_with_config(
        "name=&vec[0]=Vector",
        &Form {
            id: 0,
            name: "".to_string(),
            vec: vec!["Vector".to_string()],
        },
        default_config,
    );

    deserialize_test_with_config(
        "name=test&vec[0]=Vector",
        &Form {
            id: 0,
            name: "test".to_string(),
            vec: vec!["Vector".to_string()],
        },
        default_config,
    );
}

#[test]
fn deserialize_serde_json_value() {
    let value = serde_json::json!({ "hello": "10" });
    assert_eq!(serde_qs::to_string(&value).unwrap(), "hello=10");

    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("hello=10").unwrap(),
        serde_json::json!({ "hello": "10" })
    );
    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("").unwrap(),
        serde_json::Value::Null,
    );
    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("foo").unwrap(),
        serde_json::json!( { "foo": null })
    );
    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("foo=").unwrap(),
        serde_json::json!( { "foo": serde_json::Value::String("".to_string()) })
    );
    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("[0]=foo&[1]=bar").unwrap(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("foo".to_string()),
            serde_json::Value::String("bar".to_string())
        ]),
    );
    assert_eq!(
        serde_qs::from_str::<serde_json::Value>("=foo&=bar").unwrap(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("foo".to_string()),
            serde_json::Value::String("bar".to_string())
        ]),
    );
}

#[test]
fn deserialize_primitive_errors() {
    deserialize_test_err::<String>("hello", "invalid type: map, expected a string");
}

#[test]
fn deserialize_into_tuple() {
    deserialize_test("=1&=2", &(1u8, 2u8));
    deserialize_test_err::<(u8,)>("=1&=2", "expected a tuple of length 1, got length 2");

    #[derive(Deserialize, Debug, PartialEq)]
    struct Tuple(u8, u8);

    deserialize_test("=1&=2", &Tuple(1, 2));
    deserialize_test_err::<Tuple>(
        "=1",
        "expected tuple struct `Tuple` of length 2, got length 1",
    );
}

#[test]
fn deserialize_option() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        id: u32,
    }

    deserialize_test("id=1", &Some(Query { id: 1 }));
    deserialize_test("", &None::<Query>);
}

#[test]
fn deserialize_unit_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Query;

    deserialize_test("", &Query);
}

#[test]
fn empty_vec() {
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Data {
        values: Vec<String>,
    }

    let data = Data { values: Vec::new() };
    let serialized = serde_qs::to_string(&data).unwrap();

    let deserialized = serde_qs::from_str::<Data>(&serialized).unwrap();
    assert_eq!(deserialized, data);

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct SmallerData {
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        values: Vec<String>,
    }

    let data = SmallerData { values: Vec::new() };
    let serialized = serde_qs::to_string(&data).unwrap();

    let deserialized = serde_qs::from_str::<SmallerData>(&serialized).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn empty_map() {
    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Data {
        values: HashMap<String, String>,
    }

    let data = Data {
        values: HashMap::new(),
    };
    let serialized = serde_qs::to_string(&data).unwrap();

    let deserialized = serde_qs::from_str::<Data>(&serialized).unwrap();
    assert_eq!(deserialized, data);

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct SmallerData {
        #[serde(skip_serializing_if = "HashMap::is_empty", default)]
        values: HashMap<String, String>,
    }

    let data = SmallerData {
        values: HashMap::new(),
    };
    let serialized = serde_qs::to_string(&data).unwrap();
    let deserialized = serde_qs::from_str::<SmallerData>(&serialized).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn nested_tuple() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        vec: Vec<(u32, String)>,
    }

    deserialize_test(
        "vec[0][0]=1&vec[0][1]=test",
        &Query {
            vec: vec![(1, "test".to_string())],
        },
    );

    deserialize_test_err::<Query>("vec[0][0]=1", "expected a tuple of length 2, got length 1");
}

#[test]
fn untagged_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct A {
        x: String,
        a: String,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct B {
        x: String,
        b: String,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Q {
        A(A),
        B(B),
    }

    deserialize_test(
        "x=1&a=2",
        &Q::A(A {
            x: "1".to_string(),
            a: "2".to_string(),
        }),
    );

    deserialize_test(
        "x=1&b=2",
        &Q::B(B {
            x: "1".to_string(),
            b: "2".to_string(),
        }),
    );
}

#[test]
fn newtype_structs() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct NewU8(u8);

    #[derive(Deserialize, Debug, PartialEq)]
    struct NewU16(u16);

    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        a: NewU8,
        b: NewU16,
    }

    deserialize_test(
        "a=1&b=2",
        &Query {
            a: NewU8(1),
            b: NewU16(2),
        },
    );

    #[derive(Deserialize, Debug, PartialEq)]
    struct NewQuery(Query);

    deserialize_test(
        "a=1&b=2",
        &NewQuery(Query {
            a: NewU8(1),
            b: NewU16(2),
        }),
    );
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
fn invalid_utf8() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct StringQueryParam {
        field: String,
    }

    // Invalid UTF8 characters cause errors _if_ they are used
    deserialize_test_err::<StringQueryParam>(
        "field=%E9",
        "incomplete utf-8 byte sequence from inde",
    );

    // But if they are not used, we can still deserialize
    deserialize_test(
        "field=valid&unused=%E9",
        &StringQueryParam {
            field: "valid".to_string(),
        },
    );
}

/// NOTE: we cannot represent `Some(Some(""))` in any meaningful way
/// but I'm okay with that -- `serde_json` cannot differentiate betwee
/// nested `Option`s at all: https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=e3c3db811eeb12388302055d50232ecb`
#[test]
fn levels_of_option() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query<T> {
        a: Option<T>,
        b: Option<Option<T>>,
        c: Option<Option<Option<T>>>,
    }

    deserialize_test(
        "a=1&b=2&c=3",
        &Query::<String> {
            a: Some("1".to_string()),
            b: Some(Some("2".to_string())),
            c: Some(Some(Some("3".to_string()))),
        },
    );

    deserialize_test(
        "a=1&b=2&c=3",
        &Query::<u8> {
            a: Some(1),
            b: Some(Some(2)),
            c: Some(Some(Some(3))),
        },
    );

    deserialize_test(
        "a&b&c",
        &Query::<String> {
            a: None,
            b: None,
            c: None,
        },
    );

    deserialize_test(
        "a=&b=&c=",
        &Query::<String> {
            a: Some("".to_string()),
            b: Some(None),
            c: Some(None),
        },
    );

    deserialize_test(
        "a&b=&c=",
        &Query::<u8> {
            a: None,
            b: Some(None),
            c: Some(None),
        },
    );
}

#[test]
fn empty_keys() {
    deserialize_test("=123", &123u8);
    deserialize_test("=1&=2", &vec![1u8, 2]);
    deserialize_test("=foo", &"foo");
    deserialize_test("=foo", &vec!["foo"]);
    // empty key -> keep the last value if its a string
    deserialize_test("=foo&=bar", &"bar");
    deserialize_test("=foo&=bar", &vec!["foo", "bar"]);

    let opt_string_none: Option<String> = None;
    let opt_string_some: Option<String> = Some("foo".to_string());
    let opt_string_some_empty: Option<String> = Some("".to_string());
    deserialize_test("=foo", &opt_string_some);
    deserialize_test("=", &opt_string_some_empty);
    deserialize_test("", &opt_string_none);

    // empty key -> empty key if its a map
    deserialize_test(
        "=foo",
        &HashMap::<String, String>::from([("".to_string(), "foo".to_string())]),
    );

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Query {
        a: String,
        b: String,
    }

    // empty keys are not allowed
    deserialize_test_err::<Query>(
        "=1&b=2",
        "invalid input: the same key is used for both a value and a nested ma",
    );

    // but we can have empty values
    deserialize_test(
        "a=&b=2",
        &Query {
            a: "".to_string(),
            b: "2".to_string(),
        },
    );
}

#[test]
fn int_key_parsing() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Query<K: std::hash::Hash + Eq> {
        a: HashMap<K, u32>,
    }

    // Test that we can parse integer keys correctly
    deserialize_test(
        "a[1]=2&a[3]=4",
        &Query {
            a: HashMap::from([(1, 2), (3, 4)]),
        },
    );

    // errors if numbers are too larger
    deserialize_test_err::<Query<u8>>("a[1000]=2", "number too large to fit in target type");

    // Test that we can parse integer keys with leading zeros
    deserialize_test(
        "a[01]=2&a[03]=4",
        &Query {
            a: HashMap::from([(1, 2), (3, 4)]),
        },
    );

    // if we use a string key, it should still work
    // although we lose the leading zeros unfortunately
    deserialize_test(
        "a[01]=2&a[03]=4",
        &Query {
            a: HashMap::from([("1".to_string(), 2), ("3".to_string(), 4)]),
        },
    );

    #[derive(Debug, Deserialize, PartialEq)]
    struct VecQuery<K: std::hash::Hash + Eq> {
        a: Vec<K>,
    }

    // Test that we can parse integer keys in a vector
    deserialize_test("a[0]=1&a[1]=2", &VecQuery { a: vec![1, 2] });

    // but if we use a string key, it will error
    deserialize_test_err::<VecQuery<String>>(
        "a[x]=1&a[0]=2",
        "expected an integer index, found a string key `x`",
    );
}

#[test]
fn suggests_form_encoding() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Query {
        data: Nested,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Nested {
        a: String,
    }

    // this is a common mistake when using querystring encoding
    // so we suggest using form encoding instead
    deserialize_test_err::<Query>(
        "data%5Ba%5D=foo",
        "unknown field `data[a]`, expected `data`\nInvalid field contains an encoded bracket -- consider using form encoding mode",
    );
}

#[test]
fn boolean_values() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        #[serde(default)]
        a: bool,
    }

    // Test various boolean representations
    deserialize_test("a=true", &Query { a: true });
    deserialize_test("a=false", &Query { a: false });
    deserialize_test("a", &Query { a: true });
    deserialize_test("", &Query { a: false });
}

#[test]
fn empty_values() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Query {
        a: String,
    }

    deserialize_test("a", &Query { a: "".to_string() });
}
