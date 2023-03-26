//! An alternative and may convenient way to operate on untyped json is provided
//! through operator overloading based on json pointer. 
//! The core start point is path operator `/` to create json pointer struct,
//! which is no more than `Option<&Value>` that point to a node in json tree.
//!
//! ```rust
//! use serde_json::json;
//! use serde_json::PathOperator;
//!
//! let v = json!({"root": true, "usr": {
//!     "include": ["a.h", "b.h"],
//!     "lib": ["a.so", "b.so", {"name": "c.so", "version": "0.0.1"}],
//!     "lib64": null,
//!     "local": {"include": [], "lib": null}}
//! });
//!
//! let p1 = v.path() / "usr" / "lib" / 2 / "name";
//! let p2 = v.path() / "usr/lib/2/name";
//! let p3 = v.path() / "/usr/lib/2/name";
//! let p4 = v.pointer("/usr/lib/2/name");
//! assert!(p1 == p2 && p2 == p3 && p3.unwrap() == p4.unwrap());
//! assert!(v.path().unwrap() == &v);
//!
//! let usr = v.path() / "usr";
//! let local = v.path() / "usr" / "local";
//! let lib = "lib64";
//! let p1 = usr / lib;
//! let p2 = local / lib;
//!
//! assert!(p1.is_some());
//! assert!(p1.unwrap().is_null());
//! assert!(p2.is_none());
//! assert!(v["usr"]["local"]["lib64 or any absent"].is_null());
//! ```
//!
//! The `path()` method call can also replace with reference to `Value`, as `&v`,
//! And there is the other struct for mutable json pointer as well.
//!
//! The difference between `/` operator and `pointer()` method:
//!
//! * operator `/` can be chained and manually split path token in compile time.
//! * each path token can be use variable and modify seperately.
//! * joined path can optional omit the leading `/` required by json syntax.
//! * split path no need to escpace special char when key contins `/` or `~`.
//! * easy to save middle node pointer as variable and reuse later.
//!
//! The difference between `/` operator and `[]` index:
//!
//! * a bit more consistent and compact.
//! * can distinguish json null node and non-exist node.
//! * mutable pointer won't auto insert key to json object as index does. 
//! * mutable pointer won't panic when beyond range of json array as index does. 
//!
//! The pointer struct (and reference to `Value`) can further use 
//! operator `|` to read the primitive value hold in node with default fallback, and
//! operator `<<` to overwrite leaf node or push new item to array or object node.
//!
//! ```rust
//! use serde_json::json;
//! use serde_json::PathOperator;
//!
//! let mut v = json!({"int":10, "float":3.14, "array":["pi", null, true]});
//!
//! let node = v.path() / "int";
//! let val = node | 0;
//! assert_eq!(val, 10);
//! assert_eq!(v.path() / "float" | 0.0, 3.14);
//!
//! let _ode = v.path_mut() / "float" << 31.4;
//! let node = v.path_mut() / "array" / 2 << "true";
//! assert!(node.is_string()); // changed node type
//!
//! let _ode = v.path_mut() << ("key", "val");
//! let _ode = v.path_mut() / "array" << ("val",) << ["more"] << [100];
//! let _ode = v.path_mut() / "int" << ();
//! assert!(v["int"].is_null());
//!
//! assert_eq!(v, json!({"int":null, "float":31.4, "key":"val", "array":["pi",null,"true","val","more",100]}));
//! ```
//!
//! The `path_mut()` method above can replace with `&mut v` to create mutable pointer.
//!
//! And more, using `|`, the `&mut v` can also chained pipe to some function
//! or custom closure that modify the json tree continuously, which is much
//! different from operation on leaf node with `get_or` meaning and so would
//! finalize the operator chains.
//! 

use crate::Value;
use crate::value::Index;
use crate::json;
use std::ops::{Div, BitOr, BitAnd, Shl, Deref, DerefMut};

