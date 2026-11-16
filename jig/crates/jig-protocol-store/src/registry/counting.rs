//! Test-only per-planning tally (jig/docs/design/protocol/store/layouts.md, 7).
#![expect(clippy::disallowed_types, reason = "the test tally isolates each test thread")]
use super::Digest;
use alloc::collections::BTreeMap;
use std::{sync::Mutex, thread::ThreadId};

#[derive(Clone, Debug)]
pub(super) struct Counts {
    pub(super) parses: BTreeMap<Digest, u32>,
    pub(super) layouts: u32,
    pub(super) held: BTreeMap<(Digest, bool), u32>,
}
impl Counts {
    fn new() -> Self {
        Self { parses: BTreeMap::new(), layouts: 0, held: BTreeMap::new() }
    }
    fn apply(&mut self, event: Event) {
        match event {
            Event::Reset => *self = Self::new(),
            Event::Parse(digest) => {
                let count = self.parses.get(&digest).copied().unwrap_or(0);
                self.parses.insert(digest, count.checked_add(1).expect("parse count fits"));
            }
            Event::Held(digest, current) => {
                let key = (digest, current);
                let count = self.held.get(&key).copied().unwrap_or(0);
                self.held.insert(key, count.checked_add(1).expect("held count fits"));
            }
            Event::Layout => self.layouts = self.layouts.checked_add(1).expect("layout count fits"),
            Event::Read => {}
        }
    }
}
enum Event {
    Reset,
    Parse(Digest),
    Held(Digest, bool),
    Layout,
    Read,
}
static COUNTS: Mutex<Vec<(ThreadId, Counts)>> = Mutex::new(Vec::new());

fn record(event: Event) -> Counts {
    let thread = std::thread::current().id();
    let mut counts = COUNTS.lock().expect("test counter lock");
    for (id, counts) in counts.iter_mut() {
        if *id == thread {
            counts.apply(event);
            return counts.clone();
        }
    }
    let mut tally = Counts::new();
    tally.apply(event);
    counts.push((thread, tally.clone()));
    tally
}
pub(super) fn reset() {
    record(Event::Reset);
}
pub(super) fn parsed(digest: Digest) {
    record(Event::Parse(digest));
}
pub(super) fn held(digest: Digest, current: bool) {
    record(Event::Held(digest, current));
}
pub(super) fn layout() {
    record(Event::Layout);
}
pub(super) fn get() -> Counts {
    record(Event::Read)
}
