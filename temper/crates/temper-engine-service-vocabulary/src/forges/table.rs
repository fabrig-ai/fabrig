//! The forges table shared by readers and the component (protocol/services.md, 4.2).
use crate::settings::Forge;
use skein_lib::{List, Map};

#[derive(Debug)]
pub struct Forges {
    forges: Map<u16, Forge>,
    // Record bots distinguish a login rewrite from an author change.
    bots: Map<u16, temper_engine_domain_forge::Bot>,
}

impl Forges {
    /// Record bots the component reads and updates when keeping a forge record.
    pub fn bots(&mut self) -> &mut Map<u16, temper_engine_domain_forge::Bot> {
        &mut self.bots
    }

    #[must_use]
    pub const fn with_capacity(capacity: u32) -> Self {
        Self { forges: Map::with_capacity(capacity), bots: Map::with_capacity(capacity) }
    }

    #[must_use]
    pub fn get(&self, forge: u16) -> Option<&Forge> {
        self.forges.get(&forge)
    }

    /// The kept bot login for the component's public forge description.
    #[must_use]
    pub fn bot_login(&self, forge: u16) -> Option<&[u8]> {
        match self.bots.get(&forge) {
            Some(bot) => Some(&bot.login),
            None => None,
        }
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
        self.bots.remove(&forge);
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
