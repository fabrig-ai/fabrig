//! Bounded nesting (programming-model.md, section 8): what a parser keeps
//! instead of recursing, growing as input fills it under the fixed depth cap.

#![expect(clippy::disallowed_types, reason = "a stack is a Vec grown exactly as it fills, never past its cap")]

use core::hash::{Hash, Hasher};

use alloc::vec::Vec;
use core::mem::size_of;

use crate::grow::growth;

/// A last-in, first-out stack that never grows past its capacity, and
/// refuses past it: input nested deeper than the capacity is the caller's to
/// refuse. It allocates as it fills and never shrinks.
#[derive(Debug)]
pub struct Stack<T> {
    items: Vec<T>,
    capacity: u32,
}

impl<T> Stack<T> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Stack<T> {
        Stack { items: Vec::new(), capacity }
    }

    /// The heap a stack of `capacity` takes for its items, or `None` past a
    /// `u64`. What the items own is theirs to count.
    #[must_use]
    pub fn price(capacity: u32) -> Option<u64> {
        u64::try_from(size_of::<T>()).ok()?.checked_mul(u64::from(capacity))
    }

    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.items.len()).expect("no longer than its capacity")
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Pushes an item on top, or hands it back when the stack is full.
    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.len() == self.capacity {
            return Err(item);
        }
        self.grow();
        self.items.push(item);
        Ok(())
    }

    /// Makes room for one more item, by lib's growth rule.
    fn grow(&mut self) {
        let len = self.items.len();
        let needed = len.checked_add(1).expect("room for another item");
        self.items.reserve_exact(growth(len, self.items.capacity(), needed, self.capacity));
    }

    /// Empties the stack, keeping its grown buffer: a stack refilled to its
    /// previous depth allocates nothing.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Takes the top item.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// The top item: the innermost level.
    #[must_use]
    pub fn top(&self) -> Option<&T> {
        self.items.last()
    }

    /// The top item, to change in place.
    #[must_use]
    pub fn top_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }
}

// lib.md, 5.1: hash held state, never capacity or storage layout.
impl<T: Hash> Hash for Stack<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.len().hash(state);
        for item in &self.items {
            item.hash(state);
        }
    }
}
