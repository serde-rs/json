// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor, Unexpected};

use std::{i64};
use std::fmt::{self, Display};

use error::{Error};

#[cfg(not(feature = "arbitrary_precision"))]
use num_traits::NumCast;

#[cfg(feature = "arbitrary_precision")]
use dtoa;

#[cfg(feature = "arbitrary_precision")]
use itoa;

#[cfg(feature = "arbitrary_precision")]
use serde::de::{IntoDeserializer, MapAccess};

#[cfg(feature = "arbitrary_precision")]
use std::borrow::{Cow};
#[cfg(not(feature = "arbitrary_precision"))]
use std::fmt::{Debug};

#[cfg(feature = "arbitrary_precision")]
use error::{ErrorCode};

#[cfg(feature = "arbitrary_precision")]
/// Not public API. Should be pub(crate). The deserializer specializes on this
/// type.
#[doc(hidden)]
pub const SERDE_STRUCT_FIELD_NAME: &'static str = "$__toml_private_number";

#[cfg(feature = "arbitrary_precision")]
/// Not public API. Should be pub(crate). The deserializer specializes on this
/// type.
#[doc(hidden)]
pub const SERDE_STRUCT_NAME: &'static str = "$__toml_private_Number";

/// Represents a JSON number, whether integer or floating point.
#[cfg_attr(feature = "arbitrary_precision", derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct Number {
    n: N,
}

// "N" is a prefix of "NegInt"... this is a false positive.
// https://github.com/Manishearth/rust-clippy/issues/1241
#[cfg(not(feature = "arbitrary_precision"))]
#[cfg_attr(feature = "cargo-clippy", allow(enum_variant_names))]
#[derive(Copy, Clone, Debug, PartialEq)]
enum N {
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
}

#[cfg(feature = "arbitrary_precision")]
type N = String;

#[cfg(not(feature = "arbitrary_precision"))]
macro_rules! cast_methods {
    ($(
        #[doc = $doc:tt]
        pub fn $method:ident(&self) -> Option<$ty:ident>;
    )+) => {
        $(
            #[doc = $doc]
            pub fn $method(&self) -> Option<$ty> {
                match self.n {
                    N::PosInt(n) => NumCast::from(n),
                    N::NegInt(n) => NumCast::from(n),
                    N::Float(n) => NumCast::from(n),
                }
            }
        )*
    };
}

impl Number {
    /// Returns true if the `Number` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Number on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_json;
    /// #
    /// # use std::i64;
    /// #
    /// # fn main() {
    /// let big = i64::MAX as u64 + 10;
    /// let v = json!({ "a": 64, "b": big, "c": 256.0 });
    ///
    /// assert!(v["a"].is_i64());
    ///
    /// // Greater than i64::MAX.
    /// assert!(!v["b"].is_i64());
    ///
    /// // Can be converted to a signed integer.
    /// assert!(v["c"].is_i64());
    /// # }
    /// ```
    #[inline]
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// Returns true if the `Number` is an integer between zero and `u64::MAX`.
    ///
    /// For any Number on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_json;
    /// #
    /// # fn main() {
    /// let v = json!({ "a": 64, "b": -64, "c": 256.0 });
    ///
    /// assert!(v["a"].is_u64());
    ///
    /// // Negative integer.
    /// assert!(!v["b"].is_u64());
    ///
    /// // Can be converted to an unsigned integer.
    /// assert!(v["c"].is_u64());
    /// # }
    /// ```
    #[inline]
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// Returns true if the `Number` can be represented by f64.
    ///
    /// For any Number on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_json;
    /// #
    /// # fn main() {
    /// let v = json!({ "a": 256.0, "b": 64, "c": -64 });
    ///
    /// assert!(v["a"].is_f64());
    ///
    /// // Integers.
    /// assert!(v["b"].is_f64());
    /// assert!(v["c"].is_f64());
    /// # }
    /// ```
    #[inline]
    pub fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    cast_methods! {
        /// Returns the number represented as `i64` if possible, or else None.
        pub fn as_i64(&self) -> Option<i64>;

        /// Returns the number represented as `u64` if possible, or else None.
        pub fn as_u64(&self) -> Option<u64>;

        /// Returns the number represented as `f64` if possible, or else `None`.
        pub fn as_f64(&self) -> Option<f64>;
    }

    #[cfg(feature = "arbitrary_precision")]
    /// Returns the number represented as `i64` if possible, or else `None`.
    pub fn as_i64(&self) -> Option<i64> {
        self.n.splitn(2, '.').next().and_then(|n| n.parse().ok())
    }

    #[cfg(feature = "arbitrary_precision")]
    /// Returns the number represented as `u64` if possible, or else `None`.
    pub fn as_u64(&self) -> Option<u64> {
        self.n.splitn(2, '.').next().and_then(|n| n.parse().ok())
    }

