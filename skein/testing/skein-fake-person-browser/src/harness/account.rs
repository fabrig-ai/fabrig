//! Run and step accounts on the driving world's clock (skein/docs/design/ui/web/testing.md, 2).
use skein_lib::{Duration, Time};
use skein_world::{Deadline, Expired};

pub(super) enum Account {
    Native(Deadline),
    Hosted { what: &'static str, start: Time, bound: Duration },
}

pub(super) enum Expiry {
    Native(Expired),
    Hosted { what: &'static str, bound: Duration, waited: Duration },
}

impl Account {
    pub fn arm(native: bool, what: &'static str, bound: Duration, now: Time) -> Self {
        if native { Self::Native(Deadline::arm(what, bound)) } else { Self::Hosted { what, start: now, bound } }
    }

    pub fn until(&self) -> Time {
        match self {
            Self::Native(deadline) => deadline.until(),
            Self::Hosted { start, bound, .. } => start.saturating_add(*bound),
        }
    }

    pub fn watch_process(&mut self, pid: u32) {
        if let Self::Native(deadline) = self {
            deadline.watch_process(pid);
        }
    }

    pub fn expired(&self, now: Time) -> Expiry {
        match self {
            Self::Native(deadline) => Expiry::Native(deadline.expired()),
            Self::Hosted { what, start, bound } => {
                Expiry::Hosted { what, bound: *bound, waited: now.saturating_since(*start) }
            }
        }
    }

    pub fn expired_for(&self, what: &'static str, bound: Duration, now: Time) -> Expiry {
        match self {
            Self::Native(deadline) => Expiry::Native(deadline.expired_for(what, bound)),
            Self::Hosted { start, .. } => Expiry::Hosted { what, bound, waited: now.saturating_since(*start) },
        }
    }
}
