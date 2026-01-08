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

//! Condition operand.

use crate::id::filter::expression::Operator;
use crate::id::matcher::Matches;

// ----------------------------------------------------------------------------
// Enums
// ----------------------------------------------------------------------------

/// Condition group.
#[derive(Debug, PartialEq, Eq)]
pub enum Group {
    /// Group with operator.
    Operator(Operator, Vec<Group>),
    /// Group with terms.
    Terms(Matches),
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl Group {
    /// Maps a condition operand in post-order with the given function.
    ///
    /// This method traverses the condition in post-order, and allows to map
    /// each node using the provided function. The function is applied after
    /// mapping the child nodes, so a condition can be rewritten from the
    /// bottom up, e.g., to optimize or transform the condition operand.
    ///
    /// This implementation deliberately uses an explicit stack, because the
    /// Rust compiler will run into stack overflows for deep recursion.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(Group) -> Group,
    {
        // Initialize stack for input and processed conditions
        let mut input = Vec::from([(self, false, 0)]);
        let mut stack = Vec::new();

        // Process input stack until empty, transforming conditions in pre-
        // order, moving them onto the stack of processed conditions
        while let Some((condition, visited, arity)) = input.pop() {
            match condition {
                // 1st visit: push condition onto stack and mark as visited,
                // then push children in reverse onto stack for correct order
                Group::Operator(operator, operands) if !visited => {
                    let condition = Group::Operator(operator, Vec::new());
                    input.push((condition, true, operands.len()));

                    // Push children in reverse onto stack
                    for condition in operands.into_iter().rev() {
                        input.push((condition, false, 0));
                    }
                }
                // 2nd visit: pop exactly as many children as the condition
                // expects from the stack and process the condition
                Group::Operator(operator, ..) => {
                    let operands = stack.split_off(stack.len() - arity);
                    stack.push(f(Group::Operator(operator, operands)));
                }
                // Push terms onto stack for processing
                Group::Terms(_) if !visited => {
                    stack.push(f(condition));
                }
                // This should never happen
                Group::Terms(_) => unreachable!(),
            }
        }

        // Return the top-most processed condition
        stack.pop().expect("invariant")
    }
}