/// Wrap `Option<&Value>` as pointer to json node for operator overload.
///
/// It can used as `Option` implicitly at most time, as overload `*` Deref trait,
/// where `None` means refer to non-exist node, and `'tr` lifetime refers to 
/// the overall json tree.
/// Most method is hidden behind at the operator overload interface, except those
/// `is_*` methods that check the data type of pointed json node.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct JsonPtr<'tr> {
    ptr: Option<&'tr Value>,
}

/// Mutable josn pointer wrapper of `Optione<&mut Value>` for operator overload.
///
/// It can used as `Option` implicitly at most time, as overload `*` Deref trait,
/// where `None` means refer to non-exist node, and `'tr` lifetime refers to 
/// the overall json tree.
/// Most method is hidden behind at the operator overload interface, except those
/// `is_*` methods that check the data type of pointed json node.
///
/// Note that mutable reference don't support copy, only use it when you really 
/// need to modify the pointed json node, otherwise use the immutable pointer.
#[derive(Eq, PartialEq, Debug)]
pub struct JsonPtrMut<'tr> {
    ptr: Option<&'tr mut Value>,
}

/// Provide json pointer to supported operator overload.
pub trait PathOperator {
    /// Construct immutable json pointer to some initial node.
    fn path<'tr>(&'tr self) -> JsonPtr<'tr>;

    /// Construct immutable json pointer and move it follwoing sub path.
    fn pathto<'tr>(&'tr self, p: &str) -> JsonPtr<'tr>;

    /// Construct mutable json pointer to some initial node.
    fn path_mut<'tr>(&'tr mut self) -> JsonPtrMut<'tr>;

    /// Construct mutable json pointer and move it follwoing sub path.
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> JsonPtrMut<'tr>;
}

/// Create json pointer directely from `json::Value`.
impl PathOperator for Value {
    /// Create a `JsonPtr` instance which point to self node.
    /// Similar to `pointer()` method with empty str arguemnt, except raw `Option`.
    fn path<'tr>(&'tr self) -> JsonPtr<'tr> {
        JsonPtr::new(Some(self))
    }

    /// Create a `JsonPtr` instance which point to some subpath under self node.
    /// Similar to `pointer()` method except leading `/` is optionally.
    fn pathto<'tr>(&'tr self, p: &str) -> JsonPtr<'tr> {
        self.path().pathto(p)
    }

    /// Create a `JsonPtrMut` instance which point to self node.
    /// Similar to `pointer_mut()` method with empty str arguemnt.
    fn path_mut<'tr>(&'tr mut self) -> JsonPtrMut<'tr> {
        JsonPtrMut::new(Some(self))
    }

    /// Create a `JsonPtrMut` instance which point to some subpath under self node.
    /// Similar to `pointer_mut()` method except leading `/` is optionally.
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> JsonPtrMut<'tr> {
        self.path_mut().pathto(p)
    }
}

/// The rust type for scalar json node, which can used after operator `|` to read,
/// or/and operator `<<` to write. Only support `i64` for integer, to make use literal
/// number more convenient.
trait JsonScalar {}
impl JsonScalar for String {}
impl JsonScalar for &str {}
impl JsonScalar for i64 {}
impl JsonScalar for f64 {}
impl JsonScalar for bool {}
impl JsonScalar for () {}

/// extend method to read Value.
trait JsonReader {
    fn get_type(&self) -> &'static str;
    fn get_str<'tr>(&'tr self, rhs: &'tr str) -> &'tr str;
    fn get_string(&self, rhs: String) -> String;
    fn get_i64(&self, rhs: i64) -> i64;
    fn get_f64(&self, rhs: f64) -> f64;
    fn get_bool(&self, rhs: bool) -> bool;
}

