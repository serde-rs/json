use std::f64;
use std::fmt::Debug;
use std::i64;
use std::iter;
use std::marker::PhantomData;
use std::u64;

use serde::de;
use serde::ser;
use serde::bytes::{ByteBuf, Bytes};

use serde_json::{
    self,
    StreamDeserializer,
    Value,
    Map,
    from_iter,
    from_slice,
    from_str,
    from_value,
    to_value,
};

use serde_json::error::{Error, ErrorCode};

macro_rules! treemap {
    () => {
        Map::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = Map::new();
            $(m.insert($k, $v);)+
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
    where T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = serde_json::to_string(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value);
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, out);
    }
}

fn test_pretty_encode_ok<T>(errors: &[(T, &str)])
    where T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = serde_json::to_string_pretty(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value);
        let s = serde_json::to_string_pretty(&v).unwrap();
        assert_eq!(s, out);
    }
}

#[test]
fn test_write_null() {
    let tests = &[
        ((), "null"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_u64() {
    let tests = &[
        (3u64, "3"),
        (u64::MAX, &u64::MAX.to_string()),
    ];
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
    let v = to_value(::std::f64::NAN);
    assert!(v.is_null());

    let v = to_value(::std::f64::INFINITY);
    assert!(v.is_null());

    let v = to_value(::std::f32::NAN);
    assert!(v.is_null());

    let v = to_value(::std::f32::INFINITY);
    assert!(v.is_null());
}

#[test]
fn test_write_str() {
    let tests = &[
        ("", "\"\""),
        ("foo", "\"foo\""),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[
        (true, "true"),
        (false, "false"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_list() {
    test_encode_ok(&[
        (vec![], "[]"),
        (vec![true], "[true]"),
        (vec![true, false], "[true,false]"),
    ]);

    test_encode_ok(&[
        (vec![vec![], vec![], vec![]], "[[],[],[]]"),
        (vec![vec![1, 2, 3], vec![], vec![]], "[[1,2,3],[],[]]"),
        (vec![vec![], vec![1, 2, 3], vec![]], "[[],[1,2,3],[]]"),
        (vec![vec![], vec![], vec![1, 2, 3]], "[[],[],[1,2,3]]"),
    ]);

    test_pretty_encode_ok(&[
        (
            vec![vec![], vec![], vec![]],
            indoc!("
                [
                  [],
                  [],
                  []
                ]"
            ),
        ),
        (
            vec![vec![1, 2, 3], vec![], vec![]],
            indoc!("
                [
                  [
                    1,
                    2,
                    3
                  ],
                  [],
                  []
                ]"
            ),
        ),
        (
            vec![vec![], vec![1, 2, 3], vec![]],
            indoc!("
                [
                  [],
                  [
                    1,
                    2,
                    3
                  ],
                  []
                ]"
            ),
        ),
        (
            vec![vec![], vec![], vec![1, 2, 3]],
            indoc!("
                [
                  [],
                  [],
                  [
                    1,
                    2,
                    3
                  ]
                ]"
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (vec![], "[]"),
        (
            vec![true],
            indoc!("
                [
                  true
                ]"
            ),
        ),
        (
            vec![true, false],
            indoc!("
                [
                  true,
                  false
                ]"
            ),
        ),
    ]);

    let long_test_list = Value::Array(vec![
        Value::Bool(false),
        Value::Null,
        Value::Array(vec![Value::String("foo\nbar".to_string()), Value::F64(3.5)])]);

    test_encode_ok(&[
        (
            long_test_list.clone(),
            "[false,null,[\"foo\\nbar\",3.5]]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            long_test_list,
            indoc!(r#"
                [
                  false,
                  null,
                  [
                    "foo\nbar",
                    3.5
                  ]
                ]"#
            ),
        )
    ]);
}

#[test]
fn test_write_object() {
    test_encode_ok(&[
        (treemap!(), "{}"),
        (treemap!("a".to_string() => true), "{\"a\":true}"),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            "{\"a\":true,\"b\":false}"),
    ]);

    test_encode_ok(&[
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
            "{\"a\":{},\"b\":{},\"c\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}}}",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            indoc!(r#"
                {
                  "a": {},
                  "b": {},
                  "c": {}
                }"#
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            indoc!(r#"
                {
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
                }"#
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
            indoc!(r#"
                {
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
                }"#
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
            indoc!(r#"
                {
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
                }"#
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (treemap!(), "{}"),
        (
            treemap!("a".to_string() => true),
            indoc!(r#"
                {
                  "a": true
                }"#
            ),
        ),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            indoc!(r#"
                {
                  "a": true,
                  "b": false
                }"#
            ),
        ),
    ]);

    let complex_obj = Value::Object(treemap!(
        "b".to_string() => Value::Array(vec![
            Value::Object(treemap!("c".to_string() => Value::String("\x0c\x1f\r".to_string()))),
            Value::Object(treemap!("d".to_string() => Value::String("".to_string())))
        ])
    ));

    test_encode_ok(&[
        (
            complex_obj.clone(),
            "{\
                \"b\":[\
                    {\"c\":\"\\f\\u001f\\r\"},\
                    {\"d\":\"\"}\
                ]\
            }"
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            complex_obj.clone(),
            indoc!(r#"
                {
                  "b": [
                    {
                      "c": "\f\u001f\r"
                    },
                    {
                      "d": ""
                    }
                  ]
                }"#
            ),
        )
    ]);
}

#[test]
fn test_write_tuple() {
    test_encode_ok(&[
        (
            (5,),
            "[5]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            (5,),
            indoc!("
                [
                  5
                ]"
            ),
        ),
    ]);

    test_encode_ok(&[
        (
            (5, (6, "abc")),
            "[5,[6,\"abc\"]]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            (5, (6, "abc")),
            indoc!(r#"
                [
                  5,
                  [
                    6,
                    "abc"
                  ]
                ]"#
            ),
        ),
    ]);
}

#[test]
fn test_write_enum() {
    test_encode_ok(&[
        (
            Animal::Dog,
            "\"Dog\"",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            "{\"Frog\":[\"Henry\",[]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            "{\"Frog\":[\"Henry\",[349]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            "{\"Frog\":[\"Henry\",[349,102]]}",
        ),
        (
            Animal::Cat { age: 5, name: "Kate".to_string() },
            "{\"Cat\":{\"age\":5,\"name\":\"Kate\"}}"
        ),
        (
            Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
            "{\"AntHive\":[\"Bob\",\"Stuart\"]}",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            Animal::Dog,
            "\"Dog\"",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            indoc!(r#"
                {
                  "Frog": [
                    "Henry",
                    []
                  ]
                }"#
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            indoc!(r#"
                {
                  "Frog": [
                    "Henry",
                    [
                      349
                    ]
                  ]
                }"#
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            indoc!(r#"
                {
                  "Frog": [
                    "Henry",
                    [
                      349,
                      102
                    ]
                  ]
                }"#
            ),
        ),
    ]);
}

#[test]
fn test_write_option() {
    test_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_encode_ok(&[
        (None, "null"),
        (Some(vec!["foo", "bar"]), "[\"foo\",\"bar\"]"),
    ]);

    test_pretty_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_pretty_encode_ok(&[
        (None, "null"),
        (
            Some(vec!["foo", "bar"]),
            indoc!(r#"
                [
                  "foo",
                  "bar"
                ]"#
            ),
        ),
    ]);
}

#[test]
fn test_write_newtype_struct() {
    #[derive(Serialize, PartialEq, Debug)]
    struct Newtype(Map<String, i32>);

    let inner = Newtype(treemap!(String::from("inner") => 123));
    let outer = treemap!(String::from("outer") => to_value(&inner));

    test_encode_ok(&[
        (inner, r#"{"inner":123}"#),
    ]);

    test_encode_ok(&[
        (outer, r#"{"outer":{"inner":123}}"#),
    ]);
}

fn test_parse_ok<T>(tests: Vec<(&str, T)>)
    where T: Clone + Debug + PartialEq + ser::Serialize + de::Deserialize,
{
    for (s, value) in tests {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_slice(s.as_bytes()).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_iter(s.bytes().map(Ok)).unwrap();
        assert_eq!(v, value.clone());

        // Make sure we can deserialize into a `Value`.
        let json_value: Value = from_str(s).unwrap();
        assert_eq!(json_value, to_value(&value));

        // Make sure we can deserialize from a `Value`.
        let v: T = from_value(json_value.clone()).unwrap();
        assert_eq!(v, value);

        // Make sure we can round trip back to `Value`.
        let json_value2: Value = from_value(json_value.clone()).unwrap();
        assert_eq!(json_value2, json_value);
    }
}

// For testing representations that the deserializer accepts but the serializer
// never generates. These do not survive a round-trip through Value.
fn test_parse_unusual_ok<T>(tests: Vec<(&str, T)>)
    where T: Clone + Debug + PartialEq + ser::Serialize + de::Deserialize,
{
    for (s, value) in tests {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_slice(s.as_bytes()).unwrap();
        assert_eq!(v, value.clone());

        let v: T = from_iter(s.bytes().map(Ok)).unwrap();
        assert_eq!(v, value.clone());
    }
}

macro_rules! test_parse_err {
    ($name:ident::<$($ty:ty),*>($arg:expr) => $err:expr) => {
        match ($err, &$name::<$($ty),*>($arg).unwrap_err()) {
            (
                &Error::Syntax(ref expected_code, expected_line, expected_col),
                &Error::Syntax(ref actual_code, actual_line, actual_col),
            ) if expected_code == actual_code
                && expected_line == actual_line
                && expected_col == actual_col => { /* pass */ }
            (expected_err, actual_err) => {
                panic!("unexpected {} error: {}, expected: {}", stringify!($name), actual_err, expected_err)
            }
        }
    };
}

// FIXME (#5527): these could be merged once UFCS is finished.
fn test_parse_err<T>(errors: Vec<(&str, Error)>)
    where T: Debug + PartialEq + de::Deserialize,
{
    for &(s, ref err) in &errors {
        test_parse_err!(from_str::<T>(s) => err);
        test_parse_err!(from_slice::<T>(s.as_bytes()) => err);
        test_parse_err!(from_iter::<_, T>(s.bytes().map(Ok)) => err);
    }
}

fn test_parse_slice_err<T>(errors: Vec<(&[u8], Error)>)
    where T: Debug + PartialEq + de::Deserialize,
{
    for &(s, ref err) in &errors {
        test_parse_err!(from_slice::<T>(s) => err);
        test_parse_err!(from_iter::<_, T>(s.iter().cloned().map(Ok)) => err);
    }
}

#[test]
fn test_parse_null() {
    test_parse_err::<()>(vec![
        ("n", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("nul", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("nulla", Error::Syntax(ErrorCode::TrailingCharacters, 1, 5)),
    ]);

    test_parse_ok(vec![
        ("null", ()),
    ]);
}

#[test]
fn test_parse_bool() {
    test_parse_err::<bool>(vec![
        ("t", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("truz", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 4)),
        ("f", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("faz", Error::Syntax(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("truea", Error::Syntax(ErrorCode::TrailingCharacters, 1, 5)),
        ("falsea", Error::Syntax(ErrorCode::TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(vec![
        ("true", true),
        (" true ", true),
        ("false", false),
        (" false ", false),
    ]);
}

#[test]
fn test_parse_number_errors() {
    test_parse_err::<f64>(vec![
        ("+", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 1)),
        (".", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("-", Error::Syntax(ErrorCode::InvalidNumber, 1, 1)),
        ("00", Error::Syntax(ErrorCode::InvalidNumber, 1, 2)),
        ("0x80", Error::Syntax(ErrorCode::TrailingCharacters, 1, 2)),
        ("\\0", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("1.", Error::Syntax(ErrorCode::InvalidNumber, 1, 2)),
        ("1.a", Error::Syntax(ErrorCode::InvalidNumber, 1, 3)),
        ("1.e1", Error::Syntax(ErrorCode::InvalidNumber, 1, 3)),
        ("1e", Error::Syntax(ErrorCode::InvalidNumber, 1, 2)),
        ("1e+", Error::Syntax(ErrorCode::InvalidNumber, 1, 3)),
        ("1a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 2)),
        ("100e777777777777777777777777777", Error::Syntax(ErrorCode::NumberOutOfRange, 1, 14)),
        ("-100e777777777777777777777777777", Error::Syntax(ErrorCode::NumberOutOfRange, 1, 15)),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000", // 1e309
           Error::Syntax(ErrorCode::NumberOutOfRange, 1, 310)),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           .0e9", // 1e309
           Error::Syntax(ErrorCode::NumberOutOfRange, 1, 305)),
        ("1000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           000000000000000000000000000000000000000000000000000000000000\
           e9", // 1e309
           Error::Syntax(ErrorCode::NumberOutOfRange, 1, 303)),
    ]);
}

#[test]
fn test_parse_i64() {
    test_parse_ok(vec![
        ("-2", -2),
        ("-1234", -1234),
        (" -1234 ", -1234),
        (&i64::MIN.to_string(), i64::MIN),
        (&i64::MAX.to_string(), i64::MAX),
    ]);
}

#[test]
fn test_parse_u64() {
    test_parse_ok(vec![
        ("0", 0u64),
        ("3", 3u64),
        ("1234", 1234),
        (&u64::MAX.to_string(), u64::MAX),
    ]);
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
        assert_eq!(0, from_str::<u32>(negative_zero).unwrap());
        assert!(from_str::<f64>(negative_zero).unwrap().is_sign_negative(),
            "should have been negative: {:?}", negative_zero);
    }
}

#[test]
fn test_parse_f64() {
    test_parse_ok(vec![
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
    ]);
}

#[test]
fn test_parse_string() {
    test_parse_err::<String>(vec![
        ("\"", Error::Syntax(ErrorCode::EOFWhileParsingString, 1, 1)),
        ("\"lol", Error::Syntax(ErrorCode::EOFWhileParsingString, 1, 4)),
        ("\"lol\"a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 6)),
        ("\"\\uD83C\\uFFFF\"", Error::Syntax(ErrorCode::LoneLeadingSurrogateInHexEscape, 1, 13)),
    ]);

    test_parse_slice_err::<String>(vec![
        (&[b'"', 159, 146, 150, b'"'],
            Error::Syntax(ErrorCode::InvalidUnicodeCodePoint, 1, 5)),
        (&[b'"', b'\\', b'n', 159, 146, 150, b'"'],
            Error::Syntax(ErrorCode::InvalidUnicodeCodePoint, 1, 7)),
    ]);

    test_parse_ok(vec![
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
    ]);
}

#[test]
fn test_parse_list() {
    test_parse_err::<Vec<f64>>(vec![
        ("[", Error::Syntax(ErrorCode::EOFWhileParsingList, 1, 1)),
        ("[ ", Error::Syntax(ErrorCode::EOFWhileParsingList, 1, 2)),
        ("[1", Error::Syntax(ErrorCode::EOFWhileParsingList,  1, 2)),
        ("[1,", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 3)),
        ("[1,]", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 4)),
        ("[1 2]", Error::Syntax(ErrorCode::ExpectedListCommaOrEnd, 1, 4)),
        ("[]a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(vec![
        ("[]", vec![]),
        ("[ ]", vec![]),
        ("[null]", vec![()]),
        (" [ null ] ", vec![()]),
    ]);

    test_parse_ok(vec![
        ("[true]", vec![true]),
    ]);

    test_parse_ok(vec![
        ("[3,1]", vec![3u64, 1]),
        (" [ 3 , 1 ] ", vec![3, 1]),
    ]);

    test_parse_ok(vec![
        ("[[3], [1, 2]]", vec![vec![3u64], vec![1, 2]]),
    ]);

    test_parse_ok(vec![
        ("[1]", (1u64,)),
    ]);

    test_parse_ok(vec![
        ("[1, 2]", (1u64, 2u64)),
    ]);

    test_parse_ok(vec![
        ("[1, 2, 3]", (1u64, 2u64, 3u64)),
    ]);

    test_parse_ok(vec![
        ("[1, [2, 3]]", (1u64, (2u64, 3u64))),
    ]);

    let v: () = from_str("[]").unwrap();
    assert_eq!(v, ());
}

#[test]
fn test_parse_object() {
    test_parse_err::<Map<String, u32>>(vec![
        ("{", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 1)),
        ("{ ", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 2)),
        ("{1", Error::Syntax(ErrorCode::KeyMustBeAString, 1, 2)),
        ("{ \"a\"", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\"", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 4)),
        ("{\"a\" ", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\" 1", Error::Syntax(ErrorCode::ExpectedColon, 1, 6)),
        ("{\"a\":", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 5)),
        ("{\"a\":1", Error::Syntax(ErrorCode::EOFWhileParsingObject, 1, 6)),
        ("{\"a\":1 1", Error::Syntax(ErrorCode::ExpectedObjectCommaOrEnd, 1, 8)),
        ("{\"a\":1,", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{}a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(vec![
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
    ]);

    test_parse_ok(vec![
        (
            "{\"a\": {\"b\": 3, \"c\": 4}}",
            treemap!(
                "a".to_string() => treemap!(
                    "b".to_string() => 3u64,
                    "c".to_string() => 4
                )
            ),
        ),
    ]);
}

#[test]
fn test_parse_struct() {
    test_parse_err::<Outer>(vec![
        ("5", Error::Syntax(ErrorCode::InvalidType(de::Type::U64), 1, 1)),
        ("\"hello\"", Error::Syntax(ErrorCode::InvalidType(de::Type::Str), 1, 7)),
        ("{\"inner\": true}", Error::Syntax(ErrorCode::InvalidType(de::Type::Bool), 1, 14)),
        ("{}", Error::Syntax(ErrorCode::MissingField("inner"), 1, 2)),
        (r#"{"inner": [{"b": 42, "c": []}]}"#, Error::Syntax(ErrorCode::MissingField("a"), 1, 29)),
    ]);

    test_parse_ok(vec![
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
    ]);

    let v: Outer = from_str(
        "[
            [
                [ null, 2, [\"abc\", \"xyz\"] ]
            ]
        ]").unwrap();

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
    test_parse_ok(vec![
        ("null", None::<String>),
        ("\"jodhpurs\"", Some("jodhpurs".to_string())),
    ]);

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        x: Option<isize>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    test_parse_ok(vec![
        ("{\"x\": null}", Foo { x: None }),
        ("{\"x\": 5}", Foo { x: Some(5) }),
    ]);
}

#[test]
fn test_parse_enum_errors() {
    test_parse_err::<Animal>(vec![
        ("{}", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 2)),
        ("[]", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("\"unknown\"", Error::Syntax(ErrorCode::UnknownVariant("unknown".to_string()), 1, 9)),
        ("{\"unknown\":[]}", Error::Syntax(ErrorCode::UnknownVariant("unknown".to_string()), 1, 10)),
        ("{\"Dog\":", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{\"Dog\":}", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 8)),
        ("{\"Dog\":{}}", Error::Syntax(ErrorCode::InvalidType(de::Type::Map), 1, 8)),
        ("{\"Dog\":[0]}", Error::Syntax(ErrorCode::TrailingCharacters, 1, 9)),
        ("\"Frog\"", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 6)),
        ("{\"Frog\":{}}", Error::Syntax(ErrorCode::InvalidType(de::Type::Map), 1, 9)),
        ("{\"Cat\":[]}", Error::Syntax(ErrorCode::InvalidLength(0), 1, 9)),
        ("{\"Cat\":[0]}", Error::Syntax(ErrorCode::InvalidLength(1), 1, 10)),
        ("{\"Cat\":[0, \"\", 2]}", Error::Syntax(ErrorCode::TrailingCharacters, 1, 14)),
        (
            "{\"Cat\":{\"age\": 5, \"name\": \"Kate\", \"foo\":\"bar\"}",
            Error::Syntax(ErrorCode::UnknownField("foo".to_string()), 1, 39)
        ),
    ]);
}

#[test]
fn test_parse_enum() {
    test_parse_ok(vec![
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
    ]);

    test_parse_unusual_ok(vec![
        ("{\"Dog\":[]}", Animal::Dog),
        (" { \"Dog\" : [ ] } ", Animal::Dog),
    ]);

    test_parse_ok(vec![
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
    ]);
}

#[test]
fn test_parse_trailing_whitespace() {
    test_parse_ok(vec![
        ("[1, 2] ", vec![1u64, 2]),
        ("[1, 2]\n", vec![1, 2]),
        ("[1, 2]\t", vec![1, 2]),
        ("[1, 2]\t \n", vec![1, 2]),
    ]);
}

#[test]
fn test_multiline_errors() {
    test_parse_err::<Map<String, String>>(vec![
        ("{\n  \"foo\":\n \"bar\"", Error::Syntax(ErrorCode::EOFWhileParsingObject, 3, 6)),
    ]);
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

    let value: Foo = from_value(Value::Object(treemap!())).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(Value::Object(treemap!(
        "x".to_string() => Value::I64(5)
    ))).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_missing_nonoption_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: u32,
    }

    test_parse_err::<Foo>(vec![
        ("{}", Error::Syntax(ErrorCode::MissingField("x"), 1, 2)),
    ]);
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

    let value: Foo = from_value(Value::Object(treemap!())).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(Value::Object(treemap!(
        "y".to_string() => Value::I64(5)
    ))).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_find_path() {
    let obj: Value = serde_json::from_str(r#"{"x": {"a": 1}, "y": 2}"#).unwrap();

    assert!(obj.find_path(&["x", "a"]).unwrap() == &Value::U64(1));
    assert!(obj.find_path(&["y"]).unwrap() == &Value::U64(2));
    assert!(obj.find_path(&["z"]).is_none());
}

#[test]
fn test_lookup() {
    let obj: Value = serde_json::from_str(r#"{"x": {"a": 1}, "y": 2}"#).unwrap();

    assert!(obj.lookup("x.a").unwrap() == &Value::U64(1));
    assert!(obj.lookup("y").unwrap() == &Value::U64(2));
    assert!(obj.lookup("z").is_none());
}

#[test]
fn test_serialize_seq_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyVec<T>(Vec<T>);

    impl<T> ser::Serialize for MyVec<T>
        where T: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: ser::Serializer,
        {
            let mut state = try!(serializer.serialize_seq(None));
            for elem in &self.0 {
                try!(serializer.serialize_seq_elt(&mut state, elem));
            }
            serializer.serialize_seq_end(state)
        }
    }

    struct Visitor<T> {
        marker: PhantomData<MyVec<T>>,
    }

    impl<T> de::Visitor for Visitor<T>
        where T: de::Deserialize,
    {
        type Value = MyVec<T>;

        #[inline]
        fn visit_unit<E>(&mut self) -> Result<MyVec<T>, E>
            where E: de::Error,
        {
            Ok(MyVec(Vec::new()))
        }

        #[inline]
        fn visit_seq<V>(&mut self, mut visitor: V) -> Result<MyVec<T>, V::Error>
            where V: de::SeqVisitor,
        {
            let mut values = Vec::new();

            while let Some(value) = try!(visitor.visit()) {
                values.push(value);
            }

            try!(visitor.end());

            Ok(MyVec(values))
        }
    }

    impl<T> de::Deserialize for MyVec<T>
        where T: de::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<MyVec<T>, D::Error>
            where D: de::Deserializer,
        {
            deserializer.deserialize_map(Visitor { marker: PhantomData })
        }
    }

    let mut vec = Vec::new();
    vec.push(MyVec(Vec::new()));
    vec.push(MyVec(Vec::new()));
    let vec: MyVec<MyVec<u32>> = MyVec(vec);

    test_encode_ok(&[
        (
            vec.clone(),
            "[[],[]]",
        ),
    ]);

    let s = serde_json::to_string_pretty(&vec).unwrap();
    let expected = indoc!("
        [
          [
          ],
          [
          ]
        ]");
    assert_eq!(s, expected);
}

#[test]
fn test_serialize_map_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyMap<K, V>(Map<K, V>);

    impl<K, V> ser::Serialize for MyMap<K, V>
        where K: ser::Serialize + Ord,
              V: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: ser::Serializer,
        {
            let mut state = try!(serializer.serialize_map(None));
            for (k, v) in &self.0 {
                try!(serializer.serialize_map_key(&mut state, k));
                try!(serializer.serialize_map_value(&mut state, v));
            }
            serializer.serialize_map_end(state)
        }
    }

    struct Visitor<K, V> {
        marker: PhantomData<MyMap<K, V>>,
    }

    impl<K, V> de::Visitor for Visitor<K, V>
        where K: de::Deserialize + Eq + Ord,
              V: de::Deserialize,
    {
        type Value = MyMap<K, V>;

        #[inline]
        fn visit_unit<E>(&mut self) -> Result<MyMap<K, V>, E>
            where E: de::Error,
        {
            Ok(MyMap(Map::new()))
        }

        #[inline]
        fn visit_map<Visitor>(&mut self, mut visitor: Visitor) -> Result<MyMap<K, V>, Visitor::Error>
            where Visitor: de::MapVisitor,
        {
            let mut values = Map::new();

            while let Some((key, value)) = try!(visitor.visit()) {
                values.insert(key, value);
            }

            try!(visitor.end());

            Ok(MyMap(values))
        }
    }

    impl<K, V> de::Deserialize for MyMap<K, V>
        where K: de::Deserialize + Eq + Ord,
              V: de::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<MyMap<K, V>, D::Error>
            where D: de::Deserializer,
        {
            deserializer.deserialize_map(Visitor { marker: PhantomData })
        }
    }

    let mut map = Map::new();
    map.insert("a", MyMap(Map::new()));
    map.insert("b", MyMap(Map::new()));
    let map: MyMap<_, MyMap<u32, u32>> = MyMap(map);

    test_encode_ok(&[
        (
            map.clone(),
            "{\"a\":{},\"b\":{}}",
        ),
    ]);

    let s = serde_json::to_string_pretty(&map).unwrap();
    let expected = indoc!(r#"
        {
          "a": {
          },
          "b": {
          }
        }"#);
    assert_eq!(s, expected);
}

#[test]
fn test_deserialize_from_stream() {
    use std::net;
    use std::io::Read;
    use std::thread;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Message {
        message: String,
    }

    let l = net::TcpListener::bind("localhost:20000").unwrap();

    thread::spawn(|| {
        let l = l;
        for stream in l.incoming() {
            let mut stream = stream.unwrap();
            let read_stream = stream.try_clone().unwrap();

            let mut de = serde_json::Deserializer::new(read_stream.bytes());
            let request = Message::deserialize(&mut de).unwrap();
            let response = Message { message: request.message };
            serde_json::to_writer(&mut stream, &response).unwrap();
        }
    });

    let mut stream = net::TcpStream::connect("localhost:20000").unwrap();
    let request = Message { message: "hi there".to_string() };
    serde_json::to_writer(&mut stream, &request).unwrap();

    let mut de = serde_json::Deserializer::new(stream.bytes());
    let response = Message::deserialize(&mut de).unwrap();

    assert_eq!(request, response);
}

#[test]
fn test_serialize_rejects_bool_keys() {
    let map = treemap!(
        true => 2,
        false => 4
    );

    match serde_json::to_vec(&map).unwrap_err() {
        serde_json::Error::Syntax(serde_json::ErrorCode::KeyMustBeAString, 0, 0) => {}
        _ => panic!("integers used as keys"),
    }
}

#[test]
fn test_serialize_rejects_adt_keys() {
    let map = treemap!(
        Some("a") => 2,
        Some("b") => 4,
        None => 6
    );

    match serde_json::to_vec(&map).unwrap_err() {
        serde_json::Error::Syntax(serde_json::ErrorCode::KeyMustBeAString, 0, 0) => {}
        _ => panic!("integers used as keys"),
    }
}

#[test]
fn test_effectively_string_keys() {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
    enum Enum { Zero, One }
    let map = treemap! {
        Enum::Zero => 0,
        Enum::One => 1
    };
    let expected = r#"{"Zero":0,"One":1}"#;
    assert_eq!(serde_json::to_string(&map).unwrap(), expected);
    assert_eq!(map, serde_json::from_str(expected).unwrap());

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
    struct Wrapper(String);
    let map = treemap! {
        Wrapper("zero".to_owned()) => 0,
        Wrapper("one".to_owned()) => 1
    };
    let expected = r#"{"one":1,"zero":0}"#;
    assert_eq!(serde_json::to_string(&map).unwrap(), expected);
    assert_eq!(map, serde_json::from_str(expected).unwrap());
}

#[test]
fn test_bytes_ser() {
    let buf = vec![];
    let bytes = Bytes::from(&buf);
    assert_eq!(serde_json::to_string(&bytes).unwrap(), "[]".to_string());

    let buf = vec![1, 2, 3];
    let bytes = Bytes::from(&buf);
    assert_eq!(serde_json::to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

#[test]
fn test_byte_buf_ser() {
    let bytes = ByteBuf::new();
    assert_eq!(serde_json::to_string(&bytes).unwrap(), "[]".to_string());

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    assert_eq!(serde_json::to_string(&bytes).unwrap(), "[1,2,3]".to_string());
}

#[test]
fn test_byte_buf_de() {
    let bytes = ByteBuf::new();
    let v: ByteBuf = serde_json::from_str("[]").unwrap();
    assert_eq!(v, bytes);

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let v: ByteBuf = serde_json::from_str("[1, 2, 3]").unwrap();
    assert_eq!(v, bytes);
}

#[test]
fn test_json_stream_newlines() {
    let stream = "{\"x\":39} {\"x\":40}{\"x\":41}\n{\"x\":42}".to_string();
    let mut parsed: StreamDeserializer<Value, _> = StreamDeserializer::new(
        stream.as_bytes().iter().map(|byte| Ok(*byte))
    );

    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(39));
    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(40));
    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(41));
    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(42));
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_trailing_whitespaces() {
    let stream = "{\"x\":42} \t\n".to_string();
    let mut parsed: StreamDeserializer<Value, _> = StreamDeserializer::new(
        stream.as_bytes().iter().map(|byte| Ok(*byte))
    );

    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(42));
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_truncated() {
    let stream = "{\"x\":40}\n{\"x\":".to_string();
    let mut parsed: StreamDeserializer<Value, _> = StreamDeserializer::new(
        stream.as_bytes().iter().map(|byte| Ok(*byte))
    );

    assert_eq!(parsed.next().unwrap().ok().unwrap().lookup("x").unwrap(),
               &Value::U64(40));
    assert!(parsed.next().unwrap().is_err());
    assert!(parsed.next().is_none());
}

#[test]
fn test_json_stream_empty() {
    let stream = "".to_string();
    let mut parsed: StreamDeserializer<Value, _> = StreamDeserializer::new(
        stream.as_bytes().iter().map(|byte| Ok(*byte))
    );

    assert!(parsed.next().is_none());
}

#[test]
fn test_json_pointer() {
    // Test case taken from https://tools.ietf.org/html/rfc6901#page-5
    let data: Value = serde_json::from_str(r#"{
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
    }"#).unwrap();
    assert_eq!(data.pointer("").unwrap(), &data);
    assert_eq!(data.pointer("/foo").unwrap(),
        &Value::Array(vec![Value::String("bar".to_owned()),
                           Value::String("baz".to_owned())]));
    assert_eq!(data.pointer("/foo/0").unwrap(),
        &Value::String("bar".to_owned()));
    assert_eq!(data.pointer("/").unwrap(), &Value::U64(0));
    assert_eq!(data.pointer("/a~1b").unwrap(), &Value::U64(1));
    assert_eq!(data.pointer("/c%d").unwrap(), &Value::U64(2));
    assert_eq!(data.pointer("/e^f").unwrap(), &Value::U64(3));
    assert_eq!(data.pointer("/g|h").unwrap(), &Value::U64(4));
    assert_eq!(data.pointer("/i\\j").unwrap(), &Value::U64(5));
    assert_eq!(data.pointer("/k\"l").unwrap(), &Value::U64(6));
    assert_eq!(data.pointer("/ ").unwrap(), &Value::U64(7));
    assert_eq!(data.pointer("/m~0n").unwrap(), &Value::U64(8));
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
    let mut data: Value = serde_json::from_str(r#"{
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
    }"#).unwrap();

    // Basic pointer checks
    assert_eq!(data.pointer_mut("/foo").unwrap(),
        &Value::Array(vec![Value::String("bar".to_owned()),
                           Value::String("baz".to_owned())]));
    assert_eq!(data.pointer_mut("/foo/0").unwrap(),
        &Value::String("bar".to_owned()));
    assert_eq!(data.pointer_mut("/").unwrap(), &Value::U64(0));
    assert_eq!(data.pointer_mut("/a~1b").unwrap(), &Value::U64(1));
    assert_eq!(data.pointer_mut("/c%d").unwrap(), &Value::U64(2));
    assert_eq!(data.pointer_mut("/e^f").unwrap(), &Value::U64(3));
    assert_eq!(data.pointer_mut("/g|h").unwrap(), &Value::U64(4));
    assert_eq!(data.pointer_mut("/i\\j").unwrap(), &Value::U64(5));
    assert_eq!(data.pointer_mut("/k\"l").unwrap(), &Value::U64(6));
    assert_eq!(data.pointer_mut("/ ").unwrap(), &Value::U64(7));
    assert_eq!(data.pointer_mut("/m~0n").unwrap(), &Value::U64(8));

    // Invalid pointers
    assert!(data.pointer_mut("/unknown").is_none());
    assert!(data.pointer_mut("/e^f/ertz").is_none());
    assert!(data.pointer_mut("/foo/00").is_none());
    assert!(data.pointer_mut("/foo/01").is_none());

    // Mutable pointer checks
    *data.pointer_mut("/").unwrap() = Value::U64(100);
    assert_eq!(data.pointer("/").unwrap(), &Value::U64(100));
    *data.pointer_mut("/foo/0").unwrap() = Value::String("buzz".to_owned());
    assert_eq!(data.pointer("/foo/0").unwrap(), &Value::String("buzz".to_owned()));

    // Example of ownership stealing
    assert_eq!(data.pointer_mut("/a~1b").map(|m| mem::replace(m, Value::Null)).unwrap(), Value::U64(1));
    assert_eq!(data.pointer("/a~1b").unwrap(), &Value::Null);

    // Need to compare against a clone so we don't anger the borrow checker
    // by taking out two references to a mutable value
    let mut d2 = data.clone();
    assert_eq!(data.pointer_mut("").unwrap(), &mut d2);
}

#[test]
fn test_stack_overflow() {
    let brackets: String = iter::repeat('[').take(127).chain(iter::repeat(']').take(127)).collect();
    let _: Value = serde_json::from_str(&brackets).unwrap();

    let brackets: String = iter::repeat('[').take(128).collect();
    test_parse_err::<Value>(vec![
        (&brackets, Error::Syntax(ErrorCode::Custom("recursion limit exceeded".into()), 1, 128)),
    ]);
}

#[test]
fn test_allow_integers_as_map_keys(){
    let map = treemap!(
        1 => 2,
        2 => 4,
        -1 => 6,
        -2 => 8
    );
    
    serde_json::to_vec(&map).unwrap();
}
