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
use std::collections::{btree_map, BTreeMap};

#[cfg(feature = "preserve_order")]
use linked_hash_map::{self, LinkedHashMap};

use std::fmt;
use std::io;
use std::str;
use std::vec;

use num_traits::NumCast;

use serde::de;
use serde::ser;

use error::Error;

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

#[cfg(not(feature = "preserve_order"))]
type MapVisitor<K, T> = de::impls::BTreeMapVisitor<K, T>;
#[cfg(feature = "preserve_order")]
type MapVisitor<K, T> = linked_hash_map::serde::LinkedHashMapVisitor<K, T>;

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

impl Value {
    /// If the `Value` is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &str) -> Option<&'a Value>{
        match *self {
            Value::Object(ref map) => map.get(key),
            _ => None
        }
    }

    /// Attempts to get a nested Value Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the `Value` associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&str]) -> Option<&'a Value>{
        let mut target = self;
        for key in keys {
            match target.find(key) {
                Some(t) => { target = t; },
                None => return None
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
                Some(t) => { target = t; },
                None => return None
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
        fn parse_index(s: &str) -> Option<usize> {
            if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
                return None
            }
            s.parse().ok()
        }
        if pointer == "" {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let mut target = self;
        for escaped_token in pointer.split('/').skip(1) {
            let token = escaped_token.replace("~1", "/").replace("~0", "~");
            let target_opt = match *target {
                Value::Object(ref map) => map.get(&token[..]),
                Value::Array(ref list) => parse_index(&token[..])
                    .and_then(|x| list.get(x)),
                _ => return None,
            };
            if let Some(t) = target_opt {
                target = t;
            } else { return None }
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
                                _ => ()
                            }
                        }
                        None
                    }
                }
            },
            _ => None
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
            _ => None
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match *self {
            Value::Object(ref mut map) => Some(map),
            _ => None
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
            _ => None
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self {
            Value::Array(ref mut list) => Some(list),
            _ => None
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
            _ => None
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
            _ => None
        }
    }

    /// If the `Value` is a number, return or cast it to a u64.
    /// Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::I64(n) => NumCast::from(n),
            Value::U64(n) => Some(n),
            _ => None
        }
    }

    /// If the `Value` is a number, return or cast it to a f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::I64(n) => NumCast::from(n),
            Value::U64(n) => NumCast::from(n),
            Value::F64(n) => Some(n),
            _ => None
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
            _ => None
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
            _ => None
        }
    }
}

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(v) => serializer.serialize_bool(v),
            Value::I64(v) => serializer.serialize_i64(v),
            Value::U64(v) => serializer.serialize_u64(v),
            Value::F64(v) => serializer.serialize_f64(v),
            Value::String(ref v) => serializer.serialize_str(v),
            Value::Array(ref v) => v.serialize(serializer),
            Value::Object(ref v) => v.serialize(serializer),
        }
    }
}

