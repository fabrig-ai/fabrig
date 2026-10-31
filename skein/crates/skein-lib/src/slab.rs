//! Entity storage (lib.md, 3): slots and lists grow as
//! they fill, never past the admission limit for their entity kind.

#![expect(
    clippy::disallowed_types,
    reason = "a slab's slots and lists are Vecs grown exactly as they fill, never past its cap"
)]

use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::mem::size_of;

use crate::Id;
use crate::grow::growth;

/// Entities of one kind, named by `Id<T>`.
///
/// Inserting into a full slab hands the value back: that is the refusal.
/// Retiring an entity marks it; it stays reachable through its handle until
/// `reclaim`, which the loop calls once per iteration, so every entity present
/// when an iteration begins can be looked up until the iteration ends.
#[derive(Debug)]
pub struct Slab<T> {
    slots: Vec<Slot<T>>,
    /// Reclaimed vacant slots, the next one to fill last.
    free: Vec<u32>,
    /// Slots retired since the last reclaim.
    retired: Vec<u32>,
    capacity: u32,
    len: u32,
}

#[derive(Debug)]
enum Slot<T> {
    /// The next entity here gets `generation`.
    Vacant {
        generation: u32,
    },
    Occupied {
        generation: u32,
        retired: bool,
        value: T,
    },
    /// Its generation would wrap: never used again, so no handle is reused.
    Spent,
}

impl<T> Slab<T> {
    /// Starts without allocating; slots and lists grow as they fill, up to
    /// `capacity`, and keep their storage when reclaimed.
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Slab<T> {
        Slab { slots: Vec::new(), free: Vec::new(), retired: Vec::new(), capacity, len: 0 }
    }

    /// The heap a slab of `capacity` takes itself, its slots and its lists of
    /// free and retired slots, or `None` past a `u64`. What its entities own
    /// is theirs to count.
    #[must_use]
    pub fn price(capacity: u32) -> Option<u64> {
        let slot = u64::try_from(size_of::<Slot<T>>()).ok()?;
        let lists = u64::try_from(size_of::<u32>()).ok()?.checked_mul(2)?;
        u64::from(capacity).checked_mul(slot.checked_add(lists)?)
    }

    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Entities present, retired ones included until they are reclaimed.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether an insert would be refused.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.free.is_empty() && self.slots.len() == index(self.capacity)
    }

    /// Stores `value` and names it, or hands it back when the slab is full.
    pub fn insert(&mut self, value: T) -> Result<Id<T>, T> {
        let slot = match self.free.pop() {
            Some(slot) => slot,
            None => {
                if self.slots.len() == index(self.capacity) {
                    return Err(value);
                }
                grow(&mut self.slots, self.capacity);
                let slot = u32::try_from(self.slots.len()).expect("fewer slots than the capacity");
                self.slots.push(Slot::Vacant { generation: 0 });
                slot
            }
        };
        let entry = self.slots.get_mut(index(slot)).expect("free slots are in range");
        let generation = match entry {
            Slot::Vacant { generation } => *generation,
            Slot::Occupied { .. } | Slot::Spent => unreachable!("the free list holds vacant slots only"),
        };
        *entry = Slot::Occupied { generation, retired: false, value };
        self.len = self.len.checked_add(1).expect("no more entities than slots");
        Ok(Id::new(slot, generation))
    }

    /// The entity `id` names, or `None` if it has been reclaimed.
    #[must_use]
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        match self.slots.get(index(id.slot()))? {
            Slot::Occupied { generation, retired: _, value } => (*generation == id.generation()).then_some(value),
            Slot::Vacant { .. } | Slot::Spent => None,
        }
    }

    /// The entity `id` names, or `None` if it has been reclaimed.
    #[must_use]
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        match self.slots.get_mut(index(id.slot()))? {
            Slot::Occupied { generation, retired: _, value } => (*generation == id.generation()).then_some(value),
            Slot::Vacant { .. } | Slot::Spent => None,
        }
    }

    /// The entities present and not retired, each with its handle, in slot
    /// order: a walk over a kind's live entities, with no list of handles
    /// kept beside the slab. A step that changes an entity it finds ends
    /// the walk first, then looks the entity up by its handle.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { slots: self.slots.iter(), slot: 0 }
    }

    /// Marks a live entity for reclaiming. It stays reachable until
    /// `reclaim`. Retiring anything else is a bug.
    pub fn retire(&mut self, id: Id<T>) {
        let entry = self.slots.get_mut(index(id.slot())).expect("a handle names a slot of its slab");
        match entry {
            Slot::Occupied { generation, retired, value: _ } => {
                assert!(*generation == id.generation(), "only a live entity is retired");
                assert!(!*retired, "an entity is retired once");
                *retired = true;
            }
            Slot::Vacant { .. } | Slot::Spent => unreachable!("only a live entity is retired"),
        }
        grow(&mut self.retired, self.capacity);
        self.retired.push(id.slot());
    }

    /// Frees the slots of every retired entity: the reclaim point, once per
    /// iteration, after every step.
    pub fn reclaim(&mut self) {
        // Bounded: each slot is retired at most once between reclaims.
        while let Some(slot) = self.retired.pop() {
            let entry = self.slots.get_mut(index(slot)).expect("retired slots are in range");
            let next = match entry {
                Slot::Occupied { generation, retired: true, value: _ } => generation.checked_add(1),
                Slot::Occupied { retired: false, .. } | Slot::Vacant { .. } | Slot::Spent => {
                    unreachable!("the retired list holds retired entities only")
                }
            };
            *entry = match next {
                Some(generation) => {
                    grow(&mut self.free, self.capacity);
                    self.free.push(slot);
                    Slot::Vacant { generation }
                }
                None => Slot::Spent,
            };
            self.len = self.len.checked_sub(1).expect("a retired entity was counted");
        }
    }
}

