//! Native API compatibility controls run with and without zeroize, using only alloc.

use alloc::format;
use alloc::string::ToString;

#[test]
fn borrowed_input_and_serialized_output_keep_the_native_mapping() {
    let input = r#""borrowed input""#;
    let borrowed: &str = crate::from_str(input).unwrap();
    assert_eq!(borrowed, "borrowed input");
    assert_eq!(crate::to_string(&borrowed).unwrap(), input);
}

#[test]
fn custom_error_display_debug_and_position_stay_compatible() {
    use serde::de::Error as _;
    const LINE: usize = 2;
    const COLUMN: usize = 3;
    let error = crate::Error::custom(format!("fixture error at line {LINE} column {COLUMN}"));
    assert_eq!((error.line(), error.column()), (LINE, COLUMN));
    assert_eq!(error.to_string(), "fixture error at line 2 column 3");
    assert_eq!(
        format!("{error:?}"),
        "Error(\"fixture error\", line: 2, column: 3)"
    );
    assert!(error.is_data());
}

#[test]
fn duplicate_object_members_keep_native_order_and_last_value() {
    let document: crate::Value = crate::from_str(r#"{"z":1,"a":2,"z":3}"#).unwrap();
    #[cfg(feature = "preserve_order")]
    assert_eq!(crate::to_string(&document).unwrap(), r#"{"z":3,"a":2}"#);
    #[cfg(not(feature = "preserve_order"))]
    assert_eq!(crate::to_string(&document).unwrap(), r#"{"a":2,"z":3}"#);
}

#[test]
fn negative_zero_keeps_the_current_native_numeric_behavior() {
    let number: f64 = crate::from_str("-0").unwrap();
    assert!(number.is_sign_negative());
    #[cfg(feature = "arbitrary_precision")]
    {
        let number: crate::Number = crate::from_str("-0").unwrap();
        assert_eq!(number.to_string(), "-0");
    }
}

#[test]
fn public_reader_scratch_and_parser_stream_signatures_stay_compatible() {
    use crate::de::{Read, StrRead};
    use alloc::vec::Vec;
    let mut read = StrRead::new(r#""reader\u0021""#);
    assert_eq!(read.next().unwrap(), Some(b'"'));
    let mut scratch: Vec<u8> = Vec::new();
    assert_eq!(&*read.parse_str(&mut scratch).unwrap(), "reader!");

    let mut read = StrRead::new("1 2");
    let parser = crate::Deserializer::new(&mut read);
    let mut stream = parser.into_iter::<u64>();
    assert_eq!(stream.next().unwrap().unwrap(), 1);
    drop(stream);
    assert_eq!(read.next().unwrap(), Some(b' '));
}

#[test]
#[cfg(any(feature = "raw_value", feature = "arbitrary_precision"))]
fn hidden_string_adapter_preserves_owned_string_and_enum_dispatch() {
    use alloc::string::String;
    use core::{fmt, marker::PhantomData};
    use serde::de::{self, DeserializeSeed, VariantAccess, Visitor};

    let text = String::from("owned-private").into();
    assert_eq!(
        crate::scratch::deserialize_text(PhantomData::<String>, text).unwrap(),
        "owned-private"
    );

    struct EnumSeed;
    impl<'de> DeserializeSeed<'de> for EnumSeed {
        type Value = String;
        fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<String, D::Error> {
            struct EnumVisitor;
            impl<'de> Visitor<'de> for EnumVisitor {
                type Value = String;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a unit enum")
                }
                fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<String, A::Error> {
                    let (variant, access) = tri!(data.variant::<String>());
                    tri!(access.unit_variant());
                    Ok(variant)
                }
            }
            de.deserialize_enum("Fixture", &["unit-private"], EnumVisitor)
        }
    }
    let text = String::from("unit-private").into();
    assert_eq!(
        crate::scratch::deserialize_text(EnumSeed, text).unwrap(),
        "unit-private"
    );
}
