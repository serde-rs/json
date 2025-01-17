//! A map of String to serde_json::Value.
//!
//! By default the map is backed by a [`BTreeMap`]. Enable the `preserve_order`
//! feature of serde_json to use [`IndexMap`] instead.
//!
//! [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
//! [`IndexMap`]: https://docs.rs/indexmap/*/indexmap/map/struct.IndexMap.html

use crate::error::Error;
use crate::value::Value;
use alloc::string::String;
#[cfg(feature = "preserve_order")]
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
use core::marker::PhantomData;
#[cfg(feature = "preserve_order")]
use core::mem;
use core::ops;
use serde::de;

#[cfg(not(feature = "preserve_order"))]
use alloc::collections::{btree_map, BTreeMap};
#[cfg(feature = "preserve_order")]
use indexmap::IndexMap;

/// Represents a JSON key/value type.
pub struct Map<K = String, V = Value> {
    map: MapImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type MapImpl<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapImpl<K, V> = IndexMap<K, V>;

impl<K, V> Map<K, V> {
    /// Makes a new empty Map.
    #[inline]
    pub fn new() -> Self {
        Map {
            map: MapImpl::new(),
        }
    }

    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Map {
            #[cfg(not(feature = "preserve_order"))]
            map: {
                // does not support with_capacity
                let _ = capacity;
                BTreeMap::new()
            },
            #[cfg(feature = "preserve_order")]
            map: IndexMap::with_capacity(capacity),
        }
    }

    /// Clears the map, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get(key)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.contains_key(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get_mut(key)
    }

    /// Returns the key-value pair matching the given key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.get_key_value(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Ord + Hash,
    {
        self.map.insert(k, v)
    }

    /// Insert a key-value pair in the map at the given index.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the key is moved to the new
    /// position, the value is updated, and the old value is returned.
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_insert(&mut self, index: usize, k: K, v: V) -> Option<V>
    where
        K: Eq + Hash,
    {
        self.map.shift_insert(index, k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// If serde_json's "preserve_order" is enabled, `.remove(key)` is
    /// equivalent to [`.swap_remove(key)`][Self::swap_remove], replacing this
    /// entry's position with the last element. If you need to preserve the
    /// relative order of the keys in the map, use
    /// [`.shift_remove(key)`][Self::shift_remove] instead.
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove(key);
        #[cfg(not(feature = "preserve_order"))]
        return self.map.remove(key);
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// If serde_json's "preserve_order" is enabled, `.remove_entry(key)` is
    /// equivalent to [`.swap_remove_entry(key)`][Self::swap_remove_entry],
    /// replacing this entry's position with the last element. If you need to
    /// preserve the relative order of the keys in the map, use
    /// [`.shift_remove_entry(key)`][Self::shift_remove_entry] instead.
    #[inline]
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord + Eq + Hash,
    {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove_entry(key);
        #[cfg(not(feature = "preserve_order"))]
        return self.map.remove_entry(key);
    }

    /// Removes and returns the value corresponding to the key from the map.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.swap_remove(key)
    }

    /// Remove and return the key-value pair.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.swap_remove_entry(key)
    }

    /// Removes and returns the value corresponding to the key from the map.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.shift_remove(key)
    }

    /// Remove and return the key-value pair.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        self.map.shift_remove_entry(key)
    }

    /// Moves all elements from other into self, leaving other empty.
    #[inline]
    pub fn append(&mut self, other: &mut Self)
    where
        K: Ord + Hash,
    {
        #[cfg(feature = "preserve_order")]
        self.map
            .extend(mem::replace(&mut other.map, MapImpl::default()));
        #[cfg(not(feature = "preserve_order"))]
        self.map.append(&mut other.map);
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    pub fn entry<S>(&mut self, key: S) -> Entry<K, V>
    where
        K: Ord + Hash,
        S: Into<K>,
    {
        #[cfg(not(feature = "preserve_order"))]
        use alloc::collections::btree_map::Entry as EntryImpl;
        #[cfg(feature = "preserve_order")]
        use indexmap::map::Entry as EntryImpl;

        match self.map.entry(key.into()) {
            EntryImpl::Vacant(vacant) => Entry::Vacant(VacantEntry { vacant }),
            EntryImpl::Occupied(occupied) => Entry::Occupied(OccupiedEntry { occupied }),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            iter: self.map.iter(),
        }
    }

    /// Gets a mutable iterator over the entries of the map.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    /// Gets an iterator over the keys of the map.
    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            iter: self.map.keys(),
        }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values {
            iter: self.map.values(),
        }
    }

    /// Gets an iterator over mutable values of the map.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            iter: self.map.values_mut(),
        }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.map.into_values(),
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)`
    /// returns `false`.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        K: Ord,
        F: FnMut(&K, &mut V) -> bool,
    {
        self.map.retain(f);
    }

    /// Sorts this map's entries in-place using `str`'s usual ordering.
    ///
    /// If serde_json's "preserve_order" feature is not enabled, this method
    /// does no work because all JSON maps are always kept in a sorted state.
    ///
    /// If serde_json's "preserve_order" feature is enabled, this method
    /// destroys the original source order or insertion order of this map in
    /// favor of an alphanumerical order that matches how a BTreeMap with the
    /// same contents would be ordered. This takes **O(n log n + c)** time where
    /// _n_ is the length of the map and _c_ is the capacity.
    ///
    /// Other maps nested within the values of this map are not sorted. If you
    /// need the entire data structure to be sorted at all levels, you must also
    /// call
    /// <code>map.[values_mut]\().for_each([Value::sort_all_objects])</code>.
    ///
    /// [values_mut]: Map::values_mut
    #[inline]
    pub fn sort_keys(&mut self)
    where
        K: Ord,
    {
        #[cfg(feature = "preserve_order")]
        self.map.sort_unstable_keys();
    }
}

#[allow(clippy::derivable_impls)] // clippy bug: https://github.com/rust-lang/rust-clippy/issues/7655
impl<K, V> Default for Map<K, V> {
    #[inline]
    fn default() -> Self {
        Map {
            map: MapImpl::new(),
        }
    }
}

impl<K, V> Clone for Map<K, V>
where
    K: Clone,
    V: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Map {
            map: self.map.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.map.clone_from(&source.map);
    }
}

impl<K, V> PartialEq for Map<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.map.eq(&other.map)
    }
}

impl<K, V> Eq for Map<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<K, V> Hash for Map<K, V>
where
    K: Ord + Hash,
    V: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(not(feature = "preserve_order"))]
        {
            self.map.hash(state);
        }

        #[cfg(feature = "preserve_order")]
        {
            let mut kv = Vec::from_iter(&self.map);
            kv.sort_unstable_by(|a, b| a.0.cmp(b.0));
            kv.hash(state);
        }
    }
}

/// Access an element of this map. Panics if the given key is not present in the
/// map.
///
/// ```
/// # use serde_json::Value;
/// #
/// # let val = &Value::String("".to_owned());
/// # let _ =
/// match val {
///     Value::String(s) => Some(s.as_str()),
///     Value::Array(arr) => arr[0].as_str(),
///     Value::Object(map) => map["type"].as_str(),
///     _ => None,
/// }
/// # ;
/// ```
impl<K, V, Q> ops::Index<&Q> for Map<K, V>
where
    K: Borrow<Q> + Ord,
    Q: ?Sized + Ord + Eq + Hash,
{
    type Output = V;

    fn index(&self, index: &Q) -> &V {
        self.map.index(index)
    }
}