    #[cfg(feature = "arbitrary_precision")]
    /// Returns the number represented as `f64` if possible, or else `None`.
    pub fn as_f64(&self) -> Option<f64> {
        self.n.parse().ok()
    }

    /// Converts a finite `f64` to a `Number`. Infinite or NaN values are not JSON
    /// numbers.
    ///
    /// ```rust
    /// # use std::f64;
    /// #
    /// # use serde_json::Number;
    /// #
    /// assert!(Number::from_f64(256.0).is_some());
    ///
    /// assert!(Number::from_f64(f64::NAN).is_none());
    /// ```
    #[inline]
    pub fn from_f64(f: f64) -> Option<Number> {
        if f.is_finite() {
            Some(Number { n: n_from_finite_f64(f) })
        } else {
            None
        }
    }

    /// Not public API. Should be pub(crate). The deserializer uses this.
    #[cfg(feature = "arbitrary_precision")]
    #[doc(hidden)]
    #[inline]
    pub fn from_string_unchecked(n: String) -> Self {
        Number { n: n }
    }
}

#[cfg(not(feature = "arbitrary_precision"))]
fn n_from_finite_f64(f: f64) -> N {
    N::Float(f)
}

#[cfg(feature = "arbitrary_precision")]
fn n_from_finite_f64(f: f64) -> N {
    let mut buf = Vec::new();
    dtoa::write(&mut buf, f).unwrap();
    String::from_utf8(buf).unwrap()
}

impl Display for Number {
    #[cfg(not(feature = "arbitrary_precision"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.n {
            N::PosInt(u) => Display::fmt(&u, formatter),
            N::NegInt(i) => Display::fmt(&i, formatter),
            N::Float(f) => Display::fmt(&f, formatter),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.n)
    }
}

#[cfg(not(feature = "arbitrary_precision"))]
impl Debug for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.n, formatter)
    }
}

impl Serialize for Number {
    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.n {
            N::PosInt(u) => serializer.serialize_u64(u),
            N::NegInt(i) => serializer.serialize_i64(i),
            N::Float(f) => serializer.serialize_f64(f),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct(SERDE_STRUCT_NAME, 1)?;
        s.serialize_field(SERDE_STRUCT_FIELD_NAME, &self.to_string())?;
        s.end()
    }
}

/// Not public API. Should be pub(crate). The deserializer specializes on this
/// type.
#[doc(hidden)]
pub struct NumberVisitor;

impl<'de> de::Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON number")
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
        Ok(value.into())
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
        Ok(value.into())
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn visit_f64<E>(self, value: f64) -> Result<Number, E>
        where E: de::Error
    {
        Number::from_f64(value).ok_or_else(|| de::Error::custom("not a JSON number"))
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Number, V::Error>
        where V: de::MapAccess<'de>
    {
        let value = visitor.next_key::<NumberKey>()?;
        if value.is_none() {
            return Err(de::Error::custom("number key not found"))
        }
        let v: NumberFromString = visitor.next_value()?;
        Ok(v.value)
    }
}

impl<'de> Deserialize<'de> for Number {
    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_any(NumberVisitor)
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
        where D: de::Deserializer<'de>
    {
        static FIELDS: [&'static str; 1] = [SERDE_STRUCT_FIELD_NAME];
        deserializer.deserialize_struct(SERDE_STRUCT_NAME,
                                        &FIELDS,
                                        NumberVisitor)
    }
}

#[cfg(feature = "arbitrary_precision")]
struct NumberKey;

#[cfg(feature = "arbitrary_precision")]
impl<'de> de::Deserialize<'de> for NumberKey {
    fn deserialize<D>(deserializer: D) -> Result<NumberKey, D::Error>
        where D: de::Deserializer<'de>
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid number field")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
                where E: de::Error
            {
                if s == SERDE_STRUCT_FIELD_NAME {
                    Ok(())
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        try!(deserializer.deserialize_identifier(FieldVisitor));
        Ok(NumberKey)
    }
}

#[cfg(feature = "arbitrary_precision")]
pub struct NumberFromString {
    pub value: Number,
}

#[cfg(feature = "arbitrary_precision")]
impl<'de> de::Deserialize<'de> for NumberFromString {
    fn deserialize<D>(deserializer: D) -> Result<NumberFromString, D::Error>
        where D: de::Deserializer<'de>
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = NumberFromString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string containing a number")
            }

            fn visit_string<E>(self, s: String) -> Result<NumberFromString, E>
                where E: de::Error,
            {
                Ok(NumberFromString { value: Number::from_string_unchecked(s) })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(feature = "arbitrary_precision")]
fn invalid_number() -> Error {
    Error::syntax(ErrorCode::InvalidNumber, 0, 0)
}

macro_rules! deserialize_number {
    ($deserialize:ident => $visit:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            self.deserialize_any(visitor)
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(try!(self.n.parse().map_err(|_| invalid_number())))
        }
    }
}

impl<'de> Deserializer<'de> for Number {
    type Error = Error;

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.n {
            N::PosInt(u) => visitor.visit_u64(u),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(NumberDeserializer {
            visited: false,
            number: self.n.into(),
        })
    }

