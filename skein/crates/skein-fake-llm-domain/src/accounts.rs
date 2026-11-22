//! Account strikes, bounded at configuration (testing/fakes/llm.md, 2.7).
use crate::Config;
use crate::api::{Error, Query, Strike};
use skein_lib::{Duration, List, Time, len_u64};

#[derive(Debug)]
struct Active {
    strike: Strike,
    order: u64,
    start: Time,
    window: u64,
    charged: u64,
}

#[derive(Debug)]
pub(crate) struct Accounts {
    slots: List<List<Active>>,
    unserved: u32,
    bytes: u64,
    order: u64,
}

impl Accounts {
    pub(crate) fn price(config: &Config) -> Option<u64> {
        let slots = config.unserved.checked_add(3)?;
        List::<List<Active>>::price(config.accounts)?
            .checked_add(List::<Active>::price(slots)?.checked_mul(u64::from(config.accounts))?)
    }

    pub(crate) fn new(config: &Config, bytes: u64) -> Self {
        let mut accounts = List::allocated(config.accounts);
        for _ in 0..config.accounts {
            accounts
                .push(List::allocated(config.unserved.checked_add(3).expect("admitted slots")))
                .expect("reserved accounts");
        }
        Self { slots: accounts, unserved: config.unserved, bytes, order: 0 }
    }

    pub(crate) fn strike(&mut self, credential: u32, strike: Strike, now: Time) -> Result<(), Error> {
        let account = self.slots.get_mut(credential).ok_or(Error::ContextTooLong)?;
        let mut models = 0_u32;
        for active in account.as_slice() {
            if same(&active.strike, &strike) {
                return Ok(());
            }
            match active.strike {
                Strike::Unserved { .. } => models = models.checked_add(1).expect("bounded models"),
                Strike::Spent | Strike::SignedOut | Strike::Window { .. } => {}
            }
        }
        let bytes = match &strike {
            Strike::Unserved { model } => {
                if models == self.unserved {
                    return Err(Error::ContextTooLong);
                }
                len_u64(model.len())
            }
            Strike::Window { length, .. } => {
                if *length == Duration::ZERO {
                    return Err(Error::InvalidRequest);
                }
                0
            }
            Strike::Spent | Strike::SignedOut => 0,
        };
        let remaining = self.bytes.checked_sub(bytes).ok_or(Error::ContextTooLong)?;
        let order = self.order.checked_add(1).ok_or(Error::ContextTooLong)?;
        account.push(Active { strike, order, start: now, window: 0, charged: 0 }).expect("reserved strikes");
        self.bytes = remaining;
        self.order = order;
        Ok(())
    }

    pub(crate) fn clear(&mut self, credential: u32, strike: &Strike) {
        let Some(account) = self.slots.get_mut(credential) else { return };
        let mut found = None;
        for (index, active) in account.iter().enumerate() {
            if same(&active.strike, strike) {
                found = Some(u32::try_from(index).expect("bounded strikes"));
                break;
            }
        }
        if let Some(index) = found {
            let active = account.swap_remove(index).expect("found strike");
            match active.strike {
                Strike::Unserved { model } => {
                    self.bytes = self.bytes.checked_add(len_u64(model.len())).expect("returned model bytes")
                }
                Strike::Spent | Strike::SignedOut | Strike::Window { .. } => {}
            }
        }
    }

    pub(crate) fn check(&mut self, query: &Query, now: Time) -> Option<Error> {
        let account = self.slots.get_mut(query.credential?)?;
        let mut previous = 0;
        for _ in 0..account.len() {
            let mut next = None;
            let mut order = u64::MAX;
            for (index, active) in account.iter().enumerate() {
                if active.order > previous && active.order <= order {
                    next = Some(u32::try_from(index).expect("bounded strikes"));
                    order = active.order;
                }
            }
            let active = account.get_mut(next.expect("next ordered strike")).expect("found strike");
            previous = active.order;
            match &active.strike {
                Strike::Spent => return Some(Error::Spent),
                Strike::SignedOut => return Some(Error::SignedOut),
                Strike::Unserved { model } => {
                    if *model == query.model {
                        return Some(Error::Unserved);
                    }
                }
                Strike::Window { tokens, length } => {
                    let elapsed = now.saturating_since(active.start).as_nanos();
                    let window = elapsed.checked_div(length.as_nanos()).expect("nonzero window");
                    if window != active.window {
                        active.window = window;
                        active.charged = 0;
                    }
                    let input = crate::cache::tokens(query);
                    if active.charged != 0 && input > tokens.saturating_sub(active.charged) {
                        let within = elapsed.checked_rem(length.as_nanos()).expect("nonzero window");
                        return Some(Error::Window {
                            reset: Duration::from_nanos(length.as_nanos().checked_sub(within).expect("within window")),
                            limit: *tokens,
                            remaining: tokens.saturating_sub(active.charged),
                        });
                    }
                    active.charged = active.charged.saturating_add(input);
                }
            }
        }
        None
    }
}

fn same(left: &Strike, right: &Strike) -> bool {
    match left {
        Strike::Spent => *right == Strike::Spent,
        Strike::SignedOut => *right == Strike::SignedOut,
        Strike::Unserved { model } => match right {
            Strike::Unserved { model: other } => model == other,
            Strike::Spent | Strike::SignedOut | Strike::Window { .. } => false,
        },
        Strike::Window { .. } => match right {
            Strike::Window { .. } => true,
            Strike::Spent | Strike::SignedOut | Strike::Unserved { .. } => false,
        },
    }
}
