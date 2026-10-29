//! A commit barrier for writes and outputs (lib.md, section 11).
//! Decisions and queues grow as they fill; release is bounded per call.
//!
//! The journal keeps accepted commits, their numbers, outputs held for
//! durability, and untagged outputs at its door. It never inspects writes or
//! outputs, and it learns durability only when its owner answers a commit.
//! `decide` opens one decision before the owner changes state;
//! `accept` makes at most one commit. The owner sends `commit` values to its
//! store, answers them with `committed` or `failed`, then calls `release`.
//! A root calls `failed` for a failed sent commit, or `stop` when it stops of its own.
//! `uncapped` resumes numbering after its supplied durable commit on restart.
//!
//! Commit transitions (lib.md, section 11):
//!
//! | State | Event | Next | What becomes available |
//! |---|---|---|---|
//! | open | `commit` | sent | the writes go to the owner |
//! | sent | `committed(n)` | durable | output tagged at most `n` |
//! | sent | `failed(n)` | failed | nothing tagged at or after `n`; admission stops |
//!
//! A decision without writes makes no commit. Its outputs follow the last
//! commit made, or number zero, which is already durable.

use core::hash::{Hash, Hasher};

use crate::Queue;

/// Container sizes kept by callers (lib.md, section 11).
/// The journal reads only `release`; the other fields are sizes its callers keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalLimits {
    /// Commits made but not yet answered.
    pub commits: u32,
    /// Writes in one commit.
    pub writes: u32,
    /// Outputs held across decisions.
    pub held: u32,
    /// Untagged outputs at the door.
    pub now: u32,
    /// Outputs released by one call.
    pub release: u32,
}

/// One accepted decision's writes, numbered for the store.
#[derive(Debug, Hash)]
pub struct Commit<W> {
    /// The journal's increasing commit number.
    pub number: u64,
    /// The writes the store applies together, in order.
    pub writes: Queue<W>,
}

/// Whether release moved outputs, found none ready, or found a stopped journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Released {
    /// At least one output moved to the caller's queue.
    Some,
    /// No output was ready or the caller's queue had no room.
    None,
    /// The journal stopped by a failed or invalid commit, or `stop`.
    Stopped,
}

#[derive(Debug, Hash)]
struct Held<O> {
    after: u64,
    output: O,
}

/// One decision's writes and held outputs, growing as they fill.
#[must_use]
#[derive(Debug)]
pub struct Decision<W, O> {
    writes: Queue<W>,
    held: Queue<O>,
}

impl<W, O> Decision<W, O> {
    /// Adds a write.
    pub fn push_write(&mut self, write: W) {
        self.writes.push(write);
    }

    /// Holds an output until its commit is durable.
    pub fn push_held(&mut self, output: O) {
        self.held.push(output);
    }

    /// Adds a write; an uncapped decision never refuses, so this always
    /// returns `Ok`. Kept while temper's engine-domain world test calls it;
    /// `push_write` is the form.
    pub fn write(&mut self, write: W) -> Result<(), W> {
        self.push_write(write);
        Ok(())
    }

    /// Holds an output; an uncapped decision never refuses, so this always
    /// returns `Ok`. Kept while temper's engine-domain world test calls it;
    /// `push_held` is the form.
    pub fn hold(&mut self, output: O) -> Result<(), O> {
        self.push_held(output);
        Ok(())
    }
}

/// A growing commit barrier with bounded release per call (lib.md, section 11).
#[derive(Debug)]
pub struct Journal<W, O> {
    release: u32,
    pending: Queue<Commit<W>>,
    outstanding: Queue<u64>,
    held: Queue<Held<O>>,
    now: Queue<O>,
    open: Option<()>,
    last_number: u64,
    sent: u64,
    durable: u64,
    stopped: bool,
}

impl<W, O> Journal<W, O> {
    /// Makes an uncapped journal after `durable`, releasing at most `release`
    /// outputs per call. `durable` restores numbering after a restart.
    #[must_use]
    pub fn uncapped(release: u32, durable: u64) -> Journal<W, O> {
        Journal {
            release,
            pending: Queue::uncapped(),
            outstanding: Queue::uncapped(),
            held: Queue::uncapped(),
            now: Queue::uncapped(),
            open: None,
            last_number: durable,
            sent: durable,
            durable,
            stopped: false,
        }
    }

