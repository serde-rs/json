use crate::Value;
use super::JsonReader;

/// Implement `JsonPtrMut | FnOnce` simply.
pub fn apply<F>(v: &mut Value, f: F) where F: FnOnce(&mut Value) {
    f(v);
}

/// Implement `JsonPtrMut | JsonPtrMut` to merge `rhs` json to `lhs`.
/// The `rhs` json serve as default value in each node.
pub fn merge_move(lhs: &mut Value, rhs: &mut Value) {
    match lhs {
        Value::Null => { *lhs = rhs.take(); }
        Value::Array(array_lhs) => {
            if let Value::Array(array_rhs) = rhs {
                let len_lhs = array_lhs.len();
                let len_rhs = array_rhs.len();
                if len_rhs == 0 {
                    return;
                }
                if len_rhs == 1 {
                    for i in 0 .. len_lhs {
                        merge_copy(&mut array_lhs[i], &array_rhs[0]);
                    }
                }
                else {
                    let len_min = std::cmp::min(len_lhs, len_rhs);
                    for i in 0 .. len_min {
                        merge_move(&mut array_lhs[i], &mut array_rhs[i]);
                    }
                    if len_min < len_rhs {
                        for i in len_min .. len_rhs {
                            array_lhs.push(array_rhs[i].take());
                        }
                    }
                }
            }
        }
        Value::Object(object_lhs) => {
            if let Value::Object(object_rhs) = rhs {
                for (key, val) in object_rhs {
                    if !object_lhs.contains_key(key) || object_lhs[key].is_null() {
                        object_lhs.insert(key.to_string(), val.take());
                    }
                    else {
                        merge_move(&mut object_lhs[key], val);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Implement `JsonPtrMut | JsonPtr` to merge immutable `rhs` json to `lhs`.
pub fn merge_copy(lhs: &mut Value, rhs: &Value) {
    match lhs {
        Value::Null => { *lhs = rhs.clone(); }
        Value::Array(array_lhs) => {
            if let Value::Array(array_rhs) = rhs {
                let len_lhs = array_lhs.len();
                let len_rhs = array_rhs.len();
                if len_rhs == 0 {
                    return;
                }
                if len_rhs == 1 {
                    for i in 0 .. len_lhs {
                        merge_copy(&mut array_lhs[i], &array_rhs[0]);
                    }
                }
                else {
                    let len_min = std::cmp::min(len_lhs, len_rhs);
                    for i in 0 .. len_min {
                        merge_copy(&mut array_lhs[i], &array_rhs[i]);
                    }
                    if len_min < len_rhs {
                        for i in len_min .. len_rhs {
                            array_lhs.push(array_rhs[i].clone());
                        }
                    }
                }
            }
        }
        Value::Object(object_lhs) => {
            if let Value::Object(object_rhs) = rhs {
                for (key, val) in object_rhs {
                    if !object_lhs.contains_key(key) || object_lhs[key].is_null() {
                        object_lhs.insert(key.to_string(), val.clone());
                    }
                    else {
                        merge_copy(&mut object_lhs[key], val);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Implement `JsonPtrMut & JsonPtr` to filter `lhs` by `rhs` as sample structure.
pub fn filter_shape(lhs: &mut Value, rhs: &Value) {
    if lhs.get_type() != rhs.get_type() {
        *lhs = Value::Null;
    }
    match lhs {
        Value::Array(array_lhs) => {
            if let Value::Array(array_rhs) = rhs {
                let len_lhs = array_lhs.len();
                let len_rhs = array_rhs.len();
                if len_rhs == 0 {
                    array_lhs.clear();
                    return;
                }
                if len_rhs == 1 {
                    for i in 0 .. len_lhs {
                        filter_shape(&mut array_lhs[i], &array_rhs[0]);
                    }
                }
                else {
                    let len_min = std::cmp::min(len_lhs, len_rhs);
                    for i in 0 .. len_min {
                        filter_shape(&mut array_lhs[i], &array_rhs[i]);
                    }
                    if len_min < len_lhs {
                        for i in len_min .. len_lhs {
                            array_lhs[i] = Value::Null;
                        }
                    }
                }
            }
        }
        Value::Object(object_lhs) => {
            if let Value::Object(object_rhs) = rhs {
                for (key, val) in &mut *object_lhs {
                    if !object_rhs.contains_key(key) || object_rhs[key].is_null() {
                        *val = Value::Null;
                    }
                    else {
                        filter_shape(val, &object_rhs[key]);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Check json node is empty, include null and empty array or object.
fn is_empty_node(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Array(array) => array.is_empty(),
        Value::Object(object) => object.is_empty(),
        _ => false
    }
}

/// Implement `JsonPtrMut | ()` to remove null node recursively in array or object.
pub fn remove_null (lhs: &mut Value) {
    match lhs {
        Value::Array(array) => {
            for v in &mut *array {
                remove_null(v);
            }
            array.retain(|v| !is_empty_node(v));
        }
        Value::Object(object) => {
            for (_, v) in &mut *object {
                remove_null(v);
            }
            object.retain(|_, v| !is_empty_node(v));
        }
        _ => {}
    }
}
