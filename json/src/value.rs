//! JSON Value
//!
//! This module is centered around the `Value` type, which can represent all possible JSON values.
//!
//! # Example of use:
//!
//! ```rust
//! extern crate serde_json;
//!
//! use serde_json::Value;
//!
//! fn main() {
//!     let s = "{\"x\": 1.0, \"y\": 2.0}";
//!     let value: Value = serde_json::from_str(s).unwrap();
//! }
//! ```
//!
//! It is also possible to deserialize from a `Value` type:
//!
//! ```rust
//! extern crate serde_json;
//!
//! use serde_json::{Value, Map};
//!
//! fn main() {
//!     let mut map = Map::new();
//!     map.insert(String::from("x"), 1.0.into());
//!     map.insert(String::from("y"), 2.0.into());
//!     let value = Value::Object(map);
//!
//!     let map: Map<String, Value> = serde_json::from_value(value).unwrap();
//! }
//! ```

use std::fmt;
use std::i64;
use std::io;
use std::str;
use std::vec;

use serde::de::{self, Unexpected};
use serde::ser;
use serde::de::value::ValueDeserializer;

use error::{Error, ErrorCode, TypeError};

pub use map::Map;
pub use number::Number;

/// Represents any valid JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represents a JSON null value.
    Null,

    /// Represents a JSON boolean.
    Bool(bool),

    /// Represents a JSON number, whether integer or floating point.
    Number(Number),

    /// Represents a JSON string.
    String(String),

    /// Represents a JSON array.
    Array(Vec<Value>),

    /// Represents a JSON object.
    Object(Map<String, Value>),
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}

