use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Data {
    _value: i32,
    _value2: i128,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Wrapper {
    Data(Data),
}

#[test]
fn test() {
    let json = r#"{"type":"Data","_value":123,"_value2":123244235436463}"#;
    // Okay
    let _data1: Data = serde_json::from_str(json).unwrap();
    // Fails!
    let _data2: Wrapper = serde_json::from_str(json).unwrap();
}
