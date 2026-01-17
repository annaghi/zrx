// Copyright (c) 2024 Zensical <contributors@zensical.org>

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

// ----------------------------------------------------------------------------

//! Tracking decorator, adding change tracking to a store.

use ahash::{HashMap, HashSet};
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::{fmt, mem};

use crate::store::{
    Key, Store, StoreFromIterator, StoreIntoIterator, StoreIterable, StoreKeys,
    StoreMut, StoreRange, StoreValues,
};

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Tracking decorator, adding change tracking to a store.
///
/// This is a thin wrapper around [`Store`], which allows to track changes for
/// efficient synchronization of stores, e.g., for persisting changes to disk.
///
/// Only the keys of changed items are recorded, so subsequent insertions and
/// removals of the same key are only recorded once. We use change tracking to
/// efficiently synchronize stores, e.g., for persisting changes to disk, which
/// is why we're only interested in the latest state of a key. Changes are not
/// recorded chronologically, but always returned in random order, because of
/// the use of [`HashSet`] as a data structure for change management.
///
/// Note that it's a good idea to use [`Changed::default`], since it leverages
/// [`ahash`] as a [`BuildHasher`][], which is the fastest known hasher.
///
/// [`BuildHasher`]: std::hash::BuildHasher
///
/// # Examples
///
/// ```
/// use zrx_store::decorator::Changed;
/// use zrx_store::StoreMut;
///
/// // Create store and initial state
/// let mut store = Changed::default();
/// store.insert("a", 4);
/// store.insert("b", 2);
/// store.insert("c", 3);
/// store.insert("d", 1);
///
/// // Create iterator over the store
/// for (key, value) in store {
///     println!("{key}: {value}");
/// }
/// ```
pub struct Changed<K, V, S = HashMap<K, V>>
where
    K: Key,
    S: Store<K, V>,
{
    /// Underlying store.
    store: S,
    /// Keys of changed items.
    changes: HashSet<K>,
    /// Marker for types.
    marker: PhantomData<V>,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl<K, V, S> Changed<K, V, S>
where
    K: Key,
    S: Store<K, V>,
{
    /// Creates a tracking decorator over a store.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::<_, _, HashMap<_, _>>::new();
    /// store.insert("key", 42);
    /// ```
    #[must_use]
    pub fn new() -> Self
    where
        S: Default,
    {
        Self {
            store: S::default(),
            changes: HashSet::default(),
            marker: PhantomData,
        }
    }

    /// Returns a change iterator over the store.
    ///
    /// This method returns an iterator over all changed keys since the last
    /// call to this method. The iterator yields tuples of keys and optional
    /// references to the corresponding values in the store, so if a key was
    /// removed from the store, the value will be [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("a", 4);
    /// store.insert("b", 2);
    /// store.insert("c", 3);
    /// store.insert("d", 1);
    ///
    /// // Obtain changes from store
    /// for (key, opt) in store.changes() {
    ///     println!("{key}: {opt:?}");
    /// }
    /// ```
    pub fn changes(&mut self) -> impl Iterator<Item = (K, Option<&V>)> {
        let iter = mem::take(&mut self.changes).into_iter();
        iter.map(|key| {
            let opt = self.store.get(&key);
            (key, opt)
        })
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl<K, V, S> Store<K, V> for Changed<K, V, S>
where
    K: Key,
    S: Store<K, V>,
{
    /// Returns a reference to the value identified by the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Obtain reference to value
    /// let value = store.get(&"key");
    /// assert_eq!(value, Some(&42));
    /// ```
    #[inline]
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Key,
    {
        self.store.get(key)
    }

    /// Returns whether the store contains the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Ensure presence of key
    /// let check = store.contains_key(&"key");
    /// assert_eq!(check, true);
    /// ```
    #[inline]
    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Key,
    {
        self.store.contains_key(key)
    }

    /// Returns the number of items.
    #[inline]
    fn len(&self) -> usize {
        self.store.len()
    }
}

impl<K, V, S> StoreMut<K, V> for Changed<K, V, S>
where
    K: Key,
    S: StoreMut<K, V> + StoreKeys<K, V>,
{
    /// Updates the value identified by the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and insert value
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    /// ```
    #[inline]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.changes.insert(key.clone());
        self.store.insert(key, value)
    }

    /// Updates the value identified by the key if it changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store
    /// let mut store = Changed::default();
    ///
    /// // Insert value
    /// let check = store.insert_if_changed(&"key", &42);
    /// assert_eq!(check, true);
    ///
    /// // Ignore unchanged value
    /// let check = store.insert_if_changed(&"key", &42);
    /// assert_eq!(check, false);
    ///
    /// // Update value
    /// let check = store.insert_if_changed(&"key", &84);
    /// assert_eq!(check, true);
    /// ```
    #[inline]
    fn insert_if_changed(&mut self, key: &K, value: &V) -> bool
    where
        V: Clone + Eq,
    {
        if self.store.insert_if_changed(key, value) {
            self.changes.insert(key.clone());
            true
        } else {
            false
        }
    }

    /// Removes the value identified by the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Remove and return value
    /// let value = store.remove(&"key");
    /// assert_eq!(value, Some(42));
    /// ```
    #[inline]
    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Key,
    {
        self.store.remove_entry(key).map(|(key, value)| {
            self.changes.insert(key);
            value
        })
    }

    /// Removes the value identified by the key and returns both.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Remove and return entry
    /// let entry = store.remove_entry(&"key");
    /// assert_eq!(entry, Some(("key", 42)));
    /// ```
    #[inline]
    fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Key,
    {
        self.store.remove_entry(key).inspect(|(key, _)| {
            self.changes.insert(key.clone());
        })
    }

    /// Clears the store, removing all items.
    ///
    /// Unfortunately, this operation has O(n) compexity, as all keys need to
    /// be recorded as changed, which requires iterating over the entire store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Clear store
    /// store.clear();
    /// assert!(store.is_empty());
    /// ```
    fn clear(&mut self) {
        for key in self.store.keys() {
            if !self.changes.contains(key) {
                self.changes.insert(key.clone());
            }
        }
        self.store.clear();
    }
}

impl<K, V, S> StoreIterable<K, V> for Changed<K, V, S>
where
    K: Key,
    S: StoreIterable<K, V>,
{
    /// Returns an iterator over the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{StoreIterable, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for (key, value) in store.iter() {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a K, &'a V)>
    where
        K: 'a,
        V: 'a,
    {
        self.store.iter()
    }
}

impl<K, V, S> StoreKeys<K, V> for Changed<K, V, S>
where
    K: Key,
    S: StoreKeys<K, V>,
{
    /// Creates a key iterator over the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{StoreKeys, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for key in store.keys() {
    ///     println!("{key}");
    /// }
    /// ```
    #[inline]
    fn keys<'a>(&'a self) -> impl Iterator<Item = &'a K>
    where
        K: 'a,
    {
        self.store.keys()
    }
}

impl<K, V, S> StoreValues<K, V> for Changed<K, V, S>
where
    K: Key,
    S: StoreValues<K, V>,
{
    /// Creates a value iterator over the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{StoreMut, StoreValues};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for value in store.values() {
    ///     println!("{value}");
    /// }
    /// ```
    #[inline]
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        self.store.values()
    }
}