impl de::Deserialize for Value {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<Value, D::Error>
        where D: de::Deserializer,
    {
        struct ValueVisitor;

        impl de::Visitor for ValueVisitor {
            type Value = Value;

            #[inline]
            fn visit_bool<E>(&mut self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(&mut self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value::I64(value))
                } else {
                    Ok(Value::U64(value as u64))
                }
            }

            #[inline]
            fn visit_u64<E>(&mut self, value: u64) -> Result<Value, E> {
                Ok(Value::U64(value))
            }

            #[inline]
            fn visit_f64<E>(&mut self, value: f64) -> Result<Value, E> {
                Ok(Value::F64(value))
            }

            #[inline]
            fn visit_str<E>(&mut self, value: &str) -> Result<Value, E>
                where E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(&mut self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_none<E>(&mut self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Value, D::Error>
                where D: de::Deserializer,
            {
                de::Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: de::SeqVisitor,
            {
                let values = try!(de::impls::VecVisitor::new().visit_seq(visitor));
                Ok(Value::Array(values))
            }

            #[inline]
            fn visit_map<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: de::MapVisitor,
            {
                let values = try!(MapVisitor::new().visit_map(visitor));
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
        let mut wr = WriterFormatter { inner: f };
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
    }
}

impl fmt::Display for Value {
    /// Serializes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut wr = WriterFormatter { inner: f };
        super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
    }
}

impl str::FromStr for Value {
    type Err = Error;
    fn from_str(s: &str) -> Result<Value, Error> {
        super::de::from_str(s)
    }
}

#[derive(Debug)]
enum State {
    Value(Value),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}

/// Create a `serde::Serializer` that serializes a `Serialize`e into a `Value`.
pub struct Serializer {
    state: Vec<State>,
}

impl Serializer {
    /// Construct a new `Serializer`.
    pub fn new() -> Serializer {
        Serializer {
            state: Vec::with_capacity(4),
        }
    }

    /// Unwrap the `Serializer` and return the `Value`.
    pub fn unwrap(mut self) -> Value {
        match self.state.pop().expect("state is empty") {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        }
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Serializer::new()
    }
}

#[doc(hidden)]
pub enum MapSerializer {
    Map,
    Enum,
}

impl ser::MapSerializer for MapSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, K, V>(&mut self, serializer: &mut S, key: K, value: V) -> Result<(), Self::Error>
        where K: ser::Serialize,
              V: ser::Serialize,
              S: ser::Serializer<Error = Error> {
        serializer.serialize_map_elt(key, value)
    }

    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<(), Self::Error> where S: ser::Serializer<Error = Error> {
        match self {
            MapSerializer::Map => serializer.serialize_map_end(),
            MapSerializer::Enum => serializer.serialize_struct_variant_end(),
        }
    }
}

#[doc(hidden)]
pub enum SeqSerializer {
    Seq,
    Enum,
}

impl ser::SeqSerializer for SeqSerializer {
    type Error = Error;

    fn serialize_elt<S: ?Sized, T>(&mut self, serializer: &mut S, value: T) -> Result<(), Self::Error>
        where T: ser::Serialize, S: ser::Serializer<Error = Error> {
        serializer.serialize_seq_elt(value)
    }

    fn drop<S: ?Sized>(self, serializer: &mut S) -> Result<(), Self::Error> where S: ser::Serializer<Error = Error> {
        match self {
            SeqSerializer::Seq => serializer.serialize_seq_end(),
            SeqSerializer::Enum => serializer.serialize_tuple_variant_end(),
        }
    }
}

impl ser::Serializer for Serializer {
    type Error = Error;
    type SeqSerializer = SeqSerializer;
    type MapSerializer = MapSerializer;

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<(), Error> {
        self.state.push(State::Value(Value::Bool(value)));
        Ok(())
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<(), Error> {
        if value < 0 {
            self.state.push(State::Value(Value::I64(value)));
        } else {
            self.state.push(State::Value(Value::U64(value as u64)));
        }
        Ok(())
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<(), Error> {
        self.state.push(State::Value(Value::U64(value)));
        Ok(())
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<(), Error> {
        self.state.push(State::Value(Value::F64(value as f64)));
        Ok(())
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<(), Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<(), Error> {
        self.state.push(State::Value(Value::String(String::from(value))));
        Ok(())
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<(), Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> Result<(), Error>
        where V: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(&mut self) -> Result<(), Error> {
        self.state.push(State::Value(Value::Null));
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<(), Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(&mut self,
                                   _name: &'static str,
                                   value: T) -> Result<(), Self::Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        let mut values = Map::new();
        values.insert(String::from(variant), to_value(&value));

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_seq<'a>(&'a mut self, len: Option<usize>) -> Result<ser::SeqHelper<'a, Self>, Error> {
        let len = len.unwrap_or(0);
        let values = Vec::with_capacity(len);

        self.state.push(State::Array(values));

        Ok(ser::SeqHelper::new(self, SeqSerializer::Seq))
    }

    #[inline]
    fn serialize_tuple_variant<'a>(&'a mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              len: usize) -> Result<ser::SeqHelper<'a, Self>, Error>
    {
        self.state.push(State::Value(Value::String(String::from(variant))));
        try!(self.serialize_seq(Some(len)));
        Ok(ser::SeqHelper::new(self, SeqSerializer::Enum))
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        try!(value.serialize(self));

        let value = match self.state.pop().expect("state is empty") {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().expect("state is empty") {
            State::Array(ref mut values) => { values.push(value); }
            ref state => panic!("expected array, found {:?}", state),
        }

        Ok(())
    }

    fn serialize_seq_end(&mut self) -> Result<(), Error>
    {
        let values = match self.state.pop().expect("state is empty") {
            State::Array(values) => values,
            state => panic!("Expected array, found {:?}", state),
        };

        self.state.push(State::Value(Value::Array(values)));
        Ok(())
    }

    fn serialize_tuple_variant_end(&mut self) -> Result<(), Error>
    {
        try!(self.serialize_seq_end());
        let value = match self.state.pop().expect("state is empty") {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        let variant = match self.state.pop().expect("state is empty") {
            State::Value(Value::String(s)) => s,
            state => panic!("expected tuple variant name, found {:?}", state),
        };

        let mut object = Map::new();

        object.insert(variant, value);

        self.state.push(State::Value(Value::Object(object)));
        Ok(())
    }

    #[inline]
    fn serialize_map<'a>(&'a mut self, _len: Option<usize>) -> Result<ser::MapHelper<'a, Self>, Error> {
        self.state.push(State::Object(Map::new()));

        Ok(ser::MapHelper::new(self, MapSerializer::Map))
    }

    #[inline]
    fn serialize_struct_variant<'a>(&'a mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               _len: usize) -> Result<ser::MapHelper<'a, Self>, Error>
    {
        self.state.push(State::Value(Value::String(String::from(variant))));
        self.state.push(State::Object(Map::new()));

        Ok(ser::MapHelper::new(self, MapSerializer::Enum))
    }

    #[inline]
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(key.serialize(self));

        let key = match self.state.pop().expect("state is empty") {
            State::Value(Value::String(value)) => value,
            state => panic!("expected key, found {:?}", state),
        };

        try!(value.serialize(self));

        let value = match self.state.pop().expect("state is empty") {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().expect("state is empty") {
            State::Object(ref mut values) => { values.insert(key, value); }
            ref state => panic!("expected object, found {:?}", state),
        }

        Ok(())
    }

    fn serialize_map_end(&mut self) -> Result<(), Error>
    {
        let values = match self.state.pop().expect("state is empty") {
            State::Object(values) => values,
            state => panic!("expected object, found {:?}", state),
        };

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    fn serialize_struct_variant_end(&mut self) -> Result<(), Error>
    {
        try!(self.serialize_map_end());
        let value = match self.state.pop().expect("state is empty") {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };
        let variant = match self.state.pop().expect("state is empty, expected variant name") {
            State::Value(Value::String(s)) => s,
            other => panic!("expected string value, found {:?}", other),
        };

        let mut object = Map::new();

        object.insert(variant, value);

        self.state.push(State::Value(Value::Object(object)));

        Ok(())
    }
}

/// Creates a `serde::Deserializer` from a `json::Value` object.
pub struct Deserializer {
    value: Option<Value>,
}

impl Deserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(value: Value) -> Deserializer {
        Deserializer {
            value: Some(value),
        }
    }
}

impl de::Deserializer for Deserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let value = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream()); }
        };

