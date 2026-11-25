//! Run and step accounts on the driving world's clock (skein/docs/design/ui/web/testing.md, 2).
use skein_lib::{Duration, Time};
use skein_world::deadline::ProcessSampler;
use skein_world::{Deadline, DeadlineOrigin, Expired, ProcessWatch};

pub(super) enum Account {
    Native(Deadline),
    Hosted { what: &'static str, start: Time, bound: Duration },
}

pub(super) enum Expiry {
    Native(Expired),
    Hosted { what: &'static str, bound: Duration, waited: Duration },
}

/// The accounts begun together in one dispatch. A later dispatch always
/// starts a new origin, even if it is still playing the same step.
pub(super) struct Arming {
    native: bool,
    now: Time,
    roots: [Option<u32>; 2],
    origin: Option<DeadlineOrigin>,
    common: Option<ProcessWatch>,
    commands: Vec<(u32, ProcessWatch)>,
}

impl Arming {
    pub fn new(native: bool, now: Time, server: Option<u32>, browser: Option<u32>) -> Self {
        Self { native, now, roots: [server, browser], origin: None, common: None, commands: Vec::new() }
    }

    pub fn arm(
        &mut self,
        what: &'static str,
        bound: Duration,
        command: Option<u32>,
        sampler: &mut ProcessSampler,
    ) -> Account {
        if !self.native {
            return Account::arm(false, what, bound, self.now);
        }
        let origin = self.origin.get_or_insert_with(DeadlineOrigin::capture);
        let mut deadline = origin.arm(what, bound);
        if self.roots.iter().any(Option::is_some) {
            let watch = self.common.get_or_insert_with(|| {
                let roots = self.roots.into_iter().flatten().collect::<Vec<_>>();
                sampler.capture(&roots)
            });
            deadline.watch_snapshot(watch);
        }
        if let Some(pid) = command {
            let watch = if let Some((_, watch)) = self.commands.iter().find(|(root, _)| *root == pid) {
                watch
            } else {
                self.commands.push((pid, ProcessWatch::capture(&[pid])));
                &self.commands.last().expect("command watch just added").1
            };
            deadline.watch_snapshot(watch);
        }
        Account::Native(deadline)
    }
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

    pub fn watch_snapshot(&mut self, watch: &ProcessWatch) {
        if let Self::Native(deadline) = self {
            deadline.watch_snapshot(watch);
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
