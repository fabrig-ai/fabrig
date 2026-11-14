//! The referee (testing-strategy.md, 7): a scenario's expectations, a step
//! machine beside the processes, at every tier. It watches from outside,
//! what the processes show of themselves (a fake's observations, a service's
//! public face), never their state. Safety is checked on every observation
//! and fails at once; liveness is a deadline, and one that passes unmet
//! fails the test with what is still pending. On the real clock, an unanswered
//! deadline classified as starved instead asks the world to end what it owns
//! and pass with the referee's figures (testing-strategy.md, 2.10). The
//! simulated world never asks for this verdict. It ends the test once every
//! expectation is met and the world has settled, and it may inject what
//! belongs to no process: a shutdown, a directory of addresses.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Write};
use core::marker::PhantomData;

use alloc::rc::Rc;
use core::cell::{Cell, RefCell};
use skein_io::kernel::ServiceSignal;
use skein_lib::{Queue, Time};

use crate::Starved;

/// Where the referee places a scenario's last word (testing-strategy.md, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastWord {
    /// When the referee's last expectation is met, the ordinary default.
    LastExpectation,
    /// When its last expectation is met, and no earlier than this time.
    After(Time),
}

impl LastWord {
    pub(crate) fn at(self, met: Time) -> Time {
        match self {
            LastWord::LastExpectation => met,
            LastWord::After(after) => met.max(after),
        }
    }
}

/// A scenario's referee, over its processes `P`.
///
/// Its methods never panic. A finding from `act`, `idle` or `passed` is
/// kept for the next `watch`, with `passed` answering false while it is kept
/// (skein/docs/design/testing/README.md, 5).
pub trait Referee<P> {
    /// Injects what belongs to no process, before they run in an iteration.
    /// It changes only what a process lets be changed between iterations (a
    /// flag it acts on in its next one), and allocates nothing of theirs. By
    /// default it injects nothing.
    fn act(&mut self, _now: Time, _procs: &mut [P]) {}

    /// Watches every process after one iterated, returning a broken safety
    /// expectation's finding, naming the seed. The loop stops at its `Err`.
    fn watch(&mut self, now: Time, procs: &[P]) -> Result<(), String>;

    /// The world is idle: nothing runs before the clock moves. A referee may change here what it will answer.
    fn idle(&mut self, _now: Time, _procs: &[P], _settled: bool) {}

    /// The earliest of its expectations' deadlines and of what it will act
    /// on, for the world to wake at; by default none.
    fn next_deadline(&self) -> Option<Time> {
        None
    }

    /// What is overdue at `now`, if anything: the test fails with it. By
    /// default nothing is.
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }

    /// Whether every expectation is met.
    fn passed(&self) -> bool;

    /// A deadline this referee keeps itself passed unanswered and was starved
    /// (testing-strategy.md, 2.10): the real loop ends what the world owns and
    /// passes with these figures. The simulated world never asks.
    fn starved(&mut self) -> Option<Starved> {
        None
    }

    /// The scenario's last word, later only for a named keep, idle or delay.
    fn last_word(&self) -> LastWord {
        LastWord::LastExpectation
    }
}

/// One expectation of a scenario's, about its processes `P`.
pub trait Expectation<P>: Debug {
    /// Whether it is met by what the processes show at `now`; `Err`, saying
    /// why, if a safety rule is broken.
    fn check(&self, now: Time, procs: &[P]) -> Result<bool, String>;

    /// When it must be met by.
    fn deadline(&self) -> Time;
}

/// A scenario's expectations, each pending until met: what a referee keeps.
pub struct Expectations<P, X> {
    seed: u64,
    pending: Vec<X>,
    met: Vec<X>,
    procs: PhantomData<fn(&[P])>,
}

impl<P, X: Expectation<P>> Expectations<P, X> {
    #[must_use]
    pub fn new(seed: u64, expectations: Vec<X>) -> Expectations<P, X> {
        Expectations { seed, pending: expectations, met: Vec::new(), procs: PhantomData }
    }

    /// Checks every pending expectation: a broken one panics, naming the
    /// seed; a met one is set aside.
    pub fn observe(&mut self, now: Time, procs: &[P]) {
        self.check(now, procs).unwrap_or_else(|why| crate::fail(&why));
    }