impl JsonReader for Value {
    /// return the node type in string representation.
    fn get_type(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// operator `| &str`
    fn get_str<'tr>(&'tr self, rhs: &'tr str) -> &'tr str {
        match self.as_str() {
            Some(val) => val,
            None => rhs,
        }
    }

    /// operator `| String`.
    fn get_string(&self, rhs: String) -> String {
        match self {
            Value::String(s) => s.to_string(),
            Value::Number(i) if rhs == "0" && i.is_i64() => i.to_string(),
            Value::Number(u) if rhs == "0" && u.is_u64() => u.to_string(),
            Value::Number(f) if rhs == "0.0" && f.is_f64() => f.to_string(),
            Value::Bool(tf) if rhs == "bool" => tf.to_string(),
            Value::Array(_) if rhs == "[]" => self.to_string(),
            Value::Object(_) if rhs == "{}" => self.to_string(),
            _ if rhs.is_empty() => self.to_string(),
            _ => rhs
        }
    }

    /// operator `| i64`.
    fn get_i64(&self, rhs: i64) -> i64 {
        match self {
            Value::Number(n) if n.is_i64() => n.as_i64().unwrap_or(rhs),
            Value::String(s) => s.parse().unwrap_or(rhs),
            Value::Bool(tf) => if *tf { 1 } else { 0 },
            _ => rhs
        }
    }

    /// operator `| f64`.
    fn get_f64(&self, rhs: f64) -> f64 {
        match self {
            Value::Number(n) if n.is_f64() => n.as_f64().unwrap_or(rhs),
            Value::String(s) => s.parse().unwrap_or(rhs),
            _ => rhs
        }
    }

    /// operator `| bool`.
    fn get_bool(&self, rhs: bool) -> bool {
        match self {
            Value::Bool(tf) => *tf,
            Value::Number(n) if n.is_i64() => n.as_i64().unwrap_or(0) != 0,
            Value::Number(n) if n.is_u64() => true,
            Value::String(s) => s.parse().unwrap_or(rhs),
            _ => rhs
        }
    }
}

/// extend method to read Value.
trait JsonWriter {
    // put scalar or push item to one onde using `<<`
    fn put_value<T>(&mut self, rhs: T) -> &mut Self where Value: From<T>, T: JsonScalar;
    fn push_object<K: ToString, T>(&mut self, key: K, val: T) -> &mut Self where Value: From<T>;
    fn push_array<T>(&mut self, val: T) -> &mut Self where Value: From<T>;

    // pipe the whole json tree to function with `|`
    fn pipe_fn<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut Self);
    fn merge_move(&mut self, other: &mut Self) -> &mut Self;
    fn merge_copy(&mut self, other: &Self) -> &mut Self;
    fn filter_shape(&mut self, other: &Self) -> &mut Self;
    fn remove_null(&mut self) -> &mut Self;
}

impl JsonWriter for Value {
    /// For operator `<<` with scalar string, integer, float, bool and unit.
    fn put_value<T>(&mut self, rhs: T) -> &mut Self where Value: From<T> , T: JsonScalar {
        *self = Value::from(rhs);
        self
    }

    /// For operator `<<` to jsob objecet.
    fn push_object<K: ToString, T>(&mut self, key: K, val: T) -> &mut Self where Value: From<T> {
        if !self.is_object() {
            *self = json!({});
        }
        if let Some(v) = self.as_object_mut() {
            v.insert(key.to_string(), Value::from(val));
        }
        self
    }

    /// For operator `<<` to jsob array.
    fn push_array<T>(&mut self, val: T) -> &mut Self where Value: From<T> {
        if !self.is_array() {
            *self = json!([]);
        }
        if let Some(v) = self.as_array_mut() {
            v.push(Value::from(val));
        }
        self
    }

    /// Forward `JsonPtrMut | FnOnce`
    fn pipe_fn<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut Self) {
        pipe::apply(self, f);
        self
    }

    /// Forward `JsonPtrMut | JsonPtrMut`
    fn merge_move(&mut self, other: &mut Self) -> &mut Self {
        pipe::merge_move(self, other);
        self
    }

    /// Forward `JsonPtrMut | JsonPtr`
    fn merge_copy(&mut self, other: &Self) -> &mut Self {
        pipe::merge_copy(self, other);
        self
    }

    /// Forward `JsonPtrMut & JsonPtr`
    fn filter_shape(&mut self, other: &Self) -> &mut Self {
        pipe::filter_shape(self, other);
        self
    }

    /// Forward `JsonPtrMut | ()`
    fn remove_null(&mut self) -> &mut Self {
        pipe::remove_null(self);
        self
    }
}

