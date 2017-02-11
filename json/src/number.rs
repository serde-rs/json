use dtoa;
use error::Error;
use itoa;
use serde::de::{self, Visitor};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::{self, Display};
use std::i64;

#[cfg(feature = "arbitrary_precision")]
use std::io;

/// Represents a JSON number, whether integer or floating point.
#[derive(Clone, Debug, PartialEq)]
pub struct Number {
    n: String,
}

impl Number {
    /// Returns true if the number can be represented by i64.
    #[inline]
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// Returns true if the number can be represented as u64.
    #[inline]
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// Returns true if the number can be represented as f64.
    #[inline]
    pub fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    /// Returns the number represented as i64 if possible, or else None.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        self.n.parse().ok()
    }

    /// Returns the number represented as u64 if possible, or else None.
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        self.n.parse().ok()
    }

    /// Returns the number represented as f64 if possible, or else None.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        self.n.parse().ok()
    }

    /// Converts a finite f64 to a Number. Infinite or NaN values are not JSON
    /// numbers.
    #[inline]
    pub fn from_f64(f: f64) -> Option<Number> {
        if f.is_finite() {
            let mut buf = Vec::new();
            dtoa::write(&mut buf, f).unwrap();
            Some(Number { n: String::from_utf8(buf).unwrap() })
        } else {
            None
        }
    }

    /// Not public API. Should be pub(crate). The deserializer uses this.
    #[doc(hidden)]
    #[inline]
    pub fn from_string_unchecked(n: String) -> Self {
        Number { n: n }
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.n)
    }
}

impl Serialize for Number {
    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serialize_number(serializer, self)
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        trait SerializeNumber: Serializer {
            fn serialize_number(self, number: &Number) -> Result<Self::Ok, Self::Error>;
        }

        impl<T> SerializeNumber for T where T: Serializer {
            #[inline]
            default fn serialize_number(self, number: &Number) -> Result<T::Ok, T::Error> {
                serialize_number(self, number)
            }
        }

        impl<'a, W, F> SerializeNumber for &'a mut ::ser::Serializer<W, F>
            where W: io::Write,
                  F: ::ser::Formatter
        {
            #[inline]
            fn serialize_number(self, number: &Number) -> Result<(), Error> {
                self.write_number_str(&number.n)
            }
        }

        serializer.serialize_number(self)
    }
}

fn serialize_number<S>(serializer: S, number: &Number) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    if let Some(u) = number.as_u64() {
        serializer.serialize_u64(u)
    } else if let Some(i) = number.as_i64() {
        serializer.serialize_i64(i)
    } else if let Some(f) = number.as_f64() {
        serializer.serialize_f64(f)
    } else {
        serializer.serialize_str(&number.n)
    }
}

/// Not public API. Should be pub(crate). The deserializer specializes on this
/// type.
#[doc(hidden)]
pub struct NumberVisitor;

impl Visitor for NumberVisitor {
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
        where E: de::Error
    {
        Number::from_f64(value).ok_or_else(|| de::Error::custom("not a JSON number"))
    }
}

impl Deserialize for Number {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(NumberVisitor)
    }
}

impl Deserializer for Number {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        if let Some(u) = self.as_u64() {
            visitor.visit_u64(u)
        } else if let Some(i) = self.as_i64() {
            visitor.visit_i64(i)
        } else if let Some(f) = self.as_f64() {
            visitor.visit_f64(f)
        } else {
            visitor.visit_string(self.n)
        }
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

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

from_primitive!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);
