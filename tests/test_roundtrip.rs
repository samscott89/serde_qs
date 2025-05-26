#![allow(clippy::approx_constant)]

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

/// Wrapper type for f32 that compares NaN values as equal
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
struct F32(f32);

impl PartialEq for F32 {
    fn eq(&self, other: &Self) -> bool {
        (self.0.is_nan() && other.0.is_nan()) || self.0 == other.0
    }
}

/// Wrapper type for f64 that compares NaN values as equal
#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
struct F64(f64);

impl PartialEq for F64 {
    fn eq(&self, other: &Self) -> bool {
        (self.0.is_nan() && other.0.is_nan()) || self.0 == other.0
    }
}

/// macro for testing roundtrip serialization and deserialization
///
/// This is a macro so that `insta` generates snapshots with
/// the function name
macro_rules! roundtrip_test {
    (
        $input:expr
        $(, $params:ident)*
    ) => {
        let data = $input;

        for form_encoding in [false, true] {
            let config = serde_qs::Config::new().use_form_encoding(form_encoding);

            insta::with_settings!({
                prepend_module_to_snapshot => false,
                raw_info => &insta::internals::Content::Map(vec![(
                    "use_form_encoding".into(),
                    form_encoding.into()
                )])
            }, {
                let serialized = config.serialize_string(&data).expect("serialize");

                #[allow(unused_mut)]
                let mut serialized_lines = serialized
                    .split('&').collect::<Vec<_>>();

                // Sort the parameters if requested

                $(
                    if stringify!($params) == "sort_params" {
                        serialized_lines.sort_unstable();
                    }
                )*
                let serialized_pretty = serialized_lines.join("&\n");

                // snapshot the serialized string for easy introspection
                // and to track changes
                insta::assert_snapshot!(serialized_pretty);

                let deserialized = config
                    .deserialize_str(serialized.as_str())
                    .expect("deserialize");

                // check we get the same data back
                pretty_assertions::assert_eq!(&data, &deserialized);
            });
        }
    };
}

// ========== BASIC STRUCTS ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct FlatStruct {
    a: u8,
    b: u8,
}

#[test]
fn flat_struct() {
    roundtrip_test!(FlatStruct { a: 1, b: 2 });
}

// ========== PRIMITIVE TYPES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct PrimitiveTypes {
    bool_val: bool,
    i8_val: i8,
    i16_val: i16,
    i32_val: i32,
    i64_val: i64,
    u8_val: u8,
    u16_val: u16,
    u32_val: u32,
    u64_val: u64,
    f32_val: f32,
    f64_val: f64,
    char_val: char,
    string_val: String,
}

#[test]
fn primitive_types() {
    roundtrip_test!(PrimitiveTypes {
        bool_val: true,
        i8_val: -128,
        i16_val: -32768,
        i32_val: -2147483648,
        i64_val: -9223372036854775808,
        u8_val: 255,
        u16_val: 65535,
        u32_val: 4294967295,
        u64_val: 18446744073709551615,
        f32_val: 3.14159,
        f64_val: 2.718281828,
        char_val: '🦀',
        string_val: "Hello, world! 你好世界".to_string(),
    });
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct EdgeCasePrimitives {
    empty_string: String,
    zero_int: i32,
    zero_float: f64,
    false_bool: bool,
    space_string: String,
    special_chars: String,
}

#[test]
fn edge_case_primitives() {
    roundtrip_test!(EdgeCasePrimitives {
        empty_string: String::new(),
        zero_int: 0,
        zero_float: 0.0,
        false_bool: false,
        space_string: "   ".to_string(),
        special_chars: "!@#$%^&*()_+-=[]{}|;':,.<>?/~`".to_string(),
    });
}

// ========== OPTION TYPES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct OptionTypes {
    opt_none: Option<String>,
    opt_some_string: Option<String>,
    opt_some_int: Option<i32>,
    opt_some_bool: Option<bool>,
    opt_some_vec: Option<Vec<u8>>,
}

