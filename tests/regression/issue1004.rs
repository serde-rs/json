#![cfg(feature = "arbitrary_precision")]

#[test]
fn test() {
    let float = 5.55f32;
    let value = serde_json::to_value(&float).unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "5.550000190734863"); // FIXME
}
