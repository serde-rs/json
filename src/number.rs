// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use error::Error;
use num_traits::NumCast;
use serde::de::{self, Visitor, Unexpected};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::{self, Debug, Display};
use std::i64;

/// Represents a JSON number, whether integer or floating point.
#[derive(Clone, PartialEq)]
pub struct Number {
    n: N,
}

// "N" is a prefix of "NegInt"... this is a false positive.
// https://github.com/Manishearth/rust-clippy/issues/1241
#[cfg_attr(feature = "cargo-clippy", allow(enum_variant_names))]
#[derive(Copy, Clone, Debug, PartialEq)]
enum N {
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
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
    /// // Numbers with a decimal point are not considered integers.
    /// assert!(!v["c"].is_i64());
    /// # }
    /// ```
    #[inline]
    pub fn is_i64(&self) -> bool {
        match self.n {
            N::PosInt(v) => v <= i64::MAX as u64,
            N::NegInt(_) => true,
            N::Float(_) => false,
        }
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
    /// // Numbers with a decimal point are not considered integers.
    /// assert!(!v["c"].is_u64());
    /// # }
    /// ```
    #[inline]
    pub fn is_u64(&self) -> bool {
        match self.n {
            N::PosInt(_) => true,
            N::NegInt(_) | N::Float(_) => false,
        }
    }

    /// Returns true if the `Number` can be represented by f64.
    ///
    /// For any Number on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// Currently this function returns true if and only if both `is_i64` and
    /// `is_u64` return false but this is not a guarantee in the future.
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
    /// assert!(!v["b"].is_f64());
    /// assert!(!v["c"].is_f64());
    /// # }
    /// ```
    #[inline]
    pub fn is_f64(&self) -> bool {
        match self.n {
            N::Float(_) => true,
            N::PosInt(_) | N::NegInt(_) => false,
        }
    }

    /// If the `Number` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
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
    /// assert_eq!(v["a"].as_i64(), Some(64));
    /// assert_eq!(v["b"].as_i64(), None);
    /// assert_eq!(v["c"].as_i64(), None);
    /// # }
    /// ```
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            N::PosInt(n) => NumCast::from(n),
            N::NegInt(n) => Some(n),
            N::Float(_) => None,
        }
    }

    /// If the `Number` is an integer, represent it as u64 if possible. Returns
    /// None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_json;
    /// #
    /// # fn main() {
    /// let v = json!({ "a": 64, "b": -64, "c": 256.0 });
    ///
    /// assert_eq!(v["a"].as_u64(), Some(64));
    /// assert_eq!(v["b"].as_u64(), None);
    /// assert_eq!(v["c"].as_u64(), None);
    /// # }
    /// ```
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self.n {
            N::PosInt(n) => Some(n),
            N::NegInt(n) => NumCast::from(n),
            N::Float(_) => None,
        }
    }

    /// Represents the number as f64 if possible. Returns None otherwise.
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate serde_json;
    /// #
    /// # fn main() {
    /// let v = json!({ "a": 256.0, "b": 64, "c": -64 });
    ///
    /// assert_eq!(v["a"].as_f64(), Some(256.0));
    /// assert_eq!(v["b"].as_f64(), Some(64.0));
    /// assert_eq!(v["c"].as_f64(), Some(-64.0));
    /// # }
    /// ```
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            N::PosInt(n) => NumCast::from(n),
            N::NegInt(n) => NumCast::from(n),
            N::Float(n) => Some(n),
        }
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
            Some(Number { n: N::Float(f) })
        } else {
            None
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.n {
            N::PosInt(i) => Display::fmt(&i, formatter),
            N::NegInt(i) => Display::fmt(&i, formatter),
            N::Float(f) => Display::fmt(&f, formatter),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.n, formatter)
    }
}

impl Serialize for Number {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.n {
            N::PosInt(i) => serializer.serialize_u64(i),
            N::NegInt(i) => serializer.serialize_i64(i),
            N::Float(f) => serializer.serialize_f64(f),
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number")
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Number, E>
            where
                E: de::Error,
            {
                Number::from_f64(value).ok_or_else(|| de::Error::custom("not a JSON number"))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

impl<'de> Deserializer<'de> for Number {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.n {
            N::PosInt(i) => visitor.visit_u64(i),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a> Deserializer<'de> for &'a Number {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.n {
            N::PosInt(i) => visitor.visit_u64(i),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

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

from_signed!(i8 i16 i32 i64 isize);
from_unsigned!(u8 u16 u32 u64 usize);

impl Number {
    // Not public API. Should be pub(crate).
    #[doc(hidden)]
    pub fn unexpected(&self) -> Unexpected {
        match self.n {
            N::PosInt(u) => Unexpected::Unsigned(u),
            N::NegInt(i) => Unexpected::Signed(i),
            N::Float(f) => Unexpected::Float(f),
        }
    }
}
