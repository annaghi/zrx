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
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Index, RangeBounds};

use crate::store::comparator::Comparator;
use crate::store::key::Key;
use crate::store::{
    Store, StoreFromIterator, StoreIntoIterator, StoreIterable, StoreKeys,
    StoreMut, StoreRange, StoreValues, StoreWithComparator,
};

mod changes;

pub use changes::Changes;

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
/// Note that it's a good idea to use [`Tracked::default`][], since it leverages
/// [`ahash`] as a [`BuildHasher`][], which is the fastest known hasher.
///
/// [`BuildHasher`]: std::hash::BuildHasher
/// [`Tracked::default`]: Default::default
///
/// # Examples
///
/// ```
/// use zrx_store::decorator::Tracked;
/// use zrx_store::StoreMut;
///
/// // Create store and initial state
/// let mut store = Tracked::default();
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
#[derive(Clone, PartialEq, Eq)]
pub struct Tracked<K, V, S = HashMap<K, V>>
where
    K: Key,
    S: Store<K, V>,
{
    /// Underlying store.
    store: S,
    /// Keys of changed items.
    changed: HashSet<K>,
    /// Capture types.
    marker: PhantomData<V>,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl<K, V, S> Tracked<K, V, S>
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
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store
    /// let mut store = Tracked::<_, _, HashMap<_, _>>::new();
    ///
    /// // Insert value
    /// store.insert("key", 42);
    /// ```
    #[must_use]
    pub fn new() -> Self
    where
        S: Default,
    {
        Self {
            store: S::default(),
            changed: HashSet::default(),
            marker: PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl<K, V, S> Store<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: Store<K, V>,
{
    /// Returns a reference to the value identified by the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
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
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
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

impl<K, V, S> StoreMut<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: StoreMut<K, V>,
{
    /// Updates the value identified by the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store
    /// let mut store = Tracked::default();
    ///
    /// // Insert value
    /// store.insert("key", 42);
    /// ```
    #[inline]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.changed.insert(key.clone());
        self.store.insert(key, value)
    }

    /// Updates the value identified by the key if it changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store
    /// let mut store = Tracked::default();
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
            self.changed.insert(key.clone());
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
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
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
            self.changed.insert(key);
            value
        })
    }

    /// Removes the value identified by the key and returns both.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
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
            self.changed.insert(key.clone());
        })
    }

    /// Clears the store, removing all items.
    ///
    /// Note that this also clears all recorded changes. In order to know which
    /// which items are cleared, iterate over the store via [`Tracked::keys`][]
    /// or [`Tracked::iter`][] before clearing the store, which emits all keys
    /// as well as values, if necessary.
    ///
    /// [`Tracked::iter`]: crate::store::StoreIterable::iter
    /// [`Tracked::keys`]: crate::store::StoreKeys::keys
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{Store, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
    /// store.insert("key", 42);
    ///
    /// // Clear store
    /// store.clear();
    /// assert!(store.is_empty());
    /// ```
    fn clear(&mut self) {
        self.changed.clear();
        self.store.clear();
    }
}

impl<K, V, S> StoreIterable<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: StoreIterable<K, V>,
{
    type Iter<'a> = S::Iter<'a>
    where
        Self: 'a;

    /// Returns an iterator over the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{StoreIterable, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for (key, value) in store.iter() {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.store.iter()
    }
}

impl<K, V, S> StoreKeys<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: StoreKeys<K, V>,
{
    type Keys<'a> = S::Keys<'a>
    where
        Self: 'a;

    /// Creates an iterator over the keys of a store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{StoreKeys, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for key in store.keys() {
    ///     println!("{key}");
    /// }
    /// ```
    #[inline]
    fn keys(&self) -> Self::Keys<'_> {
        self.store.keys()
    }
}

impl<K, V, S> StoreValues<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: StoreValues<K, V>,
{
    type Values<'a> = S::Values<'a>
    where
        Self: 'a;

    /// Creates an iterator over the values of a store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{StoreMut, StoreValues};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for value in store.values() {
    ///     println!("{value}");
    /// }
    /// ```
    #[inline]
    fn values(&self) -> Self::Values<'_> {
        self.store.values()
    }
}

