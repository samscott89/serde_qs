#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;

#[test]
fn serialize_option_map_int() {
    let params = &[("first", Some(23)), ("middle", None), ("last", Some(42))];

    assert_eq!(serde_urlencoded::to_string(params),
               Ok("first=23&last=42".to_owned()));
}

#[test]
fn serialize_option_map_string() {
    let params =
        &[("first", Some("hello")), ("middle", None), ("last", Some("world"))];

    assert_eq!(serde_urlencoded::to_string(params),
               Ok("first=hello&last=world".to_owned()));
}

#[test]
fn serialize_option_map_bool() {
    let params = &[("one", Some(true)), ("two", Some(false))];

    assert_eq!(serde_urlencoded::to_string(params),
               Ok("one=true&two=false".to_owned()));
}

#[test]
fn serialize_map_bool() {
    let params = &[("one", true), ("two", false)];

    assert_eq!(serde_urlencoded::to_string(params),
               Ok("one=true&two=false".to_owned()));
}

#[derive(Serialize, Deserialize)]
struct A { b: B, c: C }
#[derive(Serialize, Deserialize)]
struct B { b1: u8, b2: String }
#[derive(Serialize, Deserialize)]
struct C { c1: String, c2: u8 }

#[test]
fn serialize_struct() {
    let params = A {
      b: B {
        b1: 10,
        b2: "Ten".to_owned()
      },
      c: C {
        c1: "Seven".to_owned(),
        c2: 7
      }
    };

    assert_eq!(serde_urlencoded::to_string(&params),
      Ok(urlencode("b[b1]=10&b[b2]=Ten&c[c1]=Seven&c[c2]=7")));
}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}
