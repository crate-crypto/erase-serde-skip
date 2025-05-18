use erase_serde_skip::erase_skip_serializing_if;
use serde::{Deserialize, Serialize};

// This struct lives in the _test_ crate, not in your proc-macro crate’s src/
#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo {
    a: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<u32>,
    c: u16,
}

#[test]
fn roundtrip_none() {
    let foo = Foo {
        a: 1,
        b: None,
        c: 0x1234,
    };
    let bytes = bincode::serialize(&foo).unwrap();
    let decoded: Foo = bincode::deserialize(&bytes).unwrap();
    assert_eq!(foo, decoded);
}

#[test]
fn roundtrip_some() {
    let foo = Foo {
        a: 2,
        b: Some(0xDEADBEEF),
        c: 0x4321,
    };
    let bytes = bincode::serialize(&foo).unwrap();
    let decoded: Foo = bincode::deserialize(&bytes).unwrap();
    assert_eq!(foo, decoded);
}

/// 1.  The field `b` is `None`; after the macro expansion it must still be
///     encoded (`"b":null` in JSON) because the `skip_serializing_if` is gone.
#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct HasOption {
    #[serde(skip_serializing_if = "Option::is_none", rename = "id")]
    b: Option<u8>,
}

#[test]
fn option_none_is_serialized() {
    let v = HasOption { b: None };
    let json = serde_json::to_string(&v).unwrap();
    assert!(
        json.contains("\"id\":null"),
        "skip_serializing_if should have been removed, got {json}"
    );
    // make sure we can read it back
    let round: HasOption = serde_json::from_str(&json).unwrap();
    assert_eq!(v, round);
}

/// 2.  Two different kinds of `skip_serializing_if` in the same attribute list
///     should *both* be dropped, while other items (here: `default`) survive.
#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MultipleSkips {
    #[serde(
        skip_serializing_if = "Option::is_none",
        skip_serializing_if = "Vec::<u8>::is_empty",
        default
    )]
    data: Vec<u8>,
}

#[test]
fn multiple_skips_are_removed() {
    let v = MultipleSkips { data: Vec::new() };
    let json = serde_json::to_string(&v).unwrap();
    assert!(
        json.contains("\"data\":[]"),
        "all skip_serializing_if clauses should have been erased, got {json}"
    );
    let round: MultipleSkips = serde_json::from_str(&json).unwrap();
    assert_eq!(v, round);
}

/// 3.  Make sure the macro does *not* touch tuple structs or unit structs,
///     i.e. it compiles but leaves their attributes unchanged.
#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TupleSkip(#[serde(skip_serializing_if = "Option::is_none")] Option<i32>);

#[test]
fn tuple_struct_is_untouched() {
    // The macro should compile, do nothing to the attribute,
    // and we should still round-trip through JSON.
    let v = TupleSkip(None);

    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, "null");

    let round: TupleSkip = serde_json::from_str(&json).unwrap();
    assert_eq!(v, round);
}

#[erase_skip_serializing_if]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TestEnum {
    /// Named-field variant
    V {
        x: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<u32>,
    },
    /// Tuple variant
    T(
        #[serde(skip_serializing_if = "Option::is_none")] Option<u16>,
        u8,
    ),
}

#[test]
fn enum_struct_variant_skip_is_removed() {
    // `y` is None → predicate is true
    let v = TestEnum::V { x: 7, y: None };

    // After macro expansion `skip_serializing_if` is gone,
    // so the field MUST still appear in the JSON.
    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, r#"{"V":{"x":7,"y":null}}"#);
}

#[test]
fn enum_tuple_variant_skip_is_removed() {
    // First element is None, but it must stay in output ([null, 3])
    let v = TestEnum::T(None, 3);

    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, r#"{"T":[null,3]}"#);
}
