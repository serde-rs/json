#![cfg(feature = "raw_value")]

// Trivial behavior check to go with the added `// SAFETY:` comments on the
// three `unsafe { mem::transmute(...) } ` calls in `src/raw.rs`
// (`RawValue::from_borrowed`/`from_owned`/`into_owned`). The comments are
// doc-only, so this just exercises all three code paths and checks the JSON
// text round-trips through them unchanged.

use serde_json::value::RawValue;

#[test]
fn test() {
    // `RawValue::NULL`/`TRUE`/`FALSE` are built with `from_borrowed`.
    assert_eq!(RawValue::NULL.get(), "null");
    assert_eq!(RawValue::TRUE.get(), "true");
    assert_eq!(RawValue::FALSE.get(), "false");

    // `RawValue::from_string` goes through `from_owned` (its fast path, when
    // the input has no surrounding whitespace and exact capacity).
    let owned: Box<RawValue> = RawValue::from_string("[1,2,3]".to_owned()).unwrap();
    assert_eq!(owned.get(), "[1,2,3]");

    // `to_owned`/`Clone` on a borrowed `RawValue` also goes through
    // `from_owned`.
    let borrowed: &RawValue = RawValue::NULL;
    let cloned: Box<RawValue> = borrowed.to_owned();
    assert_eq!(cloned.get(), "null");

    // `Box<RawValue> -> Box<str>` goes through `into_owned`.
    let json: Box<str> = Box::<str>::from(owned);
    assert_eq!(&*json, "[1,2,3]");
}
