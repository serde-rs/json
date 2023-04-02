#![cfg(not(feature = "preserve_order"))]
#![allow(clippy::assertions_on_result_states)]

use serde_json::{json, Deserializer, Value};

// Rustfmt issue https://github.com/rust-lang-nursery/rustfmt/issues/2740
#[rustfmt::skip]
macro_rules! test_stream {
    ($data:expr, $ty:ty, |$stream:ident| $test:block) => {
        {
            let de = Deserializer::from_str($data);
            let mut $stream = de.into_iter::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
        {
            let de = Deserializer::from_slice($data.as_bytes());
            let mut $stream = de.into_iter::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
        {
            let mut bytes = $data.as_bytes();
            let de = Deserializer::from_reader(&mut bytes);
            let mut $stream = de.into_iter::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
    };
}

#[test]
fn test_json_stream_newlines() {
    let data = "{\"x\":39} {\"x\":40}{\"x\":41}\n{\"x\":42}";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 39);
        assert_eq!(stream.byte_offset(), 8);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 40);
        assert_eq!(stream.byte_offset(), 17);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 41);
        assert_eq!(stream.byte_offset(), 25);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 42);
        assert_eq!(stream.byte_offset(), 34);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), 34);
    });
}

#[test]
fn test_json_stream_trailing_whitespaces() {
    let data = "{\"x\":42} \t\n";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 42);
        assert_eq!(stream.byte_offset(), 8);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), 11);
    });
}

#[test]
fn test_json_stream_truncated() {
    let data = "{\"x\":40}\n{\"x\":";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 40);
        assert_eq!(stream.byte_offset(), 8);

        assert!(stream.next().unwrap().unwrap_err().is_eof());
        assert_eq!(stream.byte_offset(), 9);
    });
}

#[test]
fn test_json_stream_truncated_decimal() {
    let data = "{\"x\":4.";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().unwrap().unwrap_err().is_eof());
        assert_eq!(stream.byte_offset(), 0);
    });
}

#[test]
fn test_json_stream_truncated_negative() {
    let data = "{\"x\":-";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().unwrap().unwrap_err().is_eof());
        assert_eq!(stream.byte_offset(), 0);
    });
}

#[test]
fn test_json_stream_truncated_exponent() {
    let data = "{\"x\":4e";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().unwrap().unwrap_err().is_eof());
        assert_eq!(stream.byte_offset(), 0);
    });
}

#[test]
fn test_json_stream_empty() {
    let data = "";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), 0);
    });
}

#[test]
fn test_json_stream_primitive() {
    let data = "{} true{}1[]\nfalse\"hey\"2 ";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap(), json!({}));
        assert_eq!(stream.byte_offset(), 2);

        assert_eq!(stream.next().unwrap().unwrap(), true);
        assert_eq!(stream.byte_offset(), 7);

        assert_eq!(stream.next().unwrap().unwrap(), json!({}));
        assert_eq!(stream.byte_offset(), 9);

        assert_eq!(stream.next().unwrap().unwrap(), 1);
        assert_eq!(stream.byte_offset(), 10);

        assert_eq!(stream.next().unwrap().unwrap(), json!([]));
        assert_eq!(stream.byte_offset(), 12);

        assert_eq!(stream.next().unwrap().unwrap(), false);
        assert_eq!(stream.byte_offset(), 18);

        assert_eq!(stream.next().unwrap().unwrap(), "hey");
        assert_eq!(stream.byte_offset(), 23);

        assert_eq!(stream.next().unwrap().unwrap(), 2);
        assert_eq!(stream.byte_offset(), 24);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), 25);
    });
}

#[test]
fn test_json_stream_invalid_literal() {
    let data = "truefalse";

    test_stream!(data, Value, |stream| {
        let second = stream.next().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing characters at line 1 column 5");
    });
}

#[test]
fn test_json_stream_invalid_number() {
    let data = "1true";

    test_stream!(data, Value, |stream| {
        let second = stream.next().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing characters at line 1 column 2");
    });
}

#[test]
fn test_json_stream_expected_value() {
    let data = r#"{
        "a": 10,
        "b": 20,
        "c": ,
    }"#;

    test_stream!(data, Value, |stream| {
        let second = stream.next().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "expected value at line 4 column 14");
    });
}

#[test]
fn test_json_stream_late_error_position() {
    // Test that the position of an error is computed correctly when it's later in the stream.
    // This is specifically intended to test the current implementation of the stream-based reader,
    // which uses a 128 byte buffer. We want the error to occur after the first 128 bytes.
    let data = r#"{
        "a": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "b": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "c": ,
    }"#;

    test_stream!(data, Value, |stream| {
        let second = stream.next().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "expected value at line 4 column 14");
    });
}

#[test]
fn test_json_stream_long_string() {
    // This is specifically intended to test the current implementation of the stream-based reader,
    // which uses a 128 byte buffer. We want to test parsing long strings with escapes at various
    // points before and after the buffer length.
    let data = "{\"x\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"y\":\"b\"}";

    test_stream!(data, Value, |stream| {
        let obj = stream.next().unwrap().unwrap();
        assert_eq!(
            obj["x"],
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(obj["y"], "b");
        assert!(stream.next().is_none());
    });

    let data = "{\"x\":\"\\raaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"y\":\"b\"}";

    test_stream!(data, Value, |stream| {
        let obj = stream.next().unwrap().unwrap();
        assert_eq!(
            obj["x"],
            "\raaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(obj["y"], "b");
        assert!(stream.next().is_none());
    });

    let data = "{\"x\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\raaaaaa\
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"y\":\"b\"}";

    test_stream!(data, Value, |stream| {
        let obj = stream.next().unwrap().unwrap();
        assert_eq!(
            obj["x"],
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\raaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(obj["y"], "b");
        assert!(stream.next().is_none());
    });

    let data = "{\"x\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\raaaaaa\",\"y\":\"b\"}";

    test_stream!(data, Value, |stream| {
        let obj = stream.next().unwrap().unwrap();
        assert_eq!(
            obj["x"],
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\raaaaaa"
        );
        assert_eq!(obj["y"], "b");
        assert!(stream.next().is_none());
    });

    let data = "{\"x\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\r\",\"y\":\"b\"}";

    test_stream!(data, Value, |stream| {
        let obj = stream.next().unwrap().unwrap();
        assert_eq!(
            obj["x"],
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r"
        );
        assert_eq!(obj["y"], "b");
        assert!(stream.next().is_none());
    });
}

#[test]
fn test_error() {
    let data = "true wrong false";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap(), true);
        assert!(stream.next().unwrap().is_err());
        assert!(stream.next().is_none());
    });
}
