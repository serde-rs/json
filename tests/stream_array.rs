#![cfg(not(feature = "preserve_order"))]

use serde_json::{json, Deserializer, Value};

// Rustfmt issue https://github.com/rust-lang-nursery/rustfmt/issues/2740
#[rustfmt::skip]
macro_rules! test_stream {
    ($data:expr, $ty:ty, |$stream:ident| $test:block) => {
        {
            let de = Deserializer::from_str($data);
            let mut $stream = de.into_array::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
        {
            let de = Deserializer::from_slice($data.as_bytes());
            let mut $stream = de.into_array::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
        {
            let mut bytes = $data.as_bytes();
            let de = Deserializer::from_reader(&mut bytes);
            let mut $stream = de.into_array::<$ty>();
            assert_eq!($stream.byte_offset(), 0);
            $test
        }
    };
}

#[test]
fn test_json_stream_array_newlines() {
    let data = "[{\"x\":39}, {\"x\":40},{\"x\":41}\n,{\"x\":42}]";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 39);
        assert_eq!(stream.byte_offset(), 9);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 40);
        assert_eq!(stream.byte_offset(), 19);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 41);
        assert_eq!(stream.byte_offset(), 28);

        assert_eq!(stream.next().unwrap().unwrap()["x"], 42);
        assert_eq!(stream.byte_offset(), 38);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), data.len());
    });
}

#[test]
fn test_json_stream_array_trailing_whitespaces() {
    let data = "[{\"x\":42}] \t\n";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 42);
        assert_eq!(stream.byte_offset(), 9);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), data.len());
    });
}

#[test]
fn test_json_stream_array_truncated() {
    let data = "[{\"x\":40}\n,{\"x\":";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap()["x"], 40);
        assert_eq!(stream.byte_offset(), 9);

        assert!(stream.next().unwrap().unwrap_err().is_eof());
        assert_eq!(stream.byte_offset(), 11);
    });
}

#[test]
fn test_json_stream_array_empty() {
    let data = "  [    \t \n  ]  ";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), data.len());
    });
}

#[test]
fn test_json_stream_array_empty_comma() {
    let data = " [  , ] ";

    test_stream!(data, Value, |stream| {
        assert!(stream.next().unwrap().is_err());
        assert_eq!(stream.byte_offset(), 4);
    });
}

#[test]
fn test_json_stream_array_primitive() {
    let data = "[{}, true ,{},1 , [],\nfalse,\"hey\"  ,2 \n ] \t ";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap(), json!({}));
        assert_eq!(stream.byte_offset(), 3);

        assert_eq!(stream.next().unwrap().unwrap(), true);
        assert_eq!(stream.byte_offset(), 9);

        assert_eq!(stream.next().unwrap().unwrap(), json!({}));
        assert_eq!(stream.byte_offset(), 13);

        assert_eq!(stream.next().unwrap().unwrap(), 1);
        assert_eq!(stream.byte_offset(), 15);

        assert_eq!(stream.next().unwrap().unwrap(), json!([]));
        assert_eq!(stream.byte_offset(), 20);

        assert_eq!(stream.next().unwrap().unwrap(), false);
        assert_eq!(stream.byte_offset(), 27);

        assert_eq!(stream.next().unwrap().unwrap(), "hey");
        assert_eq!(stream.byte_offset(), 33);

        assert_eq!(stream.next().unwrap().unwrap(), 2);
        assert_eq!(stream.byte_offset(), 37);

        assert!(stream.next().is_none());
        assert_eq!(stream.byte_offset(), data.len());
    });
}

#[test]
fn test_error() {
    let data = "[true, wrong, false]";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap(), true);
        assert!(stream.next().unwrap().is_err());
        assert!(stream.next().is_none());
    });
}
