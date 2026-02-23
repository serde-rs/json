#![cfg(feature = "partial_parsing")]

use serde::{Deserialize, Serialize};
use serde_json::{
    from_reader, from_slice, from_str, from_value, json, to_string, to_string_pretty, to_value,
    to_vec, Deserializer, Number, Value,
};
use std::io::Cursor;

#[test]
fn test_partial_json_object() {
    let json = r#"{"foo": ["bar", "baz"], "test": "val""#;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    deserializer.allow_partial_object();
    let value = Value::deserialize(&mut deserializer).unwrap();
    assert_eq!(
        value,
        json!({
            "foo": ["bar", "baz"],
            "test": "val"
        })
    )
}

#[test]
fn test_partial_json_list() {
    let json = r#"{"foo": ["bar", "baz""#;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    deserializer.allow_partial_object();
    deserializer.allow_partial_list();
    let value = Value::deserialize(&mut deserializer).unwrap();
    assert_eq!(
        value,
        json!({
            "foo": ["bar", "baz"],
        })
    )
}

#[test]
fn test_partial_json_string() {
    let json = r#"{"test": "val"#;
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    deserializer.allow_partial_object();
    deserializer.allow_partial_string();
    let value = Value::deserialize(&mut deserializer).unwrap();
    assert_eq!(
        value,
        json!({
            "test": "val"
        })
    )
}

#[test]
fn test_partial_json_reader() {
    let cursor = Cursor::new(r#"{"foo": ["bar", "baz"], "test": "val"#);
    let mut deserializer = serde_json::Deserializer::from_reader(cursor);
    deserializer.allow_partial_object();
    deserializer.allow_partial_list();
    deserializer.allow_partial_string();
    let value = Value::deserialize(&mut deserializer).unwrap();
    assert_eq!(
        value,
        json!({
            "foo": ["bar", "baz"],
            "test": "val"
        })
    )
}