impl Value {
    /// If the `Value` is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &str) -> Option<&'a Value> {
        match *self {
            Value::Object(ref map) => map.get(key),
            _ => None,
        }
    }

    /// Attempts to get a nested Value Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the `Value` associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&str]) -> Option<&'a Value> {
        let mut target = self;
        for key in keys {
            match target.find(key) {
                Some(t) => {
                    target = t;
                }
                None => return None,
            }
        }
        Some(target)
    }

    /// Looks up a value by a JSON Pointer.
    ///
    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    pub fn pointer<'a>(&'a self, pointer: &str) -> Option<&'a Value> {
        if pointer == "" {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let tokens = pointer.split('/').skip(1).map(|x| x.replace("~1", "/").replace("~0", "~"));
        let mut target = self;

        for token in tokens {
            let target_opt = match *target {
                Value::Object(ref map) => map.get(&token),
                Value::Array(ref list) => parse_index(&token).and_then(|x| list.get(x)),
                _ => return None,
            };
            if let Some(t) = target_opt {
                target = t;
            } else {
                return None;
            }
        }
        Some(target)
    }

    /// Looks up a value by a JSON Pointer and returns a mutable reference to
    /// that value.
    ///
    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    ///
    /// # Example of Use
    ///
    /// ```rust
    /// extern crate serde_json;
    ///
    /// use serde_json::Value;
    /// use std::mem;
    ///
    /// fn main() {
    ///     let s = r#"{"x": 1.0, "y": 2.0}"#;
    ///     let mut value: Value = serde_json::from_str(s).unwrap();
    ///
    ///     // Check value using read-only pointer
    ///     assert_eq!(value.pointer("/x"), Some(&1.0.into()));
    ///     // Change value with direct assignment
    ///     *value.pointer_mut("/x").unwrap() = 1.5.into();
    ///     // Check that new value was written
    ///     assert_eq!(value.pointer("/x"), Some(&1.5.into()));
    ///
    ///     // "Steal" ownership of a value. Can replace with any valid Value.
    ///     let old_x = value.pointer_mut("/x").map(|x| mem::replace(x, Value::Null)).unwrap();
    ///     assert_eq!(old_x, 1.5.into());
    ///     assert_eq!(value.pointer("/x").unwrap(), &Value::Null);
    /// }
    /// ```
    pub fn pointer_mut<'a>(&'a mut self, pointer: &str) -> Option<&'a mut Value> {
        if pointer == "" {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let tokens = pointer.split('/').skip(1).map(|x| x.replace("~1", "/").replace("~0", "~"));
        let mut target = self;

        for token in tokens {
            // borrow checker gets confused about `target` being mutably borrowed too many times because of the loop
            // this once-per-loop binding makes the scope clearer and circumvents the error
            let target_once = target;
            let target_opt = match *target_once {
                Value::Object(ref mut map) => map.get_mut(&token),
                Value::Array(ref mut list) => parse_index(&token).and_then(move |x| list.get_mut(x)),
                _ => return None,
            };
            if let Some(t) = target_opt {
                target = t;
            } else {
                return None;
            }
        }
        Some(target)
    }

    /// If the `Value` is an Object, performs a depth-first search until
    /// a value associated with the provided key is found. If no value is found
    /// or the `Value` is not an Object, returns None.
    pub fn search<'a>(&'a self, key: &str) -> Option<&'a Value> {
        match *self {
            Value::Object(ref map) => {
                match map.get(key) {
                    Some(json_value) => Some(json_value),
                    None => {
                        for (_, v) in map.iter() {
                            match v.search(key) {
                                x if x.is_some() => return x,
                                _ => (),
                            }
                        }
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /// If the `Value` is of type `T`, returns borrow of internal value.
    /// Returns Err otherwise.
    ///
    /// Useful for generics with larger/allocated types to speed things up.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate serde_json;
    ///
    /// use serde_json::{to_value,Value};
    /// use serde_json::value::ValueType;
    /// use std::collections::BTreeMap;
    ///
    /// fn main() {
    ///     let null = to_value(());
    ///     let boolean = to_value(true);
    ///     let int64 = to_value(-42i64);
    ///     let uint64 = to_value(18446744073709551337u64);
    ///     let float = to_value(3.1415926535f64);
    ///     let string = to_value("Hello world!");
    ///     let array = to_value(&[1, 2, 3, 4, 5]);
    ///     let mut map = BTreeMap::new();
    ///     map.insert("foo", to_value("bar"));
    ///     let map = to_value(&map);
    ///
    ///     let _: &() = null.as_borrow().unwrap();
    ///     let _: &bool = boolean.as_borrow().unwrap();
    ///     let _: &i64 = int64.as_borrow().unwrap();
    ///     let _: &u64 = uint64.as_borrow().unwrap();
    ///     let _: &f64 = float.as_borrow().unwrap();
    ///     let _: &str = string.as_borrow().unwrap();
    ///     let _: &String = string.as_borrow().unwrap();
    ///     let _: &[Value] = array.as_borrow().unwrap();
    ///     let _: &Vec<Value> = array.as_borrow().unwrap();
    ///     let _: &BTreeMap<String, Value> = map.as_borrow().unwrap();
    /// }
    pub fn as_borrow<'a, T: TryBorrow<'a>>(&'a self) -> Result<T, TypeError> {
        T::try_borrow(self)
    }

    /// If the `Value` is of type `T`, returns mutable borrow of internal value.
    /// Returns Err otherwise.
    ///
    /// Useful for generics with larger/allocated types to speed things up.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate serde_json;
    ///
    /// use serde_json::{to_value,Value};
    /// use serde_json::value::ValueType;
    /// use std::collections::BTreeMap;
    ///
    /// fn main() {
    ///     let mut null = to_value(());
    ///     let mut boolean = to_value(true);
    ///     let mut int64 = to_value(-42i64);
    ///     let mut uint64 = to_value(18446744073709551337u64);
    ///     let mut float = to_value(3.1415926535f64);
    ///     let mut string = to_value("Hello world!");
    ///     let mut array = to_value(&[1, 2, 3, 4, 5]);
    ///     let mut map = BTreeMap::new();
    ///     map.insert("foo", to_value("bar"));
    ///     let mut map = to_value(&map);
    ///
    ///     let _: &mut () = null.as_borrow_mut().unwrap();
    ///     let _: &mut bool = boolean.as_borrow_mut().unwrap();
    ///     let _: &mut i64 = int64.as_borrow_mut().unwrap();
    ///     let _: &mut u64 = uint64.as_borrow_mut().unwrap();
    ///     let _: &mut f64 = float.as_borrow_mut().unwrap();
    ///     {
    ///         let _: &mut str = string.as_borrow_mut().unwrap();
    ///     }
    ///     let _: &mut String = string.as_borrow_mut().unwrap();
    ///     {
    ///         let _: &mut [Value] = array.as_borrow_mut().unwrap();
    ///     }
    ///     let _: &mut Vec<Value> = array.as_borrow_mut().unwrap();
    ///     let _: &mut BTreeMap<String, Value> = map.as_borrow_mut().unwrap();
    /// }
    pub fn as_borrow_mut<'a, T: TryBorrowMut<'a>>(&'a mut self) -> Result<T, TypeError> {
        T::try_borrow_mut(self)
    }

    /// Returns true if the `Value` is an Object. Returns false otherwise.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// If the `Value` is an Object, returns the associated Map.
    /// Returns None otherwise.
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match *self {
            Value::Object(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match *self {
            Value::Object(ref mut map) => Some(map),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// If the `Value` is an Array, returns the associated vector.
    /// Returns None otherwise.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref array) => Some(&*array),
            _ => None,
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self {
            Value::Array(ref mut list) => Some(list),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// If the `Value` is a String, returns the associated str.
    /// Returns None otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by i64.
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_i64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by u64.
    pub fn is_u64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_u64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    /// If the `Value` is a number, represent it as i64 if possible.
    /// Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    /// If the `Value` is a number, represent it as u64 if possible.
    /// Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Number(ref n) => n.as_u64(),
            _ => None,
        }
    }

    /// If the `Value` is a number, represent it as f64 if possible.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(ref n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the `Value` is a Null, returns ().
    /// Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Null => Some(()),
            _ => None,
        }
    }
}

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

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Value::Null, Value::Number)
    }
}

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::Number(ref n) => n.serialize(serializer),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Array(ref v) => v.serialize(serializer),
            Value::Object(ref m) => {
                use serde::ser::SerializeMap;
                let mut map = try!(serializer.serialize_map(Some(m.len())));
                for (k, v) in m {
                    try!(map.serialize_key(k));
                    try!(map.serialize_value(v));
                }
                map.end()
            }
        }
    }
}

