//! Call and judge deadlines (skein/docs/foundation/testing-strategy.md, 2.8 and 2.10).
use skein_lib::{Duration, Time};
use skein_world::Expired;

#[derive(Debug)]
pub(crate) enum Deadline {
    Real(skein_world::Deadline),
    #[cfg(test)]
    Scripted {
        what: &'static str,
        after: Duration,
        start: Time,
    },
}

impl Deadline {
    pub(crate) fn arm(what: &'static str, after: Duration, _now: Time) -> Self {
        Self::Real(skein_world::Deadline::arm(what, after))
    }

    #[cfg(test)]
    pub(crate) const fn scripted(what: &'static str, after: Duration, start: Time) -> Self {
        Self::Scripted { what, after, start }
    }

    pub(crate) fn until(&self) -> Time {
        match self {
            Self::Real(deadline) => deadline.until(),
            #[cfg(test)]
            Self::Scripted { start, after, .. } => start.saturating_add(*after),
        }
    }

    pub(crate) fn expiration(&self, now: Time) -> Option<Expired> {
        #[cfg(not(test))]
        let _ = now;
        match self {
            Self::Real(deadline) => deadline.passed().then(|| deadline.expired()),
            #[cfg(test)]
            Self::Scripted { what, after, start } => (now >= self.until()).then(|| {
                Expired::Failed(skein_world::Figures {
                    what,
                    patience: *after,
                    elapsed: now.saturating_since(*start),
                    thread_waited: None,
                    thread_cpu: None,
                    tree_waited: None,
                })
            }),
        }
    }
}
