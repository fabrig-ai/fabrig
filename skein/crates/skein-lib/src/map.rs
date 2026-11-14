//! Bounded ordered maps (programming-model.md, 6.1): domain tables keyed by
//! value, up to a limit.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::{self, Entry};
use core::borrow::Borrow;
use core::mem::{align_of, size_of};
use core::ops::RangeBounds;

use crate::btree;

/// An ordered map that holds at most a capacity fixed when it is made.
///
/// Inserting a new key into a full map hands the key and value back: that is
/// the refusal. Replacing the value of a key already present always works.
/// Lookups take the key borrowed, so a map keyed by `Box<[u8]>` is queried
/// with a `&[u8]`.
///
/// The capacity bounds the map, but unlike a slab or a list it is not allocated
/// up front: the B-tree allocates nodes as it fills (programming-model.md,
/// 6.1), and
/// [`Map::price`] counts those nodes.
#[derive(Debug)]
pub struct Map<K, V> {
    entries: BTreeMap<K, V>,
    capacity: u32,
}

impl<K: Ord, V> Map<K, V> {
    #[must_use]
    pub const fn with_capacity(capacity: u32) -> Map<K, V> {
        Map { entries: BTreeMap::new(), capacity }
    }

    /// For a service's own correlation and collection, never for what a peer can
    /// push. It grows as it fills, up to `u32::MAX` entries.
    #[must_use]
    pub fn uncapped() -> Map<K, V> {
        Self::with_capacity(u32::MAX)
    }

    /// The most heap a map of `capacity` entries takes, its tree nodes
    /// included, or `None` past a `u64`. What the keys and values own, such as
    /// the bytes of a `Box<[u8]>`, is the owner's to count.
    #[must_use]
    pub fn price(capacity: u32) -> Option<u64> {
        btree::price(capacity, size_of::<K>(), size_of::<V>(), align_of::<(K, V)>())
    }

    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.entries.len()).expect("no more entries than the capacity")
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.contains_key(key)
    }

    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.get(key)
    }

    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.get_mut(key)
    }

    /// Maps `key` to `value`, returning the value it replaces, if any. Hands
    /// both back when `key` is new and the map is full. A replaced entry keeps
    /// the key it had, and drops the one given.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        let full = self.len() >= self.capacity;
        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => Ok(Some(entry.insert(value))),
            Entry::Vacant(entry) => {
                if full {
                    return Err((entry.into_key(), value));
                }
                let _: &mut V = entry.insert(value);
                Ok(None)
            }
        }
    }

    /// Removes `key`, returning its value if it was present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.remove(key)
    }

    /// The entry with the least key.
    #[must_use]
    pub fn first(&self) -> Option<(&K, &V)> {
        self.entries.first_key_value()
    }

    /// The entry with the greatest key.
    #[must_use]
    pub fn last(&self) -> Option<(&K, &V)> {
        self.entries.last_key_value()
    }

    /// The entries, in key order.
    pub fn iter(&self) -> btree_map::Iter<'_, K, V> {
        self.entries.iter()
    }

    /// Each key of either map once, in key order (runtime/lib.md, 5.2).
    pub fn join<'a>(&'a self, other: &'a Map<K, V>) -> Join<'a, K, V> {
        let mut first = self.entries.iter();
        let mut second = other.entries.iter();
        let next_first = first.next();
        let next_second = second.next();
        Join { first, second, next_first, next_second }
    }

    /// The entries in a key range, in key order.
    pub fn range<Q, R>(&self, range: R) -> btree_map::Range<'_, K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        self.entries.range(range)
    }
}

/// The values at a joined key, first map then second (runtime/lib.md, 5.2).
#[derive(Debug, PartialEq, Eq)]
pub enum Joined<'a, V> {
    First(&'a V),
    Second(&'a V),
    Both(&'a V, &'a V),
}

impl<V> Copy for Joined<'_, V> {}

impl<V> Clone for Joined<'_, V> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A borrowing, allocation-free walk over two maps by key (runtime/lib.md, 5.2).
pub struct Join<'a, K, V> {
    first: btree_map::Iter<'a, K, V>,
    second: btree_map::Iter<'a, K, V>,
    next_first: Option<(&'a K, &'a V)>,
    next_second: Option<(&'a K, &'a V)>,
}

impl<'a, K: Ord, V> Iterator for Join<'a, K, V> {
    type Item = (&'a K, Joined<'a, V>);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((first_key, first_value)) = self.next_first else {
            let (key, value) = self.next_second?;
            self.next_second = self.second.next();
            return Some((key, Joined::Second(value)));
        };
        let Some((second_key, second_value)) = self.next_second else {
            self.next_first = self.first.next();
            return Some((first_key, Joined::First(first_value)));
        };
        match first_key.cmp(second_key) {
            Ordering::Less => {
                self.next_first = self.first.next();
                Some((first_key, Joined::First(first_value)))
            }
            Ordering::Equal => {
                self.next_first = self.first.next();
                self.next_second = self.second.next();
                Some((first_key, Joined::Both(first_value, second_value)))
            }
            Ordering::Greater => {
                self.next_second = self.second.next();
                Some((second_key, Joined::Second(second_value)))
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> btree_map::Iter<'a, K, V> {
        self.entries.iter()
    }
}

// runtime/lib.md, 5.1: hash held state, never capacity or storage layout.
impl<K: Hash, V: Hash> Hash for Map<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entries.hash(state);
    }
}
