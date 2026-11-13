//! The map's bounded, in-memory census (skein/docs/design/data/kv.md, 2.2).

#![expect(clippy::disallowed_types, reason = "sorted census tables grow exactly as filled, under the census constants")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::Op;

pub const CENSUS_PREFIX: usize = 8;
/// Named keyspaces; the empty overflow prefix is beside these.
pub const CENSUS_KEYSPACES: usize = 128;
/// Includes the no-head entry: at most seven heads carry a u16.
pub const CENSUS_HEADS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct CensusRule {
    pub prefix: [u8; 256],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Census {
    since: u64,
    keyspaces: Vec<Keyspace>,
    // Bound overflow only: the empty key's own row does not delay admission.
    folded: u64,
}

#[expect(clippy::partial_pub_fields, reason = "the census contract exposes counts as fields and tables through slices")]
#[derive(Debug, PartialEq, Eq)]
pub struct Keyspace {
    prefix: Box<[u8]>,
    pub rows: u64,
    pub keys: u64,
    pub values: u64,
    pub puts: u64,
    pub erases: u64,
    pub written: u64,
    heads: Vec<Head>,
    // Bound overflow only: short values share no-head but do not delay admission.
    folded: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Head {
    pub head: Option<u16>,
    pub rows: u64,
}

impl Census {
    pub(crate) fn new() -> Census {
        Census { since: 1, keyspaces: Vec::new(), folded: 0 }
    }

    #[must_use]
    pub const fn since(&self) -> u64 {
        self.since
    }

    #[must_use]
    pub fn keyspaces(&self) -> &[Keyspace] {
        &self.keyspaces
    }

    pub(crate) fn restart(&mut self, since: u64) {
        self.since = since;
        for index in (0..self.keyspaces.len()).rev() {
            let space = self.keyspaces.get_mut(index).expect("index is inside the table");
            space.puts = 0;
            space.erases = 0;
            space.written = 0;
            if space.rows == 0 {
                self.keyspaces.remove(index);
            }
        }
    }

    pub(crate) fn apply(&mut self, rule: &CensusRule, op: &Op, old: Option<&[u8]>, writes: bool) {
        let key = op.key();
        let prefix = match key.first() {
            Some(first) => {
                let length = usize::from(*rule.prefix.get(usize::from(*first)).expect("a byte indexes the rule"));
                let length = length.clamp(1, CENSUS_PREFIX).min(key.len());
                key.get(..length).expect("prefix fits the key")
            }
            None => key,
        };
        let index = self.space(prefix);
        let space = self.keyspaces.get_mut(index).expect("space found or inserted");
        let folded = !prefix.is_empty() && space.prefix.is_empty();
        if let Some(value) = old {
            if folded {
                self.folded = self.folded.checked_sub(1).expect("the old folded row is counted");
            }
            space.rows = space.rows.checked_sub(1).expect("the old row is counted");
            space.keys = space.keys.checked_sub(skein_lib::len_u64(key.len())).expect("the old key is counted");
            space.values = space.values.checked_sub(skein_lib::len_u64(value.len())).expect("the old value is counted");
            space.remove_head(value);
        }
        match op {
            Op::Put { value, .. } => {
                if folded {
                    self.folded = self.folded.checked_add(1).expect("folded rows fit the map");
                }
                space.rows = space.rows.checked_add(1).expect("map rows fit u64");
                space.keys = space.keys.checked_add(skein_lib::len_u64(key.len())).expect("map keys fit u64");
                space.values = space.values.checked_add(skein_lib::len_u64(value.len())).expect("map values fit u64");
                space.add_head(value);
                if writes {
                    space.puts = space.puts.checked_add(1).expect("puts fit the commit period");
                    space.written =
                        space.written.checked_add(skein_lib::len_u64(value.len())).expect("written bytes fit u64");
                }
            }
            Op::Erase { .. } => {
                if writes {
                    space.erases = space.erases.checked_add(1).expect("erases fit the commit period");
                }
            }
        }
        if writes {
            space.written = space.written.checked_add(skein_lib::len_u64(key.len())).expect("written bytes fit u64");
        }
    }

    fn space(&mut self, prefix: &[u8]) -> usize {
        let index = match self.find(prefix) {
            Ok(index) => return index,
            Err(index) => index,
        };
        let overflow = match self.keyspaces.first() {
            Some(space) => space.prefix.is_empty(),
            None => false,
        };
        let named = self.keyspaces.len().checked_sub(usize::from(overflow)).expect("overflow is in the table");
        if !prefix.is_empty() && (named == CENSUS_KEYSPACES || self.folded != 0) {
            if overflow {
                return 0;
            }
            self.keyspaces.reserve_exact(1);
            self.keyspaces.insert(0, Keyspace::new(&[]));
            return 0;
        }
        self.keyspaces.reserve_exact(1);
        self.keyspaces.insert(index, Keyspace::new(prefix));
        index
    }

    fn find(&self, prefix: &[u8]) -> Result<usize, usize> {
        let mut low = 0;
        let mut high = self.keyspaces.len();
        // At most 129 entries, so eight comparisons reach an entry or an insertion point.
        for _ in 0..8_u8 {
            if low == high {
                break;
            }
            let mid =
                low.checked_add(high.checked_sub(low).expect("ordered search bounds") / 2).expect("table index fits");
            let space = self.keyspaces.get(mid).expect("mid is inside the search range");
            match space.prefix().cmp(prefix) {
                Ordering::Less => low = mid.checked_add(1).expect("table index fits"),
                Ordering::Equal => return Ok(mid),
                Ordering::Greater => high = mid,
            }
        }
        Err(low)
    }
}

impl Keyspace {
    fn new(prefix: &[u8]) -> Keyspace {
        Keyspace {
            prefix: Box::from(prefix),
            rows: 0,
            keys: 0,
            values: 0,
            puts: 0,
            erases: 0,
            written: 0,
            heads: Vec::new(),
            folded: 0,
        }
    }

    #[must_use]
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    #[must_use]
    pub fn heads(&self) -> &[Head] {
        &self.heads
    }

    fn find_head(&self, head: Option<u16>) -> Result<usize, usize> {
        let mut low = 0;
        let mut high = self.heads.len();
        for _ in 0..4_u8 {
            if low == high {
                break;
            }
            let mid =
                low.checked_add(high.checked_sub(low).expect("ordered search bounds") / 2).expect("head index fits");
            match self.heads.get(mid).expect("mid is inside the search range").head.cmp(&head) {
                Ordering::Less => low = mid.checked_add(1).expect("head index fits"),
                Ordering::Equal => return Ok(mid),
                Ordering::Greater => high = mid,
            }
        }
        Err(low)
    }

    fn add_head(&mut self, value: &[u8]) {
        let head = value_head(value);
        let index = match self.find_head(head) {
            Ok(index) => index,
            Err(index) => {
                let none = match self.heads.first() {
                    Some(entry) => entry.head.is_none(),
                    None => false,
                };
                let named = self.heads.len().checked_sub(usize::from(none)).expect("no-head is in the table");
                let named_limit = CENSUS_HEADS.checked_sub(1).expect("room for no-head");
                if head.is_some() && (named == named_limit || self.folded != 0) {
                    self.folded = self.folded.checked_add(1).expect("folded heads fit the map");
                    if !none {
                        self.heads.reserve_exact(1);
                        self.heads.insert(0, Head { head: None, rows: 0 });
                    }
                    0
                } else {
                    self.heads.reserve_exact(1);
                    self.heads.insert(index, Head { head, rows: 0 });
                    index
                }
            }
        };
        let entry = self.heads.get_mut(index).expect("head found or inserted");
        entry.rows = entry.rows.checked_add(1).expect("head rows fit the map");
    }

    fn remove_head(&mut self, value: &[u8]) {
        let index = match self.find_head(value_head(value)) {
            Ok(index) => index,
            Err(_) => {
                self.folded = self.folded.checked_sub(1).expect("the old folded head is counted");
                0
            }
        };
        let entry = self.heads.get_mut(index).expect("the old head is counted, or folded under no head");
        entry.rows = entry.rows.checked_sub(1).expect("the old head's row is counted");
        if entry.rows == 0 {
            self.heads.remove(index);
        }
    }
}

fn value_head(value: &[u8]) -> Option<u16> {
    let first = *value.first()?;
    let second = *value.get(1)?;
    Some(u16::from_be_bytes([first, second]))
}
