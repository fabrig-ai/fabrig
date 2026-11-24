//! Small ordered tables for the fake filesystem's metadata.
use alloc::vec::Vec;
use core::borrow::Borrow;

// Keep entries together, without reserving a whole B-tree node in a small
// table. Keys still determine the order of listings and seeded crash replay.
#[derive(Clone, Debug, Hash)]
pub(crate) struct Table<K, V>(Vec<(K, V)>);

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K, V> Table<K, V> {
    fn position<Q: Ord + ?Sized>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
    {
        self.0.binary_search_by(|(found, _)| found.borrow().cmp(key))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.position(key).ok().and_then(|at| self.0.get(at)).map(|(_, value)| value)
    }

    pub(super) fn get_mut<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        self.position(key).ok().and_then(|at| self.0.get_mut(at)).map(|(_, value)| value)
    }

    pub(super) fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        match self.position(&key) {
            Ok(at) => Some(core::mem::replace(&mut self.0.get_mut(at).expect("a found position").1, value)),
            Err(at) => {
                if self.len() == self.0.capacity() {
                    self.0.reserve_exact((self.len() / 2).max(1));
                }
                self.0.insert(at, (key, value));
                None
            }
        }
    }

    pub(super) fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let at = self.position(key).ok()?;
        let (_, value) = self.0.remove(at);
        if self.len() <= self.0.capacity() / 2 {
            self.0.shrink_to_fit();
        }
        Some(value)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(key, value)| (key, value))
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.0.iter_mut().map(|(key, value)| (&*key, value))
    }
}
