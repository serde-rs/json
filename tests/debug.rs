use serde_json::{json, Number, Value};

#[test]
fn number() {
    assert_eq!(format!("{:?}", Number::from(1)), "1");
    assert_eq!(format!("{:?}", Number::from(-1)), "-1");
    assert_eq!(format!("{:?}", Number::from_f64(1.0).unwrap()), "1.0");
}

#[test]
fn value_null() {
    assert_eq!(format!("{:?}", json!(null)), "null");
}

#[test]
fn value_bool() {
    assert_eq!(format!("{:?}", json!(true)), "true");
    assert_eq!(format!("{:?}", json!(false)), "false");
}

#[test]
fn value_number() {
    assert_eq!(format!("{:?}", json!(1)), "1");
    assert_eq!(format!("{:?}", json!(-1)), "-1");
    assert_eq!(format!("{:?}", json!(1.0)), "1.0");
}

#[test]
fn value_string() {
    assert_eq!(format!("{:?}", json!("s")), "\"s\"");
}

#[test]
fn value_array() {
    assert_eq!(format!("{:?}", json!([])), "[]");
}

#[test]
fn value_object() {
    assert_eq!(format!("{:?}", json!({})), "{}");
}

#[test]
fn error() {
    let err = serde_json::from_str::<Value>("{0}").unwrap_err();
    let expected = "Error(\"key must be a string\", line: 1, column: 2)";
    assert_eq!(format!("{:?}", err), expected);
}

const INDENTED_EXPECTED: &str = r#"{
    "array": [
        0,
        1,
    ],
}"#;

#[test]
fn indented() {
    let j = json!({ "array": [0, 1] });
    assert_eq!(format!("{:#?}", j), INDENTED_EXPECTED);
}
