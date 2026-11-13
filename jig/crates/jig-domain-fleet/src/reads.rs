//! A read of a provider's list, held until a worker can take it (domain/engine/settings.md, 12).
//! Keeps the queue and which channel holds each. It never holds a value: a grant is a name.

use crate::{Domain, Grant, HostKind, Request};
use jig_domain_link::ListOutcome;
use skein_lib::{List, Queue, Time, Token};

#[derive(Hash, Debug)]
pub(crate) struct Reads {
    held: List<Held>,
    head: i64,
    tail: i64,
}

#[derive(Hash, Debug)]
struct Held {
    read: u64,
    endpoint: u32,
    grant: Grant,
    lapses: Time,
    state: State,
}

#[derive(Hash, Debug)]
enum State {
    Queued { order: i64 },
    InFlight { channel: Token },
}

impl Reads {
    pub(crate) fn new(capacity: u32) -> Self {
        Self { held: List::with_capacity(capacity), head: 0, tail: 0 }
    }

    pub(crate) fn push(&mut self, read: u64, endpoint: u32, grant: Grant, now: Time) {
        let order = self.tail;
        self.tail = self.tail.checked_add(1).expect("a signed 64-bit count orders queued reads");
        self.held
            .push(Held {
                read,
                endpoint,
                grant,
                lapses: now.saturating_add(grant.valid),
                state: State::Queued { order },
            })
            .expect("one read per account fits the accounts bound");
    }

    pub(crate) fn renew(&mut self, grant: Grant, now: Time) {
        for held in self.held.iter_mut() {
            match held.state {
                State::InFlight { .. } => continue,
                State::Queued { .. } => {}
            }
            if held.grant.account == grant.account
                && held.grant.form.table() == grant.form.table()
                && held.grant.generation < grant.generation
            {
                held.grant = grant;
                held.lapses = now.saturating_add(grant.valid);
            }
        }
    }

    pub(crate) fn lost(&mut self, channel: Token) {
        for held in self.held.iter_mut() {
            match held.state {
                State::InFlight { channel: owner } if owner == channel => {
                    self.head = self.head.checked_sub(1).expect("a signed 64-bit count orders retried reads");
                    held.state = State::Queued { order: self.head };
                }
                State::InFlight { .. } | State::Queued { .. } => {}
            }
        }
    }

    pub(crate) fn answered(&mut self, channel: Token, read: u64, outcome: ListOutcome, out: &mut Queue<Request>) {
        for index in 0..self.held.len() {
            let held = self.held.get(index).expect("held read index");
            let owner = match held.state {
                State::InFlight { channel } => Some(channel),
                State::Queued { .. } => None,
            };
            if held.read == read && owner == Some(channel) {
                self.held.swap_remove(index).expect("answered read is held");
                out.push(Request::ListRead { read, outcome });
                return;
            }
        }
    }

    fn on(&self, channel: Token) -> bool {
        for held in &self.held {
            match held.state {
                State::InFlight { channel: owner } if owner == channel => return true,
                State::InFlight { .. } | State::Queued { .. } => {}
            }
        }
        false
    }
}

/// Reads need no free slot, but require a ready worker declaring the endpoint.
fn choose(domain: &Domain, endpoint: u32) -> Option<Token> {
    let mut best = None;
    for (_, &id) in &domain.tokens {
        let channel = domain.channels.get(id).expect("connected channel");
        if channel.kind != HostKind::Worker
            || !channel.sources.ready()
            || channel.draining
            || channel.shutting_down
            || channel.mismatched
            || domain.reads.on(channel.token)
        {
            continue;
        }
        let mut serves = false;
        for bound in &channel.evidence.endpoints {
            serves |= bound.endpoint == endpoint;
        }
        if !serves {
            continue;
        }
        let free = channel.slots.saturating_sub(channel.hosts.len());
        match best {
            Some((_, most)) if most >= free => {}
            Some(_) | None => best = Some((channel.token, free)),
        }
    }
    let (channel, _) = best?;
    Some(channel)
}

fn next(domain: &Domain) -> Option<(u32, Token)> {
    let mut next = None;
    for index in 0..domain.reads.held.len() {
        let held = domain.reads.held.get(index).expect("held read index");
        let order = match held.state {
            State::Queued { order } => order,
            State::InFlight { .. } => continue,
        };
        if let Some((_, _, first)) = next
            && first <= order
        {
            continue;
        }
        if let Some(channel) = choose(domain, held.endpoint) {
            next = Some((index, channel, order));
        }
    }
    let (index, channel, _) = next?;
    Some((index, channel))
}

pub(crate) fn ready(domain: &Domain) -> bool {
    next(domain).is_some()
}

pub(crate) fn resume(domain: &mut Domain, now: Time, out: &mut Queue<Request>) {
    let Some((index, channel)) = next(domain) else { return };
    let held = domain.reads.held.get_mut(index).expect("queued read");
    held.state = State::InFlight { channel };
    let grant = Grant { valid: held.lapses.saturating_since(now), ..held.grant };
    out.push(Request::ReadList { channel, read: held.read, endpoint: held.endpoint, grant });
}
