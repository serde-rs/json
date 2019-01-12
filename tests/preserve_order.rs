extern crate serde_json;

use serde_json::{from_str, Value};

#[test]
fn test_map_order() {
    // Sorted order
    #[cfg(not(feature = "preserve_order"))]
    const EXPECTED: &[&str] = &["a", "b", "c"];

    // Insertion order
    #[cfg(feature = "preserve_order")]
    const EXPECTED: &[&str] = &["b", "a", "c"];

    let v: Value = from_str(r#"{"b":null,"a":null,"c":null}"#).unwrap();
    let keys: Vec<_> = v.as_object().unwrap().keys().collect();
    assert_eq!(keys, EXPECTED);
}