impl<K, V, S> StoreRange<K, V> for Changed<K, V, S>
where
    K: Key,
    S: StoreRange<K, V>,
{
    /// Creates a range iterator over the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::{StoreMut, StoreRange};
    ///
    /// // Create store and initial state
    /// let mut store = Changed::<_, _, BTreeMap<_, _>>::new();
    /// store.insert("a", 42);
    /// store.insert("b", 84);
    ///
    /// // Create iterator over the store
    /// for (key, value) in store.range("b"..) {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn range<'a, R>(&'a self, range: R) -> impl Iterator<Item = (&'a K, &'a V)>
    where
        R: RangeBounds<K>,
        K: 'a,
        V: 'a,
    {
        self.store.range(range)
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S> FromIterator<(K, V)> for Changed<K, V, S>
where
    K: Key,
    S: StoreMut<K, V> + StoreFromIterator<K, V>,
{
    /// Creates a store from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create a vector of key-value pairs
    /// let items = vec![
    ///     ("a", 4),
    ///     ("b", 2),
    ///     ("c", 3),
    ///     ("d", 1),
    /// ];
    ///
    /// // Create store from iterator
    /// let store: Changed<_, _, HashMap<_, _>> =
    ///     items.into_iter().collect();
    ///
    /// // Create iterator over the store
    /// for (key, value) in store {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        Self {
            store: S::from_iter(iter),
            changes: HashSet::default(),
            marker: PhantomData,
        }
    }
}

impl<K, V, S> IntoIterator for Changed<K, V, S>
where
    K: Key,
    S: Store<K, V> + StoreIntoIterator<K, V>,
{
    type Item = (K, V);
    type IntoIter = S::IntoIter;

    /// Creates an iterator over the store.
    ///
    /// This method consumes the store, and collects it into a vector, since
    /// there's currently no way to implement this due to the absence of ATPIT
    /// (associated type position impl trait) support in stable Rust. When the
    /// feature is stabilized, we can switch to a more efficient approach.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for (key, value) in store {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.store.into_iter()
    }
}

// ----------------------------------------------------------------------------

#[allow(clippy::implicit_hasher)]
impl<K, V> Default for Changed<K, V, HashMap<K, V>>
where
    K: Key,
{
    /// Creates a tracking decorator with [`HashMap::default`] as a store.
    ///
    /// Note that this method does not allow to customize the [`BuildHasher`][],
    /// but uses [`ahash`] by default, which is the fastest known hasher.
    ///
    /// [`BuildHasher`]: std::hash::BuildHasher
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Changed;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Changed::default();
    /// store.insert("key", 42);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S> fmt::Debug for Changed<K, V, S>
where
    K: fmt::Debug + Key,
    S: fmt::Debug + Store<K, V>,
{
    /// Formats the tracking decorator for debugging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Changed")
            .field("store", &self.store)
            .field("changes", &self.changes)
            .finish()
    }
}
