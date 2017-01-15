use serde::{ser, de};
use std::fmt::{self, Debug};
use value::Value;
use std::hash::Hash;
use std::borrow::Borrow;

#[cfg(not(feature = "preserve_order"))]
use std::collections::{BTreeMap, btree_map};

#[cfg(feature = "preserve_order")]
use linked_hash_map::{self, LinkedHashMap};

/// Represents a key/value type.
pub struct Map<K, V>(MapImpl<K, V>);

#[cfg(not(feature = "preserve_order"))]
type MapImpl<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapImpl<K, V> = LinkedHashMap<K, V>;

impl Map<String, Value> {
    /// Makes a new empty Map.
    pub fn new() -> Self {
        Map(MapImpl::new())
    }

    #[cfg(not(feature = "preserve_order"))]
    /// Makes a new empty Map with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let _ = capacity;
        Map(BTreeMap::new()) // does not support with_capacity
    }

    #[cfg(feature = "preserve_order")]
    /// Makes a new empty Map with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Map(LinkedHashMap::with_capacity(capacity))
    }

    /// Clears the map, removing all values.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Value>
        where String: Borrow<Q>,
              Q: Ord + Eq + Hash
    {
        self.0.get(key)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
        where String: Borrow<Q>,
              Q: Ord + Eq + Hash
    {
        self.0.contains_key(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Value>
        where String: Borrow<Q>,
              Q: Ord + Eq + Hash
    {
        self.0.get_mut(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    pub fn insert(&mut self, k: String, v: Value) -> Option<Value> {
        self.0.insert(k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Value>
        where String: Borrow<Q>,
              Q: Ord + Eq + Hash
    {
        self.0.remove(key)
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    pub fn iter(&self) -> MapIter {
        MapIter(self.0.iter())
    }

    /// Gets a mutable iterator over the entries of the map.
    pub fn iter_mut(&mut self) -> MapIterMut {
        MapIterMut(self.0.iter_mut())
    }

    /// Gets an iterator over the keys of the map.
    pub fn keys(&self) -> MapKeys {
        MapKeys(self.0.keys())
    }

    /// Gets an iterator over the values of the map.
    pub fn values(&self) -> MapValues {
        MapValues(self.0.values())
    }
}

impl Default for Map<String, Value> {
    fn default() -> Self {
        Map(MapImpl::new())
    }
}

impl Clone for Map<String, Value> {
    fn clone(&self) -> Self {
        Map(self.0.clone())
    }
}

impl PartialEq for Map<String, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Debug for Map<String, Value> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl ser::Serialize for Map<String, Value> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        use serde::ser::SerializeMap;
        let mut map = try!(serializer.serialize_map(Some(self.len())));
        for (k, v) in self {
            try!(map.serialize_key(k));
            try!(map.serialize_value(v));
        }
        map.end()
    }
}

impl de::Deserialize for Map<String, Value> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer
    {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = Map<String, Value>;

            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where E: de::Error
            {
                Ok(Map::new())
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: de::MapVisitor
            {
                let mut values = Map::with_capacity(visitor.size_hint().0);

                while let Some((key, value)) = try!(visitor.visit()) {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

macro_rules! delegate_iterator {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* Iterator for $name $($generics)* {
            type Item = $item;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }
        }

        impl $($generics)* DoubleEndedIterator for $name $($generics)* {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        impl $($generics)* ExactSizeIterator for $name $($generics)* {
            fn len(&self) -> usize {
                self.0.len()
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> IntoIterator for &'a Map<String, Value> {
    type Item = (&'a String, &'a Value);
    type IntoIter = MapIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        MapIter(self.0.iter())
    }
}

pub struct MapIter<'a>(MapIterImpl<'a>);

#[cfg(not(feature = "preserve_order"))]
type MapIterImpl<'a> = btree_map::Iter<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type MapIterImpl<'a> = linked_hash_map::Iter<'a, String, Value>;

delegate_iterator!((MapIter<'a>) => (&'a String, &'a Value));

//////////////////////////////////////////////////////////////////////////////

impl<'a> IntoIterator for &'a mut Map<String, Value> {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = MapIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        MapIterMut(self.0.iter_mut())
    }
}

pub struct MapIterMut<'a>(MapIterMutImpl<'a>);

#[cfg(not(feature = "preserve_order"))]
type MapIterMutImpl<'a> = btree_map::IterMut<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type MapIterMutImpl<'a> = linked_hash_map::IterMut<'a, String, Value>;

delegate_iterator!((MapIterMut<'a>) => (&'a String, &'a mut Value));

//////////////////////////////////////////////////////////////////////////////

impl IntoIterator for Map<String, Value> {
    type Item = (String, Value);
    type IntoIter = MapIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        MapIntoIter(self.0.into_iter())
    }
}

pub struct MapIntoIter(MapIntoIterImpl);

#[cfg(not(feature = "preserve_order"))]
type MapIntoIterImpl = btree_map::IntoIter<String, Value>;
#[cfg(feature = "preserve_order")]
type MapIntoIterImpl = linked_hash_map::IntoIter<String, Value>;

delegate_iterator!((MapIntoIter) => (String, Value));

//////////////////////////////////////////////////////////////////////////////

pub struct MapKeys<'a>(MapKeysImpl<'a>);

#[cfg(not(feature = "preserve_order"))]
type MapKeysImpl<'a> = btree_map::Keys<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type MapKeysImpl<'a> = linked_hash_map::Keys<'a, String, Value>;

delegate_iterator!((MapKeys<'a>) => &'a String);

//////////////////////////////////////////////////////////////////////////////

pub struct MapValues<'a>(MapValuesImpl<'a>);

#[cfg(not(feature = "preserve_order"))]
type MapValuesImpl<'a> = btree_map::Values<'a, String, Value>;
#[cfg(feature = "preserve_order")]
type MapValuesImpl<'a> = linked_hash_map::Values<'a, String, Value>;

delegate_iterator!((MapValues<'a>) => &'a Value);
