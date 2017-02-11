use error::Error;
use serde::de::{self, Visitor};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::{self, Display};
use std::i64;

#[cfg(not(feature = "arbitrary_precision"))]
use num_traits::NumCast;
#[cfg(not(feature = "arbitrary_precision"))]
use std::fmt::Debug;

#[cfg(feature = "arbitrary_precision")]
use std::io;
#[cfg(feature = "arbitrary_precision")]
use dtoa;
#[cfg(feature = "arbitrary_precision")]
use itoa;
#[cfg(feature = "arbitrary_precision")]
use value::{Value, ValueVisitor};

/// Represents a JSON number, whether integer or floating point.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "arbitrary_precision", derive(Debug))]
pub struct Number {
    n: N,
}

#[cfg(not(feature = "arbitrary_precision"))]
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

#[cfg(feature = "arbitrary_precision")]
type N = String;

macro_rules! cast_methods {
    ($(
        #[doc = $doc:tt]
        pub fn $method:ident(&self) -> Option<$ty:ident>;
    )+) => {
        $(
            #[cfg(not(feature = "arbitrary_precision"))]
            #[doc = $doc]
            pub fn $method(&self) -> Option<$ty> {
                match self.n {
                    N::PosInt(n) => NumCast::from(n),
                    N::NegInt(n) => NumCast::from(n),
                    N::Float(n) => NumCast::from(n),
                }
            }

            #[cfg(feature = "arbitrary_precision")]
            #[doc = $doc]
            pub fn $method(&self) -> Option<$ty> {
                self.n.parse().ok()
            }
        )*
    };
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

    cast_methods! {
        /// Returns the number represented as i64 if possible, or else None.
        pub fn as_i64(&self) -> Option<i64>;

        /// Returns the number represented as u64 if possible, or else None.
        pub fn as_u64(&self) -> Option<u64>;

        /// Returns the number represented as f64 if possible, or else None.
        pub fn as_f64(&self) -> Option<f64>;
    }

    /// Converts a finite f64 to a Number. Infinite or NaN values are not JSON
    /// numbers.
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
        where S: Serializer
    {
        match self.n {
            N::PosInt(i) => serializer.serialize_u64(i),
            N::NegInt(i) => serializer.serialize_i64(i),
            N::Float(f) => serializer.serialize_f64(f),
        }
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
                if let Some(u) = number.as_u64() {
                    self.serialize_u64(u)
                } else if let Some(i) = number.as_i64() {
                    self.serialize_i64(i)
                } else if let Some(f) = number.as_f64() {
                    self.serialize_f64(f)
                } else {
                    self.serialize_str(&number.n)
                }
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

        impl SerializeNumber for ::value::Serializer {
            fn serialize_number(self, number: &Number) -> Result<Value, Error> {
                Ok(Value::Number(number.clone()))
            }
        }

        serializer.serialize_number(self)
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

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        match self.n {
            N::PosInt(u) => visitor.visit_u64(u),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        trait VisitNumber: Visitor {
            fn visit_number(self, number: Number) -> Result<Self::Value, Error>;
        }

        impl<V> VisitNumber for V where V: Visitor {
            default fn visit_number(self, number: Number) -> Result<V::Value, Error> {
                if let Some(u) = number.as_u64() {
                    self.visit_u64(u)
                } else if let Some(i) = number.as_i64() {
                    self.visit_i64(i)
                } else if let Some(f) = number.as_f64() {
                    self.visit_f64(f)
                } else {
                    self.visit_string(number.n)
                }
            }
        }

        impl VisitNumber for NumberVisitor {
            #[inline]
            fn visit_number(self, number: Number) -> Result<Number, Error> {
                Ok(number)
            }
        }

        impl VisitNumber for ValueVisitor {
            #[inline]
            fn visit_number(self, number: Number) -> Result<Value, Error> {
                Ok(Value::Number(number))
            }
        }

        visitor.visit_number(self)
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
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