        match value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::I64(v) => visitor.visit_i64(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::F64(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => {
                let len = v.len();
                visitor.visit_seq(SeqDeserializer {
                    de: self,
                    iter: v.into_iter(),
                    len: len,
                })
            }
            Value::Object(v) => {
                let len = v.len();
                visitor.visit_map(MapDeserializer {
                    de: self,
                    iter: v.into_iter(),
                    value: None,
                    len: len,
                })
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.value {
            Some(Value::Null) => visitor.visit_none(),
            Some(_) => visitor.visit_some(self),
            None => Err(de::Error::end_of_stream()),
        }
    }

    #[inline]
    fn deserialize_enum<V>(&mut self,
                     _name: &str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        let (variant, value) = match self.value.take() {
            Some(Value::Object(value)) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => { return Err(de::Error::invalid_type(de::Type::VariantName)); }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_type(de::Type::Map));
                }
                (variant, Some(value))
            }
            Some(Value::String(variant)) => {
                (variant, None)
            },
            Some(_) => { return Err(de::Error::invalid_type(de::Type::Enum)); }
            None => { return Err(de::Error::end_of_stream()); }
        };

        visitor.visit(VariantDeserializer {
            de: self,
            val: value,
            variant: Some(Value::String(variant)),
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(&mut self,
                               _name: &'static str,
                               mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }
}

struct VariantDeserializer<'a> {
    de: &'a mut Deserializer,
    val: Option<Value>,
    variant: Option<Value>,
}

impl<'a> de::VariantVisitor for VariantDeserializer<'a> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize,
    {
        let variant = self.variant.take().expect("variant is missing");
        de::Deserialize::deserialize(&mut Deserializer::new(variant))
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        match self.val.take() {
            Some(val) => de::Deserialize::deserialize(&mut Deserializer::new(val)),
            None => Ok(()),
        }
    }

