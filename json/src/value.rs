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
//!     map.insert(String::from("x"), Value::F64(1.0));
//!     map.insert(String::from("y"), Value::F64(2.0));
//!     let value = Value::Object(map);
//!
//!     let map: Map<String, f64> = serde_json::from_value(value).unwrap();
//! }
//! ```

#[cfg(not(feature = "preserve_order"))]
use std::collections::{BTreeMap, btree_map};

#[cfg(feature = "preserve_order")]
use linked_hash_map::{self, LinkedHashMap};

use std::fmt;
use std::io;
use std::str;
use std::vec;

use num_traits::NumCast;

use serde::de;
use serde::ser;
use serde::de::value::ValueDeserializer;

use error::{Error, ErrorCode};

/// Represents a key/value type.
#[cfg(not(feature = "preserve_order"))]
pub type Map<K, V> = BTreeMap<K, V>;
/// Represents a key/value type.
#[cfg(feature = "preserve_order")]
pub type Map<K, V> = LinkedHashMap<K, V>;

/// Represents the `IntoIter` type.
#[cfg(not(feature = "preserve_order"))]
pub type MapIntoIter<K, V> = btree_map::IntoIter<K, V>;
/// Represents the IntoIter type.
#[cfg(feature = "preserve_order")]
pub type MapIntoIter<K, V> = linked_hash_map::IntoIter<K, V>;

/// Represents a JSON value
#[derive(Clone, PartialEq)]
pub enum Value {
    /// Represents a JSON null value
    Null,

    /// Represents a JSON Boolean
    Bool(bool),

    /// Represents a JSON signed integer
    I64(i64),

    /// Represents a JSON unsigned integer
    U64(u64),

    /// Represents a JSON floating point number
    F64(f64),

    /// Represents a JSON string
    String(String),

    /// Represents a JSON array
    Array(Vec<Value>),

    /// Represents a JSON object
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

    /// **Deprecated**: Use `Value.pointer()` and pointer syntax instead.
    ///
    /// Looks up a value by path.
    ///
    /// This is a convenience method that splits the path by `'.'`
    /// and then feeds the sequence of keys into the `find_path`
    /// method.
    ///
    /// ``` ignore
    /// let obj: Value = json::from_str(r#"{"x": {"a": 1}}"#).unwrap();
    ///
    /// assert!(obj.lookup("x.a").unwrap() == &Value::U64(1));
    /// ```
    pub fn lookup<'a>(&'a self, path: &str) -> Option<&'a Value> {
        let mut target = self;
        for key in path.split('.') {
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
    ///     assert_eq!(value.pointer("/x"), Some(&Value::F64(1.0)));
    ///     // Change value with direct assignment
    ///     *value.pointer_mut("/x").unwrap() = Value::F64(1.5);
    ///     // Check that new value was written
    ///     assert_eq!(value.pointer("/x"), Some(&Value::F64(1.5)));
    ///
    ///     // "Steal" ownership of a value. Can replace with any valid Value.
    ///     let old_x = value.pointer_mut("/x").map(|x| mem::replace(x, Value::Null)).unwrap();
    ///     assert_eq!(old_x, Value::F64(1.5));
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
            Value::I64(_) | Value::U64(_) | Value::F64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a i64. Returns false otherwise.
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::I64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a u64. Returns false otherwise.
    pub fn is_u64(&self) -> bool {
        match *self {
            Value::U64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::F64(_) => true,
            _ => false,
        }
    }

    /// If the `Value` is a number, return or cast it to a i64.
    /// Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(n) => Some(n),
            Value::U64(n) => NumCast::from(n),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a u64.
    /// Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::I64(n) => NumCast::from(n),
            Value::U64(n) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::I64(n) => NumCast::from(n),
            Value::U64(n) => NumCast::from(n),
            Value::F64(n) => Some(n),
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

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::I64(i) => serializer.serialize_i64(i),
            Value::U64(u) => serializer.serialize_u64(u),
            Value::F64(f) => serializer.serialize_f64(f),
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

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value::I64(value))
                } else {
                    Ok(Value::U64(value as u64))
                }
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::U64(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::F64(value))
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
                // Work around not having attributes on statements in Rust 1.12.0
                #[cfg(not(feature = "preserve_order"))]
                macro_rules! init {
                    () => {
                        BTreeMap::new()
                    };
                }
                #[cfg(feature = "preserve_order")]
                macro_rules! init {
                    () => {
                        LinkedHashMap::with_capacity(visitor.size_hint().0)
                    };
                }
                let mut values = init!();

                while let Some((key, value)) = try!(visitor.visit()) {
                    values.insert(key, value);
                }

                Ok(Value::Object(values))
            }
        }

        deserializer.deserialize(ValueVisitor)
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

