//! The logins' table both dialects keep (forge.md, 4.4): bounded, its oldest use evicted.
use alloc::boxed::Box;
use skein_lib::Map;

#[derive(Debug)]
pub struct UserEntry {
    pub(crate) login: Box<[u8]>,
    pub(crate) used: u64,
}
/// Logins by forge and id. The owner's `Tables` gives each use its clock.
#[derive(Debug)]
pub struct Users {
    entries: Map<(u16, u64), UserEntry>,
}
impl Users {
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { entries: Map::with_capacity(capacity) }
    }
    #[must_use]
    pub fn get(&mut self, forge: u16, id: u64, used: u64) -> Option<&[u8]> {
        let entry = self.entries.get_mut(&(forge, id))?;
        entry.used = used;
        Some(&entry.login)
    }
    /// A zero capacity keeps nothing.
    pub fn keep(&mut self, forge: u16, id: u64, login: Box<[u8]>, used: u64) {
        if self.entries.capacity() == 0 {
            return;
        }
        let key = (forge, id);
        if !self.entries.contains_key(&key) && self.entries.len() == self.entries.capacity() {
            let mut oldest = None;
            for (key, entry) in &self.entries {
                let order = (entry.used, *key);
                if oldest.is_none() || Some(order) < oldest {
                    oldest = Some(order);
                }
            }
            if let Some((_, key)) = oldest {
                drop(self.entries.remove(&key));
            }
        }
        drop(self.entries.insert(key, UserEntry { login, used }));
    }
}
