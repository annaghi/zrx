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

//! Iterator over filter.

use std::iter::Peekable;

use crate::id::matcher::matches::IntoIter;
use crate::id::matcher::Matches;
use crate::id::TryIntoId;

use super::condition::Condition;
use super::error::Result;
use super::Filter;

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Iterator over filter.
pub struct Iter<'a> {
    /// Iterator over matches.
    matches: Peekable<IntoIter>,
    /// Iterator over conditions.
    conditions: slab::Iter<'a, Condition>,
    /// Current condition index.
    index: usize,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl Filter {
    /// Returns the indices of expressions that match the filter.
    ///
    /// This method compares all expressions within the filter against the given
    /// identifier, and returns an iterator over the indices of the expressions
    /// that match. Note that the order of the returned indices corresponds to
    /// the order in which the expressions were added to the filter.
    ///
    /// # Errors
    ///
    /// This method returns [`Error::Matcher`][] if the identifier is invalid.
    ///
    /// [`Error::Matcher`]: crate::id::filter::Error::Matcher
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zrx_id::{selector, Expression, Filter, Id};
    ///
    /// // Create filter builder and insert expression
    /// let mut builder = Filter::builder();
    /// builder.insert(Expression::any(|expr| {
    ///     expr.with(selector!(location = "**/*.md")?)?
    ///         .with(selector!(provider = "file")?)
    /// })?);
    ///
    /// // Create filter from builder
    /// let filter = builder.build()?;
    ///
    /// // Create identifier and obtain matched expressions
    /// let id: Id = "zri:file:::docs:index.md:".parse()?;
    /// for index in filter.matches(&id)? {
    ///     println!("{index:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn matches<T>(&self, id: &T) -> Result<Iter<'_>>
    where
        T: TryIntoId,
    {
        let matches = self.matcher.matches(id)?;
        Ok(Iter {
            matches: matches.into_iter().peekable(),
            conditions: self.conditions.iter(),
            index: 0,
        })
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl Iterator for Iter<'_> {
    type Item = usize;

    /// Returns the next satisfied condition.
    fn next(&mut self) -> Option<Self::Item> {
        let mut matches = Matches::default();

        // Obtain next condition and check for matches
        for (index, condition) in self.conditions.by_ref() {
            let end = self.index + condition.terms().len();

            // Collect all matches for the current condition
            while self.matches.peek().is_some_and(|&index| index < end) {
                let index = self.matches.next().expect("invariant");
                matches.insert(index - self.index);
            }

            // Advance index to the end of the current condition
            self.index = end;

            // Verify if condition is satisfied and return it
            let check = condition.is_universal() || !matches.is_empty();
            if check && condition.satisfies(&matches) {
                return Some(index);
            }
        }

        // No more satisfied conditions to return
        None
    }
}
