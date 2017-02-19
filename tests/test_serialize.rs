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
struct Foo { bar: Bar, baz: Baz }
#[derive(Serialize, Deserialize)]
struct Bar { x: u8, y: String }
#[derive(Serialize, Deserialize)]
struct Baz { thing: String, other: u8 }

#[test]
fn serialize_struct() {
    let params = Foo {
      bar: Bar {
        x: 10,
        y: "Ten".to_owned()
      },
      baz: Baz {
        thing: "Thing".to_owned(),
        other: 12
      }
    };

    assert_eq!(serde_urlencoded::to_string(&params),
      Ok(urlencode("bar[x]=10&bar[y]=Ten&baz[thing]=Thing&baz[other]=12")));
}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}
