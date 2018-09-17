use std::borrow::Cow;
use std::fmt::{self, Debug};

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, DeserializeSeed, IntoDeserializer, MapAccess, Unexpected, Visitor};
use serde::de::value::BorrowedStrDeserializer;

use error::Error;

/// Represents any valid JSON value as a series of raw bytes.
///
/// This type can be used to defer parsing parts of a payload until later,
/// or to embed it verbatim into another JSON payload.
///
/// When serializing, a value of this type will retain its original formatting
/// and will not be minified or pretty-printed.
///
/// When deserializing, this type can not be used with the `#[serde(flatten)]` attribute,
/// as it relies on the original input buffer.

#[derive(Clone)]
pub struct RawValue<'a> {
    cow: Cow<'a, str>,
}

impl<'a> Debug for RawValue<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_tuple("RawValue")
            .field(&format_args!("{}", self.cow))
            .finish()
    }
}

impl<'a> AsRef<str> for RawValue<'a> {
    fn as_ref(&self) -> &str {
        &self.cow
    }
}

impl<'a> fmt::Display for RawValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.cow)
    }
}

pub const TOKEN: &'static str = "$serde_json::private::RawValue";

impl<'a> Serialize for RawValue<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct(TOKEN, 1)?;
        s.serialize_field(TOKEN, &self.cow)?;
        s.end()
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for RawValue<'a> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawValueVisitor;

        impl<'de> Visitor<'de> for RawValueVisitor {
            type Value = RawValue<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "any valid JSON value")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let value = visitor.next_key::<RawKey>()?;
                if value.is_none() {
                    return Err(de::Error::invalid_type(Unexpected::Map, &self));
                }
                visitor.next_value_seed(RawValueFromString)
            }
        }

        deserializer.deserialize_newtype_struct(TOKEN, RawValueVisitor)
    }
}

struct RawKey;

impl<'de> Deserialize<'de> for RawKey {
    fn deserialize<D>(deserializer: D) -> Result<RawKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("raw value")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                if s == TOKEN {
                    Ok(())
                } else {
                    Err(de::Error::custom("unexpected raw value"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(RawKey)
    }
}

pub struct RawValueFromString;

impl<'de> DeserializeSeed<'de> for RawValueFromString {
    type Value = RawValue<'de>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for RawValueFromString {
    type Value = RawValue<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("raw value")
    }

    fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RawValue { cow: Cow::Borrowed(s) })
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RawValue { cow: Cow::Owned(s) })
    }
}

struct RawKeyDeserializer;

impl<'de> Deserializer<'de> for RawKeyDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(TOKEN)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct ignored_any
        unit_struct tuple_struct tuple enum identifier
    }
}

pub struct OwnedRawDeserializer {
    pub raw_value: Option<String>,
}

impl<'de> MapAccess<'de> for OwnedRawDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.raw_value.is_none() {
            return Ok(None);
        }
        seed.deserialize(RawKeyDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.raw_value.take().unwrap().into_deserializer())
    }
}

pub struct BorrowedRawDeserializer<'de> {
    pub raw_value: Option<&'de str>,
}

impl<'de> MapAccess<'de> for BorrowedRawDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.raw_value.is_none() {
            return Ok(None);
        }
        seed.deserialize(RawKeyDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(BorrowedStrDeserializer::new(self.raw_value.take().unwrap()))
    }
}