/// Proxy `is_*` methods of `Value` for json pointer.
macro_rules! type_checker {
    ($func_name:ident) => {
        /// Check pointer is valid and the node match the type;
        pub fn $func_name(&self) -> bool {
            self.is_some() && self.as_ref().unwrap().$func_name()
        }
    };
}

/// Proxy `get_*` methods of `Value` for json pointer.
macro_rules! scalar_getter {
    ($func_name:ident | $ret:ty) => {
        /// Forward the getter method to pointed node, or return `rhs` by default.
        fn $func_name(&self, rhs: $ret) -> $ret {
            match self.ptr {
                Some(v) => v.$func_name(rhs),
                None => rhs,
            }
        }
    };
}

impl<'tr> JsonPtr<'tr> {
    /// Trivial new constructor.
    /// Usually there is no need to create `JsonPtr` instance directly, but yield one
    /// from existed json `Value`, except `None`.
    pub fn new(ptr: Option<&'tr Value>) -> Self {
        Self { ptr }
    }

    /// Resolve to sub path, by single index or joined path.
    /// Used in operator `/`.
    fn path<B>(&self, p: B) -> Self where B: Index + Copy + ToString {
        if self.is_none() {
            return Self::new(None);
        }

        let v = self.unwrap();
        let target = v.get(p);
        if target.is_some() {
            Self::new(target)
        }
        else {
            self.pathto(&p.to_string())
        }
    }

    /// Resolve to sub path, by joined path, auto prefix '/' for json pointer syntax.
    fn pathto(&self, p: &str) -> Self {
        if self.is_none() {
            return Self::new(None);
        }

        let v = self.unwrap();
        if !p.is_empty() && p.chars().nth(0) == Some('/') {
            return Self::new(v.pointer(p));
        }

        let mut fixp = String::from("/");
        fixp.push_str(p);
        return Self::new(v.pointer(&fixp));
    }

    /// Get a str ref if the value type matches, or defalut `rhs`.
    /// Used in operator `| ""` or `| &str`.
    fn get_str(&self, rhs: &'tr str) -> &'tr str {
        match self.ptr {
            Some(v) => v.get_str(rhs),
            None => rhs,
        }
    }

    scalar_getter!(get_string | String);
    scalar_getter!(get_i64 | i64);
    scalar_getter!(get_f64 | f64);
    scalar_getter!(get_bool | bool);

    // Check if the pointer is valid and refer to node of specific type.
    type_checker!(is_string);
    type_checker!(is_i64);
    type_checker!(is_u64);
    type_checker!(is_f64);
    type_checker!(is_boolean);
    type_checker!(is_null);
    type_checker!(is_array);
    type_checker!(is_object);
}

impl<'tr> JsonPtrMut<'tr> {
    /// Trivial new constructor.
    /// Usually there is no need to create `JsonPtr` instance directly, but yield one
    /// from existed json `Value`, except `None`.
    pub fn new(ptr: Option<&'tr mut Value>) -> Self {
        Self { ptr }
    }

    /// Convert to immutable pointer, leave self None.
    pub fn immut(&mut self) -> JsonPtr<'tr> {
        if self.ptr.is_none() {
            return JsonPtr::new(None);
        }
        let v = self.ptr.take().unwrap();
        JsonPtr::new(Some(v))
    }