/// Mutably access an element of this map. Panics if the given key is not
/// present in the map.
///
/// ```
/// # use serde_json::json;
/// #
/// # let mut map: serde_json::Map = serde_json::Map::new();
/// # map.insert("key".to_owned(), serde_json::Value::Null);
/// #
/// map["key"] = json!("value");
/// ```
impl<K, V, Q> ops::IndexMut<&Q> for Map<K, V>
where
    K: Borrow<Q> + Ord,
    Q: ?Sized + Ord + Eq + Hash,
{
    fn index_mut(&mut self, index: &Q) -> &mut V {
        self.map.get_mut(index).expect("no entry found for key")
    }
}

impl<K, V> Debug for Map<K, V>
where
    K: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.map.fmt(formatter)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<K, V> serde::ser::Serialize for Map<K, V>
where
    K: serde::ser::Serialize,
    V: serde::ser::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = tri!(serializer.serialize_map(Some(self.len())));
        for (k, v) in self {
            tri!(map.serialize_entry(k, v));
        }
        map.end()
    }
}

impl<'de, K, V> de::Deserialize<'de> for Map<K, V>
where
    K: de::Deserialize<'de> + Ord + Hash,
    V: de::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
        where
            K: de::Deserialize<'de> + Ord + Hash,
            V: de::Deserialize<'de>,
        {
            type Value = Map<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Map::new())
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut values = Map::new();

                while let Some((key, value)) = tri!(map.next_entry()) {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor(PhantomData))
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for Map<K, V>
where
    K: Ord + Hash,
{
    fn from(arr: [(K, V); N]) -> Self {
        Map {
            map: From::from(arr),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Ord + Hash,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        Map {
            map: FromIterator::from_iter(iter),
        }
    }
}

impl<K, V> Extend<(K, V)> for Map<K, V>
where
    K: Ord + Hash,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        self.map.extend(iter);
    }
}

