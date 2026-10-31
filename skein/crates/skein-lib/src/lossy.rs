//! Facts a reader may miss: kept up to a cap, the rest dropped and counted
//! (skein/docs/design/lib.md, 5).

use crate::Queue;

/// A capped first-in, first-out queue that counts each item it cannot hold.
#[derive(Debug, Hash)]
pub struct Lossy<T> {
    queue: Queue<T>,
    lost: u64,
}

impl<T> Lossy<T> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Lossy<T> {
        Self { queue: Queue::with_capacity(capacity), lost: 0 }
    }

    /// The heap for `capacity` items, or `None` past a `u64`.
    /// What the items own is theirs to count.
    #[must_use]
    pub fn price(capacity: u32) -> Option<u64> {
        Queue::<T>::price(capacity)
    }

    /// Keeps the item if there is room; otherwise drops it and counts the loss.
    /// The count saturates at `u64::MAX`.
    pub fn push(&mut self, item: T) {
        if self.queue.try_push(item).is_err() {
            self.lost = self.lost.saturating_add(1);
        }
    }

    /// Takes the oldest kept item.
    pub fn pop(&mut self) -> Option<T> {
        self.queue.pop()
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.queue.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Items dropped since the queue was made, including those before a pop.
    #[must_use]
    pub const fn lost(&self) -> u64 {
        self.lost
    }
}

#[cfg(test)]
#[path = "tests/lossy.rs"]
mod tests;