impl de::Deserialize for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
        where D: de::Deserializer,
    {
        struct ValueVisitor;

        impl de::Visitor for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
                where E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_some<D>(
                self,
                deserializer: D
            ) -> Result<Value, D::Error>
                where D: de::Deserializer,
            {
                de::Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_seq<V>(self, visitor: V) -> Result<Value, V::Error>
                where V: de::SeqVisitor,
            {
                let values = try!(de::impls::VecVisitor::new()
                    .visit_seq(visitor));
                Ok(Value::Array(values))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: de::MapVisitor,
            {
                let mut values = Map::with_capacity(visitor.size_hint().0);

                while let Some((key, value)) = try!(visitor.visit()) {
                    values.insert(key, value);
                }

                Ok(Value::Object(values))
            }
        }

        deserializer.deserialize(ValueVisitor)
    }
}

/// Represents type of a JSON value
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ValueType {
    /// Represents a JSON null value type
    Null,

    /// Represents a JSON Boolean type
    Bool,

    /// Represents a JSON signed integer type
    I64,

    /// Represents a JSON unsigned integer type
    U64,

    /// Represents a JSON floating point number type
    F64,

    /// Represents a JSON string type
    String,

    /// Represents a JSON array type
    Array,

    /// Represents a JSON object type
    Object,
}

impl ValueType {

    /// Returns type of a value
    /// 
    /// # Example
    ///
    /// ```rust
    /// extern crate serde_json;
    ///
    /// use serde_json::to_value;
    /// use serde_json::value::ValueType;
    /// use std::collections::BTreeMap;
    ///
    /// fn main() {
    ///     let null = to_value(());
    ///     let boolean = to_value(true);
    ///     let int64 = to_value(-42i64);
    ///     let uint64 = to_value(18446744073709551337u64);
    ///     let float = to_value(3.1415926535f64);
    ///     let string = to_value("Hello world!");
    ///     let array = to_value(&[1, 2, 3, 4, 5]);
    ///     let mut map = BTreeMap::new();
    ///     map.insert("foo", to_value("bar"));
    ///     let map = to_value(&map);
    ///
    ///     assert_eq!(ValueType::of(&null), ValueType::Null);
    ///     assert_eq!(ValueType::of(&boolean), ValueType::Bool);
    ///     assert_eq!(ValueType::of(&int64), ValueType::I64);
    ///     assert_eq!(ValueType::of(&uint64), ValueType::U64);
    ///     assert_eq!(ValueType::of(&float), ValueType::F64);
    ///     assert_eq!(ValueType::of(&string), ValueType::String);
    ///     assert_eq!(ValueType::of(&array), ValueType::Array);
    ///     assert_eq!(ValueType::of(&map), ValueType::Object);
    /// }
    pub fn of(val: &Value) -> Self {
        match *val {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::I64(_) => ValueType::I64,
            Value::U64(_) => ValueType::U64,
            Value::F64(_) => ValueType::F64,
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ValueType::*;

        write!(f, "{}", match *self {
            Null => "Null",
            Bool => "Bool",
            I64 => "I64",
            U64 => "U64",
            F64 => "F64",
            String => "String",
            Array => "Array",
            Object => "Object",
        })
    }
}

struct WriterFormatter<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> io::Write for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        fn io_error<E>(_: E) -> io::Error {
            // Value does not matter because fmt::Debug and fmt::Display impls
            // below just map it to fmt::Error
            io::Error::new(io::ErrorKind::Other, "fmt error")
        }
        let s = try!(str::from_utf8(buf).map_err(io_error));
        try!(self.inner.write_str(s).map_err(io_error));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut wr = WriterFormatter {
            inner: f,
        };
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
    }
}

