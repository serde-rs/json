use serde::ser::Serializer;
use serde::Serialize;

#[test]
#[should_panic = "end called before serialize_value"]
fn key_and_no_value_str() {
    let _ = serde_json::to_string(&ForgetsValue { id: None });
}

#[test]
#[should_panic = "end called before serialize_value: \"id\""]
fn key_and_no_value_val() {
    let _ = serde_json::value::to_value(&ForgetsValue { id: None });
}

#[test]
#[should_panic = "serialize_value called before serialize_key"]
fn no_key_but_value_str() {
    let _ = serde_json::to_string(&ForgetsKey { id: Some(42) });
}

#[test]
#[should_panic = "serialize_value called before serialize_key"]
fn no_key_but_value_val() {
    let _ = serde_json::value::to_value(&ForgetsKey { id: Some(42) });
}

struct ForgetsKey {
    id: Option<u32>,
}

impl Serialize for ForgetsKey {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut state = ser.serialize_map(Some(1))?;
        match self.id {
            Some(ref x) => {
                // should panic here
                state.serialize_value(x)?
            }
            None => {}
        }
        state.end()
    }
}

struct ForgetsValue {
    id: Option<u32>,
}

impl Serialize for ForgetsValue {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut state = ser.serialize_map(Some(1))?;
        match self.id {
            Some(ref x) => state.serialize_entry("id", x)?,
            None => state.serialize_key("id")?,
        }
        // should panic here
        state.end()
    }
}
