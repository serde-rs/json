// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(not(feature = "preserve_order"))]

#![cfg_attr(feature = "cargo-clippy", allow(float_cmp))]

#![cfg_attr(feature = "trace-macros", feature(trace_macros))]
#[cfg(feature = "trace-macros")]
trace_macros!(true);

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_json;

#[macro_use]
mod macros;

use std::collections::BTreeMap;
use std::{f32, f64};
use std::fmt::{self, Debug};
use std::{i8, i16, i32, i64};
use std::io;
use std::iter;
use std::marker::PhantomData;
use std::{u8, u16, u32, u64};

use serde::de::{self, Deserialize, IgnoredAny};
use serde::ser::{self, Serialize, Serializer};

use serde_bytes::{ByteBuf, Bytes};

use serde_json::{Deserializer, Value, from_reader, from_slice, from_str, from_value,
                 to_string, to_string_pretty, to_value, to_vec, to_writer};

macro_rules! treemap {
    () => {
        BTreeMap::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = BTreeMap::new();
            $(
                m.insert($k, $v);
            )+
            m
        }
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
    AntHive(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    a: (),
    b: usize,
    c: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Vec<Inner>,
}

fn test_encode_ok<T>(errors: &[(T, &str)])
where
    T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = to_string(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value).unwrap();
        let s = to_string(&v).unwrap();
        assert_eq!(s, out);
    }
}

fn test_pretty_encode_ok<T>(errors: &[(T, &str)])
where
    T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = to_string_pretty(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value).unwrap();
        let s = to_string_pretty(&v).unwrap();
        assert_eq!(s, out);
    }
}

