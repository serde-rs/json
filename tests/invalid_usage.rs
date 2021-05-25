use serde::ser::Serializer;
use serde::Serialize;

#[test]
fn custom_serialize_with_key_and_no_value_str() {
    struct Response {
        id: Option<u32>,
    }

    impl Serialize for Response {
        fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeMap;
            let mut state = ser.serialize_map(Some(1))?;
            match self.id {
                Some(ref x) => state.serialize_entry("id", x)?,
                None => state.serialize_key("id")?,
            }
            state.end()
        }
    }

    // this used to return `{"id"}`
    serde_json::to_string(&Response { id: None }).unwrap_err();
}