#[test]
fn option_types() {
    roundtrip_test!(OptionTypes {
        opt_none: None,
        opt_some_string: Some("Hello".to_string()),
        opt_some_int: Some(42),
        opt_some_bool: Some(true),
        opt_some_vec: Some(vec![1, 2, 3]),
    });
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct NestedOptions {
    opt_struct: Option<FlatStruct>,
    opt_empty_string: Option<String>,
}

#[test]
fn nested_options() {
    roundtrip_test!(NestedOptions {
        opt_struct: Some(FlatStruct { a: 10, b: 20 }),
        opt_empty_string: Some(String::new()),
    });

    roundtrip_test!(NestedOptions {
        opt_struct: None,
        opt_empty_string: None,
    });
}

// ========== VECTORS AND SEQUENCES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct VectorTypes {
    empty_vec: Vec<i32>,
    single_vec: Vec<String>,
    multi_vec: Vec<u8>,
    vec_of_structs: Vec<FlatStruct>,
}

#[test]
fn vector_types() {
    roundtrip_test!(VectorTypes {
        empty_vec: vec![],
        single_vec: vec!["only one".to_string()],
        multi_vec: vec![1, 2, 3, 4, 5],
        vec_of_structs: vec![FlatStruct { a: 1, b: 2 }, FlatStruct { a: 3, b: 4 },],
    });
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct NestedVectors {
    vec_of_vecs: Vec<Vec<u8>>,
    vec_of_options: Vec<Option<String>>,
    option_vec: Option<Vec<i32>>,
}

#[test]
fn nested_vectors() {
    roundtrip_test!(NestedVectors {
        vec_of_vecs: vec![vec![1, 2], vec![], vec![3, 4, 5]],
        vec_of_options: vec![Some("first".to_string()), None, Some("third".to_string())],
        option_vec: Some(vec![10, 20, 30]),
    });

    // empty cases
    roundtrip_test!(NestedVectors {
        vec_of_vecs: vec![vec![], vec![]],
        vec_of_options: vec![Some("".to_string())],
        option_vec: Some(vec![]),
    });
}

// Test arrays of fixed size
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ArrayTypes {
    small_array: [u8; 3],
    string_array: [String; 2],
}

#[test]
fn array_types() {
    roundtrip_test!(ArrayTypes {
        small_array: [1, 2, 3],
        string_array: ["first".to_string(), "second".to_string()],
    });
}

// ========== MAPS ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct MapTypes {
    empty_map: HashMap<String, i32>,
    string_map: HashMap<String, String>,
    int_key_map: HashMap<u32, String>,
    struct_value_map: HashMap<String, FlatStruct>,
}

#[test]
fn map_types() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    string_map.insert("key2".to_string(), "value2".to_string());

    let mut int_key_map = HashMap::new();
    int_key_map.insert(1, "one".to_string());
    int_key_map.insert(2, "two".to_string());

    let mut struct_value_map = HashMap::new();
    struct_value_map.insert("first".to_string(), FlatStruct { a: 10, b: 20 });
    struct_value_map.insert("second".to_string(), FlatStruct { a: 30, b: 40 });

    roundtrip_test!(
        MapTypes {
            empty_map: HashMap::new(),
            string_map,
            int_key_map,
            struct_value_map,
        },
        sort_params
    );
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct BTreeMapTypes {
    ordered_map: BTreeMap<String, i32>,
    nested_map: BTreeMap<String, BTreeMap<String, String>>,
}

#[test]
fn btree_map_types() {
    let mut ordered_map = BTreeMap::new();
    ordered_map.insert("alpha".to_string(), 1);
    ordered_map.insert("beta".to_string(), 2);
    ordered_map.insert("gamma".to_string(), 3);

    let mut inner_map1 = BTreeMap::new();
    inner_map1.insert("x".to_string(), "foo".to_string());
    inner_map1.insert("y".to_string(), "bar".to_string());

    let mut inner_map2 = BTreeMap::new();
    inner_map2.insert("a".to_string(), "baz".to_string());

    let mut nested_map = BTreeMap::new();
    nested_map.insert("first".to_string(), inner_map1);
    nested_map.insert("second".to_string(), inner_map2);
    nested_map.insert("empty".to_string(), BTreeMap::new());

    roundtrip_test!(BTreeMapTypes {
        ordered_map,
        nested_map,
    });
}

// ========== TUPLE TYPES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TupleTypes {
    unit: (),
    pair: (i32, String),
    triple: (bool, f64, u8),
    nested_tuple: ((String, i32), Vec<u8>),
}

#[test]
fn tuple_types() {
    roundtrip_test!(TupleTypes {
        unit: (),
        pair: (42, "answer".to_string()),
        triple: (true, 3.14, 255),
        nested_tuple: (("nested".to_string(), -100), vec![1, 2, 3]),
    });
}

