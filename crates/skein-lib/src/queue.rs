//! Queues for the records between stages (lib.md, section 5), capped at
//! startup, or uncapped between service components. Capped rings of at most
//! 1 MiB allocate at construction; larger and uncapped rings grow as they fill.

#![expect(clippy::disallowed_types, reason = "a queue is a VecDeque allocated under its cap, grown above 1 MiB")]

use core::hash::{Hash, Hasher};

use alloc::collections::VecDeque;
use alloc::collections::vec_deque;
use core::mem::size_of;

/// Test worlds' harnesses fill rings outside the metered process calls, so
/// rings through this many bytes allocate with their owner at construction.
const EAGER_BYTES: u32 = 1 << 20;

/// A first-in, first-out queue, allocated whole up to 1 MiB, otherwise grown
/// as it fills, never past its cap.
#[derive(Debug)]
pub struct Queue<T> {
    items: VecDeque<T>,
    capacity: u32,
}

impl<T> Queue<T> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Queue<T> {
        let cap = usize::try_from(capacity).expect("a u32 fits in a usize");
        let item = size_of::<T>();
        let eager = usize::try_from(EAGER_BYTES).expect("one MiB fits in a usize");
        let small = item == 0 || cap <= eager.checked_div(item).expect("nonzero item size");
        let items = if small { VecDeque::with_capacity(cap) } else { VecDeque::new() };
        Queue { items, capacity }
    }

    /// For hops between a service's own components, never for what a peer
    /// can push. It grows as it fills, up to `u32::MAX` items.
    #[must_use]
    pub fn uncapped() -> Queue<T> {
        Queue { items: VecDeque::new(), capacity: u32::MAX }
    }

    /// The most heap a queue of `capacity` takes for its items, or `None` past a
    /// `u64`, including old and new buffers during growth. What the items own
    /// is theirs to count.
    #[must_use]
    pub fn worst_case(capacity: u32) -> Option<u64> {
        growth_peak(capacity, u64::try_from(size_of::<T>()).ok()?)
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

    /// Appends an item to a queue that has room for it.
    ///
    /// This is how a step emits: the loop reserved `MAX_OUT` slots before
    /// calling it, so a full queue here is a bug, not an outcome.
    pub fn push(&mut self, item: T) {
        assert!(self.room() > 0, "the loop reserves room for every output");
        self.grow();
        self.items.push_back(item);
    }

    /// Appends an item, or hands it back when the queue is full.
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.room() == 0 {
            return Err(item);
        }
        self.grow();
        self.items.push_back(item);
        Ok(())
    }

    /// Reserves the next doubling, clamped to the cap.
    fn grow(&mut self) {
        if self.items.len() < self.items.capacity() {
            return;
        }
        let cap = usize::try_from(self.capacity).expect("a u32 fits in a usize");
        let next = self.items.capacity().saturating_mul(2).max(1).min(cap);
        let additional = next.checked_sub(self.items.len()).expect("room was checked before growing");
        self.items.reserve_exact(additional);
    }

    /// Takes the oldest item.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    /// The items, oldest first.
    #[must_use]
    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.items.iter()
    }
}

impl<'a, T> IntoIterator for &'a Queue<T> {
    type Item = &'a T;
    type IntoIter = vec_deque::Iter<'a, T>;

    fn into_iter(self) -> vec_deque::Iter<'a, T> {
        self.items.iter()
    }
}

// lib.md, 5.1: hash held state, never capacity or storage layout.
impl<T: Hash> Hash for Queue<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

/// The growth peak counts the cap's buffer and the largest buffer reallocated
/// on the way, live together while copying. Rings of at most 1 MiB allocate
/// their cap at construction and hold only that buffer.
fn growth_peak(capacity: u32, item: u64) -> Option<u64> {
    let cap = u64::from(capacity);
    let full = cap.checked_mul(item)?;
    if full <= u64::from(EAGER_BYTES) || capacity <= 1 {
        return Some(full);
    }
    let mut previous = 1_u64;
    // At most 32 doublings reach the u32 cap.
    while previous.checked_mul(2)? < cap {
        previous = previous.checked_mul(2)?;
    }
    full.checked_add(previous.checked_mul(item)?)
}
