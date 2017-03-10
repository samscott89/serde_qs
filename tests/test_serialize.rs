#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

#[derive(Serialize, Deserialize)]
struct Foo { bar: Bar, baz: Baz }
#[derive(Serialize, Deserialize)]
struct Bar { x: u8, y: String }
#[derive(Serialize, Deserialize)]
struct Baz { thing: String, other: u8 }

#[derive(Serialize, Deserialize)]
struct Complex { x: Vec<u8>, y: Vec<Baz> }


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

    let params = Complex {
        x: vec![0,1,2],
        y: vec![params.baz],
    };


    assert_eq!(qs::to_string(&params),
      Ok(urlencode("x[0]=0&x[1]=1&x[2]=2&y[0][thing]=Thing&y[0][other]=12")));
}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}
