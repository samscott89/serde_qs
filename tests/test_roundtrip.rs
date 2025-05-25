use serde::{Deserialize, Serialize};

/// macro for testing roundtrip serialization and deserialization
///
/// This is a macro so that `insta` generates snapshots with
/// the function name
macro_rules! roundtrip_test {
    (
        $data:expr
    ) => {
        let data = &$data;

        for form_encoding in [false, true] {
            let config = serde_qs::Config::new().use_form_encoding(form_encoding);

            insta::with_settings!({
                raw_info => &insta::internals::Content::Map(vec![(
                    "use_form_encoding".into(),
                    form_encoding.into()
                )])
            }, {
                let serialized = config.serialize_string(data).expect("serialize");
                // snapshot the serialized string for easy introspection
                // and to track changes
                insta::assert_snapshot!(serialized);

                let deserialized = config
                    .deserialize_str(serialized.as_str())
                    .expect("deserialize");

                // check we get the same data back
                pretty_assertions::assert_eq!(data, &deserialized);
            });
        }
    };
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct FlatStruct {
    a: u8,
    b: u8,
}

#[test]
fn flat_struct() {
    roundtrip_test!(FlatStruct { a: 1, b: 2 });
}
