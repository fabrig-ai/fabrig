//! Opening the store before a builder call (hearth/docs/design/domain/store.md, 8.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod enter;
mod index;
pub mod layers;
pub mod limits;
pub mod resolutions;
mod scan;
mod staging;
mod start;
pub mod vocabulary;
use limits::Limits;
use skein_lib::{Deadlines, Env, Queue, Time};
use vocabulary::{Event, Output, Request};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Timer {
    Retry,
    Said,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Phase {
    Closed,
    Locking { said: bool },
    Recording,
    Writing,
    Scanning,
    Ready,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Standing {
    Usable,
    Declining(vocabulary::Decline),
}
#[derive(Debug, Hash)]
#[expect(clippy::large_enum_variant, reason = "the chain stays inline without allocating for each request")]
enum Flight {
    Idle,
    Finding,
    Writing(hearth_lib::encode::Digest),
    Entering(enter::Entering),
    Chain(staging::Working),
}
impl Flight {
    fn chain(&self) -> Option<&staging::Working> {
        match self {
            Self::Chain(working) => Some(working),
            Self::Idle | Self::Finding | Self::Writing(_) | Self::Entering(_) => None,
        }
    }
}
#[derive(Debug, Hash)]
pub struct Store {
    phase: Phase,
    uid: u32,
    timers: Deadlines<Timer>,
    scan: scan::Scan,
    standing: Standing,
    flight: Flight,
    staged: Option<staging::Staged>,
}
impl Store {
    #[must_use]
    pub fn new(limits: &Limits, uid: u32) -> Self {
        Self {
            phase: Phase::Closed,
            uid,
            timers: Deadlines::with_capacity(2),
            scan: scan::Scan::new(limits),
            standing: Standing::Declining(vocabulary::Decline::Unusable { step: vocabulary::Step::List }),
            flight: Flight::Idle,
            staged: None,
        }
    }
}
pub fn step(store: &mut Store, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    if let Event::Want { key } = ev {
        layers::want(store, key, out);
    } else if let Event::Stage { key, parent } = ev {
        staging::stage(store, env, key, parent, out);
    } else if let Event::Seal { key, parent, inputs } = ev {
        staging::seal(store, env, key, parent, inputs, out);
    } else if let Event::Discard { key } = ev {
        staging::discard(store, env, key, out);
    } else {
        match store.flight {
            Flight::Chain(_) => return staging::step(store, ev, out),
            Flight::Writing(_) => return resolutions::record(store, env, ev, out),
            Flight::Idle | Flight::Finding | Flight::Entering(_) => {}
        }
        if let Event::Record { .. } | Event::Room = ev {
            resolutions::record(store, env, ev, out);
        } else if let Event::Find { .. } = ev {
            resolutions::find(store, env, ev, out);
        } else {
            match store.flight {
                Flight::Finding => resolutions::find(store, env, ev, out),
                Flight::Entering(_) => enter::step(store, env, ev, out),
                Flight::Idle => {
                    if let Event::Arrived { .. } = ev {
                        enter::step(store, env, ev, out);
                    } else if store.phase == Phase::Scanning {
                        scan::step(store, env, ev, out);
                    } else {
                        start::step(store, env, ev, out);
                    }
                }
                Flight::Chain(_) | Flight::Writing(_) => unreachable!("flight dispatched"),
            }
        }
    }
}
pub fn fire(store: &mut Store, env: &Env<Limits>, out: &mut Queue<Output>) {
    match store.timers.expire(env.now) {
        Some(Timer::Retry) => out.push(Output::Request(Request::Lock)),
        Some(Timer::Said) => out.push(Output::Answer(vocabulary::Answer::Waiting)),
        None => {}
    }
}
#[must_use]
pub fn next_deadline(store: &Store) -> Option<Time> {
    store.timers.next()
}
#[cfg(test)]
mod tests;