macro_rules! delegate_iterator {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl $($generics)* DoubleEndedIterator for $name $($generics)* {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl $($generics)* ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl $($generics)* FusedIterator for $name $($generics)* {}
    }
}

impl<'de, K, V> de::IntoDeserializer<'de, Error> for Map<K, V>
where
    Self: de::Deserializer<'de, Error = Error>,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de, K, V> de::IntoDeserializer<'de, Error> for &'de Map<K, V>
where
    Self: de::Deserializer<'de, Error = Error>,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

//////////////////////////////////////////////////////////////////////////////

/// A view into a single entry in a map, which may either be vacant or occupied.
/// This enum is constructed from the [`entry`] method on [`Map`].
///
/// [`entry`]: struct.Map.html#method.entry
/// [`Map`]: struct.Map.html
pub enum Entry<'a, K, V> {
    /// A vacant Entry.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied Entry.
    Occupied(OccupiedEntry<'a, K, V>),
}

/// A vacant Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K, V> {
    vacant: VacantEntryImpl<'a, K, V>,
}

/// An occupied Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K, V> {
    occupied: OccupiedEntryImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type VacantEntryImpl<'a, K, V> = btree_map::VacantEntry<'a, K, V>;
#[cfg(feature = "preserve_order")]
type VacantEntryImpl<'a, K, V> = indexmap::map::VacantEntry<'a, K, V>;

