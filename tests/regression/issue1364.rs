use std::collections::HashMap;

#[test]
fn map_key_deserialize_bool_keeps_full_key() {
    let err = serde_json::from_str::<HashMap<bool, i32>>(r#"{"yes": 1}"#).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("\"yes\""),
        "expected the full key in the error, got {msg:?}"
    );
    assert!(
        !msg.contains("\"es\""),
        "first byte of the key was consumed before parse_str: {msg:?}"
    );
}

#[test]
fn map_key_deserialize_bool_multibyte_key_is_valid_utf8() {
    let err = serde_json::from_str::<HashMap<bool, i32>>(r#"{"éx": 1}"#).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.is_ascii() || std::str::from_utf8(msg.as_bytes()).is_ok(),
        "error message was not valid UTF-8: {msg:?}"
    );
    assert!(
        msg.contains("éx") || msg.contains("\"éx\""),
        "expected the full multibyte key in the error, got {msg:?}"
    );
    let _ = format!("{err:?}");
}

#[test]
fn map_key_deserialize_bool_true_still_works() {
    let map: HashMap<bool, i32> = serde_json::from_str(r#"{"true": 1}"#).unwrap();
    assert_eq!(map.get(&true), Some(&1));
}