impl str::FromStr for Value {
    type Err = Error;
    fn from_str(s: &str) -> Result<Value, Error> {
        super::de::from_str(s)
    }
}

struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value, Error> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn serialize_isize(self, value: isize) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Value, Error> {
        Ok(Value::Number(value.into()))
    }

    #[inline]
    fn serialize_usize(self, value: usize) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, Error> {
        Ok(Value::Number(value.into()))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, Error> {
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, Error> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value, Error> {
        let vec = value.iter().map(|&b| Value::Number(b.into())).collect();
        Ok(Value::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str
    ) -> Result<Value, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str
    ) -> Result<Value, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: T
    ) -> Result<Value, Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        value: T
    ) -> Result<Value, Error>
        where T: ser::Serialize,
    {
        let mut values = Map::new();
        values.insert(String::from(variant), try!(to_value(&value)));
        Ok(Value::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(self, value: V) -> Result<Value, Error>
        where V: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0))
        })
    }

    fn serialize_seq_fixed_size(
        self,
        size: usize
    ) -> Result<Self::SerializeSeq, Error> {
        self.serialize_seq(Some(size))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(
        self,
        _len: Option<usize>
    ) -> Result<Self::SerializeMap, Error> {
        Ok(SerializeMap {
            map: Map::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: usize,
        variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Map::new(),
        })
    }
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Value>,
}

#[doc(hidden)]
pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

#[doc(hidden)]
pub struct SerializeMap {
    map: Map<String, Value>,
    next_key: Option<String>,
}

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: String,
    map: Map<String, Value>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        self.vec.push(try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        self.vec.push(try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        let mut object = Map::new();

        object.insert(self.name, Value::Array(self.vec));

        Ok(Value::Object(object))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        match try!(to_value(&key)) {
            Value::String(s) => self.next_key = Some(s),
            _ => return Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0)),
        };
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Object(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: V) -> Result<(), Error>
        where V: ser::Serialize
    {
        try!(ser::SerializeMap::serialize_key(self, key));
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        ser::SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: V) -> Result<(), Error>
        where V: ser::Serialize
    {
        self.map.insert(String::from(key), try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        let mut object = Map::new();

        object.insert(self.name, Value::Object(self.map));

        Ok(Value::Object(object))
    }
}

impl de::Deserializer for Value {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(n) => n.deserialize(visitor),
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = try!(visitor.visit_seq(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
            Value::Object(v) => {
                let len = v.len();
                let mut deserializer = MapDeserializer::new(v);
                let map = try!(visitor.visit_map(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in map"))
                }
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(
        self,
        visitor: V
    ) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(de::Error::invalid_value(Unexpected::Map, &"map with a single key"));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_value(Unexpected::Map, &"map with a single key"));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(de::Error::invalid_type(other.unexpected(), &"string or map"));
            }
        };

        visitor.visit_enum(EnumDeserializer {
            variant: variant,
            value: value,
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit seq seq_fixed_size bytes byte_buf map unit_struct tuple_struct
        struct struct_field tuple ignored_any
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl de::EnumVisitor for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
        where V: de::DeserializeSeed,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl de::VariantVisitor for VariantDeserializer {
    type Error = Error;

    fn visit_unit(self) -> Result<(), Error> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where T: de::DeserializeSeed,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant")),
        }
    }

    fn visit_tuple<V>(
        self,
        _len: usize,
        visitor: V
    ) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.value {
            Some(Value::Array(v)) => {
                de::Deserializer::deserialize(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"tuple variant")),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
        }
    }

    fn visit_struct<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.value {
            Some(Value::Object(v)) => {
                de::Deserializer::deserialize(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"struct variant")),
            _ => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"))
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl de::Deserializer for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(de::Error::invalid_length(len, &"fewer elements in array"))
            }
        }
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit option seq seq_fixed_size bytes byte_buf map unit_struct
        newtype_struct tuple_struct struct struct_field tuple enum ignored_any
    }
}

