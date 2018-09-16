use std::borrow::Cow;
use std::fmt;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

#[derive(Debug, Clone, PartialEq)]
pub struct RawValue<'a>(Cow<'a, str>);

impl<'a> AsRef<str> for RawValue<'a> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> fmt::Display for RawValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Not public API. Should be pub(crate).
#[doc(hidden)]
pub const SERDE_STRUCT_NAME: &'static str = "$__serde_private_RawValue";

impl<'a> Serialize for RawValue<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct(SERDE_STRUCT_NAME, &self.0)
    }
}

impl<'a, 'de> Deserialize<'de> for RawValue<'a>
where
    'de: 'a,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawValueVisitor;

        impl<'de> Visitor<'de> for RawValueVisitor {
            type Value = RawValue<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a deserializable RawValue")
            }

            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(RawValue(Cow::Borrowed(s)))
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(RawValue(Cow::Owned(s)))
            }
        }

        deserializer.deserialize_newtype_struct(SERDE_STRUCT_NAME, RawValueVisitor)
    }
}
