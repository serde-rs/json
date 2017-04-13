#![cfg(not(feature = "preserve_order"))]

extern crate serde;
extern crate serde_json;

use serde_json::{Deserializer, Map, Value};

macro_rules! test_stream {
    ($data:expr, $ty:ty, |$stream:ident| $test:block) => {
        {
            let de = Deserializer::from_str($data);
            let mut $stream = de.into_iter::<$ty>();
            $test
        }
        {
            let de = Deserializer::from_slice($data.as_bytes());
            let mut $stream = de.into_iter::<$ty>();
            $test
        }
        {
            let de = Deserializer::from_iter($data.bytes().map(Ok));
            let mut $stream = de.into_iter::<$ty>();
            $test
        }
        {
            let mut bytes = $data.as_bytes();
            let de = Deserializer::from_reader(&mut bytes);
            let mut $stream = de.into_iter::<$ty>();
            $test
        }
    }
}

#[test]
fn test_json_stream_newlines() {
    let data = "{\"x\":39} {\"x\":40}{\"x\":41}\n{\"x\":42}";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 39);
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 40);
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 41);
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 42);
        assert!(stream.next().is_none());
    });
}

#[test]
fn test_json_stream_trailing_whitespaces() {
    let data = "{\"x\":42} \t\n";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 42);
        assert!(stream.next().is_none());
    });
}

#[test]
fn test_json_stream_truncated() {
    let data = "{\"x\":40}\n{\"x\":";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().ok().unwrap().pointer("/x").unwrap(), 40);
        assert!(stream.next().unwrap().is_err());
        assert!(stream.next().is_none());
    });
}

#[test]
fn test_json_stream_empty() {
    let data = "";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().is_none());
    });
}

#[test]
fn test_json_stream_primitive() {
    let data = "{} true";

    test_stream!(data, Value, |stream| {
        let first = stream.next().unwrap().unwrap();
        assert_eq!(first, Value::Object(Map::new()));

        let second = stream.next().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "expected `{` or `[` at line 1 column 4");
    });
}
