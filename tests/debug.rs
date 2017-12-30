#[macro_use]
extern crate serde_json;

use serde_json::Number;

#[test]
fn number() {
    assert_eq!(format!("{:?}", Number::from(1)), "PosInt(1)");
    assert_eq!(format!("{:?}", Number::from(-1)), "NegInt(-1)");
    assert_eq!(format!("{:?}", Number::from_f64(1.0).unwrap()), "Float(1.0)");
}

#[test]
fn value_null() {
    assert_eq!(format!("{:?}", json!(null)), "Null");
}

#[test]
fn value_bool() {
    assert_eq!(format!("{:?}", json!(true)), "Bool(true)");
    assert_eq!(format!("{:?}", json!(false)), "Bool(false)");
}

#[test]
fn value_number() {
    assert_eq!(format!("{:?}", json!(1)), "Number(PosInt(1))");
    assert_eq!(format!("{:?}", json!(-1)), "Number(NegInt(-1))");
    assert_eq!(format!("{:?}", json!(1.0)), "Number(Float(1.0))");
}

#[test]
fn value_string() {
    assert_eq!(format!("{:?}", json!("s")), "String(\"s\")");
}

#[test]
fn value_array() {
    assert_eq!(format!("{:?}", json!([])), "Array([])");
}

#[test]
fn value_object() {
    assert_eq!(format!("{:?}", json!({})), "Object({})");
}
