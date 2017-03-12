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

macro_rules! map_test {
    ($string:expr, $($mapvars:tt)*) => {
        let expected_map = hash_to_map!(New $($mapvars)*);
        let testmap: HashMap<_, _> = qs::from_str($string).unwrap();
        assert_eq!(expected_map, testmap);
    }
}


macro_rules! hash_to_map {
    // Base case: a map with no inputs, do nothing 
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

    // This parses the first entry as a nested entry, and tail calls the remaining
    // in rest.
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
        user_ids: vec!(1,2,3,4),
    };

    let rec_params: QueryParams = qs::from_str("name=Acme&id=42&phone=12345&address[postcode]=12345&address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&user_ids[2]=3&user_ids[3]=4").unwrap();
    assert_eq!(rec_params, params);
    let rec_params: QueryParams = qs::from_str("name=Acme&id=42&phone=12345&address[postcode]=12345&address[city]=Carrot+City&user_ids[]=1&user_ids[]=2&user_ids[]=3&user_ids[]=4").unwrap();
    assert_eq!(rec_params, params);

}

#[test]
fn qs_test_simple() {
// test('parse()', function (t) {
    // t.test('parses a simple string', function (st) {
    // st.deepEqual(qs.parse('0=foo'), { 0: 'foo' });
    map_test!("0=foo", 0["foo"]);

    // st.deepEqual(qs.parse('foo=c++'), { foo: 'c  ' });
    map_test!("foo=c++", "foo"["c  "]);

    // st.deepEqual(qs.parse('a[>=]=23'), { a: { '>=': '23' } });
    map_test!("a[>=]=23", "a"[">="[23]]);

    // st.deepEqual(qs.parse('a[<=>]==23'), { a: { '<=>': '=23' } });
    map_test!("a[<=>]==23", "a"["<=>"["=23"]]);

    // st.deepEqual(qs.parse('a[==]=23'), { a: { '==': '23' } });
    map_test!("a[==]=23", "a"["=="[23]]);

    // st.deepEqual(qs.parse('foo', { strictNullHandling: true }), { foo: null });
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

    // st.deepEqual(qs.parse('foo2=bar2&baz2='), { foo2: 'bar2', baz2: '' });
    map_test!("foo2=bar2&baz2=", "foo2"["bar2"] "baz2"[""]);

    // st.deepEqual(qs.parse('foo=bar&baz', { strictNullHandling: true }), { foo: 'bar', baz: null });
    map_test!("foo=bar&baz", "foo"[e Some("bar".to_string())] "baz"[e None]);

    // st.deepEqual(qs.parse('foo=bar&baz'), { foo: 'bar', baz: '' });
    map_test!("foo=bar&baz", "foo"["bar"] "baz"[""]);

    // st.deepEqual(qs.parse('cht=p3&chd=t:60,40&chs=250x100&chl=Hello|World'), {
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
fn qs_nesting() {
    // t.deepEqual(qs.parse('a[b]=c'), { a: { b: 'c' } }, 'parses a single nested string');
    // map_test!("a[b]=c", "a"["b"["c"]]);

    // t.deepEqual(qs.parse('a[b][c]=d'), { a: { b: { c: 'd' } } }, 'parses a double nested string');
    map_test!("a[b][c]=d", "a"["b"["c"["d"]]]);
    // t.deepEqual(
    //     qs.parse('a[b][c][d][e][f][g][h]=i'),
    //     { a: { b: { c: { d: { e: { f: { '[g][h]': 'i' } } } } } } },
    //     'defaults to a depth of 5'
    // );
    // map_test!("a[b][c][d][e][f][g][h]=i", "a"["b"["c"["d"["e"["f"["[g][h]"["i"]]]]]]]);
}
