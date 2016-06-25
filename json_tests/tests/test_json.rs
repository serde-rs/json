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
    StreamDeserializer,
    Value,
    Map,
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

        // special characters
        ("\u{0}", "\"\\u0000\""),
        ("\u{1F}", "\"\\u001F\""),
        ("\u{7F}", "\"\\u007F\""),
        ("\u{80}", "\"\\u0080\""),
        ("\u{9F}", "\"\\u009F\""),
        ("\"", "\"\\\"\""),
        ("\\", "\"\\\\\""),
        ("\u{8}", "\"\\b\""),
        ("\u{C}", "\"\\f\""),
        ("\n", "\"\\n\""),
        ("\r", "\"\\r\""),
        ("\t", "\"\\t\""),
        ("\u{FFFF}", "\"\u{FFFF}\""),
        ("\u{10FFFF}", "\"\u{10FFFF}\""),

        // special characters twice
        ("\u{0}\u{0}", "\"\\u0000\\u0000\""),
        ("\u{1F}\u{1F}", "\"\\u001F\\u001F\""),
        ("\u{7F}\u{7F}", "\"\\u007F\\u007F\""),
        ("\u{80}\u{80}", "\"\\u0080\\u0080\""),
        ("\u{9F}\u{9F}", "\"\\u009F\\u009F\""),
        ("\"\"", "\"\\\"\\\"\""),
        ("\\\\", "\"\\\\\\\\\""),
        ("\u{8}\u{8}", "\"\\b\\b\""),
        ("\u{C}\u{C}", "\"\\f\\f\""),
        ("\n\n", "\"\\n\\n\""),
        ("\r\r", "\"\\r\\r\""),
        ("\t\t", "\"\\t\\t\""),
        ("\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{FFFF}\""),
        ("\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with a one unit code utf-8 character
        ("a\u{0}a\u{0}\u{0}a", "\"a\\u0000a\\u0000\\u0000a\""),
        ("\u{0}a\u{0}\u{0}a", "\"\\u0000a\\u0000\\u0000a\""),
        ("a\u{0}a\u{0}\u{0}", "\"a\\u0000a\\u0000\\u0000\""),
        ("\u{0}a\u{0}\u{0}", "\"\\u0000a\\u0000\\u0000\""),
        ("a\u{1F}a\u{1F}\u{1F}a", "\"a\\u001Fa\\u001F\\u001Fa\""),
        ("\u{1F}a\u{1F}\u{1F}a", "\"\\u001Fa\\u001F\\u001Fa\""),
        ("a\u{1F}a\u{1F}\u{1F}", "\"a\\u001Fa\\u001F\\u001F\""),
        ("\u{1F}a\u{1F}\u{1F}", "\"\\u001Fa\\u001F\\u001F\""),
        ("a\u{7F}a\u{7F}\u{7F}a", "\"a\\u007Fa\\u007F\\u007Fa\""),
        ("\u{7F}a\u{7F}\u{7F}a", "\"\\u007Fa\\u007F\\u007Fa\""),
        ("a\u{7F}a\u{7F}\u{7F}", "\"a\\u007Fa\\u007F\\u007F\""),
        ("\u{7F}a\u{7F}\u{7F}", "\"\\u007Fa\\u007F\\u007F\""),
        ("a\u{80}a\u{80}\u{80}a", "\"a\\u0080a\\u0080\\u0080a\""),
        ("\u{80}a\u{80}\u{80}a", "\"\\u0080a\\u0080\\u0080a\""),
        ("a\u{80}a\u{80}\u{80}", "\"a\\u0080a\\u0080\\u0080\""),
        ("\u{80}a\u{80}\u{80}", "\"\\u0080a\\u0080\\u0080\""),
        ("a\u{9F}a\u{9F}\u{9F}a", "\"a\\u009Fa\\u009F\\u009Fa\""),
        ("\u{9F}a\u{9F}\u{9F}a", "\"\\u009Fa\\u009F\\u009Fa\""),
        ("a\u{9F}a\u{9F}\u{9F}", "\"a\\u009Fa\\u009F\\u009F\""),
        ("\u{9F}a\u{9F}\u{9F}", "\"\\u009Fa\\u009F\\u009F\""),
        ("a\"a\"\"a", "\"a\\\"a\\\"\\\"a\""),
        ("\"a\"\"a", "\"\\\"a\\\"\\\"a\""),
        ("a\"a\"\"", "\"a\\\"a\\\"\\\"\""),
        ("\"a\"\"", "\"\\\"a\\\"\\\"\""),
        ("a\\a\\\\a", "\"a\\\\a\\\\\\\\a\""),
        ("\\a\\\\a", "\"\\\\a\\\\\\\\a\""),
        ("a\\a\\\\", "\"a\\\\a\\\\\\\\\""),
        ("\\a\\\\", "\"\\\\a\\\\\\\\\""),
        ("a\u{8}a\u{8}\u{8}a", "\"a\\ba\\b\\ba\""),
        ("\u{8}a\u{8}\u{8}a", "\"\\ba\\b\\ba\""),
        ("a\u{8}a\u{8}\u{8}", "\"a\\ba\\b\\b\""),
        ("\u{8}a\u{8}\u{8}", "\"\\ba\\b\\b\""),
        ("a\u{C}a\u{C}\u{C}a", "\"a\\fa\\f\\fa\""),
        ("\u{C}a\u{C}\u{C}a", "\"\\fa\\f\\fa\""),
        ("a\u{C}a\u{C}\u{C}", "\"a\\fa\\f\\f\""),
        ("\u{C}a\u{C}\u{C}", "\"\\fa\\f\\f\""),
        ("a\na\n\na", "\"a\\na\\n\\na\""),
        ("\na\n\na", "\"\\na\\n\\na\""),
        ("a\na\n\n", "\"a\\na\\n\\n\""),
        ("\na\n\n", "\"\\na\\n\\n\""),
        ("a\ra\r\ra", "\"a\\ra\\r\\ra\""),
        ("\ra\r\ra", "\"\\ra\\r\\ra\""),
        ("a\ra\r\r", "\"a\\ra\\r\\r\""),
        ("\ra\r\r", "\"\\ra\\r\\r\""),
        ("a\ta\t\ta", "\"a\\ta\\t\\ta\""),
        ("\ta\t\ta", "\"\\ta\\t\\ta\""),
        ("a\ta\t\t", "\"a\\ta\\t\\t\""),
        ("\ta\t\t", "\"\\ta\\t\\t\""),
        ("a\u{FFFF}a\u{FFFF}\u{FFFF}a", "\"a\u{FFFF}a\u{FFFF}\u{FFFF}a\""),
        ("\u{FFFF}a\u{FFFF}\u{FFFF}a", "\"\u{FFFF}a\u{FFFF}\u{FFFF}a\""),
        ("a\u{FFFF}a\u{FFFF}\u{FFFF}", "\"a\u{FFFF}a\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}a\u{FFFF}\u{FFFF}", "\"\u{FFFF}a\u{FFFF}\u{FFFF}\""),
        ("a\u{10FFFF}a\u{10FFFF}\u{10FFFF}a", "\"a\u{10FFFF}a\u{10FFFF}\u{10FFFF}a\""),
        ("\u{10FFFF}a\u{10FFFF}\u{10FFFF}a", "\"\u{10FFFF}a\u{10FFFF}\u{10FFFF}a\""),
        ("a\u{10FFFF}a\u{10FFFF}\u{10FFFF}", "\"a\u{10FFFF}a\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}a\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}a\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with multiple one unit code utf-8 character
        ("foo\u{0}foo\u{0}\u{0}foo", "\"foo\\u0000foo\\u0000\\u0000foo\""),
        ("\u{0}foo\u{0}\u{0}foo", "\"\\u0000foo\\u0000\\u0000foo\""),
        ("foo\u{0}foo\u{0}\u{0}", "\"foo\\u0000foo\\u0000\\u0000\""),
        ("\u{0}foo\u{0}\u{0}", "\"\\u0000foo\\u0000\\u0000\""),
        ("foo\u{1F}foo\u{1F}\u{1F}foo", "\"foo\\u001Ffoo\\u001F\\u001Ffoo\""),
        ("\u{1F}foo\u{1F}\u{1F}foo", "\"\\u001Ffoo\\u001F\\u001Ffoo\""),
        ("foo\u{1F}foo\u{1F}\u{1F}", "\"foo\\u001Ffoo\\u001F\\u001F\""),
        ("\u{1F}foo\u{1F}\u{1F}", "\"\\u001Ffoo\\u001F\\u001F\""),
        ("foo\u{7F}foo\u{7F}\u{7F}foo", "\"foo\\u007Ffoo\\u007F\\u007Ffoo\""),
        ("\u{7F}foo\u{7F}\u{7F}foo", "\"\\u007Ffoo\\u007F\\u007Ffoo\""),
        ("foo\u{7F}foo\u{7F}\u{7F}", "\"foo\\u007Ffoo\\u007F\\u007F\""),
        ("\u{7F}foo\u{7F}\u{7F}", "\"\\u007Ffoo\\u007F\\u007F\""),
        ("foo\u{80}foo\u{80}\u{80}foo", "\"foo\\u0080foo\\u0080\\u0080foo\""),
        ("\u{80}foo\u{80}\u{80}foo", "\"\\u0080foo\\u0080\\u0080foo\""),
        ("foo\u{80}foo\u{80}\u{80}", "\"foo\\u0080foo\\u0080\\u0080\""),
        ("\u{80}foo\u{80}\u{80}", "\"\\u0080foo\\u0080\\u0080\""),
        ("foo\u{9F}foo\u{9F}\u{9F}foo", "\"foo\\u009Ffoo\\u009F\\u009Ffoo\""),
        ("\u{9F}foo\u{9F}\u{9F}foo", "\"\\u009Ffoo\\u009F\\u009Ffoo\""),
        ("foo\u{9F}foo\u{9F}\u{9F}", "\"foo\\u009Ffoo\\u009F\\u009F\""),
        ("\u{9F}foo\u{9F}\u{9F}", "\"\\u009Ffoo\\u009F\\u009F\""),
        ("foo\"foo\"\"foo", "\"foo\\\"foo\\\"\\\"foo\""),
        ("\"foo\"\"foo", "\"\\\"foo\\\"\\\"foo\""),
        ("foo\"foo\"\"", "\"foo\\\"foo\\\"\\\"\""),
        ("\"foo\"\"", "\"\\\"foo\\\"\\\"\""),
        ("foo\\foo\\\\foo", "\"foo\\\\foo\\\\\\\\foo\""),
        ("\\foo\\\\foo", "\"\\\\foo\\\\\\\\foo\""),
        ("foo\\foo\\\\", "\"foo\\\\foo\\\\\\\\\""),
        ("\\foo\\\\", "\"\\\\foo\\\\\\\\\""),
        ("foo\u{8}foo\u{8}\u{8}foo", "\"foo\\bfoo\\b\\bfoo\""),
        ("\u{8}foo\u{8}\u{8}foo", "\"\\bfoo\\b\\bfoo\""),
        ("foo\u{8}foo\u{8}\u{8}", "\"foo\\bfoo\\b\\b\""),
        ("\u{8}foo\u{8}\u{8}", "\"\\bfoo\\b\\b\""),
        ("foo\u{C}foo\u{C}\u{C}foo", "\"foo\\ffoo\\f\\ffoo\""),
        ("\u{C}foo\u{C}\u{C}foo", "\"\\ffoo\\f\\ffoo\""),
        ("foo\u{C}foo\u{C}\u{C}", "\"foo\\ffoo\\f\\f\""),
        ("\u{C}foo\u{C}\u{C}", "\"\\ffoo\\f\\f\""),
        ("foo\nfoo\n\nfoo", "\"foo\\nfoo\\n\\nfoo\""),
        ("\nfoo\n\nfoo", "\"\\nfoo\\n\\nfoo\""),
        ("foo\nfoo\n\n", "\"foo\\nfoo\\n\\n\""),
        ("\nfoo\n\n", "\"\\nfoo\\n\\n\""),
        ("foo\rfoo\r\rfoo", "\"foo\\rfoo\\r\\rfoo\""),
        ("\rfoo\r\rfoo", "\"\\rfoo\\r\\rfoo\""),
        ("foo\rfoo\r\r", "\"foo\\rfoo\\r\\r\""),
        ("\rfoo\r\r", "\"\\rfoo\\r\\r\""),
        ("foo\tfoo\t\tfoo", "\"foo\\tfoo\\t\\tfoo\""),
        ("\tfoo\t\tfoo", "\"\\tfoo\\t\\tfoo\""),
        ("foo\tfoo\t\t", "\"foo\\tfoo\\t\\t\""),
        ("\tfoo\t\t", "\"\\tfoo\\t\\t\""),
        ("foo\u{FFFF}foo\u{FFFF}\u{FFFF}foo", "\"foo\u{FFFF}foo\u{FFFF}\u{FFFF}foo\""),
        ("\u{FFFF}foo\u{FFFF}\u{FFFF}foo", "\"\u{FFFF}foo\u{FFFF}\u{FFFF}foo\""),
        ("foo\u{FFFF}foo\u{FFFF}\u{FFFF}", "\"foo\u{FFFF}foo\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}foo\u{FFFF}\u{FFFF}", "\"\u{FFFF}foo\u{FFFF}\u{FFFF}\""),
        ("foo\u{10FFFF}foo\u{10FFFF}\u{10FFFF}foo", "\"foo\u{10FFFF}foo\u{10FFFF}\u{10FFFF}foo\""),
        ("\u{10FFFF}foo\u{10FFFF}\u{10FFFF}foo", "\"\u{10FFFF}foo\u{10FFFF}\u{10FFFF}foo\""),
        ("foo\u{10FFFF}foo\u{10FFFF}\u{10FFFF}", "\"foo\u{10FFFF}foo\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}foo\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}foo\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with a control character having a specific character replacement
        ("\u{8}\u{0}\u{8}\u{0}\u{0}\u{8}", "\"\\b\\u0000\\b\\u0000\\u0000\\b\""),
        ("\u{0}\u{8}\u{0}\u{0}\u{8}", "\"\\u0000\\b\\u0000\\u0000\\b\""),
        ("\u{8}\u{0}\u{8}\u{0}\u{0}", "\"\\b\\u0000\\b\\u0000\\u0000\""),
        ("\u{0}\u{8}\u{0}\u{0}", "\"\\u0000\\b\\u0000\\u0000\""),
        ("\u{8}\u{1F}\u{8}\u{1F}\u{1F}\u{8}", "\"\\b\\u001F\\b\\u001F\\u001F\\b\""),
        ("\u{1F}\u{8}\u{1F}\u{1F}\u{8}", "\"\\u001F\\b\\u001F\\u001F\\b\""),
        ("\u{8}\u{1F}\u{8}\u{1F}\u{1F}", "\"\\b\\u001F\\b\\u001F\\u001F\""),
        ("\u{1F}\u{8}\u{1F}\u{1F}", "\"\\u001F\\b\\u001F\\u001F\""),
        ("\u{8}\u{7F}\u{8}\u{7F}\u{7F}\u{8}", "\"\\b\\u007F\\b\\u007F\\u007F\\b\""),
        ("\u{7F}\u{8}\u{7F}\u{7F}\u{8}", "\"\\u007F\\b\\u007F\\u007F\\b\""),
        ("\u{8}\u{7F}\u{8}\u{7F}\u{7F}", "\"\\b\\u007F\\b\\u007F\\u007F\""),
        ("\u{7F}\u{8}\u{7F}\u{7F}", "\"\\u007F\\b\\u007F\\u007F\""),
        ("\u{8}\u{80}\u{8}\u{80}\u{80}\u{8}", "\"\\b\\u0080\\b\\u0080\\u0080\\b\""),
        ("\u{80}\u{8}\u{80}\u{80}\u{8}", "\"\\u0080\\b\\u0080\\u0080\\b\""),
        ("\u{8}\u{80}\u{8}\u{80}\u{80}", "\"\\b\\u0080\\b\\u0080\\u0080\""),
        ("\u{80}\u{8}\u{80}\u{80}", "\"\\u0080\\b\\u0080\\u0080\""),
        ("\u{8}\u{9F}\u{8}\u{9F}\u{9F}\u{8}", "\"\\b\\u009F\\b\\u009F\\u009F\\b\""),
        ("\u{9F}\u{8}\u{9F}\u{9F}\u{8}", "\"\\u009F\\b\\u009F\\u009F\\b\""),
        ("\u{8}\u{9F}\u{8}\u{9F}\u{9F}", "\"\\b\\u009F\\b\\u009F\\u009F\""),
        ("\u{9F}\u{8}\u{9F}\u{9F}", "\"\\u009F\\b\\u009F\\u009F\""),
        ("\u{8}\"\u{8}\"\"\u{8}", "\"\\b\\\"\\b\\\"\\\"\\b\""),
        ("\"\u{8}\"\"\u{8}", "\"\\\"\\b\\\"\\\"\\b\""),
        ("\u{8}\"\u{8}\"\"", "\"\\b\\\"\\b\\\"\\\"\""),
        ("\"\u{8}\"\"", "\"\\\"\\b\\\"\\\"\""),
        ("\u{8}\\\u{8}\\\\\u{8}", "\"\\b\\\\\\b\\\\\\\\\\b\""),
        ("\\\u{8}\\\\\u{8}", "\"\\\\\\b\\\\\\\\\\b\""),
        ("\u{8}\\\u{8}\\\\", "\"\\b\\\\\\b\\\\\\\\\""),
        ("\\\u{8}\\\\", "\"\\\\\\b\\\\\\\\\""),
        ("\u{8}\u{8}\u{8}\u{8}\u{8}\u{8}", "\"\\b\\b\\b\\b\\b\\b\""),
        ("\u{8}\u{8}\u{8}\u{8}\u{8}", "\"\\b\\b\\b\\b\\b\""),
        ("\u{8}\u{8}\u{8}\u{8}\u{8}", "\"\\b\\b\\b\\b\\b\""),
        ("\u{8}\u{8}\u{8}\u{8}", "\"\\b\\b\\b\\b\""),
        ("\u{8}\u{C}\u{8}\u{C}\u{C}\u{8}", "\"\\b\\f\\b\\f\\f\\b\""),
        ("\u{C}\u{8}\u{C}\u{C}\u{8}", "\"\\f\\b\\f\\f\\b\""),
        ("\u{8}\u{C}\u{8}\u{C}\u{C}", "\"\\b\\f\\b\\f\\f\""),
        ("\u{C}\u{8}\u{C}\u{C}", "\"\\f\\b\\f\\f\""),
        ("\u{8}\n\u{8}\n\n\u{8}", "\"\\b\\n\\b\\n\\n\\b\""),
        ("\n\u{8}\n\n\u{8}", "\"\\n\\b\\n\\n\\b\""),
        ("\u{8}\n\u{8}\n\n", "\"\\b\\n\\b\\n\\n\""),
        ("\n\u{8}\n\n", "\"\\n\\b\\n\\n\""),
        ("\u{8}\r\u{8}\r\r\u{8}", "\"\\b\\r\\b\\r\\r\\b\""),
        ("\r\u{8}\r\r\u{8}", "\"\\r\\b\\r\\r\\b\""),
        ("\u{8}\r\u{8}\r\r", "\"\\b\\r\\b\\r\\r\""),
        ("\r\u{8}\r\r", "\"\\r\\b\\r\\r\""),
        ("\u{8}\t\u{8}\t\t\u{8}", "\"\\b\\t\\b\\t\\t\\b\""),
        ("\t\u{8}\t\t\u{8}", "\"\\t\\b\\t\\t\\b\""),
        ("\u{8}\t\u{8}\t\t", "\"\\b\\t\\b\\t\\t\""),
        ("\t\u{8}\t\t", "\"\\t\\b\\t\\t\""),
        ("\u{8}\u{FFFF}\u{8}\u{FFFF}\u{FFFF}\u{8}", "\"\\b\u{FFFF}\\b\u{FFFF}\u{FFFF}\\b\""),
        ("\u{FFFF}\u{8}\u{FFFF}\u{FFFF}\u{8}", "\"\u{FFFF}\\b\u{FFFF}\u{FFFF}\\b\""),
        ("\u{8}\u{FFFF}\u{8}\u{FFFF}\u{FFFF}", "\"\\b\u{FFFF}\\b\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{8}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\\b\u{FFFF}\u{FFFF}\""),
        ("\u{8}\u{10FFFF}\u{8}\u{10FFFF}\u{10FFFF}\u{8}", "\"\\b\u{10FFFF}\\b\u{10FFFF}\u{10FFFF}\\b\""),
        ("\u{10FFFF}\u{8}\u{10FFFF}\u{10FFFF}\u{8}", "\"\u{10FFFF}\\b\u{10FFFF}\u{10FFFF}\\b\""),
        ("\u{8}\u{10FFFF}\u{8}\u{10FFFF}\u{10FFFF}", "\"\\b\u{10FFFF}\\b\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{8}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\\b\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with a one unit code utf-8 control character
        ("\u{7F}\u{0}\u{7F}\u{0}\u{0}\u{7F}", "\"\\u007F\\u0000\\u007F\\u0000\\u0000\\u007F\""),
        ("\u{0}\u{7F}\u{0}\u{0}\u{7F}", "\"\\u0000\\u007F\\u0000\\u0000\\u007F\""),
        ("\u{7F}\u{0}\u{7F}\u{0}\u{0}", "\"\\u007F\\u0000\\u007F\\u0000\\u0000\""),
        ("\u{0}\u{7F}\u{0}\u{0}", "\"\\u0000\\u007F\\u0000\\u0000\""),
        ("\u{7F}\u{1F}\u{7F}\u{1F}\u{1F}\u{7F}", "\"\\u007F\\u001F\\u007F\\u001F\\u001F\\u007F\""),
        ("\u{1F}\u{7F}\u{1F}\u{1F}\u{7F}", "\"\\u001F\\u007F\\u001F\\u001F\\u007F\""),
        ("\u{7F}\u{1F}\u{7F}\u{1F}\u{1F}", "\"\\u007F\\u001F\\u007F\\u001F\\u001F\""),
        ("\u{1F}\u{7F}\u{1F}\u{1F}", "\"\\u001F\\u007F\\u001F\\u001F\""),
        ("\u{7F}\u{7F}\u{7F}\u{7F}\u{7F}\u{7F}", "\"\\u007F\\u007F\\u007F\\u007F\\u007F\\u007F\""),
        ("\u{7F}\u{7F}\u{7F}\u{7F}\u{7F}", "\"\\u007F\\u007F\\u007F\\u007F\\u007F\""),
        ("\u{7F}\u{7F}\u{7F}\u{7F}\u{7F}", "\"\\u007F\\u007F\\u007F\\u007F\\u007F\""),
        ("\u{7F}\u{7F}\u{7F}\u{7F}", "\"\\u007F\\u007F\\u007F\\u007F\""),
        ("\u{7F}\u{80}\u{7F}\u{80}\u{80}\u{7F}", "\"\\u007F\\u0080\\u007F\\u0080\\u0080\\u007F\""),
        ("\u{80}\u{7F}\u{80}\u{80}\u{7F}", "\"\\u0080\\u007F\\u0080\\u0080\\u007F\""),
        ("\u{7F}\u{80}\u{7F}\u{80}\u{80}", "\"\\u007F\\u0080\\u007F\\u0080\\u0080\""),
        ("\u{80}\u{7F}\u{80}\u{80}", "\"\\u0080\\u007F\\u0080\\u0080\""),
        ("\u{7F}\u{9F}\u{7F}\u{9F}\u{9F}\u{7F}", "\"\\u007F\\u009F\\u007F\\u009F\\u009F\\u007F\""),
        ("\u{9F}\u{7F}\u{9F}\u{9F}\u{7F}", "\"\\u009F\\u007F\\u009F\\u009F\\u007F\""),
        ("\u{7F}\u{9F}\u{7F}\u{9F}\u{9F}", "\"\\u007F\\u009F\\u007F\\u009F\\u009F\""),
        ("\u{9F}\u{7F}\u{9F}\u{9F}", "\"\\u009F\\u007F\\u009F\\u009F\""),
        ("\u{7F}\"\u{7F}\"\"\u{7F}", "\"\\u007F\\\"\\u007F\\\"\\\"\\u007F\""),
        ("\"\u{7F}\"\"\u{7F}", "\"\\\"\\u007F\\\"\\\"\\u007F\""),
        ("\u{7F}\"\u{7F}\"\"", "\"\\u007F\\\"\\u007F\\\"\\\"\""),
        ("\"\u{7F}\"\"", "\"\\\"\\u007F\\\"\\\"\""),
        ("\u{7F}\\\u{7F}\\\\\u{7F}", "\"\\u007F\\\\\\u007F\\\\\\\\\\u007F\""),
        ("\\\u{7F}\\\\\u{7F}", "\"\\\\\\u007F\\\\\\\\\\u007F\""),
        ("\u{7F}\\\u{7F}\\\\", "\"\\u007F\\\\\\u007F\\\\\\\\\""),
        ("\\\u{7F}\\\\", "\"\\\\\\u007F\\\\\\\\\""),
        ("\u{7F}\u{8}\u{7F}\u{8}\u{8}\u{7F}", "\"\\u007F\\b\\u007F\\b\\b\\u007F\""),
        ("\u{8}\u{7F}\u{8}\u{8}\u{7F}", "\"\\b\\u007F\\b\\b\\u007F\""),
        ("\u{7F}\u{8}\u{7F}\u{8}\u{8}", "\"\\u007F\\b\\u007F\\b\\b\""),
        ("\u{8}\u{7F}\u{8}\u{8}", "\"\\b\\u007F\\b\\b\""),
        ("\u{7F}\u{C}\u{7F}\u{C}\u{C}\u{7F}", "\"\\u007F\\f\\u007F\\f\\f\\u007F\""),
        ("\u{C}\u{7F}\u{C}\u{C}\u{7F}", "\"\\f\\u007F\\f\\f\\u007F\""),
        ("\u{7F}\u{C}\u{7F}\u{C}\u{C}", "\"\\u007F\\f\\u007F\\f\\f\""),
        ("\u{C}\u{7F}\u{C}\u{C}", "\"\\f\\u007F\\f\\f\""),
        ("\u{7F}\n\u{7F}\n\n\u{7F}", "\"\\u007F\\n\\u007F\\n\\n\\u007F\""),
        ("\n\u{7F}\n\n\u{7F}", "\"\\n\\u007F\\n\\n\\u007F\""),
        ("\u{7F}\n\u{7F}\n\n", "\"\\u007F\\n\\u007F\\n\\n\""),
        ("\n\u{7F}\n\n", "\"\\n\\u007F\\n\\n\""),
        ("\u{7F}\r\u{7F}\r\r\u{7F}", "\"\\u007F\\r\\u007F\\r\\r\\u007F\""),
        ("\r\u{7F}\r\r\u{7F}", "\"\\r\\u007F\\r\\r\\u007F\""),
        ("\u{7F}\r\u{7F}\r\r", "\"\\u007F\\r\\u007F\\r\\r\""),
        ("\r\u{7F}\r\r", "\"\\r\\u007F\\r\\r\""),
        ("\u{7F}\t\u{7F}\t\t\u{7F}", "\"\\u007F\\t\\u007F\\t\\t\\u007F\""),
        ("\t\u{7F}\t\t\u{7F}", "\"\\t\\u007F\\t\\t\\u007F\""),
        ("\u{7F}\t\u{7F}\t\t", "\"\\u007F\\t\\u007F\\t\\t\""),
        ("\t\u{7F}\t\t", "\"\\t\\u007F\\t\\t\""),
        ("\u{7F}\u{FFFF}\u{7F}\u{FFFF}\u{FFFF}\u{7F}", "\"\\u007F\u{FFFF}\\u007F\u{FFFF}\u{FFFF}\\u007F\""),
        ("\u{FFFF}\u{7F}\u{FFFF}\u{FFFF}\u{7F}", "\"\u{FFFF}\\u007F\u{FFFF}\u{FFFF}\\u007F\""),
        ("\u{7F}\u{FFFF}\u{7F}\u{FFFF}\u{FFFF}", "\"\\u007F\u{FFFF}\\u007F\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{7F}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\\u007F\u{FFFF}\u{FFFF}\""),
        ("\u{7F}\u{10FFFF}\u{7F}\u{10FFFF}\u{10FFFF}\u{7F}", "\"\\u007F\u{10FFFF}\\u007F\u{10FFFF}\u{10FFFF}\\u007F\""),
        ("\u{10FFFF}\u{7F}\u{10FFFF}\u{10FFFF}\u{7F}", "\"\u{10FFFF}\\u007F\u{10FFFF}\u{10FFFF}\\u007F\""),
        ("\u{7F}\u{10FFFF}\u{7F}\u{10FFFF}\u{10FFFF}", "\"\\u007F\u{10FFFF}\\u007F\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{7F}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\\u007F\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with a two unit code utf-8 control character
        ("\u{9F}\u{0}\u{9F}\u{0}\u{0}\u{9F}", "\"\\u009F\\u0000\\u009F\\u0000\\u0000\\u009F\""),
        ("\u{0}\u{9F}\u{0}\u{0}\u{9F}", "\"\\u0000\\u009F\\u0000\\u0000\\u009F\""),
        ("\u{9F}\u{0}\u{9F}\u{0}\u{0}", "\"\\u009F\\u0000\\u009F\\u0000\\u0000\""),
        ("\u{0}\u{9F}\u{0}\u{0}", "\"\\u0000\\u009F\\u0000\\u0000\""),
        ("\u{9F}\u{1F}\u{9F}\u{1F}\u{1F}\u{9F}", "\"\\u009F\\u001F\\u009F\\u001F\\u001F\\u009F\""),
        ("\u{1F}\u{9F}\u{1F}\u{1F}\u{9F}", "\"\\u001F\\u009F\\u001F\\u001F\\u009F\""),
        ("\u{9F}\u{1F}\u{9F}\u{1F}\u{1F}", "\"\\u009F\\u001F\\u009F\\u001F\\u001F\""),
        ("\u{1F}\u{9F}\u{1F}\u{1F}", "\"\\u001F\\u009F\\u001F\\u001F\""),
        ("\u{9F}\u{7F}\u{9F}\u{7F}\u{7F}\u{9F}", "\"\\u009F\\u007F\\u009F\\u007F\\u007F\\u009F\""),
        ("\u{7F}\u{9F}\u{7F}\u{7F}\u{9F}", "\"\\u007F\\u009F\\u007F\\u007F\\u009F\""),
        ("\u{9F}\u{7F}\u{9F}\u{7F}\u{7F}", "\"\\u009F\\u007F\\u009F\\u007F\\u007F\""),
        ("\u{7F}\u{9F}\u{7F}\u{7F}", "\"\\u007F\\u009F\\u007F\\u007F\""),
        ("\u{9F}\u{80}\u{9F}\u{80}\u{80}\u{9F}", "\"\\u009F\\u0080\\u009F\\u0080\\u0080\\u009F\""),
        ("\u{80}\u{9F}\u{80}\u{80}\u{9F}", "\"\\u0080\\u009F\\u0080\\u0080\\u009F\""),
        ("\u{9F}\u{80}\u{9F}\u{80}\u{80}", "\"\\u009F\\u0080\\u009F\\u0080\\u0080\""),
        ("\u{80}\u{9F}\u{80}\u{80}", "\"\\u0080\\u009F\\u0080\\u0080\""),
        ("\u{9F}\u{9F}\u{9F}\u{9F}\u{9F}\u{9F}", "\"\\u009F\\u009F\\u009F\\u009F\\u009F\\u009F\""),
        ("\u{9F}\u{9F}\u{9F}\u{9F}\u{9F}", "\"\\u009F\\u009F\\u009F\\u009F\\u009F\""),
        ("\u{9F}\u{9F}\u{9F}\u{9F}\u{9F}", "\"\\u009F\\u009F\\u009F\\u009F\\u009F\""),
        ("\u{9F}\u{9F}\u{9F}\u{9F}", "\"\\u009F\\u009F\\u009F\\u009F\""),
        ("\u{9F}\"\u{9F}\"\"\u{9F}", "\"\\u009F\\\"\\u009F\\\"\\\"\\u009F\""),
        ("\"\u{9F}\"\"\u{9F}", "\"\\\"\\u009F\\\"\\\"\\u009F\""),
        ("\u{9F}\"\u{9F}\"\"", "\"\\u009F\\\"\\u009F\\\"\\\"\""),
        ("\"\u{9F}\"\"", "\"\\\"\\u009F\\\"\\\"\""),
        ("\u{9F}\\\u{9F}\\\\\u{9F}", "\"\\u009F\\\\\\u009F\\\\\\\\\\u009F\""),
        ("\\\u{9F}\\\\\u{9F}", "\"\\\\\\u009F\\\\\\\\\\u009F\""),
        ("\u{9F}\\\u{9F}\\\\", "\"\\u009F\\\\\\u009F\\\\\\\\\""),
        ("\\\u{9F}\\\\", "\"\\\\\\u009F\\\\\\\\\""),
        ("\u{9F}\u{8}\u{9F}\u{8}\u{8}\u{9F}", "\"\\u009F\\b\\u009F\\b\\b\\u009F\""),
        ("\u{8}\u{9F}\u{8}\u{8}\u{9F}", "\"\\b\\u009F\\b\\b\\u009F\""),
        ("\u{9F}\u{8}\u{9F}\u{8}\u{8}", "\"\\u009F\\b\\u009F\\b\\b\""),
        ("\u{8}\u{9F}\u{8}\u{8}", "\"\\b\\u009F\\b\\b\""),
        ("\u{9F}\u{C}\u{9F}\u{C}\u{C}\u{9F}", "\"\\u009F\\f\\u009F\\f\\f\\u009F\""),
        ("\u{C}\u{9F}\u{C}\u{C}\u{9F}", "\"\\f\\u009F\\f\\f\\u009F\""),
        ("\u{9F}\u{C}\u{9F}\u{C}\u{C}", "\"\\u009F\\f\\u009F\\f\\f\""),
        ("\u{C}\u{9F}\u{C}\u{C}", "\"\\f\\u009F\\f\\f\""),
        ("\u{9F}\n\u{9F}\n\n\u{9F}", "\"\\u009F\\n\\u009F\\n\\n\\u009F\""),
        ("\n\u{9F}\n\n\u{9F}", "\"\\n\\u009F\\n\\n\\u009F\""),
        ("\u{9F}\n\u{9F}\n\n", "\"\\u009F\\n\\u009F\\n\\n\""),
        ("\n\u{9F}\n\n", "\"\\n\\u009F\\n\\n\""),
        ("\u{9F}\r\u{9F}\r\r\u{9F}", "\"\\u009F\\r\\u009F\\r\\r\\u009F\""),
        ("\r\u{9F}\r\r\u{9F}", "\"\\r\\u009F\\r\\r\\u009F\""),
        ("\u{9F}\r\u{9F}\r\r", "\"\\u009F\\r\\u009F\\r\\r\""),
        ("\r\u{9F}\r\r", "\"\\r\\u009F\\r\\r\""),
        ("\u{9F}\t\u{9F}\t\t\u{9F}", "\"\\u009F\\t\\u009F\\t\\t\\u009F\""),
        ("\t\u{9F}\t\t\u{9F}", "\"\\t\\u009F\\t\\t\\u009F\""),
        ("\u{9F}\t\u{9F}\t\t", "\"\\u009F\\t\\u009F\\t\\t\""),
        ("\t\u{9F}\t\t", "\"\\t\\u009F\\t\\t\""),
        ("\u{9F}\u{FFFF}\u{9F}\u{FFFF}\u{FFFF}\u{9F}", "\"\\u009F\u{FFFF}\\u009F\u{FFFF}\u{FFFF}\\u009F\""),
        ("\u{FFFF}\u{9F}\u{FFFF}\u{FFFF}\u{9F}", "\"\u{FFFF}\\u009F\u{FFFF}\u{FFFF}\\u009F\""),
        ("\u{9F}\u{FFFF}\u{9F}\u{FFFF}\u{FFFF}", "\"\\u009F\u{FFFF}\\u009F\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{9F}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\\u009F\u{FFFF}\u{FFFF}\""),
        ("\u{9F}\u{10FFFF}\u{9F}\u{10FFFF}\u{10FFFF}\u{9F}", "\"\\u009F\u{10FFFF}\\u009F\u{10FFFF}\u{10FFFF}\\u009F\""),
        ("\u{10FFFF}\u{9F}\u{10FFFF}\u{10FFFF}\u{9F}", "\"\u{10FFFF}\\u009F\u{10FFFF}\u{10FFFF}\\u009F\""),
        ("\u{9F}\u{10FFFF}\u{9F}\u{10FFFF}\u{10FFFF}", "\"\\u009F\u{10FFFF}\\u009F\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{9F}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\\u009F\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with the last BMP character
        ("\u{FFFF}\u{0}\u{FFFF}\u{0}\u{0}\u{FFFF}", "\"\u{FFFF}\\u0000\u{FFFF}\\u0000\\u0000\u{FFFF}\""),
        ("\u{0}\u{FFFF}\u{0}\u{0}\u{FFFF}", "\"\\u0000\u{FFFF}\\u0000\\u0000\u{FFFF}\""),
        ("\u{FFFF}\u{0}\u{FFFF}\u{0}\u{0}", "\"\u{FFFF}\\u0000\u{FFFF}\\u0000\\u0000\""),
        ("\u{0}\u{FFFF}\u{0}\u{0}", "\"\\u0000\u{FFFF}\\u0000\\u0000\""),
        ("\u{FFFF}\u{1F}\u{FFFF}\u{1F}\u{1F}\u{FFFF}", "\"\u{FFFF}\\u001F\u{FFFF}\\u001F\\u001F\u{FFFF}\""),
        ("\u{1F}\u{FFFF}\u{1F}\u{1F}\u{FFFF}", "\"\\u001F\u{FFFF}\\u001F\\u001F\u{FFFF}\""),
        ("\u{FFFF}\u{1F}\u{FFFF}\u{1F}\u{1F}", "\"\u{FFFF}\\u001F\u{FFFF}\\u001F\\u001F\""),
        ("\u{1F}\u{FFFF}\u{1F}\u{1F}", "\"\\u001F\u{FFFF}\\u001F\\u001F\""),
        ("\u{FFFF}\u{7F}\u{FFFF}\u{7F}\u{7F}\u{FFFF}", "\"\u{FFFF}\\u007F\u{FFFF}\\u007F\\u007F\u{FFFF}\""),
        ("\u{7F}\u{FFFF}\u{7F}\u{7F}\u{FFFF}", "\"\\u007F\u{FFFF}\\u007F\\u007F\u{FFFF}\""),
        ("\u{FFFF}\u{7F}\u{FFFF}\u{7F}\u{7F}", "\"\u{FFFF}\\u007F\u{FFFF}\\u007F\\u007F\""),
        ("\u{7F}\u{FFFF}\u{7F}\u{7F}", "\"\\u007F\u{FFFF}\\u007F\\u007F\""),
        ("\u{FFFF}\u{80}\u{FFFF}\u{80}\u{80}\u{FFFF}", "\"\u{FFFF}\\u0080\u{FFFF}\\u0080\\u0080\u{FFFF}\""),
        ("\u{80}\u{FFFF}\u{80}\u{80}\u{FFFF}", "\"\\u0080\u{FFFF}\\u0080\\u0080\u{FFFF}\""),
        ("\u{FFFF}\u{80}\u{FFFF}\u{80}\u{80}", "\"\u{FFFF}\\u0080\u{FFFF}\\u0080\\u0080\""),
        ("\u{80}\u{FFFF}\u{80}\u{80}", "\"\\u0080\u{FFFF}\\u0080\\u0080\""),
        ("\u{FFFF}\u{9F}\u{FFFF}\u{9F}\u{9F}\u{FFFF}", "\"\u{FFFF}\\u009F\u{FFFF}\\u009F\\u009F\u{FFFF}\""),
        ("\u{9F}\u{FFFF}\u{9F}\u{9F}\u{FFFF}", "\"\\u009F\u{FFFF}\\u009F\\u009F\u{FFFF}\""),
        ("\u{FFFF}\u{9F}\u{FFFF}\u{9F}\u{9F}", "\"\u{FFFF}\\u009F\u{FFFF}\\u009F\\u009F\""),
        ("\u{9F}\u{FFFF}\u{9F}\u{9F}", "\"\\u009F\u{FFFF}\\u009F\\u009F\""),
        ("\u{FFFF}\"\u{FFFF}\"\"\u{FFFF}", "\"\u{FFFF}\\\"\u{FFFF}\\\"\\\"\u{FFFF}\""),
        ("\"\u{FFFF}\"\"\u{FFFF}", "\"\\\"\u{FFFF}\\\"\\\"\u{FFFF}\""),
        ("\u{FFFF}\"\u{FFFF}\"\"", "\"\u{FFFF}\\\"\u{FFFF}\\\"\\\"\""),
        ("\"\u{FFFF}\"\"", "\"\\\"\u{FFFF}\\\"\\\"\""),
        ("\u{FFFF}\\\u{FFFF}\\\\\u{FFFF}", "\"\u{FFFF}\\\\\u{FFFF}\\\\\\\\\u{FFFF}\""),
        ("\\\u{FFFF}\\\\\u{FFFF}", "\"\\\\\u{FFFF}\\\\\\\\\u{FFFF}\""),
        ("\u{FFFF}\\\u{FFFF}\\\\", "\"\u{FFFF}\\\\\u{FFFF}\\\\\\\\\""),
        ("\\\u{FFFF}\\\\", "\"\\\\\u{FFFF}\\\\\\\\\""),
        ("\u{FFFF}\u{8}\u{FFFF}\u{8}\u{8}\u{FFFF}", "\"\u{FFFF}\\b\u{FFFF}\\b\\b\u{FFFF}\""),
        ("\u{8}\u{FFFF}\u{8}\u{8}\u{FFFF}", "\"\\b\u{FFFF}\\b\\b\u{FFFF}\""),
        ("\u{FFFF}\u{8}\u{FFFF}\u{8}\u{8}", "\"\u{FFFF}\\b\u{FFFF}\\b\\b\""),
        ("\u{8}\u{FFFF}\u{8}\u{8}", "\"\\b\u{FFFF}\\b\\b\""),
        ("\u{FFFF}\u{C}\u{FFFF}\u{C}\u{C}\u{FFFF}", "\"\u{FFFF}\\f\u{FFFF}\\f\\f\u{FFFF}\""),
        ("\u{C}\u{FFFF}\u{C}\u{C}\u{FFFF}", "\"\\f\u{FFFF}\\f\\f\u{FFFF}\""),
        ("\u{FFFF}\u{C}\u{FFFF}\u{C}\u{C}", "\"\u{FFFF}\\f\u{FFFF}\\f\\f\""),
        ("\u{C}\u{FFFF}\u{C}\u{C}", "\"\\f\u{FFFF}\\f\\f\""),
        ("\u{FFFF}\n\u{FFFF}\n\n\u{FFFF}", "\"\u{FFFF}\\n\u{FFFF}\\n\\n\u{FFFF}\""),
        ("\n\u{FFFF}\n\n\u{FFFF}", "\"\\n\u{FFFF}\\n\\n\u{FFFF}\""),
        ("\u{FFFF}\n\u{FFFF}\n\n", "\"\u{FFFF}\\n\u{FFFF}\\n\\n\""),
        ("\n\u{FFFF}\n\n", "\"\\n\u{FFFF}\\n\\n\""),
        ("\u{FFFF}\r\u{FFFF}\r\r\u{FFFF}", "\"\u{FFFF}\\r\u{FFFF}\\r\\r\u{FFFF}\""),
        ("\r\u{FFFF}\r\r\u{FFFF}", "\"\\r\u{FFFF}\\r\\r\u{FFFF}\""),
        ("\u{FFFF}\r\u{FFFF}\r\r", "\"\u{FFFF}\\r\u{FFFF}\\r\\r\""),
        ("\r\u{FFFF}\r\r", "\"\\r\u{FFFF}\\r\\r\""),
        ("\u{FFFF}\t\u{FFFF}\t\t\u{FFFF}", "\"\u{FFFF}\\t\u{FFFF}\\t\\t\u{FFFF}\""),
        ("\t\u{FFFF}\t\t\u{FFFF}", "\"\\t\u{FFFF}\\t\\t\u{FFFF}\""),
        ("\u{FFFF}\t\u{FFFF}\t\t", "\"\u{FFFF}\\t\u{FFFF}\\t\\t\""),
        ("\t\u{FFFF}\t\t", "\"\\t\u{FFFF}\\t\\t\""),
        ("\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\u{FFFF}", "\"\u{FFFF}\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\u{FFFF}\""),
        ("\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\u{FFFF}", "\"\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{FFFF}\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{FFFF}\u{10FFFF}\u{10FFFF}\""),

        // patterns of special characters interleaved with the last Unicode character (non-BMP)
        ("\u{10FFFF}\u{0}\u{10FFFF}\u{0}\u{0}\u{10FFFF}", "\"\u{10FFFF}\\u0000\u{10FFFF}\\u0000\\u0000\u{10FFFF}\""),
        ("\u{0}\u{10FFFF}\u{0}\u{0}\u{10FFFF}", "\"\\u0000\u{10FFFF}\\u0000\\u0000\u{10FFFF}\""),
        ("\u{10FFFF}\u{0}\u{10FFFF}\u{0}\u{0}", "\"\u{10FFFF}\\u0000\u{10FFFF}\\u0000\\u0000\""),
        ("\u{0}\u{10FFFF}\u{0}\u{0}", "\"\\u0000\u{10FFFF}\\u0000\\u0000\""),
        ("\u{10FFFF}\u{1F}\u{10FFFF}\u{1F}\u{1F}\u{10FFFF}", "\"\u{10FFFF}\\u001F\u{10FFFF}\\u001F\\u001F\u{10FFFF}\""),
        ("\u{1F}\u{10FFFF}\u{1F}\u{1F}\u{10FFFF}", "\"\\u001F\u{10FFFF}\\u001F\\u001F\u{10FFFF}\""),
        ("\u{10FFFF}\u{1F}\u{10FFFF}\u{1F}\u{1F}", "\"\u{10FFFF}\\u001F\u{10FFFF}\\u001F\\u001F\""),
        ("\u{1F}\u{10FFFF}\u{1F}\u{1F}", "\"\\u001F\u{10FFFF}\\u001F\\u001F\""),
        ("\u{10FFFF}\u{7F}\u{10FFFF}\u{7F}\u{7F}\u{10FFFF}", "\"\u{10FFFF}\\u007F\u{10FFFF}\\u007F\\u007F\u{10FFFF}\""),
        ("\u{7F}\u{10FFFF}\u{7F}\u{7F}\u{10FFFF}", "\"\\u007F\u{10FFFF}\\u007F\\u007F\u{10FFFF}\""),
        ("\u{10FFFF}\u{7F}\u{10FFFF}\u{7F}\u{7F}", "\"\u{10FFFF}\\u007F\u{10FFFF}\\u007F\\u007F\""),
        ("\u{7F}\u{10FFFF}\u{7F}\u{7F}", "\"\\u007F\u{10FFFF}\\u007F\\u007F\""),
        ("\u{10FFFF}\u{80}\u{10FFFF}\u{80}\u{80}\u{10FFFF}", "\"\u{10FFFF}\\u0080\u{10FFFF}\\u0080\\u0080\u{10FFFF}\""),
        ("\u{80}\u{10FFFF}\u{80}\u{80}\u{10FFFF}", "\"\\u0080\u{10FFFF}\\u0080\\u0080\u{10FFFF}\""),
        ("\u{10FFFF}\u{80}\u{10FFFF}\u{80}\u{80}", "\"\u{10FFFF}\\u0080\u{10FFFF}\\u0080\\u0080\""),
        ("\u{80}\u{10FFFF}\u{80}\u{80}", "\"\\u0080\u{10FFFF}\\u0080\\u0080\""),
        ("\u{10FFFF}\u{9F}\u{10FFFF}\u{9F}\u{9F}\u{10FFFF}", "\"\u{10FFFF}\\u009F\u{10FFFF}\\u009F\\u009F\u{10FFFF}\""),
        ("\u{9F}\u{10FFFF}\u{9F}\u{9F}\u{10FFFF}", "\"\\u009F\u{10FFFF}\\u009F\\u009F\u{10FFFF}\""),
        ("\u{10FFFF}\u{9F}\u{10FFFF}\u{9F}\u{9F}", "\"\u{10FFFF}\\u009F\u{10FFFF}\\u009F\\u009F\""),
        ("\u{9F}\u{10FFFF}\u{9F}\u{9F}", "\"\\u009F\u{10FFFF}\\u009F\\u009F\""),
        ("\u{10FFFF}\"\u{10FFFF}\"\"\u{10FFFF}", "\"\u{10FFFF}\\\"\u{10FFFF}\\\"\\\"\u{10FFFF}\""),
        ("\"\u{10FFFF}\"\"\u{10FFFF}", "\"\\\"\u{10FFFF}\\\"\\\"\u{10FFFF}\""),
        ("\u{10FFFF}\"\u{10FFFF}\"\"", "\"\u{10FFFF}\\\"\u{10FFFF}\\\"\\\"\""),
        ("\"\u{10FFFF}\"\"", "\"\\\"\u{10FFFF}\\\"\\\"\""),
        ("\u{10FFFF}\\\u{10FFFF}\\\\\u{10FFFF}", "\"\u{10FFFF}\\\\\u{10FFFF}\\\\\\\\\u{10FFFF}\""),
        ("\\\u{10FFFF}\\\\\u{10FFFF}", "\"\\\\\u{10FFFF}\\\\\\\\\u{10FFFF}\""),
        ("\u{10FFFF}\\\u{10FFFF}\\\\", "\"\u{10FFFF}\\\\\u{10FFFF}\\\\\\\\\""),
        ("\\\u{10FFFF}\\\\", "\"\\\\\u{10FFFF}\\\\\\\\\""),
        ("\u{10FFFF}\u{8}\u{10FFFF}\u{8}\u{8}\u{10FFFF}", "\"\u{10FFFF}\\b\u{10FFFF}\\b\\b\u{10FFFF}\""),
        ("\u{8}\u{10FFFF}\u{8}\u{8}\u{10FFFF}", "\"\\b\u{10FFFF}\\b\\b\u{10FFFF}\""),
        ("\u{10FFFF}\u{8}\u{10FFFF}\u{8}\u{8}", "\"\u{10FFFF}\\b\u{10FFFF}\\b\\b\""),
        ("\u{8}\u{10FFFF}\u{8}\u{8}", "\"\\b\u{10FFFF}\\b\\b\""),
        ("\u{10FFFF}\u{C}\u{10FFFF}\u{C}\u{C}\u{10FFFF}", "\"\u{10FFFF}\\f\u{10FFFF}\\f\\f\u{10FFFF}\""),
        ("\u{C}\u{10FFFF}\u{C}\u{C}\u{10FFFF}", "\"\\f\u{10FFFF}\\f\\f\u{10FFFF}\""),
        ("\u{10FFFF}\u{C}\u{10FFFF}\u{C}\u{C}", "\"\u{10FFFF}\\f\u{10FFFF}\\f\\f\""),
        ("\u{C}\u{10FFFF}\u{C}\u{C}", "\"\\f\u{10FFFF}\\f\\f\""),
        ("\u{10FFFF}\n\u{10FFFF}\n\n\u{10FFFF}", "\"\u{10FFFF}\\n\u{10FFFF}\\n\\n\u{10FFFF}\""),
        ("\n\u{10FFFF}\n\n\u{10FFFF}", "\"\\n\u{10FFFF}\\n\\n\u{10FFFF}\""),
        ("\u{10FFFF}\n\u{10FFFF}\n\n", "\"\u{10FFFF}\\n\u{10FFFF}\\n\\n\""),
        ("\n\u{10FFFF}\n\n", "\"\\n\u{10FFFF}\\n\\n\""),
        ("\u{10FFFF}\r\u{10FFFF}\r\r\u{10FFFF}", "\"\u{10FFFF}\\r\u{10FFFF}\\r\\r\u{10FFFF}\""),
        ("\r\u{10FFFF}\r\r\u{10FFFF}", "\"\\r\u{10FFFF}\\r\\r\u{10FFFF}\""),
        ("\u{10FFFF}\r\u{10FFFF}\r\r", "\"\u{10FFFF}\\r\u{10FFFF}\\r\\r\""),
        ("\r\u{10FFFF}\r\r", "\"\\r\u{10FFFF}\\r\\r\""),
        ("\u{10FFFF}\t\u{10FFFF}\t\t\u{10FFFF}", "\"\u{10FFFF}\\t\u{10FFFF}\\t\\t\u{10FFFF}\""),
        ("\t\u{10FFFF}\t\t\u{10FFFF}", "\"\\t\u{10FFFF}\\t\\t\u{10FFFF}\""),
        ("\u{10FFFF}\t\u{10FFFF}\t\t", "\"\u{10FFFF}\\t\u{10FFFF}\\t\\t\""),
        ("\t\u{10FFFF}\t\t", "\"\\t\u{10FFFF}\\t\\t\""),
        ("\u{10FFFF}\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\u{10FFFF}\""),
        ("\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\u{10FFFF}", "\"\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}", "\"\u{10FFFF}\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}", "\"\u{FFFF}\u{10FFFF}\u{FFFF}\u{FFFF}\""),
        ("\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\""),
        ("\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}", "\"\u{10FFFF}\u{10FFFF}\u{10FFFF}\u{10FFFF}\""),
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
            indoc!(r#"
                {
                  "b": [
                    {
                      "c": "\f\r"
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
            indoc!(r#"
                {
                  "Dog": []
                }"#
            ),
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
                Error::Syntax(expected_code, expected_line, expected_col),
                Error::Syntax(actual_code, actual_line, actual_col),
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
        ("1.", Error::Syntax(ErrorCode::InvalidNumber, 1, 2)),
        ("1e", Error::Syntax(ErrorCode::InvalidNumber, 1, 2)),
        ("1e+", Error::Syntax(ErrorCode::InvalidNumber, 1, 3)),
        ("1a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 2)),
        ("1e777777777777777777777777777", Error::Syntax(ErrorCode::InvalidNumber, 1, 22)),
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
        ("\"", Error::Syntax(ErrorCode::EOFWhileParsingString, 1, 1)),
        ("\"lol", Error::Syntax(ErrorCode::EOFWhileParsingString, 1, 4)),
        ("\"lol\"a", Error::Syntax(ErrorCode::TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(vec![
        ("\"\"", "".to_string()),
        ("\"foo\"", "foo".to_string()),
        (" \"foo\" ", "foo".to_string()),
        ("\"\\\"\"", "\"".to_string()),
        ("\"\\/\"", "/".to_string()),
        ("\"\\b\"", "\x08".to_string()),
        ("\"\\f\"", "\x0C".to_string()),
        ("\"\\n\"", "\n".to_string()),
        ("\"\\r\"", "\r".to_string()),
        ("\"\\t\"", "\t".to_string()),
        ("\"\\u12ab\"", "\u{12ab}".to_string()),
        ("\"\\uAB12\"", "\u{AB12}".to_string()),
        ("\"\\uDBFF\\uDFFF\"", "\u{10FFFF}".to_string()),
        ("\"\u{FFFF}\"", "\u{FFFF}".to_string()),
        ("\"\u{10FFFF}\"", "\u{10FFFF}".to_string()),
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
        ("{\"Dog\":", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{\"Dog\":}", Error::Syntax(ErrorCode::ExpectedSomeValue, 1, 8)),
        ("{\"unknown\":[]}", Error::Syntax(ErrorCode::UnknownVariant("unknown".to_string()), 1, 10)),
        ("{\"Dog\":{}}", Error::Syntax(ErrorCode::InvalidType(de::Type::Map), 1, 8)),
        ("{\"Frog\":{}}", Error::Syntax(ErrorCode::InvalidType(de::Type::Map), 1, 9)),
        ("{\"Cat\":[]}", Error::Syntax(ErrorCode::EOFWhileParsingValue, 1, 9)),
        (
            "{\"Cat\":{\"age\": 5, \"name\": \"Kate\", \"foo\":\"bar\"}",
            Error::Syntax(ErrorCode::UnknownField("foo".to_string()), 1, 39)
        ),
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
            serializer.serialize_map(ser::impls::MapIteratorVisitor::new(self.0.iter(), None))
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
fn test_serialize_rejects_non_key_maps() {
    let map = treemap!(
        1 => 2,
        3 => 4
    );

    match serde_json::to_vec(&map).unwrap_err() {
        serde_json::Error::Syntax(serde_json::ErrorCode::KeyMustBeAString, 0, 0) => {}
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
