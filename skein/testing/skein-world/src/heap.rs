//! Memory at every iteration (simulator.md, 5; testing-strategy.md, 6).
//!
//! The harness meters each hosted process's heap and checks it against the
//! scenario's ceiling when one is named. One thread, one heap: a process's
//! part is what grew within the calls that run its code, metered with a
//! [`Span`], at its peak within each call: making it, delivering into its
//! own containers, iterating and
//! draining it, and destroying it. What the harness pushes into a process's
//! containers is metered inside that process's span, including queue growth.
//! Pops of its submissions stay outside: a queue never shrinks, so popping
//! frees nothing. The simulator's submit and reap bookkeeping, trace and
//! network, and the harness's storage stay outside process spans; simulated
//! completions are staged there before delivery. The backend hands every
//! buffer back and never copies or replaces one (`skein_io::kernel`); a
//! crash cut destroys its original records under their owner's span. The
//! referee changes a process only through flags it acts on in its next
//! iteration, allocating nothing of theirs. Once settled, the harness checks it:
//! each process, dropped, frees exactly what was metered as its own
//! ([`Outcome`](crate::Outcome)), which also finds a leak.

use alloc::format;
use alloc::vec::Vec;
use core::hint;

use skein_heap::{Grown, Span};

/// Whether a world checks memory: only a test binary that declares the
/// counting allocator its global allocator can (testing.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Memory {
    /// Checked at every iteration: what each process holds, its release on
    /// drop, and its peak, reported; no ceiling. The allocator must count.
    Checked,
    /// Checked as above, and each process stays within this scenario ceiling
    /// at every iteration (testing-strategy.md, 6).
    Within(u64),
    /// No check: the system allocator.
    Unchecked,
}

/// Each process's heap, metered, by its index in the world.
#[derive(Debug)]
pub(crate) struct Heap {
    procs: Vec<Held>,
    ceiling: Option<u64>,
}

/// One process's heap.
#[derive(Clone, Copy, Debug)]
struct Held {
    /// What it holds: what grew within its calls.
    now: i64,
    /// The most it held at once.
    most: i64,
}

impl Heap {
    /// The processes' heap, from nothing, once the counting allocator is seen
    /// to count.
    pub(crate) fn new(ceiling: Option<u64>) -> Heap {
        let span = Span::start();
        let probe = hint::black_box(Box::new([0_u8; 64]));
        assert!(
            span.end().net >= 64,
            "a world that checks memory runs under the counting allocator: #[global_allocator] static HEAP: skein_heap::Counting"
        );
        drop(probe);
        Heap { procs: Vec::new(), ceiling }
    }

    /// Makes the next process, metered, and checks its construction peak
    /// against the scenario ceiling, when named.
    pub(crate) fn admit<T, F: FnOnce() -> T>(&mut self, make: F) -> T {
        let span = Span::start();
        let proc = make();
        let grown = span.end();
        self.procs.push(Held { now: 0, most: 0 });
        let at = self.procs.len().checked_sub(1).expect("just pushed");
        self.check(at, grown);
        proc
    }

    /// Runs one of process `at`'s calls, metered: what the heap grew by is
    /// the process's, and the most it held at once within the call must be
    /// within the scenario ceiling, when named.
    pub(crate) fn around<T, F: FnOnce() -> T>(&mut self, at: usize, call: F) -> T {
        let span = Span::start();
        let result = call();
        self.check(at, span.end());
        result
    }

    /// Drops a terminated process under its own span and removes its meter.
    pub(crate) fn release<T>(&mut self, at: usize, proc: T) -> u64 {
        let held = self.procs.remove(at);
        let span = Span::start();
        drop(proc);
        let freed = span.end().net.checked_neg().expect("a heap within an i64");
        if freed != held.now {
            crate::fail(&format!(
                "terminated process {at} freed {freed} bytes, not the {} it held of its own: a leak, or heap made or freed outside its calls",
                held.now
            ));
        }
        u64::try_from(held.most).expect("a nonnegative peak")
    }

    /// Drops a cut process and meters its replacement in the same slot.
    pub(crate) fn restart<T, F: FnOnce() -> T>(&mut self, at: usize, proc: T, make: F) -> T {
        let old = *self.procs.get(at).expect("a process admitted");
        let span = Span::start();
        drop(proc);
        let freed = span.end().net.checked_neg().expect("heap fits i64");
        assert_eq!(
            freed, old.now,
            "the cut process releases exactly its remaining owned heap: a leak, or heap made or freed outside its calls"
        );
        let span = Span::start();
        let proc = make();
        let grown = span.end();
        *self.procs.get_mut(at).expect("the same process slot") = Held { now: 0, most: 0 };
        self.check(at, grown);
        let held = self.procs.get_mut(at).expect("the replacement's meter");
        held.most = held.most.max(old.most);
        proc
    }

    fn check(&mut self, at: usize, grown: Grown) {
        let held = self.procs.get_mut(at).expect("a process admitted");
        let most = held.now.checked_add(grown.peak).expect("a heap within an i64");
        if let Some(ceiling) = self.ceiling
            && u64::try_from(most).expect("a nonnegative peak") > ceiling
        {
            crate::fail(&format!(
                "process {at} held {most} bytes at once, past its scenario's ceiling, {ceiling} (testing-strategy.md, 6)"
            ));
        }
        held.most = held.most.max(most);
        held.now = held.now.checked_add(grown.net).expect("a heap within an i64");
    }

    /// What each process holds of its own now, by index.
    pub(crate) fn held(&self) -> Vec<i64> {
        let mut held = Vec::with_capacity(self.procs.len());
        for proc in &self.procs {
            held.push(proc.now);
        }
        held
    }

    /// Each process's peak, by index.
    pub(crate) fn report(&self) -> Vec<u64> {
        let mut report = Vec::with_capacity(self.procs.len());
        for held in &self.procs {
            report.push(u64::try_from(held.most).unwrap_or(0));
        }
        report
    }
}

/// The simulated loop's optional heap accounting.
#[derive(Debug)]
pub(crate) struct Metered(pub(crate) Option<Heap>);

impl skein_sim_world::Meter for Metered {
    fn admit<T>(&mut self, make: impl FnOnce() -> T) -> T {
        match &mut self.0 {
            Some(heap) => heap.admit(make),
            None => make(),
        }
    }
    fn around<T>(&mut self, at: usize, call: impl FnOnce() -> T) -> T {
        match &mut self.0 {
            Some(heap) => heap.around(at, call),
            None => call(),
        }
    }
    fn release<T>(&mut self, at: usize, proc: T) -> Option<u64> {
        match &mut self.0 {
            Some(heap) => Some(heap.release(at, proc)),
            None => {
                drop(proc);
                None
            }
        }
    }
    fn restart<T>(&mut self, at: usize, proc: T, make: impl FnOnce() -> T) -> T {
        match &mut self.0 {
            Some(heap) => heap.restart(at, proc, make),
            None => {
                drop(proc);
                make()
            }
        }
    }
}