#[cfg(not(feature = "preserve_order"))]
type OccupiedEntryImpl<'a, K, V> = btree_map::OccupiedEntry<'a, K, V>;
#[cfg(feature = "preserve_order")]
type OccupiedEntryImpl<'a, K, V> = indexmap::map::OccupiedEntry<'a, K, V>;

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// assert_eq!(map.entry("serde").key(), &"serde");
    /// ```
    pub fn key(&self) -> &K {
        match self {
            Entry::Vacant(e) => e.key(),
            Entry::Occupied(e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.entry("serde").or_insert(json!(12));
    ///
    /// assert_eq!(map["serde"], 12);
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.entry("serde").or_insert_with(|| json!("hoho"));
    ///
    /// assert_eq!(map["serde"], "hoho".to_owned());
    /// ```
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.entry("serde")
    ///     .and_modify(|e| *e = json!("rust"))
    ///     .or_insert(json!("cpp"));
    ///
    /// assert_eq!(map["serde"], "cpp");
    ///
    /// map.entry("serde")
    ///     .and_modify(|e| *e = json!("rust"))
    ///     .or_insert(json!("cpp"));
    ///
    /// assert_eq!(map["serde"], "rust");
    /// ```
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord,
{
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    ///
    /// match map.entry("serde") {
    ///     Entry::Vacant(vacant) => {
    ///         assert_eq!(vacant.key(), &"serde");
    ///     }
    ///     Entry::Occupied(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        self.vacant.key()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    ///
    /// match map.entry("serde") {
    ///     Entry::Vacant(vacant) => {
    ///         vacant.insert(json!("hoho"));
    ///     }
    ///     Entry::Occupied(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        self.vacant.insert(value)
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.key(), &"serde");
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.get(), 12);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn get(&self) -> &V {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!([1, 2, 3]));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         occupied.get_mut().as_array_mut().unwrap().push(json!(4));
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    ///
    /// assert_eq!(map["serde"].as_array().unwrap().len(), 4);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!([1, 2, 3]));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         occupied.into_mut().as_array_mut().unwrap().push(json!(4));
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    ///
    /// assert_eq!(map["serde"].as_array().unwrap().len(), 4);
    /// ```
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key, and returns
    /// the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(mut occupied) => {
    ///         assert_eq!(occupied.insert(json!(13)), 12);
    ///         assert_eq!(occupied.get(), 13);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        self.occupied.insert(value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// If serde_json's "preserve_order" is enabled, `.remove()` is
    /// equivalent to [`.swap_remove()`][Self::swap_remove], replacing this
    /// entry's position with the last element. If you need to preserve the
    /// relative order of the keys in the map, use
    /// [`.shift_remove()`][Self::shift_remove] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         assert_eq!(occupied.remove(), 12);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn remove(self) -> V {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove();
        #[cfg(not(feature = "preserve_order"))]
        return self.occupied.remove();
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove(self) -> V {
        self.occupied.swap_remove()
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove(self) -> V {
        self.occupied.shift_remove()
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// If serde_json's "preserve_order" is enabled, `.remove_entry()` is
    /// equivalent to [`.swap_remove_entry()`][Self::swap_remove_entry],
    /// replacing this entry's position with the last element. If you need to
    /// preserve the relative order of the keys in the map, use
    /// [`.shift_remove_entry()`][Self::shift_remove_entry] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// use serde_json::map::Entry;
    ///
    /// let mut map: serde_json::Map = serde_json::Map::new();
    /// map.insert("serde".to_owned(), json!(12));
    ///
    /// match map.entry("serde") {
    ///     Entry::Occupied(occupied) => {
    ///         let (key, value) = occupied.remove_entry();
    ///         assert_eq!(key, "serde");
    ///         assert_eq!(value, 12);
    ///     }
    ///     Entry::Vacant(_) => unimplemented!(),
    /// }
    /// ```
    #[inline]
    pub fn remove_entry(self) -> (K, V) {
        #[cfg(feature = "preserve_order")]
        return self.swap_remove_entry();
        #[cfg(not(feature = "preserve_order"))]
        return self.occupied.remove_entry();
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// Like [`Vec::swap_remove`], the entry is removed by swapping it with the
    /// last element of the map and popping it off. This perturbs the position
    /// of what used to be the last element!
    ///
    /// [`Vec::swap_remove`]: std::vec::Vec::swap_remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn swap_remove_entry(self) -> (K, V) {
        self.occupied.swap_remove_entry()
    }

    /// Removes the entry from the map, returning the stored key and value.
    ///
    /// Like [`Vec::remove`], the entry is removed by shifting all of the
    /// elements that follow it, preserving their relative order. This perturbs
    /// the index of all of those elements!
    ///
    /// [`Vec::remove`]: std::vec::Vec::remove
    #[cfg(feature = "preserve_order")]
    #[cfg_attr(docsrs, doc(cfg(feature = "preserve_order")))]
    #[inline]
    pub fn shift_remove_entry(self) -> (K, V) {
        self.occupied.shift_remove_entry()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.map.iter(),
        }
    }
}

/// An iterator over a serde_json::Map's entries.
pub struct Iter<'a, K, V> {
    iter: IterImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IterImpl<'a, K, V> = btree_map::Iter<'a, K, V>;
#[cfg(feature = "preserve_order")]
type IterImpl<'a, K, V> = indexmap::map::Iter<'a, K, V>;

delegate_iterator!((Iter<'a, K, V>) => (&'a K, &'a V));

//////////////////////////////////////////////////////////////////////////////

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

/// A mutable iterator over a serde_json::Map's entries.
pub struct IterMut<'a, K, V> {
    iter: IterMutImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IterMutImpl<'a, K, V> = btree_map::IterMut<'a, K, V>;
#[cfg(feature = "preserve_order")]
type IterMutImpl<'a, K, V> = indexmap::map::IterMut<'a, K, V>;

delegate_iterator!((IterMut<'a, K, V>) => (&'a K, &'a mut V));

//////////////////////////////////////////////////////////////////////////////

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

/// An owning iterator over a serde_json::Map's entries.
pub struct IntoIter<K, V> {
    iter: IntoIterImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IntoIterImpl<K, V> = btree_map::IntoIter<K, V>;
#[cfg(feature = "preserve_order")]
type IntoIterImpl<K, V> = indexmap::map::IntoIter<K, V>;

delegate_iterator!((IntoIter<K, V>) => (K, V));

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a serde_json::Map's keys.
pub struct Keys<'a, K, V> {
    iter: KeysImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type KeysImpl<'a, K, V> = btree_map::Keys<'a, K, V>;
#[cfg(feature = "preserve_order")]
type KeysImpl<'a, K, V> = indexmap::map::Keys<'a, K, V>;

delegate_iterator!((Keys<'a, K, V>) => &'a K);

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a serde_json::Map's values.
pub struct Values<'a, K, V> {
    iter: ValuesImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesImpl<'a, K, V> = btree_map::Values<'a, K, V>;
#[cfg(feature = "preserve_order")]
type ValuesImpl<'a, K, V> = indexmap::map::Values<'a, K, V>;

delegate_iterator!((Values<'a, K, V>) => &'a V);

//////////////////////////////////////////////////////////////////////////////

/// A mutable iterator over a serde_json::Map's values.
pub struct ValuesMut<'a, K, V> {
    iter: ValuesMutImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesMutImpl<'a, K, V> = btree_map::ValuesMut<'a, K, V>;
#[cfg(feature = "preserve_order")]
type ValuesMutImpl<'a, K, V> = indexmap::map::ValuesMut<'a, K, V>;

delegate_iterator!((ValuesMut<'a, K, V>) => &'a mut V);

//////////////////////////////////////////////////////////////////////////////

/// An owning iterator over a serde_json::Map's values.
pub struct IntoValues<K, V> {
    iter: IntoValuesImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IntoValuesImpl<K, V> = btree_map::IntoValues<K, V>;
#[cfg(feature = "preserve_order")]
type IntoValuesImpl<K, V> = indexmap::map::IntoValues<K, V>;

delegate_iterator!((IntoValues<K, V>) => V);