impl<K, V, S> StoreRange<K, V> for Tracked<K, V, S>
where
    K: Key,
    S: StoreRange<K, V>,
{
    type Range<'a> = S::Range<'a>
    where
        Self: 'a;

    /// Creates an iterator over a range of items in a store.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{StoreMut, StoreRange};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::<_, _, BTreeMap<_, _>>::new();
    /// store.insert("a", 42);
    /// store.insert("b", 84);
    ///
    /// // Create iterator over the store
    /// for (key, value) in store.range("b"..) {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn range<R>(&self, range: R) -> Self::Range<'_>
    where
        R: RangeBounds<K>,
    {
        self.store.range(range)
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S, C> StoreWithComparator<K, V, C> for Tracked<K, V, S>
where
    K: Key,
    S: Store<K, V> + StoreWithComparator<K, V, C>,
    C: Comparator<V>,
{
    /// Creates a store with the given comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use zrx_store::comparator::Descending;
    /// use zrx_store::decorator::{Ordered, Tracked};
    /// use zrx_store::{StoreMut, StoreWithComparator};
    ///
    /// // Create store
    /// let mut store: Tracked::<_, _, Ordered<_, _, HashMap<_, _>, _>> =
    ///     Tracked::with_comparator(Descending);
    ///
    /// // Insert value
    /// store.insert("key", 42);
    /// ```
    fn with_comparator(comparator: C) -> Self {
        Self {
            store: S::with_comparator(comparator),
            changed: HashSet::default(),
            marker: PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S> Index<usize> for Tracked<K, V, S>
where
    K: Key,
    S: Store<K, V> + Index<usize, Output = K>,
{
    type Output = K;

    /// Returns a reference to the key at the index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::{Indexed, Tracked};
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::<_, _, Indexed<_, _>>::new();
    /// store.insert("a", 42);
    /// store.insert("b", 84);
    ///
    /// // Obtain reference to key
    /// let key = &store[1];
    /// assert_eq!(key, &"b");
    /// ```
    #[inline]
    fn index(&self, n: usize) -> &Self::Output {
        &self.store[n]
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S> FromIterator<(K, V)> for Tracked<K, V, S>
where
    K: Key,
    S: Store<K, V> + StoreFromIterator<K, V>,
{
    /// Creates a store from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use zrx_store::decorator::Tracked;
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
    /// let store: Tracked<_, _, HashMap<_, _>> =
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
            changed: HashSet::default(),
            marker: PhantomData,
        }
    }
}

impl<K, V, S> IntoIterator for Tracked<K, V, S>
where
    K: Key,
    S: Store<K, V> + StoreIntoIterator<K, V>,
{
    type Item = (K, V);
    type IntoIter = S::IntoIter;

    /// Creates an iterator over the items of a store.
    ///
    /// This method consumes the store, and collects it into a vector, since
    /// there's currently no way to implement this due to the absence of ATPIT
    /// (associated type position impl trait) support in stable Rust. When the
    /// feature is stabilized, we can switch to a more efficient approach.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
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

#[allow(clippy::into_iter_without_iter)]
impl<'a, K, V, S> IntoIterator for &'a Tracked<K, V, S>
where
    K: Key,
    S: StoreIterable<K, V>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = S::Iter<'a>;

    /// Creates an iterator over the items of a store.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::{StoreIterable, StoreMut};
    ///
    /// // Create store and initial state
    /// let mut store = Tracked::default();
    /// store.insert("key", 42);
    ///
    /// // Create iterator over the store
    /// for (key, value) in &store {
    ///     println!("{key}: {value}");
    /// }
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ----------------------------------------------------------------------------

#[allow(clippy::implicit_hasher)]
impl<K, V> Default for Tracked<K, V, HashMap<K, V>>
where
    K: Key,
{
    /// Creates a tracking decorator with [`HashMap::default`][] as a store.
    ///
    /// Note that this method does not allow to customize the [`BuildHasher`][],
    /// but uses [`ahash`] by default, which is the fastest known hasher.
    ///
    /// [`BuildHasher`]: std::hash::BuildHasher
    /// [`HashMap::default`]: Default::default
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::decorator::Tracked;
    /// use zrx_store::StoreMut;
    ///
    /// // Create store
    /// let mut store = Tracked::default();
    ///
    /// // Insert value
    /// store.insert("key", 42);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------

impl<K, V, S> fmt::Debug for Tracked<K, V, S>
where
    K: fmt::Debug + Key,
    S: fmt::Debug + Store<K, V>,
{
    /// Formats the tracking decorator for debugging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tracked")
            .field("store", &self.store)
            .field("changes", &self.changed)
            .finish()
    }
}
