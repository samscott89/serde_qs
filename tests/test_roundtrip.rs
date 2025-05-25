use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

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
    opt_opt_string: Option<Option<String>>,
    opt_struct: Option<FlatStruct>,
    opt_empty_string: Option<String>,
}

#[test]
fn nested_options() {
    roundtrip_test!(NestedOptions {
        opt_opt_string: Some(None),
        opt_struct: Some(FlatStruct { a: 10, b: 20 }),
        opt_empty_string: Some(String::new()),
    });
    
    roundtrip_test!(NestedOptions {
        opt_opt_string: Some(Some("nested".to_string())),
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
        vec_of_structs: vec![
            FlatStruct { a: 1, b: 2 },
            FlatStruct { a: 3, b: 4 },
        ],
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
    
    roundtrip_test!(MapTypes {
        empty_map: HashMap::new(),
        string_map,
        int_key_map,
        struct_value_map,
    });
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
    UnitVariant,
    NewtypeVariant(String),
    TupleVariant(i32, bool),
    StructVariant { x: f64, y: String },
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
        unit: BasicEnum::UnitVariant,
        newtype: BasicEnum::NewtypeVariant("hello".to_string()),
        tuple: BasicEnum::TupleVariant(42, true),
        struct_variant: BasicEnum::StructVariant {
            x: 3.14,
            y: "test".to_string(),
        },
        vec_of_enums: vec![
            BasicEnum::UnitVariant,
            BasicEnum::NewtypeVariant("in vec".to_string()),
        ],
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
    #[serde(skip_serializing)]
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
    insta::assert_snapshot!(serialized);
    
    // Deserialize and check that skip fields have their defaults
    let deserialized: SkipFields = config.deserialize_str(&serialized).expect("deserialize");
    assert_eq!(deserialized.visible, data.visible);
    assert_eq!(deserialized.skip_always, "skipped"); // default value
    assert_eq!(deserialized.skip_ser, ""); // empty default from Default trait
    assert_eq!(deserialized.skip_de, "skip deserialization"); // default value
}

// ========== SERDE ATTRIBUTES: SKIP_SERIALIZING_IF ==========

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SkipSerializingIf {
    #[serde(skip_serializing_if = "Option::is_none")]
    maybe_string: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    maybe_vec: Vec<i32>,
    #[serde(skip_serializing_if = "is_zero")]
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
    let data = SkipSerializingIf {
        maybe_string: None,
        maybe_vec: vec![],
        maybe_zero: 0,
        always_present: "still here".to_string(),
    };
    
    let config = serde_qs::Config::new();
    let serialized = config.serialize_string(&data).expect("serialize");
    insta::assert_snapshot!(serialized);
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
    assert_eq!(deserialized.default_vec, vec!["default1".to_string(), "default2".to_string()]); // custom default
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
    
    roundtrip_test!(WithFlattenedMap {
        id: 42,
        name: "Widget".to_string(),
        extra,
    });
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
    roundtrip_test!(vec![
        ExternallyTagged::Unit,
        ExternallyTagged::Newtype("hello".to_string()),
        ExternallyTagged::Tuple(1, 2),
        ExternallyTagged::Struct { a: "test".to_string(), b: true },
    ]);
}

// Internally tagged
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
enum InternallyTagged {
    Unit,
    Struct { value: String },
    Complex { x: i32, y: i32, data: Vec<String> },
}

#[test]
fn internally_tagged_enum() {
    roundtrip_test!(vec![
        InternallyTagged::Unit,
        InternallyTagged::Struct { value: "test".to_string() },
        InternallyTagged::Complex { 
            x: 10, 
            y: 20, 
            data: vec!["a".to_string(), "b".to_string()] 
        },
    ]);
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
    roundtrip_test!(vec![
        AdjacentlyTagged::Unit,
        AdjacentlyTagged::Newtype("adjacent".to_string()),
        AdjacentlyTagged::Tuple(3, 4),
        AdjacentlyTagged::Struct { name: "item".to_string(), count: 42 },
    ]);
}

// Untagged
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
enum Untagged {
    Bool(bool),
    Number(i32),
    Text(String),
    Struct { field: String },
}

#[test]
fn untagged_enum() {
    roundtrip_test!(vec![
        Untagged::Bool(true),
        Untagged::Number(42),
        Untagged::Text("untagged".to_string()),
        Untagged::Struct { field: "value".to_string() },
    ]);
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
        external: ExternallyTagged::Struct { a: "ext".to_string(), b: false },
        internal: InternallyTagged::Struct { value: "int".to_string() },
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
    
    // Deeply nested Option
    deep_option: Option<Option<Option<String>>>,
    
    // Mixed nesting with tuple
    complex_tuple: Vec<(String, Option<Vec<i32>>)>,
}

#[test]
fn complex_nested_structures() {
    let mut map_of_vecs = HashMap::new();
    map_of_vecs.insert("colors".to_string(), vec!["red".to_string(), "blue".to_string()]);
    map_of_vecs.insert("sizes".to_string(), vec!["small".to_string(), "medium".to_string(), "large".to_string()]);
    
    let mut map1 = HashMap::new();
    map1.insert("a".to_string(), 1);
    map1.insert("b".to_string(), 2);
    
    let mut map2 = HashMap::new();
    map2.insert("x".to_string(), 10);
    map2.insert("y".to_string(), 20);
    
    roundtrip_test!(ComplexNested {
        maybe_vec_maybe: Some(vec![Some("hello".to_string()), None, Some("world".to_string())]),
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
        map_of_vecs,
        vec_of_maps: vec![map1, map2],
        deep_option: Some(Some(Some("deeply nested".to_string()))),
        complex_tuple: vec![
            ("first".to_string(), Some(vec![1, 2, 3])),
            ("second".to_string(), None),
            ("third".to_string(), Some(vec![4, 5])),
        ],
    });
}

// Test with all None/empty values
#[test]
fn complex_nested_empty() {
    roundtrip_test!(ComplexNested {
        maybe_vec_maybe: None,
        matrix: vec![],
        map_of_vecs: HashMap::new(),
        vec_of_maps: vec![],
        deep_option: None,
        complex_tuple: vec![],
    });
    
    // Test with Some(empty) and Some(None) values
    roundtrip_test!(ComplexNested {
        maybe_vec_maybe: Some(vec![]),
        matrix: vec![vec![]],
        map_of_vecs: HashMap::new(),
        vec_of_maps: vec![HashMap::new()],
        deep_option: Some(None),
        complex_tuple: vec![("empty".to_string(), Some(vec![]))],
    });
}

// Recursive structures (using Box to avoid infinite size)
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TreeNode {
    value: String,
    children: Vec<Box<TreeNode>>,
}

#[test]
fn recursive_structure() {
    roundtrip_test!(TreeNode {
        value: "root".to_string(),
        children: vec![
            Box::new(TreeNode {
                value: "child1".to_string(),
                children: vec![
                    Box::new(TreeNode {
                        value: "grandchild1".to_string(),
                        children: vec![],
                    }),
                    Box::new(TreeNode {
                        value: "grandchild2".to_string(),
                        children: vec![],
                    }),
                ],
            }),
            Box::new(TreeNode {
                value: "child2".to_string(),
                children: vec![],
            }),
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
    
    roundtrip_test!(Document {
        title: "Test Document".to_string(),
        tags: Tags(vec!["rust".to_string(), "serde".to_string(), "testing".to_string()]),
        scores: Scores(scores_map),
    });
}