    /// Checks pending expectations, setting met ones aside and returning
    /// the first broken one's finding with its seed and time.
    pub fn check(&mut self, now: Time, procs: &[P]) -> Result<(), String> {
        let mut at = 0;
        while let Some(expectation) = self.pending.get(at) {
            match expectation.check(now, procs) {
                Ok(true) => {
                    let met = self.pending.remove(at);
                    self.met.push(met);
                }
                Ok(false) => at = at.checked_add(1).expect("fewer expectations than memory"),
                Err(why) => return Err(broke(self.seed, now, expectation, &why)),
            }
        }
        Ok(())
    }

    /// When the earliest pending expectation falls due.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next: Option<Time> = None;
        for expectation in &self.pending {
            let at = expectation.deadline();
            next = Some(match next {
                Some(next) => next.min(at),
                None => at,
            });
        }
        next
    }

    /// The pending expectations past their deadline at `now`, with all that
    /// is pending, if any is.
    #[must_use]
    #[expect(clippy::use_debug, reason = "what is overdue is printed for a person reading a failure")]
    pub fn overdue(&self, now: Time) -> Option<String> {
        let mut overdue = String::new();
        for expectation in &self.pending {
            if expectation.deadline() <= now {
                writeln!(overdue, "  unmet: {expectation:?}").expect("writing to a String");
            }
        }
        if overdue.is_empty() {
            return None;
        }
        writeln!(overdue, "  pending: {:?}", self.pending).expect("writing to a String");
        Some(overdue)
    }

    #[must_use]
    pub fn passed(&self) -> bool {
        self.pending.is_empty()
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }
}

impl<P, X: Debug> Debug for Expectations<P, X> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expectations").field("seed", &self.seed).field("pending", &self.pending).finish_non_exhaustive()
    }
}

/// What a broken safety expectation fails the world with.
fn broke(seed: u64, now: Time, expectation: &dyn Debug, why: &str) -> String {
    format!("seed {seed}: at {} ns, {expectation:?} broke: {why}", now.as_nanos())
}

/// Referee access to world signals and real-loop scheduler accounting.
#[derive(Clone, Debug)]
pub struct Controls {
    pub(crate) signals: Rc<RefCell<Queue<(usize, ServiceSignal)>>>,
    pub(crate) sched_start: Rc<Cell<Option<crate::real::Sched>>>,
}

impl Controls {
    pub(crate) fn new() -> Self {
        Self { signals: Rc::new(RefCell::new(Queue::with_capacity(256))), sched_start: Rc::new(Cell::new(None)) }
    }

    /// Delivers a signal to the named root or hosted child in the next turn.
    /// Root ids come from `spawn`; hosted ids follow them in admission order.
    pub fn signal(&self, host: usize, signal: ServiceSignal) {
        self.signals.borrow_mut().push((host, signal));
    }
}

#[cfg(test)]
mod tests {
    use super::{Expectation, Expectations};
    use alloc::rc::Rc;
    use core::cell::Cell;
    use core::fmt;
    use skein_lib::Time;

    struct Answer {
        calls: Rc<Cell<u32>>,
        verdict: Result<bool, String>,
    }

    impl fmt::Debug for Answer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("answer")
        }
    }

    impl Expectation<()> for Answer {
        fn check(&self, _now: Time, _procs: &[()]) -> Result<bool, String> {
            self.calls.set(self.calls.get().checked_add(1).expect("bounded test checks"));
            self.verdict.clone()
        }
        fn deadline(&self) -> Time {
            Time::from_nanos(20)
        }
    }

    #[test]
    fn a_broken_expectation_returns_its_seed_time_name_and_reason() {
        let calls = Rc::new(Cell::new(0));
        let later = Rc::new(Cell::new(0));
        let mut expectations = Expectations::new(
            17,
            vec![
                Answer { calls: calls.clone(), verdict: Err(String::from("no answer")) },
                Answer { calls: later.clone(), verdict: Ok(true) },
            ],
        );
        assert_eq!(
            expectations.check(Time::from_nanos(12), &[]),
            Err(String::from("seed 17: at 12 ns, answer broke: no answer"))
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(later.get(), 0);
    }

    #[test]
    fn a_met_expectation_is_never_checked_again() {
        let calls = Rc::new(Cell::new(0));
        let mut expectations = Expectations::new(17, vec![Answer { calls: calls.clone(), verdict: Ok(true) }]);
        assert_eq!(expectations.check(Time::from_nanos(12), &[]), Ok(()));
        assert!(expectations.passed());
        assert_eq!(expectations.next_deadline(), None);
        assert_eq!(expectations.check(Time::from_nanos(30), &[]), Ok(()));
        assert_eq!(calls.get(), 1);
    }
}
