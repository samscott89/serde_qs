#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

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

    assert_eq!(qs::to_string(&params),
      Ok(urlencode("bar[x]=10&bar[y]=Ten&baz[thing]=Thing&baz[other]=12")));
}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}
