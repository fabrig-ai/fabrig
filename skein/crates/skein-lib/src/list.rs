//! Bounded sequences (skein/docs/design/lib.md, 5): what a step appends to,
//! growing as they fill under a cap.

#![expect(clippy::disallowed_types, reason = "a list is a Vec grown exactly as it fills, never past its cap")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::mem::size_of;
use core::slice;

use crate::growth::grown;

/// A sequence that grows up to a capacity fixed when it is made, and refuses
/// past it. It allocates as it fills, never past its capacity and never shrinking.
#[derive(Debug)]
pub struct List<T> {
    items: Vec<T>,
    capacity: u32,
}

impl<T> List<T> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> List<T> {
        List { items: Vec::new(), capacity }
    }

    /// Owned items moved into a list whose capacity is their count.
    /// The count must fit a `u32`.
    #[must_use]
    #[expect(clippy::disallowed_methods, reason = "a list adopts the boxed items as its buffer without copying")]
    pub fn from_boxed(items: Box<[T]>) -> List<T> {
        let capacity = u32::try_from(items.len()).expect("a list's item count fits a u32");
        List { items: items.into_vec(), capacity }
    }

    /// For a service's own correlation and collection, never for what a peer can
    /// push. It grows as it fills, up to `u32::MAX` items.
    #[must_use]
    pub fn uncapped() -> List<T> {
        Self::with_capacity(u32::MAX)
    }

    /// The heap a list of `capacity` takes for its items, or `None` past a
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

    /// How many more items fit.
    #[must_use]
    pub fn room(&self) -> u32 {
        self.capacity.checked_sub(self.len()).expect("no longer than its capacity")
    }

    /// Appends an item, or hands it back when the list is full.
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let len = self.items.len();
        let cap = crate::usize_of(self.capacity);
        if len == cap {
            return Err(item);
        }
        if len == self.items.capacity() {
            let needed = len.checked_add(1).expect("room for another item");
            let additional = grown(len, needed, cap).checked_sub(len).expect("room for another item");
            self.items.reserve_exact(additional);
        }
        self.items.push(item);
        Ok(())
    }

    /// The item at `index`, counted from the first pushed.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&T> {
        self.items.get(usize::try_from(index).ok()?)
    }

    /// The item at `index`, to change in place: a list of slots pushed in
    /// order can be filled in any order.
    #[must_use]
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.items.get_mut(usize::try_from(index).ok()?)
    }

    /// Takes the item at `index` out, the last item taking its place. `None` past the end.
    pub fn swap_remove(&mut self, index: u32) -> Option<T> {
        let index = usize::try_from(index).ok()?;
        if index >= self.items.len() {
            return None;
        }
        Some(self.items.swap_remove(index))
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    /// Removes a suffix without changing the allocated capacity.
    pub fn truncate(&mut self, length: u32) {
        assert!(length <= self.len(), "truncate only removes a suffix");
        self.items.truncate(crate::usize_of(length));
    }

    /// Empties the list, keeping its grown buffer: a list refilled in place
    /// to its previous length allocates nothing.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.items.iter()
    }

    /// Each item, to change in place.
    #[expect(clippy::iter_without_into_iter, reason = "List exposes mutable traversal through iter_mut")]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.items.iter_mut()
    }

    /// The items, moved into a box of exactly their number.
    #[must_use]
    pub fn into_boxed(self) -> Box<[T]> {
        self.items.into_boxed_slice()
    }

    /// A copy of the items in a box of exactly their number: copy at emission.
    #[must_use]
    pub fn to_boxed(&self) -> Box<[T]>
    where
        T: Clone,
    {
        Box::from(self.items.as_slice())
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.items.iter()
    }
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> List<T> {
        let mut items = Vec::with_capacity(self.items.len());
        items.extend_from_slice(&self.items);
        List { items, capacity: self.capacity }
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &List<T>) -> bool {
        self.items == other.items
    }
}

impl<T: Eq> Eq for List<T> {}

impl<T: Hash> Hash for List<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.len().hash(state);
        for item in &self.items {
            item.hash(state);
        }
    }
}
