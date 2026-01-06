use sciformats_serde_json::span::Span;
use serde::Deserialize;
use std::{
    cell::RefCell,
    io::{Cursor, Read, Seek},
    rc::Rc,
};

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleStruct {
    foo: i32,
    bar: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NestedStructParent {
    foo: i32,
    nested: Span,
    bar: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NestedSeqParent {
    foo: i32,
    nested: Span,
    bar: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NestedStruct {
    foo: i32,
    bar: String,
}

#[test]
fn test_deserialize_no_span_simple_struct() {
    let json = br#"{"foo": 42, "bar": "baz"}"#;
    let cursor = Cursor::new(json);
    let rc_cursor = Rc::new(RefCell::new(cursor));
    let mut borrow = rc_cursor.borrow_mut();

    let mut de = sciformats_serde_json::Deserializer::from_reader(&mut *borrow);
    let value = SimpleStruct::deserialize(&mut de).unwrap();

    assert_eq!(
        value,
        SimpleStruct {
            foo: 42,
            bar: "baz".to_string()
        }
    );
}

#[test]
fn deserialize_top_level_span_struct() {
    let json = br#"{"foo": 42, "bar": "baz"}"#;
    let json_len = json.len() as u64;
    let cursor = Cursor::new(json);
    let rc_cursor = Rc::new(RefCell::new(cursor));
    let mut borrow = rc_cursor.borrow_mut();

    let mut de = sciformats_serde_json::Deserializer::from_reader(&mut *borrow);
    let value = Span::deserialize(&mut de).unwrap();

    assert_eq!(Span { span: 0..json_len }, value);
    assert_eq!(json_len, de.byte_offset());
}

#[test]
fn deserialize_nested_span_struct() {
    let json = br#"{"foo": 42, "nested": {"foo": 42, "bar": "baz"}, "bar": "baz"}"#;
    let json_len = json.len() as u64;
    let cursor = Cursor::new(json);
    let rc_cursor = Rc::new(RefCell::new(cursor));
    let mut borrow = rc_cursor.borrow_mut();
    let mut de = sciformats_serde_json::Deserializer::from_reader(&mut *borrow);
    let value = NestedStructParent::deserialize(&mut de).unwrap();

    assert_eq!(
        NestedStructParent {
            foo: 42,
            nested: Span { span: 22..47 },
            bar: "baz".to_string(),
        },
        value
    );
    assert_eq!(json_len, de.byte_offset());

    drop(de);
    drop(borrow);

    let mut borrow = rc_cursor.borrow_mut();
    borrow
        .seek(std::io::SeekFrom::Start(value.nested.span.start))
        .unwrap();
    let nested_span = (&mut *borrow).take(value.nested.span.end - value.nested.span.start);
    let mut nested_de = sciformats_serde_json::Deserializer::from_reader(nested_span);
    let nested = NestedStruct::deserialize(&mut nested_de).unwrap();

    assert_eq!(
        NestedStruct {
            foo: 42,
            bar: "baz".to_string(),
        },
        nested
    );
}

#[test]
fn deserialize_nested_span_seq() {
    let json = br#"{"foo": 42, "nested": [{"foo": 42, "bar": "baz"}, {"foo": 43, "bar": "bazz"}], "bar": "baz"}"#;
    let json_len = json.len() as u64;
    let cursor = Cursor::new(json);
    let rc_cursor = Rc::new(RefCell::new(cursor));
    let mut borrow = rc_cursor.borrow_mut();
    let mut de = sciformats_serde_json::Deserializer::from_reader(&mut *borrow);
    let value = NestedSeqParent::deserialize(&mut de).unwrap();

    assert_eq!(
        NestedSeqParent {
            foo: 42,
            nested: Span { span: 22..77 },
            bar: "baz".to_string(),
        },
        value
    );
    assert_eq!(json_len, de.byte_offset());

    drop(de);
    drop(borrow);

    let mut borrow = rc_cursor.borrow_mut();
    borrow
        .seek(std::io::SeekFrom::Start(value.nested.span.start))
        .unwrap();
    let nested_span = (&mut *borrow).take(value.nested.span.end - value.nested.span.start);
    let mut nested_de = sciformats_serde_json::Deserializer::from_reader(nested_span);
    let nested = Vec::<NestedStruct>::deserialize(&mut nested_de).unwrap();

    assert_eq!(
        vec![
            NestedStruct {
                foo: 42,
                bar: "baz".to_string(),
            },
            NestedStruct {
                foo: 43,
                bar: "bazz".to_string(),
            }
        ],
        nested
    );
}
