//! Bounded last bytes of a stream (skein/docs/design/lib.md, 6).

use crate::{List, Writer};
use alloc::boxed::Box;

/// The newest bytes, retained in a ring; putting bytes never reallocates after filling.
#[derive(Debug)]
pub struct Tail {
    bytes: List<u8>,
    next: u32,
}

impl Tail {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Self {
        Self { bytes: List::with_capacity(capacity), next: 0 }
    }

    #[must_use]
    pub fn capacity(&self) -> u32 {
        self.bytes.capacity()
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.bytes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Retains this byte, dropping the oldest when full; zero capacity discards it.
    pub fn push(&mut self, byte: u8) {
        if self.bytes.room() > 0 {
            self.bytes.push(byte).expect("tail has room");
        } else if self.capacity() > 0 {
            *self.bytes.get_mut(self.next).expect("tail slot exists") = byte;
            self.next = self
                .next
                .checked_add(1)
                .expect("index below capacity")
                .checked_rem(self.capacity())
                .expect("positive capacity");
        }
    }

    /// Retains the newest bytes across this and earlier puts.
    pub fn put(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push(*byte);
        }
    }

    /// A retained byte in arrival order, oldest first, or `None` past the end.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&u8> {
        if index >= self.len() {
            return None;
        }
        let at = u64::from(self.next)
            .checked_add(u64::from(index))
            .expect("two u32 indices fit")
            .checked_rem(u64::from(self.len()))
            .expect("nonempty tail");
        self.bytes.get(u32::try_from(at).expect("index below length"))
    }

    /// A copy of the retained bytes in arrival order, at exactly their length.
    #[must_use]
    pub fn to_boxed(&self) -> Box<[u8]> {
        let mut output = Writer::new(usize::try_from(self.len()).expect("u32 fits usize"));
        for index in 0..self.len() {
            output.put(&[*self.get(index).expect("retained index")]).expect("sized tail");
        }
        output.finish()
    }

    /// Discards retained bytes, keeping the ring's capacity.
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.next = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::Tail;

    #[test]
    fn every_chunking_keeps_the_same_suffix_at_zero_and_wrap_boundaries() {
        let input = b"abcdefghijk";
        for capacity in 0..=12 {
            for split in 0..=input.len() {
                let mut tail = Tail::with_capacity(capacity);
                tail.put(&input[..split]);
                let prefix_start = split.saturating_sub(usize::try_from(capacity).expect("small capacity"));
                assert_eq!(tail.to_boxed().as_ref(), &input[prefix_start..split]);
                tail.put(&[]);
                tail.put(&input[split..]);
                let start = input.len().saturating_sub(usize::try_from(capacity).expect("small capacity"));
                assert_eq!(tail.to_boxed().as_ref(), &input[start..]);
                assert_eq!(tail.get(tail.len()), None);
                assert_eq!(tail.capacity(), capacity);
                tail.clear();
                assert!(tail.is_empty());
                tail.put(b"xy");
                assert_eq!(
                    tail.to_boxed().as_ref(),
                    &b"xy"[2_usize.saturating_sub(usize::try_from(capacity).expect("small capacity"))..]
                );
            }
        }
    }
}
