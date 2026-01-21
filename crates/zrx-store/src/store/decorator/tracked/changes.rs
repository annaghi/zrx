// Copyright (c) 2025-2026 Zensical and contributors

// SPDX-License-Identifier: MIT
// All contributions are certified under the DCO

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

//! Iterator implementations for [`Tracked`].

use ahash::HashMap;
use std::collections::hash_set;
use std::marker::PhantomData;
use std::mem;

use crate::store::key::Key;
use crate::store::Store;

use super::Tracked;

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Iterator over the changes of a [`Tracked`] store.
pub struct Changes<'a, K, V, S = HashMap<K, V>>
where
    K: Key,
    S: Store<K, V>,
{
    /// Underlying store.
    store: &'a S,
    /// Keys of changed items.
    changed: hash_set::IntoIter<K>,
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
    /// Creates an iterator over the tracked changes of a store.
    ///
    /// This method returns an iterator over all changed keys since the last
    /// call to this method. The iterator yields tuples of keys and optional
    /// references to the corresponding values in the store, so if a key was
    /// removed from the store, the value will be [`None`].
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
    /// // Obtain changes from store
    /// for (key, opt) in store.changes() {
    ///     println!("{key}: {opt:?}");
    /// }
    /// ```
    pub fn changes(&mut self) -> Changes<'_, K, V, S> {
        Changes {
            store: &self.store,
            changed: mem::take(&mut self.changed).into_iter(),
            marker: PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl<'a, K, V, S> Iterator for Changes<'a, K, V, S>
where
    K: Key,
    V: 'a,
    S: Store<K, V>,
{
    type Item = (K, Option<&'a V>);

    /// Returns the next changed item.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.changed.next().map(|key| {
            let opt = self.store.get(&key);
            (key, opt)
        })
    }

    /// Returns the bounds on the remaining length of the iterator.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.changed.size_hint()
    }
}

impl<'a, K, V, S> ExactSizeIterator for Changes<'a, K, V, S>
where
    K: Key,
    V: 'a,
    S: Store<K, V>,
{
    /// Returns the exact remaining length of the iterator.
    #[inline]
    fn len(&self) -> usize {
        self.changed.len()
    }
}