impl de::SeqVisitor for SeqDeserializer {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: de::DeserializeSeed,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct MapDeserializer {
    iter: <Map<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Map<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl de::MapVisitor for MapDeserializer {
    type Error = Error;

    fn visit_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: de::DeserializeSeed,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn visit_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
        where T: de::DeserializeSeed,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl de::Deserializer for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit option seq seq_fixed_size bytes byte_buf map unit_struct
        newtype_struct tuple_struct struct struct_field tuple enum ignored_any
    }
}

impl Value {
    fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Null => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(b),
            Value::Number(ref n) => {
                if let Some(u) = n.as_u64() {
                    Unexpected::Unsigned(u)
                } else if let Some(i) = n.as_i64() {
                    Unexpected::Signed(i)
                } else if let Some(f) = n.as_f64() {
                    Unexpected::Float(f)
                } else {
                    panic!("unexpected number")
                }
            }
            Value::String(ref s) => Unexpected::Str(s),
            Value::Array(_) => Unexpected::Seq,
            Value::Object(_) => Unexpected::Map,
        }
    }
}

/// Convert a `T` into `serde_json::Value` which is an enum that can represent
/// any valid JSON data.
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
///
/// ```rust
/// # use serde_json::Value;
/// let val = serde_json::to_value("s").unwrap();
/// assert_eq!(val, Value::String("s".to_owned()));
/// ```
pub fn to_value<T>(value: T) -> Result<Value, Error>
    where T: ser::Serialize,
{
    value.serialize(Serializer)
}

/// Interpret a `serde_json::Value` as an instance of type `T`.
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than a JSON map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the JSON map or some number is too big to fit in the expected primitive
/// type.
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: de::Deserialize,
{
    de::Deserialize::deserialize(value)
}

/// Representation of any serializable data as a `serde_json::Value`.
pub trait ToJson {
    /// Represent `self` as a `serde_json::Value`. Note that `Value` is not a
    /// JSON string. If you need a string, use `serde_json::to_string` instead.
    ///
    /// This conversion can fail if `T`'s implementation of `Serialize` decides
    /// to fail, or if `T` contains a map with non-string keys.
    fn to_json(&self) -> Result<Value, Error>;
}

impl<T: ?Sized> ToJson for T
    where T: ser::Serialize,
{
    fn to_json(&self) -> Result<Value, Error> {
        to_value(self)
    }
}

/// Just like `std::borrow::Borrow<Value>` but borrowing may fail.
pub trait TryBorrow<'a>: 'a + ::std::marker::Sized {
    /// Returns reference to internal data if type matches.
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError>;
}

impl<'a> TryBorrow<'a> for &'a () {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        static EMPTY: () = ();
        if value.is_null() { Ok(&EMPTY) } else { Err(TypeError { expected: ValueType::Null, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a bool {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::Bool(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Bool, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a u64 {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::U64(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::U64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a i64 {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::I64(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::I64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a f64 {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::F64(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::F64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a str {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::String(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::String, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a String {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::String(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::String, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a [Value] {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::Array(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Array, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a Vec<Value> {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::Array(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Array, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrow<'a> for &'a Map<String, Value> {
    fn try_borrow(value: &'a Value) -> Result<Self, TypeError> {
        if let &Value::Object(ref val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Object, provided: ValueType::of(value) }) }
    }
}

/// Just like `std::borrow::BorrowMut` but borrowing may fail.
pub trait TryBorrowMut<'a>: 'a + ::std::marker::Sized {
    /// Returns mutable reference to internal data if type matches.
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError>;
}

impl<'a> TryBorrowMut<'a> for &'a mut () {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        static mut EMPTY: () = ();
        // Use of static &mut () is always safe, because it's value actually can't be changed
        if value.is_null() { unsafe { Ok(&mut EMPTY) } } else { Err(TypeError { expected: ValueType::Null, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut bool {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::Bool(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Bool, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut u64 {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::U64(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::U64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut i64 {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::I64(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::I64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut f64 {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::F64(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::F64, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut str {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::String(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::String, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut String {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::String(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::String, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut [Value] {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::Array(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Array, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut Vec<Value> {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::Array(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Array, provided: ValueType::of(value) }) }
    }
}

impl<'a> TryBorrowMut<'a> for &'a mut Map<String, Value> {
    fn try_borrow_mut(value: &'a mut Value) -> Result<Self, TypeError> {
        if let &mut Value::Object(ref mut val) = value { Ok(val) } else { Err(TypeError { expected: ValueType::Object, provided: ValueType::of(value) }) }
    }
}

