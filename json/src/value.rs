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
//! use serde_json::Value;
//! use std::collections::BTreeMap;
//!
//! fn main() {
//!     let mut map = BTreeMap::new();
//!     map.insert(String::from("x"), Value::F64(1.0));
//!     map.insert(String::from("y"), Value::F64(2.0));
//!     let value = Value::Object(map);
//!
//!     let map: BTreeMap<String, f64> = serde_json::from_value(value).unwrap();
//! }
//! ```

use std::collections::{BTreeMap, btree_map};
use std::fmt;
use std::io;
use std::str;
use std::vec;

use num::NumCast;

use serde::de;
use serde::ser;

use error::Error;

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
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// If the `Value` is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &str) -> Option<&'a Value>{
        match self {
            &Value::Object(ref map) => map.get(key),
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

    /// If the `Value` is an Object, performs a depth-first search until
    /// a value associated with the provided key is found. If no value is found
    /// or the `Value` is not an Object, returns None.
    pub fn search<'a>(&'a self, key: &str) -> Option<&'a Value> {
        match self {
            &Value::Object(ref map) => {
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
    pub fn is_object<'a>(&'a self) -> bool {
        self.as_object().is_some()
    }

    /// If the `Value` is an Object, returns the associated BTreeMap.
    /// Returns None otherwise.
    pub fn as_object<'a>(&'a self) -> Option<&'a BTreeMap<String, Value>> {
        match self {
            &Value::Object(ref map) => Some(map),
            _ => None
        }
    }

    /// If the `Value` is an Object, returns the associated mutable BTreeMap.
    /// Returns None otherwise.
    pub fn as_object_mut<'a>(&'a mut self) -> Option<&'a mut BTreeMap<String, Value>> {
        match self {
            &mut Value::Object(ref mut map) => Some(map),
            _ => None
        }
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    pub fn is_array<'a>(&'a self) -> bool {
        self.as_array().is_some()
    }

    /// If the `Value` is an Array, returns the associated vector.
    /// Returns None otherwise.
    pub fn as_array<'a>(&'a self) -> Option<&'a Vec<Value>> {
        match self {
            &Value::Array(ref array) => Some(&*array),
            _ => None
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut<'a>(&'a mut self) -> Option<&'a mut Vec<Value>> {
        match self {
            &mut Value::Array(ref mut list) => Some(list),
            _ => None
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    pub fn is_string<'a>(&'a self) -> bool {
        self.as_string().is_some()
    }

    /// If the `Value` is a String, returns the associated str.
    /// Returns None otherwise.
    pub fn as_string<'a>(&'a self) -> Option<&'a str> {
        match *self {
            Value::String(ref s) => Some(&s),
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
        self.as_boolean().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            &Value::Bool(b) => Some(b),
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
        match self {
            &Value::Null => Some(()),
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
            Value::String(ref v) => serializer.serialize_str(&v),
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
                let values = try!(de::impls::BTreeMapVisitor::new().visit_map(visitor));
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
        match self.inner.write_str(str::from_utf8(buf).unwrap()) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err(io::Error::last_os_error()),
        }
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

#[derive(Debug)]
enum State {
    Value(Value),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
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
        match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        }
    }
}