// Tuple structs
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TupleStruct(i32, String, bool);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ContainsTupleStruct {
    ts: TupleStruct,
    vec_of_tuples: Vec<(u8, u8)>,
}

#[test]
fn tuple_struct_types() {
    roundtrip_test!(ContainsTupleStruct {
        ts: TupleStruct(100, "tuple struct".to_string(), false),
        vec_of_tuples: vec![(1, 2), (3, 4), (5, 6)],
    });
}

// ========== ENUM TYPES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum BasicEnum {
    Unit,
    Newtype(String),
    Tuple(i32, bool),
    Struct { x: f64, y: String },
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct EnumContainer {
    unit: BasicEnum,
    newtype: BasicEnum,
    tuple: BasicEnum,
    struct_variant: BasicEnum,
    vec_of_enums: Vec<BasicEnum>,
}

#[test]
fn basic_enum_types() {
    roundtrip_test!(EnumContainer {
        unit: BasicEnum::Unit,
        newtype: BasicEnum::Newtype("hello".to_string()),
        tuple: BasicEnum::Tuple(42, true),
        struct_variant: BasicEnum::Struct {
            x: 3.14,
            y: "test".to_string(),
        },
        vec_of_enums: vec![BasicEnum::Unit, BasicEnum::Newtype("in vec".to_string()),],
    });
}

// ========== SERDE ATTRIBUTES: RENAME ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct RenameFields {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    #[serde(rename = "user-id")]
    user_id: u32,
}

#[test]
fn rename_fields() {
    roundtrip_test!(RenameFields {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        user_id: 12345,
    });
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameAllCamelCase {
    first_name: String,
    last_name: String,
    phone_number: String,
    email_address: String,
}

#[test]
fn rename_all_camel_case() {
    roundtrip_test!(RenameAllCamelCase {
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        phone_number: "555-1234".to_string(),
        email_address: "jane@example.com".to_string(),
    });
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct RenameAllScreamingSnake {
    field_one: i32,
    field_two: String,
    nested_field: Option<String>,
}

#[test]
fn rename_all_screaming_snake() {
    roundtrip_test!(RenameAllScreamingSnake {
        field_one: 42,
        field_two: "LOUD".to_string(),
        nested_field: Some("NESTED".to_string()),
    });
}

// ========== SERDE ATTRIBUTES: SKIP ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipFields {
    visible: String,
    #[serde(skip)]
    skip_always: String,
    #[serde(skip_serializing, default)]
    skip_ser: String,
    #[serde(skip_deserializing, default)]
    skip_de: String,
}

impl Default for SkipFields {
    fn default() -> Self {
        Self {
            visible: String::new(),
            skip_always: "skipped".to_string(),
            skip_ser: "skip serialization".to_string(),
            skip_de: "skip deserialization".to_string(),
        }
    }
}

#[test]
fn skip_fields() {
    // Note: skip fields won't roundtrip perfectly, but we can test serialization
    let data = SkipFields {
        visible: "you can see me".to_string(),
        skip_always: "invisible".to_string(),
        skip_ser: "not serialized".to_string(),
        skip_de: "not deserialized".to_string(),
    };

    let config = serde_qs::Config::new();
    let serialized = config.serialize_string(&data).expect("serialize");
    insta::with_settings!({
        prepend_module_to_snapshot => false
    }, {
        // snapshot the serialized string for easy introspection
        // and to track changes
        insta::assert_snapshot!(serialized);
    });

    // Deserialize and check that skip fields have their defaults
    let deserialized: SkipFields = config.deserialize_str(&serialized).expect("deserialize");
    assert_eq!(deserialized.visible, data.visible);
    assert_eq!(deserialized.skip_always, ""); // default value
    assert_eq!(deserialized.skip_ser, ""); // empty default from Default trait
    assert_eq!(deserialized.skip_de, ""); // default value
}

// ========== SERDE ATTRIBUTES: SKIP_SERIALIZING_IF ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingIf {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    maybe_string: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    maybe_vec: Vec<i32>,
    #[serde(skip_serializing_if = "is_zero", default)]
    maybe_zero: i32,
    always_present: String,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

#[test]
fn skip_serializing_if_fields() {
    // With values that should be serialized
    roundtrip_test!(SkipSerializingIf {
        maybe_string: Some("present".to_string()),
        maybe_vec: vec![1, 2, 3],
        maybe_zero: 42,
        always_present: "always".to_string(),
    });

    // With values that should be skipped
    roundtrip_test!(SkipSerializingIf {
        maybe_string: None,
        maybe_vec: vec![],
        maybe_zero: 0,
        always_present: "still here".to_string(),
    });
}

// ========== SERDE ATTRIBUTES: DEFAULT ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct WithDefaults {
    #[serde(default)]
    default_string: String,
    #[serde(default = "default_i32")]
    default_number: i32,
    #[serde(default = "default_vec")]
    default_vec: Vec<String>,
    normal_field: String,
}

