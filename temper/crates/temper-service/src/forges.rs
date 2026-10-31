//! The forges this service serves, kept while it runs (temper/docs/design/draft/playground-reset.md, 3).
use crate::settings::Forge;
use alloc::boxed::Box;
use skein_lib::{List, Map};

#[derive(Debug)]
pub struct Forges {
    forges: Map<u16, Forge>,
}

impl Forges {
    #[must_use]
    pub const fn with_capacity(capacity: u32) -> Self {
        Self { forges: Map::with_capacity(capacity) }
    }

    #[must_use]
    pub fn from_document(forges: Box<[Forge]>, capacity: u32) -> Self {
        let mut table = Self::with_capacity(capacity);
        for forge in forges {
            assert!(table.insert(forge), "validated document forges fit with unique numbers");
        }
        table
    }

    #[must_use]
    pub fn get(&self, forge: u16) -> Option<&Forge> {
        self.forges.get(&forge)
    }

    #[must_use]
    pub fn get_mut(&mut self, forge: u16) -> Option<&mut Forge> {
        self.forges.get_mut(&forge)
    }

    /// The forges in number order.
    pub fn iter(&self) -> impl Iterator<Item = &Forge> {
        Iter { entries: self.forges.iter() }
    }

    /// The record loader's positional cursor, in number order.
    #[must_use]
    pub fn nth(&self, index: usize) -> Option<&Forge> {
        self.iter().nth(index)
    }

    #[must_use]
    pub fn numbers(&self) -> List<u16> {
        let mut numbers = List::with_capacity(self.len());
        for (number, _) in &self.forges {
            numbers.push(*number).expect("one number per forge");
        }
        numbers
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.forges.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.forges.is_empty()
    }

    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.forges.capacity()
    }

    /// Refuse a full table or a number already present, preserving its forge.
    pub fn insert(&mut self, forge: Forge) -> bool {
        if self.forges.contains_key(&forge.number) {
            return false;
        }
        self.forges.insert(forge.number, forge).is_ok()
    }

    pub fn remove(&mut self, forge: u16) -> Option<Forge> {
        self.forges.remove(&forge)
    }
}

struct Iter<'a> {
    entries: alloc::collections::btree_map::Iter<'a, u16, Forge>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Forge;

    fn next(&mut self) -> Option<Self::Item> {
        match self.entries.next() {
            Some((_, forge)) => Some(forge),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forge(number: u16) -> Forge {
        let mut forges = crate::tests::fixtures::forges().into_iter();
        let mut forge = forges.next().expect("fixture forge");
        forge.number = number;
        forge
    }

    #[test]
    fn insert_refuses_a_taken_number_and_a_full_table() {
        let mut forges = Forges::with_capacity(2);
        assert_eq!(forges.capacity(), 2);
        assert!(forges.insert(forge(7)));
        let mut duplicate = forge(7);
        duplicate.user = 99;
        assert!(!forges.insert(duplicate));
        assert_eq!(forges.get(7).unwrap().user, 42);
        assert!(forges.insert(forge(3)));
        assert!(!forges.insert(forge(9)));
        assert_eq!(forges.len(), 2);
        assert!(forges.get(9).is_none());
        assert_eq!(forges.remove(7).unwrap().number, 7);
        assert!(forges.remove(7).is_none());
        assert!(forges.insert(forge(9)));
        forges.get_mut(9).unwrap().user = 17;
        assert_eq!(forges.get(9).unwrap().user, 17);
    }

    #[test]
    fn iteration_is_in_number_order() {
        let forges = Forges::from_document(Box::new([forge(9), forge(3), forge(7)]), 4);
        let mut iter = forges.iter();
        assert_eq!(iter.next().unwrap().number, 3);
        assert_eq!(iter.next().unwrap().number, 7);
        assert_eq!(iter.next().unwrap().number, 9);
        assert!(iter.next().is_none());
        assert_eq!(forges.numbers().as_slice(), &[3, 7, 9]);
        assert_eq!(forges.nth(1).unwrap().number, 7);
        assert!(forges.nth(3).is_none());
    }
}
