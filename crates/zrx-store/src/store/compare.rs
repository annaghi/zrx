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

//! Comparator.

use std::cmp::Ordering;
use std::fmt;
use std::ops::Deref;

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Comparator.
///
/// This data type is a thin wrapper around a value of type `T`, which allows
/// to set a custom function `F` to customize the ordering of values in stores.
/// It implements the [`Eq`], [`PartialEq`], [`Ord`] and [`PartialOrd`] traits,
/// as well as [`Deref`], so the original value can be used transparently.
///
/// If `F` is a zero-sized type (ZST), and doesn't capture any variables, the
/// runtime overhead is zero, as the compiler is able to optimize it away.
///
/// # Examples
///
/// ```
/// use zrx_store::Compare;
///
/// // Create comparison function
/// let f = |x: &i32, y: &i32| y.cmp(x);
///
/// // Create and compare values
/// let a = Compare(42, f);
/// let b = Compare(84, f);
/// assert!(a > b);
/// ```
#[derive(Clone)]
pub struct Compare<T, F>(pub T, pub F);

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl<T, F> PartialEq for Compare<T, F>
where
    T: Eq,
{
    /// Compares two values for equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::Compare;
    ///
    /// // Create comparison function
    /// let f = |x: &i32, y: &i32| y.cmp(x);
    ///
    /// // Create and compare values
    /// let a = Compare(42, f);
    /// let b = Compare(42, f);
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, F> Eq for Compare<T, F> where T: Eq {}

// ----------------------------------------------------------------------------

impl<T, F> PartialOrd for Compare<T, F>
where
    T: Eq,
    F: Fn(&T, &T) -> Ordering,
{
    /// Orders two values.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::Compare;
    ///
    /// // Create comparison function
    /// let f = |x: &i32, y: &i32| y.cmp(x);
    ///
    /// // Create and compare values
    /// let a = Compare(42, f);
    /// let b = Compare(84, f);
    /// assert!(a > b);
    /// ```
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, F> Ord for Compare<T, F>
where
    T: Eq,
    F: Fn(&T, &T) -> Ordering,
{
    /// Orders two values.
    ///
    /// # Examples
    ///
    /// ```
    /// use zrx_store::Compare;
    ///
    /// // Create comparison function
    /// let f = |x: &i32, y: &i32| y.cmp(x);
    ///
    /// // Create and compare values
    /// let a = Compare(42, f);
    /// let b = Compare(84, f);
    /// assert!(a > b);
    /// ```
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (self.1)(&self.0, &other.0)
    }
}

// ----------------------------------------------------------------------------

impl<T, F> Deref for Compare<T, F> {
    type Target = T;

    /// Dereferences to the wrapped value.
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ----------------------------------------------------------------------------

impl<T, F> fmt::Debug for Compare<T, F>
where
    T: fmt::Debug,
{
    /// Formats the comparator for debugging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
