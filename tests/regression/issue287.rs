// When a Visitor rejects a value, the error should point to the
// start of the value, not the end.

use serde::de::{Deserialize, Deserializer, Error, Visitor};

#[derive(Debug)]
struct Rejected;

impl<'de> Visitor<'de> for Rejected {
    type Value = Rejected;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("...")
    }
    fn visit_str<E: Error>(self, _: &str) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_bytes<E: Error>(self, _: &[u8]) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_bool<E: Error>(self, _: bool) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_i64<E: Error>(self, _: i64) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_u64<E: Error>(self, _: u64) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_f64<E: Error>(self, _: f64) -> Result<Self::Value, E> {
        Err(E::custom("rejected"))
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
        Err(Error::custom("rejected"))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
        Err(Error::custom("rejected"))
    }
}

#[derive(Debug)]
struct Str;
impl<'de> Deserialize<'de> for Str {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(Rejected).map(|_| Str)
    }
}

#[derive(Debug)]
struct Any;
impl<'de> Deserialize<'de> for Any {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(Rejected).map(|_| Any)
    }
}

#[derive(Debug)]
struct Bytes;
impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_bytes(Rejected).map(|_| Bytes)
    }
}

// String tests
//   123456789012
//     "hello"
//     ^     ^
//   start  end
//    (3)   (9)

#[test]
fn deserialize_str() {
    let err = serde_json::from_str::<Str>(r#"  "hello"  "#).unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of string, not end");
}

#[test]
fn deserialize_any_string() {
    let err = serde_json::from_str::<Any>(r#"  "hello"  "#).unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of string, not end");
}

#[test]
fn deserialize_bytes() {
    let err = serde_json::from_str::<Bytes>(r#"  "hello"  "#).unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of string, not end");
}

// Null test
//   12345678
//     null
//     ^  ^
//   start end
//    (3) (6)

#[test]
fn deserialize_any_null() {
    let err = serde_json::from_str::<Any>("  null  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of null, not end");
}

// Bool tests
//   123456789
//     true
//     ^  ^
//    (3)(6)

#[test]
fn deserialize_any_true() {
    let err = serde_json::from_str::<Any>("  true  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of true, not end");
}

//   1234567890
//     false
//     ^   ^
//    (3) (7)

#[test]
fn deserialize_any_false() {
    let err = serde_json::from_str::<Any>("  false  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of false, not end");
}

// Number tests
//   12345678
//     123
//     ^ ^
//    (3)(5)

#[test]
fn deserialize_any_positive_int() {
    let err = serde_json::from_str::<Any>("  123  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of number, not end");
}

//   123456789
//     -456
//     ^  ^
//    (3)(6)

#[test]
fn deserialize_any_negative_int() {
    let err = serde_json::from_str::<Any>("  -456  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of number, not end");
}

//   1234567890
//     3.14
//     ^  ^
//    (3)(6)

#[test]
fn deserialize_any_float() {
    let err = serde_json::from_str::<Any>("  3.14  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of number, not end");
}

// Array test
//   123456789
//     [1,2]
//     ^   ^
//    (3) (6)

#[test]
fn deserialize_any_array() {
    let err = serde_json::from_str::<Any>("  [1,2]  ").unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of array, not end");
}

// Object test
//   12345678901234
//     {"a":1}
//     ^     ^
//    (3)  (9)

#[test]
fn deserialize_any_object() {
    let err = serde_json::from_str::<Any>(r#"  {"a":1}  "#).unwrap_err();
    assert_eq!(err.column(), 3, "should point to start of object, not end");
}