fn default_i32() -> i32 {
    42
}

fn default_vec() -> Vec<String> {
    vec!["default1".to_string(), "default2".to_string()]
}

#[test]
fn default_fields() {
    // Full data roundtrip
    roundtrip_test!(WithDefaults {
        default_string: "custom".to_string(),
        default_number: 100,
        default_vec: vec!["a".to_string(), "b".to_string()],
        normal_field: "required".to_string(),
    });

    // Test deserialization with missing fields
    let config = serde_qs::Config::new();
    let partial = "normal_field=required";
    let deserialized: WithDefaults = config.deserialize_str(partial).expect("deserialize");

    assert_eq!(deserialized.default_string, ""); // Default::default()
    assert_eq!(deserialized.default_number, 42); // custom default
    assert_eq!(
        deserialized.default_vec,
        vec!["default1".to_string(), "default2".to_string()]
    ); // custom default
    assert_eq!(deserialized.normal_field, "required");
}

// ========== SERDE ATTRIBUTES: ALIAS ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct WithAliases {
    #[serde(alias = "name", alias = "username")]
    user_name: String,
    #[serde(alias = "msg")]
    message: String,
    normal: i32,
}

#[test]
fn alias_fields() {
    // Normal roundtrip uses the actual field names
    roundtrip_test!(WithAliases {
        user_name: "alice".to_string(),
        message: "hello world".to_string(),
        normal: 42,
    });

    // Test deserialization with aliases
    let config = serde_qs::Config::new();

    // Using primary alias
    let with_alias1 = "name=bob&message=hi&normal=1";
    let deserialized: WithAliases = config.deserialize_str(with_alias1).expect("deserialize");
    assert_eq!(deserialized.user_name, "bob");
    assert_eq!(deserialized.message, "hi");

    // Using secondary alias
    let with_alias2 = "username=charlie&msg=bye&normal=2";
    let deserialized: WithAliases = config.deserialize_str(with_alias2).expect("deserialize");
    assert_eq!(deserialized.user_name, "charlie");
    assert_eq!(deserialized.message, "bye");
}

// ========== SERDE ATTRIBUTES: FLATTEN ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Person {
    name: String,
    age: u32,
    #[serde(flatten)]
    address: Address,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Company {
    company_name: String,
    #[serde(flatten)]
    headquarters: Address,
    employee_count: u32,
}

#[test]
fn flatten_fields() {
    // Simple flatten
    roundtrip_test!(Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip: "12345".to_string(),
        },
    });

    // Multiple levels with flatten
    roundtrip_test!(Company {
        company_name: "Acme Corp".to_string(),
        headquarters: Address {
            street: "456 Business Ave".to_string(),
            city: "Metropolis".to_string(),
            zip: "67890".to_string(),
        },
        employee_count: 100,
    });
}

// Flatten with maps
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct WithFlattenedMap {
    id: u32,
    name: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[test]
fn flatten_map() {
    let mut extra = HashMap::new();
    extra.insert("color".to_string(), "blue".to_string());
    extra.insert("size".to_string(), "large".to_string());
    extra.insert("custom_field".to_string(), "custom_value".to_string());

    roundtrip_test!(
        WithFlattenedMap {
            id: 42,
            name: "Widget".to_string(),
            extra,
        },
        sort_params
    );
}

// ========== ENUM REPRESENTATIONS ==========

// Externally tagged (default)
#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum ExternallyTagged {
    Unit,
    Newtype(String),
    Tuple(i32, i32),
    Struct { a: String, b: bool },
}

#[test]
fn externally_tagged_enum() {
    roundtrip_test!(ExternallyTagged::Unit);
    roundtrip_test!(ExternallyTagged::Newtype("hello".to_string()));
    roundtrip_test!(ExternallyTagged::Tuple(1, 2));
    roundtrip_test!(ExternallyTagged::Struct {
        a: "test".to_string(),
        b: true,
    });
}

