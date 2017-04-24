extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    #[serde(deserialize_with="from_csv")]
    r: Vec<u8>,
    s: u8,
}

fn main() {
    let q = "s=12&r=1,2,3";
    let q: Query = qs::from_str(q).unwrap();
    println!("{:?}", q);
}


fn from_csv<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(CSVVecVisitor)
}

/// Visits a string value of the form "v1,v2,v3" into a vector of bytes Vec<u8>
struct CSVVecVisitor;

impl<'de> serde::de::Visitor<'de> for CSVVecVisitor {
    type Value = Vec<u8>;

    fn expecting(&self,
                 formatter: &mut std::fmt::Formatter)
                 -> std::fmt::Result {
        write!(formatter, "a str")
    }

    fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
        where E: serde::de::Error,
    {
        let mut output = Vec::new();
        let mut items = csv::Reader::from_string(s);
        // let items = items.next_str();
        while let csv::NextField::Data(item) = items.next_str() {
            output.push(u8::from_str_radix(item, 10).unwrap());
        }

        Ok(output)
    }
}
