
use serde_json::value::{Map, Value};
use serde_json::builder::{ArrayBuilder, ObjectBuilder};

#[test]
fn test_array_builder() {
    let value = ArrayBuilder::new().build();
    assert_eq!(value, Value::Array(Vec::new()));

    let value = ArrayBuilder::new()
        .push(1)
        .push(2)
        .push(3)
        .build();
    assert_eq!(value,
               Value::Array(vec![Value::Number(1.into()),
                                 Value::Number(2.into()),
                                 Value::Number(3.into())]));

    let value = ArrayBuilder::new()
        .push_array(|bld| bld.push(1).push(2).push(3))
        .build();
    assert_eq!(value,
               Value::Array(vec![Value::Array(vec![Value::Number(1.into()),
                                                   Value::Number(2.into()),
                                                   Value::Number(3.into())])]));

    let value = ArrayBuilder::new()
        .push_object(|bld| {
            bld.insert("a".to_string(), 1)
                .insert("b".to_string(), 2)
        })
        .build();

    let mut map = Map::new();
    map.insert("a".to_string(), Value::Number(1.into()));
    map.insert("b".to_string(), Value::Number(2.into()));
    assert_eq!(value, Value::Array(vec![Value::Object(map)]));
}

#[test]
fn test_object_builder() {
    let value = ObjectBuilder::new().build();
    assert_eq!(value, Value::Object(Map::new()));

    let value = ObjectBuilder::new()
        .insert("a".to_string(), 1)
        .insert("b".to_string(), 2)
        .build();

    let mut map = Map::new();
    map.insert("a".to_string(), Value::Number(1.into()));
    map.insert("b".to_string(), Value::Number(2.into()));
    assert_eq!(value, Value::Object(map));
}
