//! The two newest lent generations per account (protocol/llm.md, 5.3).
//! A value is a credential with its scheme (protocol/channel.md, 8).
use skein_lib::{List, Map, Secret, Time};

/// A grant's credential, put in its taker's form only when handed over.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Value {
    pub token: Secret,
    pub scheme: Scheme,
}
/// How the credential is presented (protocol/llm.md, 5.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
}
#[derive(Debug)]
pub struct Table {
    accounts: Map<u32, Generations>,
    token_bytes: u32,
}
#[derive(Debug)]
struct Generations {
    newest: Option<Entry>,
    older: Option<Entry>,
}
#[derive(Debug)]
struct Entry {
    generation: u64,
    value: Value,
    expires: Time,
}
/// An account or value beyond the configured bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit;

impl Table {
    #[must_use]
    pub fn new(accounts: u32, token_bytes: u32) -> Table {
        Table { accounts: Map::with_capacity(accounts), token_bytes }
    }
    pub fn granted(&mut self, account: u32, generation: u64, value: Value, expires: Time) -> Result<(), Unfit> {
        if value.token.len() > skein_lib::length::usize_of(self.token_bytes) {
            return Err(Unfit);
        }
        if !self.accounts.contains_key(&account) {
            self.accounts.insert(account, Generations { newest: None, older: None }).or(Err(Unfit))?;
        }
        let slots = self.accounts.get_mut(&account).expect("admitted account");
        let entry = Entry { generation, value, expires };
        match &slots.newest {
            Some(newest) if generation < newest.generation => match &slots.older {
                Some(older) if generation < older.generation => (),
                Some(_) | None => slots.older = Some(entry),
            },
            Some(newest) if generation == newest.generation => slots.newest = Some(entry),
            Some(_) | None => {
                slots.older = slots.newest.take();
                slots.newest = Some(entry);
            }
        }
        Ok(())
    }
    #[must_use]
    pub fn value(&self, account: u32, generation: u64, now: Time) -> Option<&Value> {
        let slots = self.accounts.get(&account)?;
        for entry in [&slots.newest, &slots.older].into_iter().flatten() {
            if entry.generation == generation && now < entry.expires {
                return Some(&entry.value);
            }
        }
        None
    }
    /// Forget one unheld key generation on a worker; bearers keep their expiry (channel.md, 8).
    pub fn release_key(&mut self, account: u32, generation: u64) {
        let Some(slots) = self.accounts.get_mut(&account) else { return };
        release_key(&mut slots.newest, generation);
        release_key(&mut slots.older, generation);
        if slots.newest.is_none() {
            slots.newest = slots.older.take();
        }
        if slots.newest.is_none() {
            let _removed = self.accounts.remove(&account);
        }
    }
    /// Names of key generations whose worker lifetime follows attempts and reads (channel.md, 8).
    #[must_use]
    pub fn keys(&self) -> List<(u32, u64)> {
        let mut keys = List::with_capacity(self.entries());
        for (account, slots) in &self.accounts {
            for entry in [&slots.newest, &slots.older].into_iter().flatten() {
                match entry.value.scheme {
                    Scheme::Key => keys.push((*account, entry.generation)).expect("one name per retained key"),
                    Scheme::Bearer => {}
                }
            }
        }
        keys
    }

    /// Retained credential generations, including those awaiting expiry.
    #[must_use]
    pub fn entries(&self) -> u32 {
        let mut count = 0_u32;
        for (_, slots) in &self.accounts {
            for _ in [&slots.newest, &slots.older].into_iter().flatten() {
                count = count.checked_add(1).expect("two entries per bounded account");
            }
        }
        count
    }

    /// Frees a removed LLM account source's generations and its table place.
    pub fn remove(&mut self, account: u32) {
        let _removed = self.accounts.remove(&account);
    }
    pub fn expire(&mut self, now: Time) {
        let mut after = 0;
        for _ in 0..self.accounts.len() {
            let Some((&number, _)) = self.accounts.range(after..).next() else { break };
            let slots = self.accounts.get_mut(&number).expect("named account");
            expire(&mut slots.newest, now);
            expire(&mut slots.older, now);
            if slots.newest.is_none() {
                slots.newest = slots.older.take();
            }
            let Some(next) = number.checked_add(1) else { break };
            after = next;
        }
    }
    #[must_use]
    pub fn next_expiry(&self) -> Option<Time> {
        let mut earliest: Option<Time> = None;
        for (_, slots) in &self.accounts {
            for entry in [&slots.newest, &slots.older].into_iter().flatten() {
                earliest = Some(match earliest {
                    Some(time) => time.min(entry.expires),
                    None => entry.expires,
                });
            }
        }
        earliest
    }
}
fn expire(slot: &mut Option<Entry>, now: Time) {
    if let Some(entry) = slot
        && now >= entry.expires
    {
        *slot = None;
    }
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub const fn valid(accounts: u32, token_bytes: u32) -> bool {
    accounts > 0 && token_bytes > 0
}

fn release_key(slot: &mut Option<Entry>, generation: u64) {
    if let Some(entry) = slot
        && entry.generation == generation
    {
        match entry.value.scheme {
            Scheme::Key => *slot = None,
            Scheme::Bearer => {}
        }
    }
}