    fn visit_newtype<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        let val = self.val.take().expect("val is missing");
        de::Deserialize::deserialize(&mut Deserializer::new(val))
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let val = self.val.take().expect("val is missing");
        if let Value::Array(fields) = val {
            de::Deserializer::deserialize(
                &mut SeqDeserializer {
                    de: self.de,
                    len: fields.len(),
                    iter: fields.into_iter(),
                },
                visitor,
            )
        } else {
            Err(de::Error::invalid_type(de::Type::Tuple))
        }
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static[&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        let val = self.val.take().expect("val is missing");
        if let Value::Object(fields) = val {
            de::Deserializer::deserialize(
                &mut MapDeserializer {
                    de: self.de,
                    len: fields.len(),
                    iter: fields.into_iter(),
                    value: None,
                },
                visitor,
            )
        } else {
            Err(de::Error::invalid_type(de::Type::Struct))
        }
    }
}

struct SeqDeserializer<'a> {
    de: &'a mut Deserializer,
    iter: vec::IntoIter<Value>,
    len: usize,
}

impl<'a> de::Deserializer for SeqDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        if self.len == 0 {
            visitor.visit_unit()
        } else {
            visitor.visit_seq(self)
        }
    }
}

impl<'a> de::SeqVisitor for SeqDeserializer<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        match self.iter.next() {
            Some(value) => {
                self.len -= 1;
                self.de.value = Some(value);
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            None => Ok(None),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(de::Error::invalid_length(self.len))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

struct MapDeserializer<'a> {
    de: &'a mut Deserializer,
    iter: MapIntoIter<String, Value>,
    value: Option<Value>,
    len: usize,
}

impl<'a> de::MapVisitor for MapDeserializer<'a> {
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.len -= 1;
                self.value = Some(value);
                self.de.value = Some(Value::String(key));
                Ok(Some(try!(de::Deserialize::deserialize(self.de))))
            }
            None => Ok(None),
        }
    }

    fn visit_value<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize
    {
        let value = self.value.take().expect("value is missing");
        self.de.value = Some(value);
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(de::Error::invalid_length(self.len))
        }
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = de::value::Error;

            fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                let &mut MissingFieldDeserializer(field) = self;
                Err(de::value::Error::MissingField(field))
            }

            fn deserialize_option<V>(&mut self,
                                     mut visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        let mut de = MissingFieldDeserializer(field);
        Ok(try!(de::Deserialize::deserialize(&mut de)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> de::Deserializer for MapDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self)
    }
}

/// Shortcut function to encode a `T` into a JSON `Value`
///
/// ```rust
/// use serde_json::to_value;
/// let val = to_value("foo");
/// assert_eq!(val.as_str(), Some("foo"))
/// ```
pub fn to_value<T: ?Sized>(value: &T) -> Value
    where T: ser::Serialize
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser).expect("failed to serialize");
    ser.unwrap()
}

/// Shortcut function to decode a JSON `Value` into a `T`
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: de::Deserialize
{
    let mut de = Deserializer::new(value);
    de::Deserialize::deserialize(&mut de)
}

/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Value;
}

impl<T: ?Sized> ToJson for T where T: ser::Serialize {
    fn to_json(&self) -> Value {
        to_value(&self)
    }
}
