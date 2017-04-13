#![cfg(not(feature = "preserve_order"))]

extern crate serde;
extern crate serde_json;

use serde_json::{Deserializer, Map, Value};

#[test]
fn test_json_stream_newlines() {
    let data = "{\"x\":39} {\"x\":40}{\"x\":41}\n{\"x\":42}";
    let mut parsed = Deserializer::from_str(data).into_iter::<Value>();

    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 39);
    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 40);
    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 41);
    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 42);
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_trailing_whitespaces() {
    let data = "{\"x\":42} \t\n";
    let mut parsed = Deserializer::from_str(data).into_iter::<Value>();

    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 42);
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_truncated() {
    let data = "{\"x\":40}\n{\"x\":";
    let mut parsed = Deserializer::from_str(data).into_iter::<Value>();

    assert_eq!(parsed.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 40);
    assert!(parsed.next().unwrap().is_err());
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_empty() {
    let data = "";
    let mut parsed = Deserializer::from_str(data).into_iter::<Value>();

    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_primitive() {
    let data = "{} true";
    let mut parsed = Deserializer::from_str(data).into_iter::<Value>();

    let first = parsed.next().unwrap().unwrap();
    assert_eq!(first, Value::Object(Map::new()));

    let second = parsed.next().unwrap().unwrap_err();
    assert_eq!(second.to_string(), "expected `{` or `[` at line 1 column 4");
}