// Internally tagged
#[serde_with::serde_as]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
enum InternallyTagged {
    Unit,
    Struct {
        value: String,
    },
    // Similarly to untagged enums, we need a few workarounds to
    // let complex internally tagged enums work
    // in this case, the fields get buffered and the default
    // inferred types don't work well with the deserializer
    Complex {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        x: i32,
        #[serde_as(as = "serde_with::DisplayFromStr")]
        y: i32,
        data: Vec<String>,
    },
}

#[test]
fn internally_tagged_enum() {
    roundtrip_test!(InternallyTagged::Unit);
    roundtrip_test!(InternallyTagged::Struct {
        value: "test".to_string()
    });
    roundtrip_test!(InternallyTagged::Complex {
        x: 10,
        y: 20,
        data: vec!["a".to_string(), "b".to_string()]
    });
}

// Adjacently tagged
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged {
    Unit,
    Newtype(String),
    Tuple(i32, i32),
    Struct { name: String, count: u32 },
}

#[test]
fn adjacently_tagged_enum() {
    roundtrip_test!(AdjacentlyTagged::Unit);
    roundtrip_test!(AdjacentlyTagged::Newtype("adjacent".to_string()));
    roundtrip_test!(AdjacentlyTagged::Tuple(3, 4));
    roundtrip_test!(AdjacentlyTagged::Struct {
        name: "item".to_string(),
        count: 42
    });
}