    deserialize_number!(deserialize_i8 => visit_i8);
    deserialize_number!(deserialize_i16 => visit_i16);
    deserialize_number!(deserialize_i32 => visit_i32);
    deserialize_number!(deserialize_i64 => visit_i64);
    deserialize_number!(deserialize_u8 => visit_u8);
    deserialize_number!(deserialize_u16 => visit_u16);
    deserialize_number!(deserialize_u32 => visit_u32);
    deserialize_number!(deserialize_u64 => visit_u64);
    deserialize_number!(deserialize_f32 => visit_f32);
    deserialize_number!(deserialize_f64 => visit_f64);

    #[cfg(not(feature = "arbitrary_precision"))]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[cfg(feature = "arbitrary_precision")]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.n)
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[cfg(feature = "arbitrary_precision")]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(&self.n)
    }

    forward_to_deserialize_any! {
        bool char bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

impl<'de> Deserializer<'de> for &'de Number {
    type Error = Error;

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.n {
            N::PosInt(u) => visitor.visit_u64(u),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(NumberDeserializer {
            visited: false,
            number: (&self.n as &str).into(),
        })
    }

    deserialize_number!(deserialize_i8 => visit_i8);
    deserialize_number!(deserialize_i16 => visit_i16);
    deserialize_number!(deserialize_i32 => visit_i32);
    deserialize_number!(deserialize_i64 => visit_i64);
    deserialize_number!(deserialize_u8 => visit_u8);
    deserialize_number!(deserialize_u16 => visit_u16);
    deserialize_number!(deserialize_u32 => visit_u32);
    deserialize_number!(deserialize_u64 => visit_u64);
    deserialize_number!(deserialize_f32 => visit_f32);
    deserialize_number!(deserialize_f64 => visit_f64);

    #[cfg(not(feature = "arbitrary_precision"))]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[cfg(feature = "arbitrary_precision")]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.n.clone())
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[cfg(feature = "arbitrary_precision")]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(&self.n)
    }

    forward_to_deserialize_any! {
        bool char bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

#[cfg(feature = "arbitrary_precision")]
// Not public API. Should be pub(crate).
#[doc(hidden)]
pub struct NumberDeserializer<'a> {
    pub visited: bool,
    pub number: Cow<'a, str>,
}

#[cfg(feature = "arbitrary_precision")]
impl<'de> MapAccess<'de> for NumberDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: de::DeserializeSeed<'de>,
    {
        if self.visited {
            return Ok(None)
        }
        self.visited = true;
        seed.deserialize(NumberFieldDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.number.to_owned().into_deserializer())
    }
}

#[cfg(feature = "arbitrary_precision")]
struct NumberFieldDeserializer;

#[cfg(feature = "arbitrary_precision")]
impl<'de> Deserializer<'de> for NumberFieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(SERDE_STRUCT_FIELD_NAME)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

#[cfg(not(feature = "arbitrary_precision"))]
macro_rules! from_signed {
    ($($signed_ty:ident)*) => {
        $(
            impl From<$signed_ty> for Number {
                #[inline]
                fn from(i: $signed_ty) -> Self {
                    if i < 0 {
                        Number { n: N::NegInt(i as i64) }
                    } else {
                        Number { n: N::PosInt(i as u64) }
                    }
                }
            }
        )*
    };
}

#[cfg(not(feature = "arbitrary_precision"))]
macro_rules! from_unsigned {
    ($($unsigned_ty:ident)*) => {
        $(
            impl From<$unsigned_ty> for Number {
                #[inline]
                fn from(u: $unsigned_ty) -> Self {
                    Number { n: N::PosInt(u as u64) }
                }
            }
        )*
    };
}

#[cfg(not(feature = "arbitrary_precision"))]
from_signed!(i8 i16 i32 i64 isize);
#[cfg(not(feature = "arbitrary_precision"))]
from_unsigned!(u8 u16 u32 u64 usize);

#[cfg(feature = "arbitrary_precision")]
macro_rules! from_primitive {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(primitive: $ty) -> Self {
                    let mut buf = Vec::new();
                    itoa::write(&mut buf, primitive).unwrap();
                    Number { n: String::from_utf8(buf).unwrap() }
                }
            }
        )*
    };
}

#[cfg(feature = "arbitrary_precision")]
from_primitive!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);

impl Number {
    #[cfg(not(feature = "arbitrary_precision"))]
    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn unexpected(&self) -> Unexpected {
        match self.n {
            N::PosInt(u) => Unexpected::Unsigned(u),
            N::NegInt(i) => Unexpected::Signed(i),
            N::Float(f) => Unexpected::Float(f),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn unexpected(&self) -> Unexpected {
        Unexpected::Other("number")
    }
}
