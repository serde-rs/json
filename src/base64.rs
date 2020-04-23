//! Convenience functions for the base64 alternate byte encoding mode.

use crate::de::Deserializer;
use crate::error::Result;
use crate::io;
use crate::read::{self, Read};
use crate::ser::{CompactFormatter, PrettyFormatter, SerializerBuilder};
use crate::value;
use crate::BytesMode;
use serde::de;
use serde::ser::Serialize;

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    let mut de = Deserializer::with_bytes_mode(read, BytesMode::Base64);
    let value = tri!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    tri!(de.end());
    Ok(value)
}

/// Like `from_reader`, except it uses BytesMode::Base64.
#[cfg(feature = "std")]
pub fn from_reader<R, T>(rdr: R) -> Result<T>
where
    R: crate::io::Read,
    T: de::DeserializeOwned,
{
    from_trait(read::IoRead::new(rdr))
}

/// Like `from_slice`, except it uses BytesMode::Base64.
pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

/// Like `from_str`, except it uses BytesMode::Base64.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::StrRead::new(s))
}

/// Like `to_writer`, except it uses BytesMode::Base64.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = SerializerBuilder::with_formatter(writer, CompactFormatter)
        .bytes_mode(BytesMode::Base64)
        .build();
    tri!(value.serialize(&mut ser));
    Ok(())
}

/// Like `to_writer_pretty`, except it uses BytesMode::Base64.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = SerializerBuilder::with_formatter(writer, PrettyFormatter::new())
        .bytes_mode(BytesMode::Base64)
        .build();
    tri!(value.serialize(&mut ser));
    Ok(())
}

/// Like `to_vec`, except it uses BytesMode::Base64.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    tri!(to_writer(&mut writer, value));
    Ok(writer)
}

/// Like `to_vec_pretty`, except it uses BytesMode::Base64.
#[inline]
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    tri!(to_writer_pretty(&mut writer, value));
    Ok(writer)
}

/// Like `to_string`, except it uses BytesMode::Base64.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = tri!(to_vec(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Like `to_string_pretty`, except it uses BytesMode::Base64.
#[inline]
pub fn to_string_pretty<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = tri!(to_vec_pretty(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Like `to_value`, except it uses BytesMode::Base64.
pub fn to_value<T>(value: T) -> Result<value::Value>
where
    T: Serialize,
{
    value.serialize(value::Serializer::with_bytes_mode(BytesMode::Base64))
}
