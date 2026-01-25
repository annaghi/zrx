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
use std::fmt::Debug;

mod comparable;

pub use comparable::Comparable;

// ----------------------------------------------------------------------------
// Traits
// ----------------------------------------------------------------------------

/// Comparator.
///
/// This data type defines a comparator for values of type `T`, which can be
/// used to customize the ordering of values in stores. If it's a zero-sized
/// type (ZST), e.g., a struct without fields or closure that doesn't capture
/// any variables, it's optimized away, resulting in zero-runtime overhead.
pub trait Comparator<T> {
    /// Compares two values.
    fn cmp(&self, a: &T, b: &T) -> Ordering;
}

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Comparator for ascending order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ascending;

/// Comparator for descending order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descending;

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl<T> Comparator<T> for Ascending
where
    T: Ord,
{
    /// Compares two values in ascending order.
    #[inline]
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<T> Comparator<T> for Descending
where
    T: Ord,
{
    /// Compares two values in descending order.
    #[inline]
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        b.cmp(a)
    }
}

// ----------------------------------------------------------------------------
// Blanket implementations
// ----------------------------------------------------------------------------

impl<T, F> Comparator<T> for F
where
    F: Fn(&T, &T) -> Ordering,
{
    #[inline]
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        self(a, b)
    }
}
