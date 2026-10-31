//! Bounded credential generations (protocol/channel.md, section 6).
use crate::{Error, GrantValue};
use skein_lib::Map;

/// The last two credential values per configured account.
#[derive(Debug)]
pub(crate) struct Credentials {
    values: Map<(u32, u64), GrantValue>,
}

impl Credentials {
    pub(crate) fn new(capacity: u32) -> Credentials {
        Credentials { values: Map::with_capacity(capacity) }
    }

    pub(crate) fn value(&self, account: u32, generation: u64) -> Option<&GrantValue> {
        self.values.get(&(account, generation))
    }

    pub(crate) fn keep(&mut self, account: u32, generation: u64, value: GrantValue) -> Result<(), Error> {
        if generation == 0 {
            return Err(Error::Grants);
        }
        let mut oldest = None;
        let mut newest = 0_u64;
        let mut count = 0_u32;
        for ((held_account, held_generation), _) in &self.values {
            if *held_account == account {
                count = count.checked_add(1).ok_or(Error::Grants)?;
                if oldest.is_none() {
                    oldest = Some(*held_generation);
                }
                newest = newest.max(*held_generation);
            }
        }
        if generation <= newest && count > 0 {
            return Err(Error::Grants);
        }
        if count >= 2 {
            let oldest = oldest.ok_or(Error::Grants)?;
            self.values.remove(&(account, oldest));
        }
        if self.values.insert((account, generation), value).is_err() {
            return Err(Error::Grants);
        }
        Ok(())
    }
}
