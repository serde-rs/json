// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(not(feature = "preserve_order"))]

extern crate serde;

#[macro_use]
extern crate serde_json;

use serde_json::{Deserializer, Value};

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
    }
}

#[test]
fn test_json_stream_newlines() {
    let data = "{\"x\":39} {\"x\":40}{\"x\":41}\n{\"x\":42}";

    test_stream!(
        data, Value, |stream| {
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
        }
    );
}

#[test]
fn test_json_stream_trailing_whitespaces() {
    let data = "{\"x\":42} \t\n";

    test_stream!(
        data, Value, |stream| {
            assert_eq!(stream.next().unwrap().unwrap()["x"], 42);
            assert_eq!(stream.byte_offset(), 8);

            assert!(stream.next().is_none());
            assert_eq!(stream.byte_offset(), 11);
        }
    );
}

#[test]
fn test_json_stream_truncated() {
    let data = "{\"x\":40}\n{\"x\":";

    test_stream!(
        data, Value, |stream| {
            assert_eq!(stream.next().unwrap().unwrap()["x"], 40);
            assert_eq!(stream.byte_offset(), 8);

            assert!(stream.next().unwrap().unwrap_err().is_eof());
            assert_eq!(stream.byte_offset(), 9);
        }
    );
}

#[test]
fn test_json_stream_empty() {
    let data = "";

    test_stream!(
        data, Value, |stream| {
            assert!(stream.next().is_none());
            assert_eq!(stream.byte_offset(), 0);
        }
    );
}

#[test]
fn test_json_stream_primitive() {
    let data = "{} true";

    test_stream!(
        data, Value, |stream| {
            assert_eq!(stream.next().unwrap().unwrap(), json!({}));
            assert_eq!(stream.byte_offset(), 2);

            let second = stream.next().unwrap().unwrap_err();
            assert_eq!(second.to_string(), "expected `{` or `[` at line 1 column 4");
        }
    );
}
