//! Preserve StringDeserializer's owned-string and unit-enum behavior, while
//! retaining the wiping owner if a seed or enum visitor rejects the input before
//! asking for its string. Ownership transfers only when visit_string is called.

use super::{into_string, Text};
use crate::{Error, Result};
use serde::de::{self, DeserializeSeed, Unexpected, Visitor};

pub(super) struct Deserializer(pub Text);

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(into_string(self.0))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> de::EnumAccess<'de> for Deserializer {
    type Error = Error;
    type Variant = UnitVariant;

    fn variant_seed<S>(self, seed: S) -> Result<(S::Value, UnitVariant)>
    where
        S: DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(|value| (value, UnitVariant))
    }
}

pub(super) struct UnitVariant;

impl<'de> de::VariantAccess<'de> for UnitVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<S>(self, _seed: S) -> Result<S::Value>
    where
        S: DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Deserializer, Text};
    use crate::custody_tests::{watch, wiped};
    use core::fmt;
    use serde::de::{self, DeserializeSeed, Visitor};

    struct RefusingSeed;
    impl<'de> DeserializeSeed<'de> for RefusingSeed {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, _: D) -> Result<(), D::Error> {
            Err(de::Error::custom("fixture refused to read the string"))
        }
    }

    #[test]
    fn seed_rejection_wipes_the_adapter_before_any_string_is_transferred() {
        let text = Text::from_display("adapter-private");
        watch(text.as_bytes());
        assert!(crate::scratch::deserialize_text(RefusingSeed, text).is_err());
        assert_eq!(wiped(), Some(true));
    }

    #[test]
    fn enum_visitor_rejection_retains_wiping_ownership() {
        struct RefusingEnum;
        impl<'de> Visitor<'de> for RefusingEnum {
            type Value = ();
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an enum")
            }
            fn visit_enum<A: de::EnumAccess<'de>>(self, _: A) -> Result<(), A::Error> {
                Err(de::Error::custom(
                    "fixture refused the enum before reading its variant",
                ))
            }
        }
        let text = Text::from_display("enum-private");
        watch(text.as_bytes());
        assert!(serde::Deserializer::deserialize_enum(
            Deserializer(text),
            "Fixture",
            &["enum-private"],
            RefusingEnum
        )
        .is_err());
        assert_eq!(wiped(), Some(true));
    }
}