impl ser::Serializer for Serializer {
    type Error = Error;

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
        let mut values = BTreeMap::new();
        values.insert(String::from(variant), Value::Array(vec![]));

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        let mut values = BTreeMap::new();
        values.insert(String::from(variant), to_value(&value));

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor,
    {
        let len = visitor.len().unwrap_or(0);
        let values = Vec::with_capacity(len);

        self.state.push(State::Array(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        let values = match self.state.pop().unwrap() {
            State::Array(values) => values,
            state => panic!("Expected array, found {:?}", state),
        };

        self.state.push(State::Value(Value::Array(values)));

        Ok(())
    }

    #[inline]
    fn serialize_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<(), Error>
        where V: ser::SeqVisitor,
    {
        try!(self.serialize_seq(visitor));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        let mut object = BTreeMap::new();

        object.insert(String::from(variant), value);

        self.state.push(State::Value(Value::Object(object)));

        Ok(())
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        try!(value.serialize(self));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().unwrap() {
            State::Array(ref mut values) => { values.push(value); }
            ref state => panic!("expected array, found {:?}", state),
        }

        Ok(())
    }

    #[inline]
    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor,
    {
        let values = BTreeMap::new();

        self.state.push(State::Object(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        let values = match self.state.pop().unwrap() {
            State::Object(values) => values,
            state => panic!("expected object, found {:?}", state),
        };

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<(), Error>
        where V: ser::MapVisitor,
    {
        try!(self.serialize_map(visitor));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        let mut object = BTreeMap::new();

        object.insert(String::from(variant), value);

        self.state.push(State::Value(Value::Object(object)));

        Ok(())
    }

    #[inline]
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: ser::Serialize,
              V: ser::Serialize,
    {
        try!(key.serialize(self));

        let key = match self.state.pop().unwrap() {
            State::Value(Value::String(value)) => value,
            state => panic!("expected key, found {:?}", state),
        };

        try!(value.serialize(self));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().unwrap() {
            State::Object(ref mut values) => { values.insert(key, value); }
            ref state => panic!("expected object, found {:?}", state),
        }

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
        let value = match self.value.take() {
            Some(Value::Object(value)) => value,
            Some(_) => { return Err(de::Error::syntax("expected an enum")); }
            None => { return Err(de::Error::end_of_stream()); }
        };

        let mut iter = value.into_iter();

        let (variant, value) = match iter.next() {
            Some(v) => v,
            None => return Err(de::Error::syntax("expected a variant name")),
        };

        // enums are encoded in json as maps with a single key:value pair
        match iter.next() {
            Some(_) => Err(de::Error::syntax("expected map")),
            None => visitor.visit(VariantDeserializer {
                de: self,
                val: Some(value),
                variant: Some(Value::String(variant)),
            }),
        }
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
        de::Deserialize::deserialize(&mut Deserializer::new(self.variant.take().unwrap()))
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        de::Deserialize::deserialize(&mut Deserializer::new(self.val.take().unwrap()))
    }

    fn visit_newtype<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize,
    {
        de::Deserialize::deserialize(&mut Deserializer::new(self.val.take().unwrap()))
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        if let Value::Array(fields) = self.val.take().unwrap() {
            de::Deserializer::deserialize(
                &mut SeqDeserializer {
                    de: self.de,
                    len: fields.len(),
                    iter: fields.into_iter(),
                },
                visitor,
            )
        } else {
            Err(de::Error::syntax("expected a tuple"))
        }
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static[&'static str],
                       visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        if let Value::Object(fields) = self.val.take().unwrap() {
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
            Err(de::Error::syntax("expected a struct"))
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
            Err(de::Error::length_mismatch(self.len))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

struct MapDeserializer<'a> {
    de: &'a mut Deserializer,
    iter: btree_map::IntoIter<String, Value>,
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
        let value = self.value.take().unwrap();
        self.de.value = Some(value);
        Ok(try!(de::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(de::Error::length_mismatch(self.len))
        }
    }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        // See if the type can deserialize from a unit.
        struct UnitDeserializer;

        impl de::Deserializer for UnitDeserializer {
            type Error = Error;

            fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_unit()
            }

            fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        Ok(try!(de::Deserialize::deserialize(&mut UnitDeserializer)))
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
/// assert_eq!(val.as_string(), Some("foo"))
/// ```
pub fn to_value<T: ?Sized>(value: &T) -> Value
    where T: ser::Serialize
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser).ok().unwrap();
    ser.unwrap()
}

/// Shortcut function to decode a JSON `Value` into a `T`
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: de::Deserialize
{
    let mut de = Deserializer::new(value);
    de::Deserialize::deserialize(&mut de)
}
