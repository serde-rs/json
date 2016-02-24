use std::collections::BTreeMap;
use std::f64;
use std::fmt::Debug;
use std::i64;
use std::marker::PhantomData;
use std::u64;

use serde::de;
use serde::ser;
use serde::bytes::{ByteBuf, Bytes};

use serde_json::{
    self,
    Value,
    from_str,
    from_value,
    to_value,
};

use serde_json::error::{Error, ErrorCode};

macro_rules! treemap {
    () => {
        BTreeMap::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = BTreeMap::new();
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
    let min_string = format!("{:?}", f64::MIN);
    let max_string = format!("{:?}", f64::MAX);
    let epsilon_string = format!("{:?}", f64::EPSILON);

    let tests = &[
        (3.0, "3"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
        (f64::MIN, &min_string),
        (f64::MAX, &max_string),
        (f64::EPSILON, &epsilon_string),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
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
            concat!(
                "[\n",
                "  [],\n",
                "  [],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![1, 2, 3], vec![], vec![]],
            concat!(
                "[\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ],\n",
                "  [],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![], vec![1, 2, 3], vec![]],
            concat!(
                "[\n",
                "  [],\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![], vec![], vec![1, 2, 3]],
            concat!(
                "[\n",
                "  [],\n",
                "  [],\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ]\n",
                "]"
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (vec![], "[]"),
        (
            vec![true],
            concat!(
                "[\n",
                "  true\n",
                "]"
            ),
        ),
        (
            vec![true, false],
            concat!(
                "[\n",
                "  true,\n",
                "  false\n",
                "]"
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
            concat!(
                "[\n",
                "  false,\n",
                "  null,\n",
                "  [\n",
                "    \"foo\\nbar\",\n",
                "    3.5\n",
                "  ]\n",
                "]"
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
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {},\n",
                "  \"c\": {}\n",
                "}",
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
            concat!(
                "{\n",
                "  \"a\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  },\n",
                "  \"b\": {},\n",
                "  \"c\": {}\n",
                "}"
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
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  },\n",
                "  \"c\": {}\n",
                "}"
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
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {},\n",
                "  \"c\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  }\n",
                "}"
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (treemap!(), "{}"),
        (
            treemap!("a".to_string() => true),
            concat!(
                "{\n",
                "  \"a\": true\n",
                "}"
            ),
        ),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            concat!(
                "{\n",
                "  \"a\": true,\n",
                "  \"b\": false\n",
                "}"
            ),
        ),
    ]);

    let complex_obj = Value::Object(treemap!(
        "b".to_string() => Value::Array(vec![
            Value::Object(treemap!("c".to_string() => Value::String("\x0c\r".to_string()))),
            Value::Object(treemap!("d".to_string() => Value::String("".to_string())))
        ])
    ));

    test_encode_ok(&[
        (
            complex_obj.clone(),
            "{\
                \"b\":[\
                    {\"c\":\"\\f\\r\"},\
                    {\"d\":\"\"}\
                ]\
            }"
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            complex_obj.clone(),
            concat!(
                "{\n",
                "  \"b\": [\n",
                "    {\n",
                "      \"c\": \"\\f\\r\"\n",
                "    },\n",
                "    {\n",
                "      \"d\": \"\"\n",
                "    }\n",
                "  ]\n",
                "}"
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
            concat!(
                "[\n",
                "  5\n",
                "]"
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
            concat!(
                "[\n",
                "  5,\n",
                "  [\n",
                "    6,\n",
                "    \"abc\"\n",
                "  ]\n",
                "]"
            ),
        ),
    ]);
}

#[test]
fn test_write_enum() {
    test_encode_ok(&[
        (
            Animal::Dog,
            "{\"Dog\":[]}",
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
            concat!(
                "{\n",
                "  \"Dog\": []\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    []\n",
                "  ]\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    [\n",
                "      349\n",
                "    ]\n",
                "  ]\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    [\n",
                "      349,\n",
                "      102\n",
                "    ]\n",
                "  ]\n",
                "}"
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
            concat!(
                "[\n",
                "  \"foo\",\n",
                "  \"bar\"\n",
                "]"
            ),
        ),
    ]);
}

fn test_parse_ok<T>(errors: Vec<(&str, T)>)
    where T: Clone + Debug + PartialEq + ser::Serialize + de::Deserialize,
{
    for (s, value) in errors {
        let v: T = from_str(s).unwrap();
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

// FIXME (#5527): these could be merged once UFCS is finished.
fn test_parse_err<T>(errors: Vec<(&'static str, Error)>)
    where T: Debug + PartialEq + de::Deserialize,
{
    for (s, err) in errors {
        match (err, from_str::<T>(s).unwrap_err()) {
            (
                Error::SyntaxError(expected_code, expected_line, expected_col),
                Error::SyntaxError(actual_code, actual_line, actual_col),
            ) => {
                assert_eq!(
                    (expected_code, expected_line, expected_col),
                    (actual_code, actual_line, actual_col)
                )
            }
            (expected_err, actual_err) => {
                panic!("unexpected errors {} != {}", expected_err, actual_err)
            }
        }
    }
}

#[test]
fn test_parse_null() {
    test_parse_err::<()>(vec![
        ("n", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("nul", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("nulla", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 5)),
    ]);

    test_parse_ok(vec![
        ("null", ()),
    ]);
}

#[test]
fn test_parse_bool() {
    test_parse_err::<bool>(vec![
        ("t", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("truz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 4)),
        ("f", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("faz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("truea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 5)),
        ("falsea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
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
        ("+", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        (".", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("-", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 1)),
        ("00", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1.", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1e", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1e+", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 3)),
        ("1a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 2)),
        ("1e777777777777777777777777777", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 22)),
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
    assert_eq!(0, from_str::<u32>("-0").unwrap());
    assert_eq!(0, from_str::<u32>("-0.0").unwrap());
    assert_eq!(0, from_str::<u32>("-0e2").unwrap());
    assert_eq!(0, from_str::<u32>("-0.0e2").unwrap());
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
    ]);
}

#[test]
fn test_parse_string() {
    test_parse_err::<String>(vec![
        ("\"", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 1)),
        ("\"lol", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 4)),
        ("\"lol\"a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
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
    ]);
}

#[test]
fn test_parse_list() {
    test_parse_err::<Vec<f64>>(vec![
        ("[", Error::SyntaxError(ErrorCode::EOFWhileParsingList, 1, 1)),
        ("[ ", Error::SyntaxError(ErrorCode::EOFWhileParsingList, 1, 2)),
        ("[1", Error::SyntaxError(ErrorCode::EOFWhileParsingList,  1, 2)),
        ("[1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 3)),
        ("[1,]", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 4)),
        ("[1 2]", Error::SyntaxError(ErrorCode::ExpectedListCommaOrEnd, 1, 4)),
        ("[]a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
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
    test_parse_err::<BTreeMap<String, u32>>(vec![
        ("{", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 1)),
        ("{ ", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 2)),
        ("{1", Error::SyntaxError(ErrorCode::KeyMustBeAString, 1, 2)),
        ("{ \"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 4)),
        ("{\"a\" ", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\" 1", Error::SyntaxError(ErrorCode::ExpectedColon, 1, 6)),
        ("{\"a\":", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 5)),
        ("{\"a\":1", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 6)),
        ("{\"a\":1 1", Error::SyntaxError(ErrorCode::ExpectedObjectCommaOrEnd, 1, 8)),
        ("{\"a\":1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{}a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
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
        ("5", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("\"hello\"", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 7)),
        ("{\"inner\": true}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 14)),
        ("{}", Error::SyntaxError(ErrorCode::MissingField("inner"), 1, 2)),
        (r#"{"inner": [{"b": 42, "c": []}]}"#, Error::SyntaxError(ErrorCode::MissingField("a"), 1, 29)),
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
        ("{}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 2)),
        ("{\"Dog\":", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{\"Dog\":}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 8)),
        ("{\"unknown\":[]}", Error::SyntaxError(ErrorCode::UnknownField("unknown".to_string()), 1, 10)),
        ("{\"Dog\":{}}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 8)),
        ("{\"Frog\":{}}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 9)),
        ("{\"Cat\":[]}", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 9)),
    ]);
}

#[test]
fn test_parse_enum() {
    test_parse_ok(vec![
        ("{\"Dog\":[]}", Animal::Dog),
        (" { \"Dog\" : [ ] } ", Animal::Dog),
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

    test_parse_ok(vec![
        (
            concat!(
                "{",
                "  \"a\": {\"Dog\": []},",
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
    test_parse_err::<BTreeMap<String, String>>(vec![
        ("{\n  \"foo\":\n \"bar\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 3, 6)),
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
        ("{}", Error::SyntaxError(ErrorCode::MissingField("x"), 1, 2)),
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
            serializer.serialize_seq(ser::impls::SeqIteratorVisitor::new(self.0.iter(), None))
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
    assert_eq!(
        s,
        concat!(
            "[\n",
            "  [\n",
            "  ],\n",
            "  [\n",
            "  ]\n",
            "]"
        )
    );
}

#[test]
fn test_serialize_map_with_no_len() {
    #[derive(Clone, Debug, PartialEq)]
    struct Map<K, V>(BTreeMap<K, V>);

    impl<K, V> ser::Serialize for Map<K, V>
        where K: ser::Serialize + Ord,
              V: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: ser::Serializer,
        {
            serializer.serialize_map(ser::impls::MapIteratorVisitor::new(self.0.iter(), None))
        }
    }

    struct Visitor<K, V> {
        marker: PhantomData<Map<K, V>>,
    }

    impl<K, V> de::Visitor for Visitor<K, V>
        where K: de::Deserialize + Eq + Ord,
              V: de::Deserialize,
    {
        type Value = Map<K, V>;

        #[inline]
        fn visit_unit<E>(&mut self) -> Result<Map<K, V>, E>
            where E: de::Error,
        {
            Ok(Map(BTreeMap::new()))
        }

        #[inline]
        fn visit_map<Visitor>(&mut self, mut visitor: Visitor) -> Result<Map<K, V>, Visitor::Error>
            where Visitor: de::MapVisitor,
        {
            let mut values = BTreeMap::new();

            while let Some((key, value)) = try!(visitor.visit()) {
                values.insert(key, value);
            }

            try!(visitor.end());

            Ok(Map(values))
        }
    }

    impl<K, V> de::Deserialize for Map<K, V>
        where K: de::Deserialize + Eq + Ord,
              V: de::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Map<K, V>, D::Error>
            where D: de::Deserializer,
        {
            deserializer.deserialize_map(Visitor { marker: PhantomData })
        }
    }

    let mut map = BTreeMap::new();
    map.insert("a", Map(BTreeMap::new()));
    map.insert("b", Map(BTreeMap::new()));
    let map: Map<_, Map<u32, u32>> = Map(map);

    test_encode_ok(&[
        (
            map.clone(),
            "{\"a\":{},\"b\":{}}",
        ),
    ]);

    let s = serde_json::to_string_pretty(&map).unwrap();
    assert_eq!(
        s,
        concat!(
            "{\n",
            "  \"a\": {\n",
            "  },\n",
            "  \"b\": {\n",
            "  }\n",
            "}"
        )
    );
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
fn test_serialize_rejects_non_key_maps() {
    let map = treemap!(
        1 => 2,
        3 => 4
    );

    match serde_json::to_vec(&map).unwrap_err() {
        serde_json::Error::SyntaxError(serde_json::ErrorCode::KeyMustBeAString, 0, 0) => {}
        _ => panic!("integers used as keys"),
    }
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