/// A walk over a slab's live entities, in slot order: `Slab::iter`.
#[derive(Debug)]
pub struct Iter<'a, T> {
    slots: core::slice::Iter<'a, Slot<T>>,
    /// The slot the next one taken from `slots` is.
    slot: u32,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id<T>, &'a T);

    fn next(&mut self) -> Option<(Id<T>, &'a T)> {
        // Bounded: each turn takes one of the slab's slots.
        for entry in self.slots.by_ref() {
            let slot = self.slot;
            self.slot = slot.checked_add(1).expect("no more slots than the capacity");
            match entry {
                Slot::Occupied { generation, retired: false, value } => {
                    return Some((Id::new(slot, *generation), value));
                }
                Slot::Occupied { retired: true, .. } | Slot::Vacant { .. } | Slot::Spent => {}
            }
        }
        None
    }
}

impl<'a, T> IntoIterator for &'a Slab<T> {
    type Item = (Id<T>, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// Makes room for one more item in one of the slab's buffers, by lib's
/// growth rule.
fn grow<I>(items: &mut Vec<I>, capacity: u32) {
    let len = items.len();
    let needed = len.checked_add(1).expect("room for another item");
    items.reserve_exact(growth(len, items.capacity(), needed, capacity));
}

fn index(slot: u32) -> usize {
    usize::try_from(slot).expect("a u32 fits in a usize")
}

// lib.md, 5.1: slot identity and reuse order are state; capacity and len are not.
impl<T: Hash> Hash for Slab<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for slot in &self.slots {
            match slot {
                Slot::Vacant { generation } => {
                    state.write_u8(0);
                    generation.hash(state);
                }
                Slot::Occupied { generation, retired, value } => {
                    state.write_u8(1);
                    generation.hash(state);
                    retired.hash(state);
                    value.hash(state);
                }
                Slot::Spent => state.write_u8(2),
            }
        }
        self.free.len().hash(state);
        for slot in &self.free {
            slot.hash(state);
        }
        self.retired.len().hash(state);
        for slot in &self.retired {
            slot.hash(state);
        }
    }
}
