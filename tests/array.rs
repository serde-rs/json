#![cfg(not(feature = "preserve_order"))]

extern crate serde;

#[macro_use]
extern crate serde_json;

use serde_json::{Deserializer, Value};

// Rustfmt issue https://github.com/rust-lang-nursery/rustfmt/issues/2740
#[cfg_attr(rustfmt, rustfmt_skip)]
macro_rules! test_stream {
    ($data:expr, |$stream:ident| $test:block) => {
        {
            let de = Deserializer::from_str($data);
            let mut $stream = de.into_array();
            $test
        }
        {
            let de = Deserializer::from_slice($data.as_bytes());
            let mut $stream = de.into_array();
            $test
        }
        {
            let mut bytes = $data.as_bytes();
            let de = Deserializer::from_reader(&mut bytes);
            let mut $stream = de.into_array();
            $test
        }
    };
}

#[test]
fn test_json_array_empty() {
    let data = "[]";

    test_stream!(data, |stream| {
        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_whitespace() {
    let data = "\r [\n{\"x\":42}\t, {\"y\":43}\n] \t\n";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap()["x"], 42);

        assert_eq!(stream.next::<Value>().unwrap().unwrap()["y"], 43);

        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_truncated() {
    let data = "[{\"x\":40},{\"x\":";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap()["x"], 40);

        assert!(stream.next::<Value>().unwrap().unwrap_err().is_eof());
    });
}

#[test]
fn test_json_array_primitive() {
    let data = "[{}, true, 1, [], 1.0, \"hey\", null]";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap(), json!({}));

        assert_eq!(stream.next::<bool>().unwrap().unwrap(), true);

        assert_eq!(stream.next::<u32>().unwrap().unwrap(), 1);

        assert_eq!(stream.next::<Value>().unwrap().unwrap(), json!([]));

        assert_eq!(stream.next::<f32>().unwrap().unwrap(), 1.0);

        assert_eq!(stream.next::<String>().unwrap().unwrap(), "hey");

        assert_eq!(stream.next::<Value>().unwrap().unwrap(), Value::Null);

        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_tailing_data() {
    let data = "[]e";

    test_stream!(data, |stream| {
        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing characters at line 1 column 3");
    });
}

#[test]
fn test_json_array_tailing_comma() {
    let data = "[true,]";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap(), true);

        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing comma at line 1 column 7");
    });
}

#[test]
fn test_json_array_eof() {
    let data = "";

    test_stream!(data, |stream| {
        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "EOF while parsing a value at line 1 column 0");
    });
}
