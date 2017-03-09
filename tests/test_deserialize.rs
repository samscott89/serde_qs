#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct A { b: B, c: C }
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct B { b1: u8, b2: String }
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct C { c1: String, c2: u8 }

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Complex { x: Vec<u8>, y: Vec<C> }


#[test]
fn deserialize_struct() {
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

    let complex_params = Complex {
      x: vec![0,1,2],
      y: vec![params.c.clone()],
    };

    let input = "b[b1]=10&b[b2]=Ten&c[c1]=Seven&c[c2]=7";
    let input2 = "c[c1]=Seven&b[b2]=Ten&b[b1]=10&c[c2]=7";
    let result: A = qs::from_str(&urlencode(input)).unwrap();
    assert_eq!(result, params);
    let result: A = qs::from_str(&input).unwrap();
    assert_eq!(result, params);
    let result: A = qs::from_str(&urlencode(input2)).unwrap();
    assert_eq!(result, params);
    let result: A = qs::from_str(&input2).unwrap();
    assert_eq!(result, params);

    let input3 = "x[0]=0&x[1]=1&x[2]=2&y[0][c1]=Seven&y[0][c2]=7";
    let result: Complex = qs::from_str(&input3).unwrap();
    assert_eq!(complex_params, result);



}

fn urlencode(input: &str) -> String {
  str::replace(&str::replace(input, "[", "%5B"), "]", "%5D")
}