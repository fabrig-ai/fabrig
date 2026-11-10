//! Bounded least-recently-used tables (runtime/lib.md, 5 and 5.1).
use crate::{Map, Set};
use core::hash::{Hash, Hasher};

/// A bounded table that drops the entry used longest ago (runtime/lib.md, 5).
#[derive(Debug)]
pub struct Lru<K, V> {
    by_use: Set<(u64, K)>,
    by_key: Map<K, (u64, V)>,
    clock: u64,
}
impl<K: Ord + Copy, V> Lru<K, V> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Lru<K, V> {
        Lru { by_use: Set::with_capacity(capacity), by_key: Map::with_capacity(capacity), clock: 0 }
    }
    #[must_use]
    pub fn price(capacity: u32) -> Option<u64> {
        Set::<(u64, K)>::price(capacity)?.checked_add(Map::<K, (u64, V)>::price(capacity)?)
    }
    #[must_use]
    pub fn len(&self) -> u32 {
        self.by_key.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
    /// The value under `key`, now the most recently used.
    pub fn get(&mut self, key: K) -> Option<&V> {
        let (used, value) = self.by_key.get_mut(&key)?;
        assert!(self.by_use.remove(&(*used, key)), "both indexes hold every entry");
        self.clock = self.clock.saturating_add(1);
        *used = self.clock;
        assert!(self.by_use.insert((*used, key)).is_ok(), "removing the old use makes room");
        Some(value)
    }
    /// Keeps `value`, most recently used; hands back what left (runtime/lib.md, 5).
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        if self.by_key.capacity() == 0 {
            return Some((key, value));
        }
        let mut left = None;
        if let Some(old) = self.remove(key) {
            left = Some((key, old));
        } else if self.len() == self.by_key.capacity() {
            let &(_, oldest) = self.by_use.first().expect("a full nonzero table holds an entry");
            let old = self.remove(oldest).expect("both indexes hold every entry");
            left = Some((oldest, old));
        }
        self.clock = self.clock.saturating_add(1);
        assert!(self.by_key.insert(key, (self.clock, value)).is_ok(), "the new key has room after removal");
        assert!(self.by_use.insert((self.clock, key)).is_ok(), "both indexes have the same keys and capacity");
        left
    }
    pub fn remove(&mut self, key: K) -> Option<V> {
        let (used, value) = self.by_key.remove(&key)?;
        assert!(self.by_use.remove(&(used, key)), "both indexes hold every entry");
        Some(value)
    }
}
impl<K: Hash + Ord, V: Hash> Hash for Lru<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.by_key.len().hash(state);
        for (_, key) in &self.by_use {
            key.hash(state);
            self.by_key.get(key).expect("both indexes hold every entry").1.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lru;

    #[test]
    fn a_saturated_lru_evicts_equal_uses_by_key() {
        // Three rows distinguish an older use from two tied at the clock's end.
        for refresh in [false, true] {
            let mut table = Lru::<u8, u8>::with_capacity(3);
            table.clock = u64::MAX - 2;
            assert_eq!(table.insert(9, 90), None);
            assert_eq!(table.insert(3, 30), None);
            assert_eq!(table.insert(1, 10), None);
            assert_eq!(table.insert(2, 20), Some((9, 90)), "older than the tied uses");
            if refresh {
                assert_eq!(table.get(3), Some(&30), "a get also saturates");
            }
            assert_eq!(table.insert(4, 40), Some((1, 10)), "least key among equal uses");
            assert_eq!(table.clock, u64::MAX);
        }
    }
}
