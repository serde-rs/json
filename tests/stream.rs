#![cfg(not(feature = "preserve_order"))]

use serde_json::{de::SliceRead, json, Deserializer, StreamDeserializer, Value};

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
fn test_error() {
    let data = "true wrong false";

    test_stream!(data, Value, |stream| {
        assert_eq!(stream.next().unwrap().unwrap(), true);
        assert!(stream.next().unwrap().is_err());
        assert!(stream.next().is_none());
    });
}

use serde::de::{Deserialize, DeserializeSeed, SeqAccess, Visitor};

// A DeserializeSeed implementation that uses stateful deserialization to
// append array elements onto the end of an existing vector. The preexisting
// state ("seed") in this case is the Vec<T>. The `deserialize` method of
// `ExtendVec` will be traversing the inner arrays of the JSON input and
// appending each integer into the existing Vec.
struct ExtendVec<'a, T: 'a>(&'a mut Vec<T>);

impl<'de, 'a, T> DeserializeSeed<'de> for &mut ExtendVec<'a, T>
where
    T: Deserialize<'de>,
{
    // The return type of the `deserialize` method. This implementation
    // appends onto an existing vector but does not create any new data
    // structure, so the return type is ().
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // Visitor implementation that will walk an inner array of the JSON
        // input.
        struct ExtendVecVisitor<'a, T: 'a>(&'a mut Vec<T>);

        impl<'de, 'a, T> Visitor<'de> for ExtendVecVisitor<'a, T>
        where
            T: Deserialize<'de>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an array of integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Visit each element in the inner array and push it onto
                // the existing vector.
                while let Some(elem) = seq.next_element()? {
                    self.0.push(elem);
                }
                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendVecVisitor(self.0))
    }
}

#[test]
fn test_json_stream_with_seed() {
    let data = "[0] [1,2] [3] []";

    let reader = SliceRead::new(data.as_bytes());

    let mut vec: Vec<usize> = Vec::with_capacity(4);

    {
        let seed = ExtendVec(&mut vec);
        let mut stream = StreamDeserializer::new_with_seed(reader, seed);
        assert_eq!(stream.byte_offset(), 0);

        assert_eq!(stream.next().unwrap().unwrap(), ());
        assert_eq!(stream.byte_offset(), 3);

        assert_eq!(stream.next().unwrap().unwrap(), ());
        assert_eq!(stream.byte_offset(), 9);

        assert_eq!(stream.next().unwrap().unwrap(), ());
        assert_eq!(stream.byte_offset(), 13);

        assert_eq!(stream.next().unwrap().unwrap(), ());
        assert_eq!(stream.byte_offset(), 16);
    }

    assert_eq!(vec, &[0, 1, 2, 3]);
}