#[test]
fn test_write_null() {
    let tests = &[((), "null")];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_u64() {
    let tests = &[(3u64, "3"), (u64::MAX, &u64::MAX.to_string())];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_i64() {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
        (i64::MIN, &i64::MIN.to_string()),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_f64() {
    let tests = &[
        (3.0, "3.0"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
        (f64::MIN, "-1.7976931348623157e308"),
        (f64::MAX, "1.7976931348623157e308"),
        (f64::EPSILON, "2.220446049250313e-16"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_encode_nonfinite_float_yields_null() {
    let v = to_value(::std::f64::NAN).unwrap();
    assert!(v.is_null());

    let v = to_value(::std::f64::INFINITY).unwrap();
    assert!(v.is_null());

    let v = to_value(::std::f32::NAN).unwrap();
    assert!(v.is_null());

    let v = to_value(::std::f32::INFINITY).unwrap();
    assert!(v.is_null());
}

#[test]
fn test_write_str() {
    let tests = &[("", "\"\""), ("foo", "\"foo\"")];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[(true, "true"), (false, "false")];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_char() {
    let tests = &[
        ('n', "\"n\""),
        ('"', "\"\\\"\""),
        ('\\', "\"\\\\\""),
        ('/', "\"/\""),
        ('\x08', "\"\\b\""),
        ('\x0C', "\"\\f\""),
        ('\n', "\"\\n\""),
        ('\r', "\"\\r\""),
        ('\t', "\"\\t\""),
        ('\x0B', "\"\\u000b\""),
        ('\u{3A3}', "\"\u{3A3}\""),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_list() {
    test_encode_ok(
        &[
            (vec![], "[]"),
            (vec![true], "[true]"),
            (vec![true, false], "[true,false]"),
        ],
    );

    test_encode_ok(
        &[
            (vec![vec![], vec![], vec![]], "[[],[],[]]"),
            (vec![vec![1, 2, 3], vec![], vec![]], "[[1,2,3],[],[]]"),
            (vec![vec![], vec![1, 2, 3], vec![]], "[[],[1,2,3],[]]"),
            (vec![vec![], vec![], vec![1, 2, 3]], "[[],[],[1,2,3]]"),
        ],
    );

    test_pretty_encode_ok(
        &[
            (vec![vec![], vec![], vec![]], pretty_str!([[], [], []])),
            (vec![vec![1, 2, 3], vec![], vec![]], pretty_str!([[1, 2, 3], [], []])),
            (vec![vec![], vec![1, 2, 3], vec![]], pretty_str!([[], [1, 2, 3], []])),
            (vec![vec![], vec![], vec![1, 2, 3]], pretty_str!([[], [], [1, 2, 3]])),
        ],
    );

    test_pretty_encode_ok(
        &[
            (vec![], "[]"),
            (vec![true], pretty_str!([true])),
            (vec![true, false], pretty_str!([true, false])),
        ],
    );

    let long_test_list = json!([false, null, ["foo\nbar", 3.5]]);

    test_encode_ok(
        &[
            (long_test_list.clone(), json_str!([false, null, ["foo\nbar", 3.5]])),
        ],
    );

    test_pretty_encode_ok(
        &[
            (long_test_list, pretty_str!([false, null, ["foo\nbar", 3.5]])),
        ],
    );
}

#[test]
fn test_write_object() {
    test_encode_ok(
        &[
            (treemap!(), "{}"),
            (treemap!("a".to_string() => true), "{\"a\":true}"),
            (treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
             "{\"a\":true,\"b\":false}"),
        ],
    );

    test_encode_ok(
        &[
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
             "{\"a\":{},\"b\":{},\"c\":{}}"),
            (treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
             "{\"a\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"b\":{},\"c\":{}}"),
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
             "{\"a\":{},\"b\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"c\":{}}"),
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
             "{\"a\":{},\"b\":{},\"c\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}}}"),
        ],
    );

    test_pretty_encode_ok(
        &[
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
             pretty_str!({
                "a": {},
                "b": {},
                "c": {}
            })),
            (treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
             pretty_str!({
                "a": {
                    "a": {
                        "a": [
                            1,
                            2,
                            3
                        ]
                    },
                    "b": {},
                    "c": {}
                },
                "b": {},
                "c": {}
            })),
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
             pretty_str!({
                "a": {},
                "b": {
                    "a": {
                        "a": [
                            1,
                            2,
                            3
                        ]
                    },
                    "b": {},
                    "c": {}
                },
                "c": {}
            })),
            (treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
             pretty_str!({
                "a": {},
                "b": {},
                "c": {
                    "a": {
                        "a": [
                            1,
                            2,
                            3
                        ]
                    },
                    "b": {},
                    "c": {}
                }
            })),
        ],
    );

    test_pretty_encode_ok(
        &[
            (treemap!(), "{}"),
            (treemap!("a".to_string() => true),
             pretty_str!({
                "a": true
            })),
            (treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
             pretty_str!( {
                "a": true,
                "b": false
            })),
        ],
    );

    let complex_obj = json!({
        "b": [
            {"c": "\x0c\x1f\r"},
            {"d": ""}
        ]
    });

    test_encode_ok(
        &[
            (complex_obj.clone(),
             json_str!({
                "b": [
                    {
                        "c": (r#""\f\u001f\r""#)
                    },
                    {
                        "d": ""
                    }
                ]
            })),
        ],
    );

    test_pretty_encode_ok(
        &[
            (complex_obj.clone(),
             pretty_str!({
                "b": [
                    {
                        "c": (r#""\f\u001f\r""#)
                    },
                    {
                        "d": ""
                    }
                ]
            })),
        ],
    );
}

#[test]
fn test_write_tuple() {
    test_encode_ok(&[((5,), "[5]")]);

    test_pretty_encode_ok(
        &[
            ((5,),
             pretty_str!([
                5
            ])),
        ],
    );

    test_encode_ok(&[((5, (6, "abc")), "[5,[6,\"abc\"]]")]);

    test_pretty_encode_ok(
        &[
            ((5, (6, "abc")),
             pretty_str!([
                5,
                [
                    6,
                    "abc"
                ]
            ])),
        ],
    );
}

#[test]
fn test_write_enum() {
    test_encode_ok(
        &[
            (Animal::Dog, "\"Dog\""),
            (Animal::Frog("Henry".to_string(), vec![]), "{\"Frog\":[\"Henry\",[]]}"),
            (Animal::Frog("Henry".to_string(), vec![349]), "{\"Frog\":[\"Henry\",[349]]}"),
            (Animal::Frog("Henry".to_string(), vec![349, 102]), "{\"Frog\":[\"Henry\",[349,102]]}"),
            (Animal::Cat {
                 age: 5,
                 name: "Kate".to_string(),
             },
             "{\"Cat\":{\"age\":5,\"name\":\"Kate\"}}"),
            (Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
             "{\"AntHive\":[\"Bob\",\"Stuart\"]}"),
        ],
    );

    test_pretty_encode_ok(
        &[
            (Animal::Dog, "\"Dog\""),
            (Animal::Frog("Henry".to_string(), vec![]),
             pretty_str!({
                "Frog": [
                    "Henry",
                    []
                ]
            })),
            (Animal::Frog("Henry".to_string(), vec![349]),
             pretty_str!({
                "Frog": [
                    "Henry",
                    [
                        349
                    ]
                ]
            })),
            (Animal::Frog("Henry".to_string(), vec![349, 102]),
             pretty_str!({
                "Frog": [
                    "Henry",
                    [
                      349,
                      102
                    ]
                ]
            })),
        ],
    );
}

#[test]
fn test_write_option() {
    test_encode_ok(&[(None, "null"), (Some("jodhpurs"), "\"jodhpurs\"")]);

    test_encode_ok(
        &[
            (None, "null"),
            (Some(vec!["foo", "bar"]), "[\"foo\",\"bar\"]"),
        ],
    );

    test_pretty_encode_ok(&[(None, "null"), (Some("jodhpurs"), "\"jodhpurs\"")]);

    test_pretty_encode_ok(
        &[
            (None, "null"),
            (Some(vec!["foo", "bar"]),
             pretty_str!([
                "foo",
                "bar"
            ])),
        ],
    );
}

#[test]
fn test_write_newtype_struct() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Newtype(BTreeMap<String, i32>);

    let inner = Newtype(treemap!(String::from("inner") => 123));
    let outer = treemap!(String::from("outer") => to_value(&inner).unwrap());

    test_encode_ok(&[(inner, r#"{"inner":123}"#)]);

    test_encode_ok(&[(outer, r#"{"outer":{"inner":123}}"#)]);
}

fn test_parse_ok<T>(tests: Vec<(&str, T)>)
where
    T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned,
{
    for (s, value) in tests {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_slice(s.as_bytes()).unwrap();
        assert_eq!(v, value.clone());

        // Make sure we can deserialize into a `Value`.
        let json_value: Value = from_str(s).unwrap();
        assert_eq!(json_value, to_value(&value).unwrap());

        // Make sure we can deserialize from a `&Value`.
        let v = T::deserialize(&json_value).unwrap();
        assert_eq!(v, value);

        // Make sure we can deserialize from a `Value`.
        let v: T = from_value(json_value.clone()).unwrap();
        assert_eq!(v, value);

        // Make sure we can round trip back to `Value`.
        let json_value2: Value = from_value(json_value.clone()).unwrap();
        assert_eq!(json_value2, json_value);

        // Make sure we can fully ignore.
        let twoline = s.to_owned() + "\n3735928559";
        let mut de = Deserializer::from_str(&twoline);
        IgnoredAny::deserialize(&mut de).unwrap();
        assert_eq!(0xDEAD_BEEF, u64::deserialize(&mut de).unwrap());
    }
}

// For testing representations that the deserializer accepts but the serializer
// never generates. These do not survive a round-trip through Value.
fn test_parse_unusual_ok<T>(tests: Vec<(&str, T)>)
where
    T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned,
{
    for (s, value) in tests {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_slice(s.as_bytes()).unwrap();
        assert_eq!(v, value.clone());
    }
}

macro_rules! test_parse_err {
    ($name:ident::<$($ty:ty),*>($arg:expr) => $expected:expr) => {
        let actual = $name::<$($ty),*>($arg).unwrap_err().to_string();
        assert_eq!(actual, $expected, "unexpected {} error", stringify!($name));
    };
}

fn test_parse_err<T>(errors: &[(&str, &'static str)])
where
    T: Debug + PartialEq + de::DeserializeOwned,
{
    for &(s, err) in errors {
        test_parse_err!(from_str::<T>(s) => err);
        test_parse_err!(from_slice::<T>(s.as_bytes()) => err);
    }
}

fn test_parse_slice_err<T>(errors: &[(&[u8], &'static str)])
where
    T: Debug + PartialEq + de::DeserializeOwned,
{
    for &(s, err) in errors {
        test_parse_err!(from_slice::<T>(s) => err);
    }
}

#[test]
fn test_parse_null() {
    test_parse_err::<()>(
        &[
            ("n", "expected ident at line 1 column 1"),
            ("nul", "expected ident at line 1 column 3"),
            ("nulla", "trailing characters at line 1 column 5"),
        ],
    );

    test_parse_ok(
        vec![
        ("null", ()),
    ],
    );
}

#[test]
fn test_parse_bool() {
    test_parse_err::<bool>(
        &[
            ("t", "expected ident at line 1 column 1"),
            ("truz", "expected ident at line 1 column 4"),
            ("f", "expected ident at line 1 column 1"),
            ("faz", "expected ident at line 1 column 3"),
            ("truea", "trailing characters at line 1 column 5"),
            ("falsea", "trailing characters at line 1 column 6"),
        ],
    );

    test_parse_ok(
        vec![
        ("true", true),
        (" true ", true),
        ("false", false),
        (" false ", false),
    ],
    );
}

#[test]
fn test_parse_char() {
    test_parse_err::<char>(
        &[
            ("\"ab\"", "invalid value: string \"ab\", expected a character at line 1 column 4"),
            ("10", "invalid type: integer `10`, expected a character at line 1 column 2"),
        ],
    );

    test_parse_ok(
        vec![
        ("\"n\"", 'n'),
        ("\"\\\"\"", '"'),
        ("\"\\\\\"", '\\'),
        ("\"/\"", '/'),
        ("\"\\b\"", '\x08'),
        ("\"\\f\"", '\x0C'),
        ("\"\\n\"", '\n'),
        ("\"\\r\"", '\r'),
        ("\"\\t\"", '\t'),
        ("\"\\u000b\"", '\x0B'),
        ("\"\\u000B\"", '\x0B'),
        ("\"\u{3A3}\"", '\u{3A3}'),
    ],
    );
}

#[test]
fn test_parse_number_errors() {
    test_parse_err::<f64>(
        &[
            ("+", "expected value at line 1 column 1"),
            (".", "expected value at line 1 column 1"),
            ("-", "invalid number at line 1 column 1"),
            ("00", "invalid number at line 1 column 2"),
            ("0x80", "trailing characters at line 1 column 2"),
            ("\\0", "expected value at line 1 column 1"),
            ("1.", "invalid number at line 1 column 2"),
            ("1.a", "invalid number at line 1 column 3"),
            ("1.e1", "invalid number at line 1 column 3"),
            ("1e", "invalid number at line 1 column 2"),
            ("1e+", "invalid number at line 1 column 3"),
            ("1a", "trailing characters at line 1 column 2"),
            ("100e777777777777777777777777777", "number out of range at line 1 column 14"),
            ("-100e777777777777777777777777777", "number out of range at line 1 column 15"),
            ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000", // 1e309
             "number out of range at line 1 column 310"),
            ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           .0e9", // 1e309
             "number out of range at line 1 column 305"),
            ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           e9", // 1e309
             "number out of range at line 1 column 303"),
        ],
    );
}

#[test]
fn test_parse_i64() {
    test_parse_ok(
        vec![
        ("-2", -2),
        ("-1234", -1234),
        (" -1234 ", -1234),
        (&i64::MIN.to_string(), i64::MIN),
        (&i64::MAX.to_string(), i64::MAX),
    ],
    );
}

#[test]
fn test_parse_u64() {
    test_parse_ok(
        vec![
        ("0", 0u64),
        ("3", 3u64),
        ("1234", 1234),
        (&u64::MAX.to_string(), u64::MAX),
    ],
    );
}

#[test]
fn test_parse_negative_zero() {
    for negative_zero in &[
        "-0.0",
        "-0e2",
        "-0.0e2",
        "-1e-400",
        "-1e-4000000000000000000000000000000000000000000000000",
    ] {
        assert!(from_str::<f64>(negative_zero).unwrap().is_sign_negative(),
            "should have been negative: {:?}", negative_zero);
    }
}

#[test]
fn test_parse_f64() {
    test_parse_ok(
        vec![
        ("0.0", 0.0f64),
        ("3.0", 3.0f64),
        ("3.00", 3.0f64),
        ("3.1", 3.1),
        ("-1.2", -1.2),
        ("0.4", 0.4),
        ("0.4e5", 0.4e5),
        ("0.4e+5", 0.4e5),
        ("0.4e15", 0.4e15),
        ("0.4e+15", 0.4e15),
        ("0.4e-01", 0.4e-1),
        (" 0.4e-01 ", 0.4e-1),
        ("0.4e-001", 0.4e-1),
        ("0.4e-0", 0.4e0),
        ("0.00e00", 0.0),
        ("0.00e+00", 0.0),
        ("0.00e-00", 0.0),
        (&format!("{:?}", (i64::MIN as f64) - 1.0), (i64::MIN as f64) - 1.0),
        (&format!("{:?}", (u64::MAX as f64) + 1.0), (u64::MAX as f64) + 1.0),
        (&format!("{:?}", f64::EPSILON), f64::EPSILON),
        ("0.0000000000000000000000000000000000000000000000000123e50", 1.23),
        ("100e-777777777777777777777777777", 0.0),
        ("1010101010101010101010101010101010101010", 10101010101010101010e20),
        ("0.1010101010101010101010101010101010101010", 0.1010101010101010101),
        ("0e1000000000000000000000000000000000000000000000", 0.0),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           00000000", 1e308),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           .0e8", 1e308),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           e8", 1e308),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000e-10", 1e308),
    ],
    );
}

#[test]
fn test_parse_string() {
    test_parse_err::<String>(
        &[
            ("\"", "EOF while parsing a string at line 1 column 1"),
            ("\"lol", "EOF while parsing a string at line 1 column 4"),
            ("\"lol\"a", "trailing characters at line 1 column 6"),
            ("\"\\uD83C\\uFFFF\"", "lone leading surrogate in hex escape at line 1 column 13"),
        ],
    );

    test_parse_slice_err::<String>(
        &[
            (&[b'"', 159, 146, 150, b'"'], "invalid unicode code point at line 1 column 5"),
            (&[b'"', b'\\', b'n', 159, 146, 150, b'"'],
             "invalid unicode code point at line 1 column 7"),
        ],
    );

    test_parse_ok(
        vec![
        ("\"\"", "".to_string()),
        ("\"foo\"", "foo".to_string()),
        (" \"foo\" ", "foo".to_string()),
        ("\"\\\"\"", "\"".to_string()),
        ("\"\\b\"", "\x08".to_string()),
        ("\"\\n\"", "\n".to_string()),
        ("\"\\r\"", "\r".to_string()),
        ("\"\\t\"", "\t".to_string()),
        ("\"\\u12ab\"", "\u{12ab}".to_string()),
        ("\"\\uAB12\"", "\u{AB12}".to_string()),
        ("\"\\uD83C\\uDF95\"", "\u{1F395}".to_string()),
    ],
    );
}

#[test]
fn test_parse_list() {
    test_parse_err::<Vec<f64>>(
        &[
            ("[", "EOF while parsing a list at line 1 column 1"),
            ("[ ", "EOF while parsing a list at line 1 column 2"),
            ("[1", "EOF while parsing a list at line 1 column 2"),
            ("[1,", "EOF while parsing a value at line 1 column 3"),
            ("[1,]", "trailing comma at line 1 column 4"),
            ("[1 2]", "expected `,` or `]` at line 1 column 4"),
            ("[]a", "trailing characters at line 1 column 3"),
        ],
    );

    test_parse_ok(
        vec![
        ("[]", vec![]),
        ("[ ]", vec![]),
        ("[null]", vec![()]),
        (" [ null ] ", vec![()]),
    ],
    );

    test_parse_ok(
        vec![
        ("[true]", vec![true]),
    ],
    );

    test_parse_ok(
        vec![
        ("[3,1]", vec![3u64, 1]),
        (" [ 3 , 1 ] ", vec![3, 1]),
    ],
    );

    test_parse_ok(
        vec![
        ("[[3], [1, 2]]", vec![vec![3u64], vec![1, 2]]),
    ],
    );

    test_parse_ok(
        vec![
        ("[1]", (1u64,)),
    ],
    );

    test_parse_ok(
        vec![
        ("[1, 2]", (1u64, 2u64)),
    ],
    );

    test_parse_ok(
        vec![
        ("[1, 2, 3]", (1u64, 2u64, 3u64)),
    ],
    );

    test_parse_ok(
        vec![
        ("[1, [2, 3]]", (1u64, (2u64, 3u64))),
    ],
    );
}

#[test]
fn test_parse_object() {
    test_parse_err::<BTreeMap<String, u32>>(
        &[
            ("{", "EOF while parsing an object at line 1 column 1"),
            ("{ ", "EOF while parsing an object at line 1 column 2"),
            ("{1", "key must be a string at line 1 column 2"),
            ("{ \"a\"", "EOF while parsing an object at line 1 column 5"),
            ("{\"a\"", "EOF while parsing an object at line 1 column 4"),
            ("{\"a\" ", "EOF while parsing an object at line 1 column 5"),
            ("{\"a\" 1", "expected `:` at line 1 column 6"),
            ("{\"a\":", "EOF while parsing a value at line 1 column 5"),
            ("{\"a\":1", "EOF while parsing an object at line 1 column 6"),
            ("{\"a\":1 1", "expected `,` or `}` at line 1 column 8"),
            ("{\"a\":1,", "EOF while parsing a value at line 1 column 7"),
            ("{}a", "trailing characters at line 1 column 3"),
        ],
    );

    test_parse_ok(
        vec![
        ("{}", treemap!()),
        ("{ }", treemap!()),
        (
            "{\"a\":3}",
            treemap!("a".to_string() => 3u64)
        ),
        (
            "{ \"a\" : 3 }",
            treemap!("a".to_string() => 3)
        ),
        (
            "{\"a\":3,\"b\":4}",
            treemap!("a".to_string() => 3, "b".to_string() => 4)
        ),
        (
            " { \"a\" : 3 , \"b\" : 4 } ",
            treemap!("a".to_string() => 3, "b".to_string() => 4),
        ),
    ],
    );

    test_parse_ok(
        vec![
        (
            "{\"a\": {\"b\": 3, \"c\": 4}}",
            treemap!(
                "a".to_string() => treemap!(
                    "b".to_string() => 3u64,
                    "c".to_string() => 4
                )
            ),
        ),
    ],
    );
}

#[test]
fn test_parse_struct() {
    test_parse_err::<Outer>(
        &[
            ("5", "invalid type: integer `5`, expected struct Outer at line 1 column 1"),
            ("\"hello\"", "invalid type: string \"hello\", expected struct Outer at line 1 column 7"),
            ("{\"inner\": true}",
             "invalid type: boolean `true`, expected a sequence at line 1 column 14"),
            ("{}", "missing field `inner` at line 1 column 2"),
            (r#"{"inner": [{"b": 42, "c": []}]}"#, "missing field `a` at line 1 column 29"),
        ],
    );

    test_parse_ok(
        vec![
        (
            "{
                \"inner\": []
            }",
            Outer {
                inner: vec![]
            },
        ),
        (
            "{
                \"inner\": [
                    { \"a\": null, \"b\": 2, \"c\": [\"abc\", \"xyz\"] }
                ]
            }",
            Outer {
                inner: vec![
                    Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
                ]
            },
        ),
    ],
    );

    let v: Outer = from_str(
        "[
            [
                [ null, 2, [\"abc\", \"xyz\"] ]
            ]
        ]",
    )
            .unwrap();

    assert_eq!(
        v,
        Outer {
            inner: vec![
                Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
            ],
        }
    );
}

#[test]
fn test_parse_option() {
    test_parse_ok(
        vec![
        ("null", None::<String>),
        ("\"jodhpurs\"", Some("jodhpurs".to_string())),
    ],
    );

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        x: Option<isize>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    test_parse_ok(
        vec![
        ("{\"x\": null}", Foo { x: None }),
        ("{\"x\": 5}", Foo { x: Some(5) }),
    ],
    );
}

#[test]
fn test_parse_enum_errors() {
    test_parse_err::<Animal>(
        &[
            ("{}", "expected value at line 1 column 2"),
            ("[]", "expected value at line 1 column 1"),
            ("\"unknown\"",
             "unknown variant `unknown`, expected one of `Dog`, `Frog`, `Cat`, `AntHive` at line 1 column 9"),
            ("{\"unknown\":null}",
             "unknown variant `unknown`, expected one of `Dog`, `Frog`, `Cat`, `AntHive` at line 1 column 10"),
            ("{\"Dog\":", "EOF while parsing a value at line 1 column 7"),
            ("{\"Dog\":}", "expected value at line 1 column 8"),
            ("{\"Dog\":{}}", "invalid type: map, expected unit at line 1 column 9"),
            ("\"Frog\"", "invalid type: unit variant, expected tuple variant"),
            ("\"Frog\" 0 ", "invalid type: unit variant, expected tuple variant"),
            ("{\"Frog\":{}}",
             "invalid type: map, expected tuple variant Animal::Frog at line 1 column 10"),
            ("{\"Cat\":[]}", "invalid length 0, expected tuple of 2 elements at line 1 column 9"),
            ("{\"Cat\":[0]}", "invalid length 1, expected tuple of 2 elements at line 1 column 10"),
            ("{\"Cat\":[0, \"\", 2]}", "trailing characters at line 1 column 16"),
            ("{\"Cat\":{\"age\": 5, \"name\": \"Kate\", \"foo\":\"bar\"}",
             "unknown field `foo`, expected `age` or `name` at line 1 column 39"),

            // JSON does not allow trailing commas in data structures
            ("{\"Cat\":[0, \"Kate\",]}", "trailing comma at line 1 column 19"),
            ("{\"Cat\":{\"age\": 2, \"name\": \"Kate\",}}",
             "trailing comma at line 1 column 34"),
        ],
    );
}

#[test]
fn test_parse_enum() {
    test_parse_ok(
        vec![
        ("\"Dog\"", Animal::Dog),
        (" \"Dog\" ", Animal::Dog),
        (
            "{\"Frog\":[\"Henry\",[]]}",
            Animal::Frog("Henry".to_string(), vec![]),
        ),
        (
            " { \"Frog\": [ \"Henry\" , [ 349, 102 ] ] } ",
            Animal::Frog("Henry".to_string(), vec![349, 102]),
        ),
        (
            "{\"Cat\": {\"age\": 5, \"name\": \"Kate\"}}",
            Animal::Cat { age: 5, name: "Kate".to_string() },
        ),
        (
            " { \"Cat\" : { \"age\" : 5 , \"name\" : \"Kate\" } } ",
            Animal::Cat { age: 5, name: "Kate".to_string() },
        ),
        (
            " { \"AntHive\" : [\"Bob\", \"Stuart\"] } ",
            Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
        ),
    ],
    );

    test_parse_unusual_ok(
        vec![
        ("{\"Dog\":null}", Animal::Dog),
        (" { \"Dog\" : null } ", Animal::Dog),
    ],
    );

    test_parse_ok(
        vec![
        (
            concat!(
                "{",
                "  \"a\": \"Dog\",",
                "  \"b\": {\"Frog\":[\"Henry\", []]}",
                "}"
            ),
            treemap!(
                "a".to_string() => Animal::Dog,
                "b".to_string() => Animal::Frog("Henry".to_string(), vec![])
            )
        ),
    ],
    );
}

#[test]
fn test_parse_trailing_whitespace() {
    test_parse_ok(
        vec![
        ("[1, 2] ", vec![1u64, 2]),
        ("[1, 2]\n", vec![1, 2]),
        ("[1, 2]\t", vec![1, 2]),
        ("[1, 2]\t \n", vec![1, 2]),
    ],
    );
}

#[test]
fn test_multiline_errors() {
    test_parse_err::<BTreeMap<String, String>>(
        &[
            ("{\n  \"foo\":\n \"bar\"", "EOF while parsing an object at line 3 column 6"),
        ],
    );
}

#[test]
fn test_missing_option_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: Option<u32>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{\"x\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });

    let value: Foo = from_value(json!({})).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(json!({"x": 5})).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_missing_nonoption_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: u32,
    }

    test_parse_err::<Foo>(&[("{}", "missing field `x` at line 1 column 2")]);
}

#[test]
fn test_missing_renamed_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        #[serde(rename="y")]
        x: Option<u32>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{\"y\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });

    let value: Foo = from_value(json!({})).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(json!({"y": 5})).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_serialize_seq_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyVec<T>(Vec<T>);

    impl<T> ser::Serialize for MyVec<T>
    where
        T: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            use serde::ser::SerializeSeq;
            let mut seq = try!(serializer.serialize_seq(None));
            for elem in &self.0 {
                try!(seq.serialize_element(elem));
            }
            seq.end()
        }
    }

    struct Visitor<T> {
        marker: PhantomData<MyVec<T>>,
    }

    impl<'de, T> de::Visitor<'de> for Visitor<T>
    where
        T: de::Deserialize<'de>,
    {
        type Value = MyVec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<MyVec<T>, E>
        where
            E: de::Error,
        {
            Ok(MyVec(Vec::new()))
        }

        #[inline]
        fn visit_seq<V>(self, mut visitor: V) -> Result<MyVec<T>, V::Error>
        where
            V: de::SeqAccess<'de>,
        {
            let mut values = Vec::new();

            while let Some(value) = try!(visitor.next_element()) {
                values.push(value);
            }

            Ok(MyVec(values))
        }
    }

    impl<'de, T> de::Deserialize<'de> for MyVec<T>
    where
        T: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<MyVec<T>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(Visitor { marker: PhantomData })
        }
    }

    let mut vec = Vec::new();
    vec.push(MyVec(Vec::new()));
    vec.push(MyVec(Vec::new()));
    let vec: MyVec<MyVec<u32>> = MyVec(vec);

    test_encode_ok(&[(vec.clone(), "[[],[]]")]);

    let s = to_string_pretty(&vec).unwrap();
    let expected = pretty_str!([
        [],
        []
    ]);
    assert_eq!(s, expected);
}

#[test]
fn test_serialize_map_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyMap<K, V>(BTreeMap<K, V>);

    impl<K, V> ser::Serialize for MyMap<K, V>
    where
        K: ser::Serialize + Ord,
        V: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            use serde::ser::SerializeMap;
            let mut map = try!(serializer.serialize_map(None));
            for (k, v) in &self.0 {
                try!(map.serialize_key(k));
                try!(map.serialize_value(v));
            }
            map.end()
        }
    }

    struct Visitor<K, V> {
        marker: PhantomData<MyMap<K, V>>,
    }

    impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        type Value = MyMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<MyMap<K, V>, E>
        where
            E: de::Error,
        {
            Ok(MyMap(BTreeMap::new()))
        }

        #[inline]
        fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<MyMap<K, V>, Visitor::Error>
        where
            Visitor: de::MapAccess<'de>,
        {
            let mut values = BTreeMap::new();

            while let Some((key, value)) = try!(visitor.next_entry()) {
                values.insert(key, value);
            }

            Ok(MyMap(values))
        }
    }

    impl<'de, K, V> de::Deserialize<'de> for MyMap<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<MyMap<K, V>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(Visitor { marker: PhantomData })
        }
    }

    let mut map = BTreeMap::new();
    map.insert("a", MyMap(BTreeMap::new()));
    map.insert("b", MyMap(BTreeMap::new()));
    let map: MyMap<_, MyMap<u32, u32>> = MyMap(map);

    test_encode_ok(&[(map.clone(), "{\"a\":{},\"b\":{}}")]);

    let s = to_string_pretty(&map).unwrap();
    let expected = pretty_str!({
        "a": {},
        "b": {}
    });
    assert_eq!(s, expected);
}

#[test]
fn test_deserialize_from_stream() {
    use std::net;
    use std::thread;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Message {
        message: String,
    }

    let l = net::TcpListener::bind("localhost:20000").unwrap();

    thread::spawn(
        || {
            let l = l;
            for stream in l.incoming() {
                let mut stream = stream.unwrap();
                let read_stream = stream.try_clone().unwrap();

                let mut de = Deserializer::from_reader(read_stream);
                let request = Message::deserialize(&mut de).unwrap();
                let response = Message { message: request.message };
                to_writer(&mut stream, &response).unwrap();
            }
        },
    );

    let mut stream = net::TcpStream::connect("localhost:20000").unwrap();
    let request = Message { message: "hi there".to_string() };
    to_writer(&mut stream, &request).unwrap();

    let mut de = Deserializer::from_reader(stream);
    let response = Message::deserialize(&mut de).unwrap();

    assert_eq!(request, response);
}

#[test]
fn test_serialize_rejects_bool_keys() {
    let map = treemap!(
        true => 2,
        false => 4
    );

    let err = to_vec(&map).unwrap_err();
    assert_eq!(err.to_string(), "key must be a string");
}

#[test]
fn test_serialize_rejects_adt_keys() {
    let map = treemap!(
        Some("a") => 2,
        Some("b") => 4,
        None => 6
    );

    let err = to_vec(&map).unwrap_err();
    assert_eq!(err.to_string(), "key must be a string");
}

#[test]
fn test_bytes_ser() {
    let buf = vec![];
    let bytes = Bytes::new(&buf);
    assert_eq!(to_string(&bytes).unwrap(), "[]".to_string());

    let buf = vec![1, 2, 3];
    let bytes = Bytes::new(&buf);
    assert_eq!(to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

#[test]
fn test_byte_buf_ser() {
    let bytes = ByteBuf::new();
    assert_eq!(to_string(&bytes).unwrap(), "[]".to_string());

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    assert_eq!(to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

#[test]
fn test_byte_buf_de() {
    let bytes = ByteBuf::new();
    let v: ByteBuf = from_str("[]").unwrap();
    assert_eq!(v, bytes);

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let v: ByteBuf = from_str("[1, 2, 3]").unwrap();
    assert_eq!(v, bytes);
}

#[test]
fn test_byte_buf_de_multiple() {
    let s: Vec<ByteBuf> = from_str(r#"["ab\nc", "cd\ne"]"#).unwrap();
    let a = ByteBuf::from(b"ab\nc".to_vec());
    let b = ByteBuf::from(b"cd\ne".to_vec());
    assert_eq!(vec![a, b], s);
}

#[test]
fn test_json_pointer() {
    // Test case taken from https://tools.ietf.org/html/rfc6901#page-5
    let data: Value = from_str(
        r#"{
        "foo": ["bar", "baz"],
        "": 0,
        "a/b": 1,
        "c%d": 2,
        "e^f": 3,
        "g|h": 4,
        "i\\j": 5,
        "k\"l": 6,
        " ": 7,
        "m~n": 8
    }"#,
    )
            .unwrap();
    assert_eq!(data.pointer("").unwrap(), &data);
    assert_eq!(data.pointer("/foo").unwrap(), &json!(["bar", "baz"]));
    assert_eq!(data.pointer("/foo/0").unwrap(), &json!("bar"));
    assert_eq!(data.pointer("/").unwrap(), &json!(0));
    assert_eq!(data.pointer("/a~1b").unwrap(), &json!(1));
    assert_eq!(data.pointer("/c%d").unwrap(), &json!(2));
    assert_eq!(data.pointer("/e^f").unwrap(), &json!(3));
    assert_eq!(data.pointer("/g|h").unwrap(), &json!(4));
    assert_eq!(data.pointer("/i\\j").unwrap(), &json!(5));
    assert_eq!(data.pointer("/k\"l").unwrap(), &json!(6));
    assert_eq!(data.pointer("/ ").unwrap(), &json!(7));
    assert_eq!(data.pointer("/m~0n").unwrap(), &json!(8));
    // Invalid pointers
    assert!(data.pointer("/unknown").is_none());
    assert!(data.pointer("/e^f/ertz").is_none());
    assert!(data.pointer("/foo/00").is_none());
    assert!(data.pointer("/foo/01").is_none());
}

#[test]
fn test_json_pointer_mut() {
    use std::mem;

    // Test case taken from https://tools.ietf.org/html/rfc6901#page-5
    let mut data: Value = from_str(
        r#"{
        "foo": ["bar", "baz"],
        "": 0,
        "a/b": 1,
        "c%d": 2,
        "e^f": 3,
        "g|h": 4,
        "i\\j": 5,
        "k\"l": 6,
        " ": 7,
        "m~n": 8
    }"#,
    )
            .unwrap();

    // Basic pointer checks
    assert_eq!(data.pointer_mut("/foo").unwrap(), &json!(["bar", "baz"]));
    assert_eq!(data.pointer_mut("/foo/0").unwrap(), &json!("bar"));
    assert_eq!(data.pointer_mut("/").unwrap(), 0);
    assert_eq!(data.pointer_mut("/a~1b").unwrap(), 1);
    assert_eq!(data.pointer_mut("/c%d").unwrap(), 2);
    assert_eq!(data.pointer_mut("/e^f").unwrap(), 3);
    assert_eq!(data.pointer_mut("/g|h").unwrap(), 4);
    assert_eq!(data.pointer_mut("/i\\j").unwrap(), 5);
    assert_eq!(data.pointer_mut("/k\"l").unwrap(), 6);
    assert_eq!(data.pointer_mut("/ ").unwrap(), 7);
    assert_eq!(data.pointer_mut("/m~0n").unwrap(), 8);

    // Invalid pointers
    assert!(data.pointer_mut("/unknown").is_none());
    assert!(data.pointer_mut("/e^f/ertz").is_none());
    assert!(data.pointer_mut("/foo/00").is_none());
    assert!(data.pointer_mut("/foo/01").is_none());

    // Mutable pointer checks
    *data.pointer_mut("/").unwrap() = 100.into();
    assert_eq!(data.pointer("/").unwrap(), 100);
    *data.pointer_mut("/foo/0").unwrap() = json!("buzz");
    assert_eq!(data.pointer("/foo/0").unwrap(), &json!("buzz"));

    // Example of ownership stealing
    assert_eq!(data.pointer_mut("/a~1b").map(|m| mem::replace(m, json!(null))).unwrap(), 1);
    assert_eq!(data.pointer("/a~1b").unwrap(), &json!(null));

    // Need to compare against a clone so we don't anger the borrow checker
    // by taking out two references to a mutable value
    let mut d2 = data.clone();
    assert_eq!(data.pointer_mut("").unwrap(), &mut d2);
}

#[test]
fn test_stack_overflow() {
    let brackets: String = iter::repeat('[')
        .take(127)
        .chain(iter::repeat(']').take(127))
        .collect();
    let _: Value = from_str(&brackets).unwrap();

    let brackets: String = iter::repeat('[').take(128).collect();
    test_parse_err::<Value>(&[(&brackets, "recursion limit exceeded at line 1 column 128")],);
}

#[test]
fn test_integer_key() {
    // map with integer keys
    let map = treemap!(
        1 => 2,
        -1 => 6
    );
    let j = r#"{"-1":6,"1":2}"#;
    test_encode_ok(&[(&map, j)]);
    test_parse_ok(vec![(j, map)]);

    let j = r#"{"x":null}"#;
    test_parse_err::<BTreeMap<i32, ()>>(
        &[
            (j, "invalid type: string \"x\", expected i32 at line 1 column 4"),
        ],
    );
}

#[test]
fn test_deny_float_key() {
    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    struct Float;
    impl Serialize for Float {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_f32(1.0)
        }
    }

    // map with float key
    let map = treemap!(Float => "x");
    assert!(serde_json::to_value(&map).is_err());
}

#[test]
fn test_borrowed_key() {
    let map: BTreeMap<&str, ()> = from_str("{\"borrowed\":null}").unwrap();
    let expected = treemap! { "borrowed" => () };
    assert_eq!(map, expected);

    #[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
    struct NewtypeStr<'a>(&'a str);

    let map: BTreeMap<NewtypeStr, ()> = from_str("{\"borrowed\":null}").unwrap();
    let expected = treemap! { NewtypeStr("borrowed") => () };
    assert_eq!(map, expected);
}

#[test]
fn test_effectively_string_keys() {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
    enum Enum {
        One,
        Two,
    }
    let map = treemap! {
        Enum::One => 1,
        Enum::Two => 2
    };
    let expected = r#"{"One":1,"Two":2}"#;
    test_encode_ok(&[(&map, expected)]);
    test_parse_ok(vec![(expected, map)]);

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
    struct Wrapper(String);
    let map = treemap! {
        Wrapper("zero".to_owned()) => 0,
        Wrapper("one".to_owned()) => 1
    };
    let expected = r#"{"one":1,"zero":0}"#;
    test_encode_ok(&[(&map, expected)]);
    test_parse_ok(vec![(expected, map)]);
}

#[test]
fn test_json_macro() {
    // This is tricky because the <...> is not a single TT and the comma inside
    // looks like an array element separator.
    let _ = json!([
        <Result<(), ()> as Clone>::clone(&Ok(())),
        <Result<(), ()> as Clone>::clone(&Err(()))
    ]);

    // Same thing but in the map values.
    let _ = json!({
        "ok": <Result<(), ()> as Clone>::clone(&Ok(())),
        "err": <Result<(), ()> as Clone>::clone(&Err(()))
    });

    // It works in map keys but only if they are parenthesized.
    let _ = json!({
        (<Result<&str, ()> as Clone>::clone(&Ok("")).unwrap()): "ok",
        (<Result<(), &str> as Clone>::clone(&Err("")).unwrap_err()): "err"
    });
}

#[test]
fn issue_220() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    enum E {
        V(u8),
    }

    assert!(from_str::<E>(r#" "V"0 "#).is_err());

    assert_eq!(from_str::<E>(r#"{"V": 0}"#).unwrap(), E::V(0));
}

macro_rules! number_partialeq_ok {
    ($($n:expr)*) => {
        $(
            let value = to_value($n).unwrap();
            let s = $n.to_string();
            assert_eq!(value, $n);
            assert_eq!($n, value);
            assert_ne!(value, s);
        )*
    }
}

#[test]
fn test_partialeq_number() {
    number_partialeq_ok!(0 1 100
        i8::MIN i8::MAX i16::MIN i16::MAX i32::MIN i32::MAX i64::MIN i64::MAX
        u8::MIN u8::MAX u16::MIN u16::MAX u32::MIN u32::MAX u64::MIN u64::MAX
        f32::MIN f32::MAX f32::MIN_EXP f32::MAX_EXP f32::MIN_POSITIVE
        f64::MIN f64::MAX f64::MIN_EXP f64::MAX_EXP f64::MIN_POSITIVE
        f32::consts::E f32::consts::PI f32::consts::LN_2 f32::consts::LOG2_E
        f64::consts::E f64::consts::PI f64::consts::LN_2 f64::consts::LOG2_E
    );
}

#[test]
fn test_partialeq_string() {
    let v = to_value("42").unwrap();
    assert_eq!(v, "42");
    assert_eq!("42", v);
    assert_ne!(v, 42);
    assert_eq!(v, String::from("42"));
    assert_eq!(String::from("42"), v);
}

#[test]
fn test_partialeq_bool() {
    let v = to_value(true).unwrap();
    assert_eq!(v, true);
    assert_eq!(true, v);
    assert_ne!(v, false);
    assert_ne!(v, "true");
    assert_ne!(v, 1);
    assert_ne!(v, 0);
}


struct FailReader(io::ErrorKind);

impl io::Read for FailReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(self.0, "oh no!"))
    }
}

#[test]
fn test_category() {
    assert!(from_str::<String>("123").unwrap_err().is_data());

    assert!(from_str::<String>("]").unwrap_err().is_syntax());

    assert!(from_str::<String>("").unwrap_err().is_eof());
    assert!(from_str::<String>("\"").unwrap_err().is_eof());
    assert!(from_str::<String>("\"\\").unwrap_err().is_eof());
    assert!(from_str::<String>("\"\\u").unwrap_err().is_eof());
    assert!(from_str::<String>("\"\\u0").unwrap_err().is_eof());
    assert!(from_str::<String>("\"\\u00").unwrap_err().is_eof());
    assert!(from_str::<String>("\"\\u000").unwrap_err().is_eof());

    assert!(from_str::<Vec<usize>>("[").unwrap_err().is_eof());
    assert!(from_str::<Vec<usize>>("[0").unwrap_err().is_eof());
    assert!(from_str::<Vec<usize>>("[0,").unwrap_err().is_eof());

    assert!(from_str::<BTreeMap<String, usize>>("{").unwrap_err().is_eof());
    assert!(from_str::<BTreeMap<String, usize>>("{\"k\"").unwrap_err().is_eof());
    assert!(from_str::<BTreeMap<String, usize>>("{\"k\":").unwrap_err().is_eof());
    assert!(from_str::<BTreeMap<String, usize>>("{\"k\":0").unwrap_err().is_eof());
    assert!(from_str::<BTreeMap<String, usize>>("{\"k\":0,").unwrap_err().is_eof());

    let fail = FailReader(io::ErrorKind::NotConnected);
    assert!(from_reader::<_, String>(fail).unwrap_err().is_io());
}

#[test]
// Clippy false positive: https://github.com/Manishearth/rust-clippy/issues/292
#[cfg_attr(feature = "cargo-clippy", allow(needless_lifetimes))]
fn test_into_io_error() {
    fn io_error<'de, T: Deserialize<'de> + Debug>(j: &'static str) -> io::Error {
        from_str::<T>(j).unwrap_err().into()
    }

    assert_eq!(io_error::<String>("\"\\u").kind(), io::ErrorKind::UnexpectedEof);
    assert_eq!(io_error::<String>("0").kind(), io::ErrorKind::InvalidData);
    assert_eq!(io_error::<String>("]").kind(), io::ErrorKind::InvalidData);

    let fail = FailReader(io::ErrorKind::NotConnected);
    let io_err: io::Error = from_reader::<_, u8>(fail).unwrap_err().into();
    assert_eq!(io_err.kind(), io::ErrorKind::NotConnected);
}

#[test]
fn test_borrow() {
    let s: &str = from_str("\"borrowed\"").unwrap();
    assert_eq!("borrowed", s);

    let s: &str = from_slice(b"\"borrowed\"").unwrap();
    assert_eq!("borrowed", s);
}

#[test]
fn null_invalid_type() {
    let err = serde_json::from_str::<String>("null").unwrap_err();
    assert_eq!(format!("{}", err), String::from("invalid type: null, expected a string at line 1 column 4"));
}

#[test]
fn test_number_kind(){
    assert_eq!(serde_json::Number::from_f64(256.0).unwrap().kind(), serde_json::NumberKind::Float);
    assert_eq!(serde_json::Number::from_f64(-256.0).unwrap().kind(), serde_json::NumberKind::Float);

    assert_eq!(serde_json::Number::from(256u64).kind(), serde_json::NumberKind::PosInt);
    assert_eq!(serde_json::Number::from(256i64).kind(), serde_json::NumberKind::PosInt);

    assert_eq!(serde_json::Number::from(-256i64).kind(), serde_json::NumberKind::NegInt);
}