impl fmt::Debug for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut wr = WriterFormatter {
            inner: f,
        };
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
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
        Ok(if value < 0 {
            Value::I64(value)
        } else {
            Value::U64(value as u64)
        })
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
        Ok(Value::U64(value))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, Error> {
        Ok(if value.is_finite() {
            Value::F64(value)
        } else {
            Value::Null
        })
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
        let vec = value.iter().map(|b| Value::U64(*b as u64)).collect();
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
        match self.next_key.take() {
            Some(key) => self.map.insert(key, try!(to_value(&value))),
            None => {
                return Err(Error::Syntax(ErrorCode::Custom("serialize_map_value without \
                                                            matching serialize_map_key".to_owned()),
                                         0, 0));
            }
        };
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
            Value::I64(v) => visitor.visit_i64(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::F64(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => {
                let mut deserializer = SeqDeserializer::new(v);
                let seq = try!(visitor.visit_seq(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(de::Error::invalid_length(remaining))
                }
            }
            Value::Object(v) => {
                let mut deserializer = MapDeserializer::new(v);
                let map = try!(visitor.visit_map(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(de::Error::invalid_length(remaining))
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
                        return Err(de::Error::invalid_type(de::Type::VariantName));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_type(de::Type::Map));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            _ => {
                return Err(de::Error::invalid_type(de::Type::Enum));
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
        unit seq seq_fixed_size bytes map unit_struct tuple_struct struct
        struct_field tuple ignored_any
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl de::EnumVisitor for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn visit_variant<V>(self) -> Result<(V, VariantDeserializer), Error>
        where V: de::Deserialize,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        de::Deserialize::deserialize(variant).map(|v| (v, visitor))
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

    fn visit_newtype<T>(self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        match self.value {
            Some(value) => de::Deserialize::deserialize(value),
            None => Err(de::Error::invalid_type(de::Type::NewtypeVariant)),
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
            Some(Value::Array(fields)) => {
                de::Deserializer::deserialize(SeqDeserializer {
                                                  iter: fields.into_iter(),
                                              },
                                              visitor)
            }
            _ => Err(de::Error::invalid_type(de::Type::TupleVariant))
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
            Some(Value::Object(fields)) => {
                de::Deserializer::deserialize(MapDeserializer {
                                                  iter: fields.into_iter(),
                                                  value: None,
                                              },
                                              visitor)
            }
            _ => Err(de::Error::invalid_type(de::Type::StructVariant))
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
        if self.iter.len() == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(de::Error::invalid_length(remaining))
            }
        }
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit option seq seq_fixed_size bytes map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

impl de::SeqVisitor for SeqDeserializer {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.iter.next() {
            Some(value) => de::Deserialize::deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct MapDeserializer {
    iter: MapIntoIter<String, Value>,
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

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                de::Deserialize::deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn visit_value<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        match self.value.take() {
            Some(value) => de::Deserialize::deserialize(value),
            None => Err(de::Error::custom("value is missing")),
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = Error;

            fn deserialize<V>(
                self,
                _visitor: V
            ) -> Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                let MissingFieldDeserializer(field) = self;
                Err(de::Error::missing_field(field))
            }

            fn deserialize_option<V>(
                self,
                visitor: V
            ) -> Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }

            forward_to_deserialize! {
                bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
                string unit seq seq_fixed_size bytes map unit_struct
                newtype_struct tuple_struct struct struct_field tuple enum
                ignored_any
            }
        }

        de::Deserialize::deserialize(MissingFieldDeserializer(field))
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
        unit option seq seq_fixed_size bytes map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

/// Shortcut function to encode a `T` into a JSON `Value`
///
/// ```rust
/// use serde_json::to_value;
/// let val = to_value("foo").unwrap();
/// assert_eq!(val.as_str(), Some("foo"))
/// ```
pub fn to_value<T>(value: T) -> Result<Value, Error>
    where T: ser::Serialize,
{
    value.serialize(Serializer)
}

/// Shortcut function to decode a JSON `Value` into a `T`
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: de::Deserialize,
{
    de::Deserialize::deserialize(value)
}

/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Result<Value, Error>;
}

impl<T: ?Sized> ToJson for T
    where T: ser::Serialize,
{
    fn to_json(&self) -> Result<Value, Error> {
        to_value(&self)
    }
}