// Untagged
#[serde_with::serde_as]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
enum Untagged {
    Bool(#[serde_as(as = "serde_with::DisplayFromStr")] bool),
    Number(#[serde_as(as = "serde_with::DisplayFromStr")] i32),
    Text(String),
    Struct { field: String },
}

#[test]
fn untagged_enum() {
    roundtrip_test!(Untagged::Bool(true));
    roundtrip_test!(Untagged::Number(42));
    roundtrip_test!(Untagged::Text("untagged".to_string()));
    roundtrip_test!(Untagged::Struct {
        field: "value".to_string()
    });
}

// Mixed enum representations in nested structures
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct MixedEnums {
    external: ExternallyTagged,
    internal: InternallyTagged,
    adjacent: AdjacentlyTagged,
    untagged: Untagged,
    optional_enum: Option<ExternallyTagged>,
}

#[test]
fn mixed_enum_representations() {
    roundtrip_test!(MixedEnums {
        external: ExternallyTagged::Struct {
            a: "ext".to_string(),
            b: false
        },
        internal: InternallyTagged::Struct {
            value: "int".to_string()
        },
        adjacent: AdjacentlyTagged::Newtype("adj".to_string()),
        untagged: Untagged::Number(100),
        optional_enum: Some(ExternallyTagged::Unit),
    });
}

// ========== COMPLEX NESTED STRUCTURES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ComplexNested {
    // Option containing Vec of Options
    maybe_vec_maybe: Option<Vec<Option<String>>>,

    // Vec of Vecs
    matrix: Vec<Vec<i32>>,

    // HashMap with Vec values
    map_of_vecs: HashMap<String, Vec<String>>,

    // Vec of HashMaps
    vec_of_maps: Vec<HashMap<String, i32>>,

    // // Nested Option
    // NOTE: we cannot support this since there's no way
    // to differentiate the layers of options
    // nested_option: Option<Option<String>>,

    // Mixed nesting with tuple
    complex_tuple: Vec<(String, Option<Vec<i32>>)>,
}

#[test]
fn complex_nested_structures() {
    let mut map_of_vecs = HashMap::new();
    map_of_vecs.insert(
        "colors".to_string(),
        vec!["red".to_string(), "blue".to_string()],
    );
    map_of_vecs.insert(
        "sizes".to_string(),
        vec![
            "small".to_string(),
            "medium".to_string(),
            "large".to_string(),
        ],
    );

    let mut map1 = HashMap::new();
    map1.insert("a".to_string(), 1);
    map1.insert("b".to_string(), 2);

    let mut map2 = HashMap::new();
    map2.insert("x".to_string(), 10);
    map2.insert("y".to_string(), 20);

    roundtrip_test!(
        ComplexNested {
            maybe_vec_maybe: Some(vec![
                Some("hello".to_string()),
                None,
                Some("world".to_string())
            ]),
            matrix: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
            map_of_vecs,
            vec_of_maps: vec![map1, map2],
            complex_tuple: vec![
                ("first".to_string(), Some(vec![1, 2, 3])),
                ("second".to_string(), None),
                ("third".to_string(), Some(vec![4, 5])),
            ],
        },
        sort_params
    );
}

// Test with all None/empty values
#[test]
fn complex_nested_empty() {
    roundtrip_test!(ComplexNested {
        maybe_vec_maybe: None,
        matrix: vec![],
        map_of_vecs: HashMap::new(),
        vec_of_maps: vec![],
        complex_tuple: vec![],
    });

    // Test with Some(empty) and Some(None) values
    roundtrip_test!(ComplexNested {
        maybe_vec_maybe: Some(vec![]),
        matrix: vec![vec![]],
        map_of_vecs: HashMap::new(),
        vec_of_maps: vec![HashMap::new()],
        complex_tuple: vec![("empty".to_string(), Some(vec![]))],
    });
}

// Recursive structures
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TreeNode {
    value: String,
    children: Vec<TreeNode>,
}

#[test]
fn recursive_structure() {
    roundtrip_test!(TreeNode {
        value: "root".to_string(),
        children: vec![
            TreeNode {
                value: "child1".to_string(),
                children: vec![
                    TreeNode {
                        value: "grandchild1".to_string(),
                        children: vec![],
                    },
                    TreeNode {
                        value: "grandchild2".to_string(),
                        children: vec![],
                    },
                ],
            },
            TreeNode {
                value: "child2".to_string(),
                children: vec![],
            },
        ],
    });
}

// ========== NEWTYPE STRUCTS AND TRANSPARENT TYPES ==========

// Basic newtype
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct UserId(u64);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Username(String);

// Transparent newtype
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
struct Email(String);

// Newtype with validation (using custom deserialize)
#[derive(Debug, PartialEq, Serialize)]
struct PositiveNumber(i32);

impl<'de> Deserialize<'de> for PositiveNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        if value > 0 {
            Ok(PositiveNumber(value))
        } else {
            Err(serde::de::Error::custom("number must be positive"))
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct UserProfile {
    id: UserId,
    name: Username,
    email: Email,
    age: PositiveNumber,
}

#[test]
fn newtype_structs() {
    roundtrip_test!(UserProfile {
        id: UserId(12345),
        name: Username("alice".to_string()),
        email: Email("alice@example.com".to_string()),
        age: PositiveNumber(25),
    });
}

// Multiple levels of newtype wrapping
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Outer(Inner);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Inner(String);

#[test]
fn nested_newtypes() {
    roundtrip_test!(Outer(Inner("nested newtype".to_string())));
}

// Newtype over collections
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Tags(Vec<String>);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Scores(HashMap<String, i32>);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Document {
    title: String,
    tags: Tags,
    scores: Scores,
}

#[test]
fn newtype_collections() {
    let mut scores_map = HashMap::new();
    scores_map.insert("relevance".to_string(), 95);
    scores_map.insert("quality".to_string(), 87);

    roundtrip_test!(
        Document {
            title: "Test Document".to_string(),
            tags: Tags(vec![
                "rust".to_string(),
                "serde".to_string(),
                "testing".to_string()
            ]),
            scores: Scores(scores_map),
        },
        sort_params
    );
}

// ========== UNIT STRUCTS AND UNIT TYPE ==========

// Unit struct
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct UnitStruct;

// Unit-like enum variants were already tested, but let's test unit structs in containers
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ContainsUnits {
    unit_value: (),
    unit_struct: UnitStruct,
    optional_unit: Option<()>,
    vec_of_units: Vec<UnitStruct>,
    tuple_with_unit: (String, (), i32),
}

#[test]
fn unit_types() {
    roundtrip_test!(ContainsUnits {
        unit_value: (),
        unit_struct: UnitStruct,
        optional_unit: Some(()),
        vec_of_units: vec![UnitStruct, UnitStruct, UnitStruct],
        tuple_with_unit: ("test".to_string(), (), 42),
    });

    // Test with None for optional unit
    roundtrip_test!(ContainsUnits {
        unit_value: (),
        unit_struct: UnitStruct,
        optional_unit: None,
        vec_of_units: vec![],
        tuple_with_unit: ("empty".to_string(), (), 0),
    });
}

// PhantomData (zero-sized type)
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TypedId<T> {
    value: u64,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> TypedId<T> {
    fn new(value: u64) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct User;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Post;

#[test]
fn phantom_data_types() {
    roundtrip_test!(TypedId::<User>::new(100));
    roundtrip_test!(TypedId::<Post>::new(1000));
}

// ========== SPECIAL CASES AND EDGE CASES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct EdgeCases {
    // Empty string vs None
    empty_string: String,
    optional_empty: Option<String>,

    // Zero values
    zero_i32: i32,
    zero_f64: f64,
    zero_u8: u8,

    // Special float values
    nan_value: F32,
    infinity: F64,
    neg_infinity: F64,

    // Very large numbers
    max_u64: u64,
    min_i64: i64,

    // Unicode strings
    unicode: String,
    emoji: String,

    // Special characters
    special_chars: String,
}

#[test]
fn edge_case_values() {
    roundtrip_test!(EdgeCases {
        empty_string: "".to_string(),
        optional_empty: Some("".to_string()),
        zero_i32: 0,
        zero_f64: 0.0,
        zero_u8: 0,
        nan_value: F32(f32::NAN),
        infinity: F64(f64::INFINITY),
        neg_infinity: F64(f64::NEG_INFINITY),
        max_u64: u64::MAX,
        min_i64: i64::MIN,
        unicode: "Hello 世界 🌍".to_string(),
        emoji: "🚀🎉🔥💯".to_string(),
        special_chars: "a&b=c?d#e".to_string(),
    });
}

// Separate test for NaN handling (not used due to wrapper types above)
#[test]
fn nan_roundtrip_direct() {
    #[derive(Debug, Deserialize, Serialize)]
    struct FloatSpecials {
        nan: f32,
        inf: f64,
        neg_inf: f64,
    }

    let data = FloatSpecials {
        nan: f32::NAN,
        inf: f64::INFINITY,
        neg_inf: f64::NEG_INFINITY,
    };

    let config = serde_qs::Config::new();
    let serialized = config.serialize_string(&data).expect("serialize");
    let deserialized: FloatSpecials = config.deserialize_str(&serialized).expect("deserialize");

    // Custom assertions for special float values
    assert!(deserialized.nan.is_nan());
    assert!(deserialized.inf.is_infinite() && deserialized.inf.is_sign_positive());
    assert!(deserialized.neg_inf.is_infinite() && deserialized.neg_inf.is_sign_negative());
}

// Test with all empty collections
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct AllEmpty {
    empty_vec: Vec<i32>,
    empty_vec_vec: Vec<Vec<String>>,
    empty_hashmap: HashMap<String, i32>,
    empty_btreemap: BTreeMap<i32, String>,
    empty_option_vec: Option<Vec<bool>>,
    empty_string: String,
}

#[test]
fn all_empty_collections() {
    roundtrip_test!(AllEmpty {
        empty_vec: vec![],
        empty_vec_vec: vec![],
        empty_hashmap: HashMap::new(),
        empty_btreemap: BTreeMap::new(),
        empty_option_vec: Some(vec![]),
        empty_string: String::new(),
    });
}

// Very deeply nested structure
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct VeryDeep {
    level1: Option<Box<VeryDeep>>,
    value: String,
}

#[test]
fn very_deep_nesting() {
    let deep = VeryDeep {
        level1: Some(Box::new(VeryDeep {
            level1: Some(Box::new(VeryDeep {
                level1: Some(Box::new(VeryDeep {
                    level1: Some(Box::new(VeryDeep {
                        level1: None,
                        value: "deepest".to_string(),
                    })),
                    value: "level4".to_string(),
                })),
                value: "level3".to_string(),
            })),
            value: "level2".to_string(),
        })),
        value: "level1".to_string(),
    };

    roundtrip_test!(deep);
}

// Single element collections
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SingleElements {
    single_vec: Vec<String>,
    single_map: HashMap<String, i32>,
    single_option: Option<Vec<u8>>,
    single_char_string: String,
}

#[test]
fn single_element_collections() {
    let mut single_map = HashMap::new();
    single_map.insert("only".to_string(), 42);

    roundtrip_test!(SingleElements {
        single_vec: vec!["single".to_string()],
        single_map,
        single_option: Some(vec![255]),
        single_char_string: "x".to_string(),
    });
}

// ========== HELPER ATTRIBUTES ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct WithHelpers {
    // Comma-separated values
    #[serde(with = "serde_qs::helpers::comma_separated")]
    comma_values: Vec<String>,

    // Pipe-delimited values
    #[serde(with = "serde_qs::helpers::pipe_delimited")]
    pipe_values: Vec<i32>,

    // Space-delimited values
    #[serde(with = "serde_qs::helpers::space_delimited")]
    space_values: Vec<String>,

    // Generic delimiter (using dot)
    #[serde(deserialize_with = "serde_qs::helpers::generic_delimiter::deserialize::<_, _, '.'>")]
    #[serde(serialize_with = "serde_qs::helpers::generic_delimiter::serialize::<_, _, '.'>")]
    dot_values: Vec<u8>,

    // Normal vec for comparison
    normal_vec: Vec<String>,
}

#[test]
fn helper_attributes() {
    roundtrip_test!(WithHelpers {
        comma_values: vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string()
        ],
        pipe_values: vec![1, 2, 3, 4, 5],
        space_values: vec!["hello".to_string(), "world".to_string()],
        dot_values: vec![10, 20, 30],
        normal_vec: vec!["normal".to_string(), "array".to_string()],
    });

    // Test with empty vecs
    roundtrip_test!(WithHelpers {
        comma_values: vec![],
        pipe_values: vec![],
        space_values: vec![],
        dot_values: vec![],
        normal_vec: vec![],
    });

    // Test with single elements
    roundtrip_test!(WithHelpers {
        comma_values: vec!["single".to_string()],
        pipe_values: vec![42],
        space_values: vec!["one".to_string()],
        dot_values: vec![255],
        normal_vec: vec!["alone".to_string()],
    });
}

// Test helpers with special characters and edge cases
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct HelperEdgeCases {
    // Values containing the delimiter
    #[serde(with = "serde_qs::helpers::comma_separated")]
    tricky_comma: Vec<String>,

    #[serde(with = "serde_qs::helpers::pipe_delimited")]
    numbers_with_negatives: Vec<i32>,

    #[serde(with = "serde_qs::helpers::space_delimited")]
    unicode_words: Vec<String>,
}

#[test]
fn helper_edge_cases() {
    roundtrip_test!(HelperEdgeCases {
        // Note: these values don't contain commas to avoid parsing issues
        tricky_comma: vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string()
        ],
        numbers_with_negatives: vec![-10, 0, 10, -5, 5],
        unicode_words: vec!["Hello".to_string(), "世界".to_string(), "🌍".to_string()],
    });
}

