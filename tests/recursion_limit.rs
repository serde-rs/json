#![allow(clippy::items_after_statements)]

use serde::Deserialize;
use serde_json::{Deserializer, Result, Value};

fn make_nested_array(depth: usize) -> String {
    let mut json = String::from("null");
    for _ in 0..depth {
        json = format!("[{}]", json);
    }
    json
}

fn make_nested_object(depth: usize) -> String {
    let mut json = String::from("null");
    for _ in 0..depth {
        json = format!(r#"{{"a":{}}}"#, json);
    }
    json
}

#[test]
fn test_default_recursion_limit_127() {
    // Default limit is 128, so depth 127 should work (limit - 1)
    let json = make_nested_array(127);
    let result: Result<Value> = serde_json::from_str(&json);
    assert!(result.is_ok(), "Depth 127 should succeed with default limit 128");
}

#[test]
fn test_default_recursion_limit_exceeded() {
    // Depth 128 should exceed the default limit of 128
    let json = make_nested_array(128);
    let result: Result<Value> = serde_json::from_str(&json);
    assert!(
        result.is_err(),
        "Depth 128 should fail with default limit 128"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("recursion limit"),
        "Error should mention recursion limit, got: {}",
        err
    );
}

#[test]
fn test_set_recursion_limit_higher() {
    // Set limit to 256, depth 200 should work
    let json = make_nested_array(200);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(255); // Max for u8
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_ok(),
        "Depth 200 should succeed with limit 255"
    );
}

#[test]
fn test_set_recursion_limit_lower() {
    // Set limit to 10, depth 20 should fail
    let json = make_nested_array(20);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(10);
    let result = Value::deserialize(&mut deserializer);
    assert!(result.is_err(), "Depth 20 should fail with limit 10");
}

#[test]
fn test_set_recursion_limit_exact() {
    // Test exact boundary: limit=50, max depth=49
    let json = make_nested_array(49);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(50);
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_ok(),
        "Depth 49 should succeed with limit 50"
    );

    // Depth 50 should fail
    let json = make_nested_array(50);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(50);
    let result = Value::deserialize(&mut deserializer);
    assert!(result.is_err(), "Depth 50 should fail with limit 50");
}

#[test]
fn test_set_recursion_limit_builder_pattern() {
    // Test that set_recursion_limit returns &mut Self for chaining
    let json = r#"{"a":{"b":{"c":null}}}"#;
    let mut deserializer = Deserializer::from_str(json);
    
    // Chain method calls
    deserializer
        .set_recursion_limit(100)
        .set_recursion_limit(50); // Should overwrite and return &mut self
    
    let result = Value::deserialize(&mut deserializer);
    assert!(result.is_ok(), "Builder pattern should work");
}

#[test]
fn test_set_recursion_limit_with_objects() {
    // Test with nested objects instead of arrays
    let json = make_nested_object(80);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(100);
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_ok(),
        "Nested objects depth 80 should succeed with limit 100"
    );

    // Should fail with lower limit
    let json = make_nested_object(80);
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(50);
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_err(),
        "Nested objects depth 80 should fail with limit 50"
    );
}

#[test]
fn test_set_recursion_limit_mixed_structures() {
    // Mix of arrays and objects
    let json = r#"[{"a":[{"b":[{"c":[{"d":null}]}]}]}]"#;
    let mut deserializer = Deserializer::from_str(json);
    deserializer.set_recursion_limit(20);
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_ok(),
        "Mixed nested structure should succeed with sufficient limit"
    );
}

#[test]
#[should_panic]
fn test_set_recursion_limit_zero() {
    // Edge case: limit of 0 causes underflow - should panic or be prevented
    // This test documents current behavior (panic on underflow)
    let json = r#"[]"#;
    let mut deserializer = Deserializer::from_str(json);
    deserializer.set_recursion_limit(0);
    let _result = Value::deserialize(&mut deserializer);
}

#[test]
fn test_set_recursion_limit_one() {
    // Limit of 1 means max depth is 0 (no nesting allowed)
    let json = r#"null"#;
    let mut deserializer = Deserializer::from_str(json);
    deserializer.set_recursion_limit(1);
    let result = Value::deserialize(&mut deserializer);
    assert!(result.is_ok(), "No nesting should work with limit 1");

    // But any array/object nesting should fail
    let json = r#"[]"#;
    let mut deserializer = Deserializer::from_str(json);
    deserializer.set_recursion_limit(1);
    let result = Value::deserialize(&mut deserializer);
    assert!(
        result.is_err(),
        "Any nesting should fail with limit 1"
    );
}

#[test]
fn test_set_recursion_limit_does_not_affect_other_deserializers() {
    // Ensure that setting limit on one deserializer doesn't affect others
    let json1 = make_nested_array(150);
    let json2 = make_nested_array(150);

    let mut deserializer1 = Deserializer::from_str(&json1);
    deserializer1.set_recursion_limit(200);

    let mut deserializer2 = Deserializer::from_str(&json2);
    // Don't set limit on deserializer2, should use default (128)

    let result1 = Value::deserialize(&mut deserializer1);
    let result2 = Value::deserialize(&mut deserializer2);

    assert!(
        result1.is_ok(),
        "Deserializer1 with limit 200 should succeed"
    );
    assert!(
        result2.is_err(),
        "Deserializer2 with default limit 128 should fail"
    );
}

#[test]
fn test_set_recursion_limit_with_from_reader() {
    use std::io::Cursor;
    
    let json = make_nested_array(100);
    let cursor = Cursor::new(json.as_bytes());
    
    let mut deserializer = Deserializer::from_reader(cursor);
    deserializer.set_recursion_limit(150);
    let result = Value::deserialize(&mut deserializer);
    
    assert!(
        result.is_ok(),
        "set_recursion_limit should work with from_reader"
    );
}

#[test]
fn test_set_recursion_limit_with_from_slice() {
    let json = make_nested_array(100);
    
    let mut deserializer = Deserializer::from_slice(json.as_bytes());
    deserializer.set_recursion_limit(150);
    let result = Value::deserialize(&mut deserializer);
    
    assert!(
        result.is_ok(),
        "set_recursion_limit should work with from_slice"
    );
}

#[test]
fn test_large_flat_structure_not_affected() {
    // Large flat arrays should not be affected by recursion limit
    let mut json = String::from("[");
    for i in 0..10000 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&i.to_string());
    }
    json.push(']');

    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(10); // Very low limit
    let result = Value::deserialize(&mut deserializer);
    
    assert!(
        result.is_ok(),
        "Flat structures should not be affected by recursion limit"
    );
}

#[test]
fn test_performance_large_limit() {
    // Ensure setting a large limit doesn't cause performance issues
    let json = make_nested_array(200);
    
    let start = std::time::Instant::now();
    let mut deserializer = Deserializer::from_str(&json);
    deserializer.set_recursion_limit(255);
    let result = Value::deserialize(&mut deserializer);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large limit should work");
    assert!(
        duration.as_millis() < 1000,
        "Parsing should complete in reasonable time ({}ms)",
        duration.as_millis()
    );
}
