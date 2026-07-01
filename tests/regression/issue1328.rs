use serde::ser::{Serialize, Serializer};
use std::fmt;

struct BadDisplay;

impl fmt::Display for BadDisplay {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}

impl Serialize for BadDisplay {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[test]
fn test() {
    let err = serde_json::to_string(&BadDisplay).unwrap_err();

    assert!(err.is_data());
    assert_eq!(
        err.to_string(),
        "Display implementation returned an error unexpectedly"
    );
}
