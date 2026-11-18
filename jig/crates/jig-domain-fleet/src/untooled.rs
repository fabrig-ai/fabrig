//! Task-local program places, until their channel or login changes (domain/engine/fleet.md).
use crate::{Domain, SourceName, channel::Channel};
use skein_lib::{List, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Untooled {
    pub channel: Token,
    /// The worker's configured position, not its number.
    pub worker: u32,
    pub source: SourceName,
}

impl Domain {
    #[must_use]
    pub fn untooled(&self, run: Token) -> Option<Untooled> {
        let mut latest = None;
        let mut order = 0;
        for (_, id) in &self.tokens {
            let host = self.channels.get(*id).expect("current channel");
            let Some(worker) = host.worker else { continue };
            for ((task, source), recorded) in &host.untooled {
                if *task == run && (latest.is_none() || *recorded > order) {
                    latest = Some(Untooled { channel: host.token, worker, source: *source });
                    order = *recorded;
                }
            }
        }
        latest
    }

    #[must_use]
    pub fn untooled_on(&self, run: Token, channel: Token, source: &SourceName) -> bool {
        let Some(id) = self.tokens.get(&channel) else { return false };
        self.channels.get(*id).expect("current channel").untooled.contains_key(&(run, *source))
    }
}

pub(crate) fn record(domain: &mut Domain, run: Token, channel: Token, source: SourceName) {
    let Some(id) = domain.tokens.get(&channel) else { return };
    let host = domain.channels.get_mut(*id).expect("current channel");
    if host.worker.is_none() {
        return;
    }
    let order = domain.untooled_order;
    domain.untooled_order = order.checked_add(1).expect("a u64 counts program faults");
    host.untooled.insert((run, source), order).expect("uncapped task-local places");
}

pub(crate) fn logins(host: &mut Channel) {
    let mut absent = List::with_capacity(host.untooled.len());
    for ((run, name), _) in &host.untooled {
        let mut usable = false;
        for login in host.sources.logins() {
            if login.name == *name {
                usable = login.state == crate::LoginState::Usable;
            }
        }
        if !usable {
            absent.push((*run, *name)).expect("one key per fault");
        }
    }
    for key in absent.into_boxed() {
        host.untooled.remove(&key);
    }
}

pub(crate) fn clear(host: &mut Channel, source: SourceName) {
    let mut keys = List::with_capacity(host.untooled.len());
    for ((run, name), _) in &host.untooled {
        if *name == source {
            keys.push((*run, *name)).expect("one key per fault");
        }
    }
    for key in keys.into_boxed() {
        host.untooled.remove(&key);
    }
}
