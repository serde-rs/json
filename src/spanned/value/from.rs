use crate::map::SpannedMap;
use crate::number::Number;
use crate::spanned::spanned::Spanned;
use crate::spanned::value::SpannedValue;
use alloc::borrow::{Cow, ToOwned};
use alloc::string::String;
use alloc::vec::Vec;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for SpannedValue {
                fn from(n: $ty) -> Self {
                    SpannedValue::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

#[cfg(feature = "arbitrary_precision")]
from_integer! {
    i128 u128
}

impl From<f32> for SpannedValue {
    /// Convert 32-bit floating point number to `Value::Number`, or
    /// `Value::Null` if infinite or NaN.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f32) -> Self {
        Number::from_f32(f).map_or(SpannedValue::Null, SpannedValue::Number)
    }
}

impl From<f64> for SpannedValue {
    /// Convert 64-bit floating point number to `Value::Number`, or
    /// `Value::Null` if infinite or NaN.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(SpannedValue::Null, SpannedValue::Number)
    }
}

impl From<bool> for SpannedValue {
    /// Convert boolean to `Value::Bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// ```
    fn from(f: bool) -> Self {
        SpannedValue::Bool(f)
    }
}

impl From<String> for SpannedValue {
    /// Convert `String` to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let s: String = "lorem".to_owned();
    /// let x: Value = s.into();
    /// ```
    fn from(f: String) -> Self {
        SpannedValue::String(f)
    }
}

impl From<&str> for SpannedValue {
    /// Convert string slice to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// ```
    fn from(f: &str) -> Self {
        SpannedValue::String(f.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for SpannedValue {
    /// Convert copy-on-write string to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// ```
    ///
    /// ```
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_owned());
    /// let x: Value = s.into();
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        SpannedValue::String(f.into_owned())
    }
}

impl From<Number> for SpannedValue {
    /// Convert `Number` to `Value::Number`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::{Number, Value};
    ///
    /// let n = Number::from(7);
    /// let x: Value = n.into();
    /// ```
    fn from(f: Number) -> Self {
        SpannedValue::Number(f)
    }
}

impl From<SpannedMap> for SpannedValue {
    /// Convert map (with string keys) to `Value::Object`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::{Map, Value};
    ///
    /// let mut m = Map::new();
    /// m.insert("Lorem".to_owned(), "ipsum".into());
    /// let x: Value = m.into();
    /// ```
    fn from(f: SpannedMap) -> Self {
        SpannedValue::Object(f)
    }
}

impl<T: Into<Spanned<SpannedValue>>> From<Vec<T>> for SpannedValue {
    /// Convert a `Vec` to `Value::Array`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: Vec<T>) -> Self {
        SpannedValue::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Spanned<SpannedValue>>, const N: usize> From<[T; N]> for SpannedValue {
    fn from(array: [T; N]) -> Self {
        SpannedValue::Array(array.into_iter().map(Into::into).collect())
    }
}

impl<T: Clone + Into<Spanned<SpannedValue>>> From<&[T]> for SpannedValue {
    /// Convert a slice to `Value::Array`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: &[T]) -> Self {
        SpannedValue::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Spanned<SpannedValue>>> FromIterator<T> for SpannedValue {
    /// Create a `Value::Array` by collecting an iterator of array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// ```
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// ```
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use serde_json::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        SpannedValue::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<Spanned<String>>, V: Into<Spanned<SpannedValue>>> FromIterator<(K, V)>
    for SpannedValue
{
    /// Create a `Value::Object` by collecting an iterator of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: Vec<_> = vec![("lorem", 40), ("ipsum", 2)];
    /// let x: Value = v.into_iter().collect();
    /// ```
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        SpannedValue::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<()> for SpannedValue {
    /// Convert `()` to `Value::Null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let u = ();
    /// let x: Value = u.into();
    /// ```
    fn from((): ()) -> Self {
        SpannedValue::Null
    }
}

impl<T> From<Option<T>> for SpannedValue
where
    T: Into<SpannedValue>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => SpannedValue::Null,
            Some(value) => Into::into(value),
        }
    }
}