// Nested structure with helpers
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct NestedWithHelpers {
    data: Vec<InnerWithHelpers>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct InnerWithHelpers {
    name: String,
    #[serde(with = "serde_qs::helpers::comma_separated")]
    tags: Vec<String>,
}

#[test]
fn nested_helpers() {
    roundtrip_test!(NestedWithHelpers {
        data: vec![
            InnerWithHelpers {
                name: "item1".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
            },
            InnerWithHelpers {
                name: "item2".to_string(),
                tags: vec!["tag3".to_string(), "tag4".to_string(), "tag5".to_string()],
            },
        ],
    });
}

// ========== BORROWED VS OWNED DATA ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct WithCow<'a> {
    // Cow<str> - can be borrowed or owned
    #[serde(borrow)]
    cow_str: Cow<'a, str>,

    // Always owned for comparison
    owned_string: String,

    // Vec of Cow strings
    #[serde(borrow)]
    cow_vec: Vec<Cow<'a, str>>,
}

#[test]
fn cow_types() {
    // Test with owned data
    roundtrip_test!(WithCow {
        cow_str: Cow::Owned("owned cow".to_string()),
        owned_string: "always owned".to_string(),
        cow_vec: vec![
            Cow::Owned("first".to_string()),
            Cow::Owned("second".to_string()),
        ],
    });

    // Note: We can't test borrowed data in roundtrip because
    // serialization always produces owned strings
}

// Test that demonstrates owned vs borrowed doesn't affect serialization
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct StringVariants {
    owned: String,
    vec_owned: Vec<String>,
}

#[test]
fn string_ownership() {
    roundtrip_test!(StringVariants {
        owned: "hello world".to_string(),
        vec_owned: vec!["one".to_string(), "two".to_string(), "three".to_string()],
    });
}

// Bytes data (borrowed slice can't be tested in roundtrip)
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct BytesData {
    #[serde(with = "serde_bytes")]
    byte_vec: Vec<u8>,

    regular_u8_vec: Vec<u8>,
}

#[test]
fn bytes_types() {
    roundtrip_test!(BytesData {
        byte_vec: vec![0, 1, 2, 255, 128, 64],
        regular_u8_vec: vec![10, 20, 30, 40],
    });
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
struct StringWrapper(String);

#[test]
fn toplevel_string() {
    roundtrip_test!(StringWrapper("just a string".to_string()));
}