    /// Opens an uncapped decision, including after the journal has stopped.
    /// Calling this with a decision open is a bug.
    pub fn decide(&mut self) -> Decision<W, O> {
        assert!(self.open.is_none(), "only one decision may be open");
        self.open = Some(());
        Decision { writes: Queue::uncapped(), held: Queue::uncapped() }
    }

    /// Commits not yet answered, held outputs, and outputs at the door, in entries.
    #[must_use]
    pub fn backlog(&self) -> u64 {
        let commits_and_held = u64::from(self.outstanding.len())
            .checked_add(u64::from(self.held.len()))
            .expect("two u32 counts fit in a u64");
        commits_and_held.checked_add(u64::from(self.now.len())).expect("three u32 counts fit in a u64")
    }

    /// Adds an untagged output at the door, returning it if the journal stopped.
    pub fn now(&mut self, output: O) -> Result<(), O> {
        if self.stopped {
            return Err(output);
        }
        self.now.push(output);
        Ok(())
    }

    /// Accepts one decision, making at most one numbered commit.
    /// A writing decision at `u64::MAX` panics: commit numbering is exhausted.
    pub fn accept(&mut self, mut decision: Decision<W, O>) {
        assert!(self.open.is_some(), "accept an open decision");
        self.open = None;
        if self.stopped {
            return;
        }
        if !decision.writes.is_empty() {
            self.last_number = self.last_number.checked_add(1).expect("commit numbering exhausted at u64::MAX");
            self.outstanding.push(self.last_number);
            self.pending.push(Commit { number: self.last_number, writes: decision.writes });
        }
        while let Some(output) = decision.held.pop() {
            self.held.push(Held { after: self.last_number, output });
        }
    }

    /// Takes the next numbered commit for the store, in order.
    pub fn commit(&mut self) -> Option<Commit<W>> {
        if self.stopped {
            return None;
        }
        let commit = self.pending.pop()?;
        self.sent = commit.number;
        Some(commit)
    }

    /// Marks a sent commit and every earlier one durable; invalid answers stop.
    pub fn committed(&mut self, number: u64) {
        if self.stopped {
            return;
        }
        if number <= self.durable || number > self.sent {
            self.stopped = true;
            return;
        }
        while let Some(front) = self.outstanding.iter().next() {
            if *front > number {
                break;
            }
            self.outstanding.pop().expect("front exists");
        }
        self.durable = number;
    }

    /// Stops after a failed sent commit; nothing tagged with it or later leaves.
    /// A root that stops of its own calls `stop`.
    pub fn failed(&mut self, number: u64) {
        if self.stopped {
            return;
        }
        if number <= self.durable || number > self.sent {
            self.stopped = true;
            return;
        }
        self.stopped = true;
    }

    /// Stops the journal of a root that stops of its own: nothing held or
    /// at the door leaves again, and `release` answers `Stopped`. A failed sent commit is
    /// `failed(number)`. Stopping twice is stopping once.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// Moves at most the release limit of ready outputs into `out`.
    pub fn release(&mut self, out: &mut Queue<O>) -> Released {
        if self.stopped {
            return Released::Stopped;
        }
        let mut moved = 0;
        while moved < self.release && out.room() > 0 {
            let held_ready = match self.held.iter().next() {
                Some(held) => held.after <= self.durable,
                None => false,
            };
            if let Some(output) = self.now.pop() {
                out.push(output);
                moved = moved.checked_add(1).expect("bounded by the release limit");
            } else if held_ready {
                let held = self.held.pop().expect("ready front exists");
                out.push(held.output);
                moved = moved.checked_add(1).expect("bounded by the release limit");
            } else {
                break;
            }
        }
        if moved > 0 { Released::Some } else { Released::None }
    }

    /// Whether the journal has stopped: a failed or invalid commit, or `stop`.
    #[must_use]
    pub const fn stopped(&self) -> bool {
        self.stopped
    }

    /// Whether the running journal has no open decision, commit, or output.
    #[must_use]
    pub fn idle(&self) -> bool {
        !self.stopped
            && self.open.is_none()
            && self.pending.is_empty()
            && self.outstanding.is_empty()
            && self.held.is_empty()
            && self.now.is_empty()
    }
}

// lib.md, 5.1: hash held state, never capacity or storage layout.
impl<W: Hash, O: Hash> Hash for Journal<W, O> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pending.hash(state);
        self.outstanding.hash(state);
        self.held.hash(state);
        self.now.hash(state);
        self.open.hash(state);
        self.last_number.hash(state);
        self.sent.hash(state);
        self.durable.hash(state);
        self.stopped.hash(state);
    }
}