    /// Resolve to sub path, by single index or joined path.
    /// Used in operator `/`.
    fn path<B>(&mut self, p: B) -> Self where B: Index + Copy + ToString {
        if self.is_none() {
            return Self::new(None);
        }

        // use immutable get to check first, avoid mutable refer twice
        let v = self.take().unwrap();
        let target = v.get(p);
        if target.is_some() {
            Self::new(v.get_mut(p))
        }
        else {
            self.ptr = Some(v); // restore reference had took out to `v`
            self.pathto(&p.to_string())
        }
    }

    /// Resolve to sub path, by joined path, auto prefix '/' for json pointer syntax.
    fn pathto(&mut self, p: &str) -> Self {
        if self.is_none() {
            return Self::new(None);
        }

        let v = self.take().unwrap();
        if !p.is_empty() && p.chars().nth(0) == Some('/') {
            return Self::new(v.pointer_mut(p));
        }

        let mut fixp = String::from("/");
        fixp.push_str(p);
        return Self::new(v.pointer_mut(&fixp));
    }

    /// Put a value to json and return pointer to it, which may change the node type.
    /// Implement for `<< (val)` , usually in scarlar node.
    fn put_value<T>(&mut self, rhs: T) -> Self where Value: From<T>, T: JsonScalar {
        match self.take() {
            Some(v) => { v.put_value(rhs); Self::new(Some(v)) },
            None => Self::new(None)
        }
    }

    /// Push a pair to object node, would invalidate the pointer if type mismatch.
    /// Implment for `<< (key, val)`.
    fn push_object<K: ToString, T>(&mut self, key: K, val: T) -> Self where Value: From<T> {
        match self.take() {
            Some(v) => { v.push_object(key, val); Self::new(Some(v)) },
            None => Self::new(None)
        }
    }

    /// Push a item to array node, would invalidate the pointer if type mismatch.
    /// Implment for `<< (val, )` or  `<< [item]` .
    fn push_array<T>(&mut self, val: T) -> Self where Value: From<T> {
        match self.take() {
            Some(v) => { v.push_array(val); Self::new(Some(v)) },
            None => Self::new(None)
        }
    }

    // Check if the pointer is valid and refer to node of specific type.
    type_checker!(is_string);
    type_checker!(is_i64);
    type_checker!(is_u64);
    type_checker!(is_f64);
    type_checker!(is_boolean);
    type_checker!(is_null);
    type_checker!(is_array);
    type_checker!(is_object);

    /// Forward `JsonPtrMut | FnOnce`
    fn pipe_fn<F>(mut self, f: F) -> Self where F: FnOnce(&mut Value) {
        match self.take() {
            Some(v) => {
                v.pipe_fn(f);
                Self::new(Some(v))
            },
            None => Self::new(None)
        }
    }

    /// Forward `JsonPtrMut | JsonPtrMut`
    fn merge_move(mut self, mut other: Self) -> Self {
        match self.take() {
            Some(v) => {
                if !other.is_null() {
                    v.merge_move(other.take().unwrap());
                }
                Self::new(Some(v))
            },
            None => Self::new(None)
        }
    }

    /// Forward `JsonPtrMut | JsonPtrMut`
    fn merge_copy(mut self, other: JsonPtr) -> Self {
        match self.take() {
            Some(v) => {
                if !other.is_null() {
                    v.merge_copy(other.unwrap());
                }
                Self::new(Some(v))
            },
            None => Self::new(None)
        }
    }

    /// Forward `JsonPtrMut & JsonPtr`
    fn filter_shape(mut self, other: JsonPtr) -> Self {
        match self.take() {
            Some(v) => {
                v.filter_shape(other.unwrap());
                Self::new(Some(v))
            },
            None => Self::new(None)
        }
    }

    /// Forward `JsonPtrMut | ()`
    fn remove_null(mut self) -> Self {
        match self.take() {
            Some(v) => {
                v.remove_null();
                Self::new(Some(v))
            },
            None => Self::new(None)
        }
    }
}

/// Extend functionily for pipe operator `|`.
mod pipe;

// Split operator overload implemntation to seperate files but not sub mod.
include!("overload_ptr.rs");
include!("overload_val.rs");

