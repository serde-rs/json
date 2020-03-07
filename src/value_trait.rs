use crate::{Map, Number, Value as JsonValue};
use std::borrow::Borrow;
use std::hash::Hash;
use value_trait::*;

impl Object for Map<String, JsonValue> {
    type Key = String;
    type Element = JsonValue;
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Map::get(self, k)
    }
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Map::get_mut(self, k)
    }
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        Map::insert(self, k.into(), v.into())
    }
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq + Ord,
    {
        Map::remove(self, k)
    }
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(Map::iter(self))
    }
}

impl PartialEq<i128> for JsonValue {
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().map(|v| v == *other).unwrap_or_default()
    }
}

impl PartialEq<u128> for JsonValue {
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().map(|v| v == *other).unwrap_or_default()
    }
}

impl PartialEq<()> for JsonValue {
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl Value for JsonValue {
    type Key = String;
    type Array = Vec<JsonValue>;
    type Object = Map<<Self as Value>::Key, Self>;
    fn value_type(&self) -> ValueType {
        unimplemented!()
    }
    fn is_null(&self) -> bool {
        self == &JsonValue::Null
    }
    fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }
    fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }
    fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(&s),
            _ => None,
        }
    }
    fn as_array(&self) -> Option<&<Self as Value>::Array> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }
    fn as_object(&self) -> Option<&<Self as Value>::Object> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl Mutable for JsonValue {
    fn as_array_mut(&mut self) -> Option<&mut <Self as Value>::Array> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }
    fn as_object_mut(&mut self) -> Option<&mut <Self as Value>::Object> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl From<StaticNode> for JsonValue {
    fn from(n: StaticNode) -> Self {
        match n {
            StaticNode::Bool(b) => JsonValue::Bool(b),
            StaticNode::F64(f) => {
                if let Some(f) = Number::from_f64(f) {
                    JsonValue::Number(f)
                } else {
                    JsonValue::Null
                }
            }
            StaticNode::Null => JsonValue::Null,
            StaticNode::I64(i) => JsonValue::Number(Number::from(i)),
            StaticNode::U64(u) => JsonValue::Number(Number::from(u)),
        }
    }
}
impl<'input> Builder<'input> for JsonValue {
    fn array_with_capacity(capacity: usize) -> Self {
        JsonValue::Array(Vec::with_capacity(capacity))
    }
    fn object_with_capacity(capacity: usize) -> Self {
        JsonValue::Object(Map::with_capacity(capacity))
    }
    fn null() -> Self {
        JsonValue::Null
    }
}
