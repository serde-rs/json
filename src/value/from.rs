#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use super::Value;
use map::Map;
use number::Number;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(n.into())
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
serde_if_integer128! {
    from_integer! {
        i128 u128
    }
}

impl From<f32> for Value {
    /// Convert 32-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    /// Convert 64-bit floating point number to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Value::Null, Value::Number)
    }
}

impl From<bool> for Value {
    /// Convert boolean to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// ```
    fn from(f: bool) -> Self {
        Value::Bool(f)
    }
}

impl From<String> for Value {
    /// Convert `String` to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Value = s.into();
    /// ```
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    /// Convert string slice to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// ```
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    /// Convert copy-on-write string to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// ```
    ///
    /// ```edition2018
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Value = s.into();
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.into_owned())
    }
}

impl From<Map<String, Value>> for Value {
    /// Convert map (with string keys) to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::{Map, Value};
    ///
    /// let mut m = Map::new();
    /// m.insert("Lorem".to_string(), "ipsum".into());
    /// let x: Value = m.into();
    /// ```
    fn from(f: Map<String, Value>) -> Self {
        Value::Object(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Convert a `Vec` to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: Vec<T>) -> Self {
        Value::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Value>> From<&'a [T]> for Value {
    /// Convert a slice to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: &'a [T]) -> Self {
        Value::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Value>> ::core::iter::FromIterator<T> for Value {
    /// Convert an iteratable type to a `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// ```
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// ```
    ///
    /// ```edition2018
    /// use std::iter::FromIterator;
    /// use serde_json::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl From<()> for Value {
    /// Convert `()` to `Value`
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// use serde_json::Value;
    ///
    /// let u = ();
    /// let x: Value = u.into();
    /// ```
    fn from((): ()) -> Self {
        Value::Null
    }
}
