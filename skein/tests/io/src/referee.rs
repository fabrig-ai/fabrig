//! The referee (testing-strategy.md, 7): a scenario's expectations, as a
//! small step machine beside the world. It watches from outside, what the
//! owners saw, never io's state. Safety is checked on every observation and
//! fails at once; liveness is a deadline, and one that passes fails the test
//! with what is still pending.

use std::collections::BTreeMap;
use std::fmt;

use skein_io::Error;
use skein_lib::Time;

use crate::owner::{Conn, Owner};

/// What a scenario expects of a connection, named by its process and its
/// plan's name.
#[derive(Clone)]
pub enum Expect {
    /// A listener fails with one of the named errors.
    ListenFails { at: usize, serve: &'static str, errors: &'static [Error], by: Time },
    /// Safety: what it received is a prefix of `bytes`, at every
    /// observation. Liveness: all of them by `by`, unless its stream broke.
    Receives { at: usize, conn: &'static str, bytes: Box<[u8]>, by: Time },
    /// Safety only: what it received is a prefix of `bytes`, at every
    /// observation; met once it is closed by `by`, or if it is never made (a
    /// server's, when its client's connect failed).
    Prefix { at: usize, conn: &'static str, bytes: Box<[u8]>, by: Time },
    /// Liveness: it is closed by `by`; or, if `optional`, it was never made.
    Closed { at: usize, conn: &'static str, by: Time, optional: bool },
    /// Liveness: its connect fails with one of `errors` by `by`.
    Fails { at: usize, conn: &'static str, errors: &'static [Error], by: Time },
    /// Safety: until `until`, it has handed io no more than `most` bytes.
    Holds { at: usize, conn: &'static str, most: u64, until: Time },
    /// Safety: it is never told `Failed`. Liveness: it hears `End` by `by`:
    /// its peer ended the stream, not a reset.
    Ends { at: usize, conn: &'static str, by: Time },
}

/// A scenario's expectations, each either pending or met.
#[derive(Debug)]
pub struct Referee {
    seed: u64,
    pending: BTreeMap<usize, Expect>,
    met: Vec<Expect>,
}

impl Referee {
    #[must_use]
    pub fn new(seed: u64, expectations: Vec<Expect>) -> Referee {
        Referee { seed, pending: expectations.into_iter().enumerate().collect(), met: Vec::new() }
    }

    /// Checks every expectation about process `at` against what its owner
    /// saw: safety at once, liveness when met.
    pub fn watch(&mut self, now: Time, at: usize, owner: &Owner) -> Result<(), String> {
        let mut met = Vec::new();
        for (key, expect) in &self.pending {
            if expect.at() == at && self.check(now, expect, owner)? {
                met.push(*key);
            }
        }
        self.meet(met);
        Ok(())
    }

    fn meet(&mut self, keys: Vec<usize>) {
        for key in keys {
            let expect = self.pending.remove(&key).expect("pending");
            self.met.push(expect);
        }
    }

    /// Whether `expect` is met, or the first broken safety expectation's finding.
    fn check(&self, now: Time, expect: &Expect, owner: &Owner) -> Result<bool, String> {
        let seed = self.seed;
        let conn = |name: &str| -> Option<&Conn> { owner.conn(name) };
        Ok(match expect {
            Expect::Receives { conn: name, bytes, .. } | Expect::Prefix { conn: name, bytes, .. } => {
                let Some(conn) = conn(name) else { return Ok(false) };
                let same = common(&conn.received, bytes);
                if same != conn.received.len() {
                    return Err(format!(
                        "seed {seed}: {name} received what its peer sent, in order: {same} bytes of {}, then {:?}",
                        bytes.len(),
                        &conn.received[same..conn.received.len().min(same + 16)],
                    ));
                }
                let whole = matches!(expect, Expect::Receives { .. }) && conn.received.len() == bytes.len();
                let closed = conn.closed_at.is_some() && (conn.broken() || matches!(expect, Expect::Prefix { .. }));
                whole || closed
            }
            Expect::Closed { conn: name, .. } => conn(name).is_some_and(|conn| conn.closed_at.is_some()),
            Expect::ListenFails { serve, errors, .. } => {
                let Some(listener) = owner.listeners.values().find(|listener| listener.serve.name == *serve) else {
                    return Ok(false);
                };
                if let Some(error) = listener.error {
                    if !errors.contains(&error) {
                        return Err(format!("seed {seed}: {serve} failed with {error:?}, not {errors:?}"));
                    }
                    true
                } else {
                    false
                }
            }
            Expect::Fails { conn: name, errors, .. } => {
                let Some(conn) = conn(name) else { return Ok(false) };
                if let Some(error) = conn.error {
                    if !errors.contains(&error) {
                        return Err(format!("seed {seed}: {name} failed with {error:?}, not {errors:?}"));
                    }
                    return Ok(true);
                }
                if conn.closed_at.is_some() {
                    return Err(format!("seed {seed}: {name} closed without failing"));
                }
                false
            }
            Expect::Holds { conn: name, most, until, .. } => {
                if now >= *until {
                    return Ok(true);
                }
                if let Some(conn) = conn(name)
                    && conn.handed > *most
                {
                    return Err(format!(
                        "seed {seed}: {name} handed io {} bytes before {} ns, past {most}: backpressure failed",
                        conn.handed,
                        until.as_nanos()
                    ));
                }
                false
            }
            Expect::Ends { conn: name, .. } => {
                let Some(conn) = conn(name) else { return Ok(false) };
                if conn.failed.is_some() {
                    return Err(format!("seed {seed}: {name} was told Failed({:?}), not the end", conn.failed));
                }
                conn.ended
            }
        })
    }

    /// The expectations that a connection never made meets, met: the world
    /// settled, or their deadline came, without it.
    fn absent(&mut self, owners: &[&Owner], due: Option<Time>) {
        let mut met = Vec::new();
        for (key, expect) in &self.pending {
            let optional = match expect {
                Expect::Prefix { .. } | Expect::Closed { optional: true, .. } => true,
                Expect::Closed { optional: false, .. }
                | Expect::Receives { .. }
                | Expect::Fails { .. }
                | Expect::Holds { .. }
                | Expect::Ends { .. }
                | Expect::ListenFails { .. } => false,
            };
            let reached = due.is_none_or(|now| expect.deadline() <= now);
            if optional && reached && owners[expect.at()].conn(expect.conn()).is_none() {
                met.push(*key);
            }
        }
        self.meet(met);
    }

    /// The world settled: nothing more will be made.
    pub fn settled(&mut self, owners: &[&Owner]) {
        self.absent(owners, None);
    }

    /// When the earliest pending expectation falls due.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.pending.values().map(Expect::deadline).min()
    }

    /// Returns a finding if an expectation fell due unmet by `now`.
    pub fn overdue(&mut self, now: Time, owners: &[&Owner], trace: &dyn Fn() -> String) -> Result<(), String> {
        self.absent(owners, Some(now));
        let overdue: Vec<&Expect> = self.pending.values().filter(|expect| expect.deadline() <= now).collect();
        if !overdue.is_empty() {
            return Err(format!(
                "seed {}: at {} ns, unmet: {overdue:?}\nstill pending: {:?}\n{}",
                self.seed,
                now.as_nanos(),
                self.pending.values().collect::<Vec<_>>(),
                trace()
            ));
        }
        Ok(())
    }

    /// Whether every expectation is met.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Expect {
    const fn at(&self) -> usize {
        match self {
            Expect::Receives { at, .. }
            | Expect::Prefix { at, .. }
            | Expect::Closed { at, .. }
            | Expect::Fails { at, .. }
            | Expect::Holds { at, .. }
            | Expect::Ends { at, .. }
            | Expect::ListenFails { at, .. } => *at,
        }
    }

    const fn conn(&self) -> &'static str {
        match self {
            Expect::Receives { conn, .. }
            | Expect::Prefix { conn, .. }
            | Expect::Closed { conn, .. }
            | Expect::Fails { conn, .. }
            | Expect::Holds { conn, .. }
            | Expect::Ends { conn, .. } => conn,
            Expect::ListenFails { serve, .. } => serve,
        }
    }

    const fn deadline(&self) -> Time {
        match self {
            Expect::Receives { by, .. }
            | Expect::Prefix { by, .. }
            | Expect::Closed { by, .. }
            | Expect::Fails { by, .. }
            | Expect::Ends { by, .. }
            | Expect::ListenFails { by, .. } => *by,
            Expect::Holds { until, .. } => *until,
        }
    }
}

/// Briefly: the bytes by their length.
impl fmt::Debug for Expect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (at, conn, when) = (self.at(), self.conn(), self.deadline().as_nanos());
        match self {
            Expect::Receives { bytes, .. } => write!(f, "{conn}@{at} receives {} bytes by {when} ns", bytes.len()),
            Expect::Prefix { bytes, .. } => {
                write!(f, "{conn}@{at} receives a prefix of {} bytes, closed by {when} ns", bytes.len())
            }
            Expect::Closed { optional, .. } => write!(f, "{conn}@{at} closed by {when} ns (optional: {optional})"),
            Expect::ListenFails { errors, .. } | Expect::Fails { errors, .. } => {
                write!(f, "{conn}@{at} fails with one of {errors:?} by {when} ns")
            }
            Expect::Holds { most, .. } => write!(f, "{conn}@{at} hands io no more than {most} bytes until {when} ns"),
            Expect::Ends { .. } => write!(f, "{conn}@{at} hears End, never Failed, by {when} ns"),
        }
    }
}

fn common(a: &[u8], b: &[u8]) -> usize {
    // Matching deliveries use the slice comparison; only a mismatch needs
    // the byte offset that the referee's failure reports.
    if b.starts_with(a) {
        return a.len();
    }
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// The scenario's expectations at each turn and idle point (runtime/io/README.md, 5).
pub(crate) struct Judge {
    expectations: Referee,
    finding: Option<String>,
}

impl Judge {
    pub(crate) const fn new(expectations: Referee) -> Self {
        Self { expectations, finding: None }
    }
}

impl skein_world::Referee<crate::world::Hosted> for Judge {
    fn watch(&mut self, now: Time, procs: &[crate::world::Hosted]) -> Result<(), String> {
        for (at, host) in procs.iter().enumerate() {
            self.expectations.watch(now, at, &host.proc.owner)?;
        }
        Ok(())
    }
    fn idle(&mut self, now: Time, procs: &[crate::world::Hosted], settled: bool) {
        let owners: Vec<&Owner> = procs.iter().map(|host| &host.proc.owner).collect();
        if settled {
            self.expectations.settled(&owners);
        }
        self.finding = self
            .expectations
            .overdue(now, &owners, &|| {
                procs.iter().map(|host| format!("{:?}", host.proc.log)).collect::<Vec<_>>().join("\n")
            })
            .err();
    }
    fn next_deadline(&self) -> Option<Time> {
        self.expectations.next_deadline()
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        self.finding.clone()
    }
    fn passed(&self) -> bool {
        self.expectations.passed()
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::After(crate::scenarios::END)
    }
}

#[cfg(test)]
mod tests {
    use super::{Expect, Judge, Referee};
    use crate::owner::{Directory, Owner, Serve, ServeClose};
    use crate::scenarios::{limits, local};
    use crate::world::{Hosted, Proc};
    use skein_io::Error;
    use skein_lib::Time;
    use skein_world::Referee as _;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn an_unexpected_listener_error_returns_its_safety_finding() {
        let mut owner = Owner::new(
            7,
            vec![Serve { name: "listener", addr: local(0), answers: vec![], close: ServeClose::Answered(Time::ZERO) }],
            vec![],
        );
        owner.listeners.values_mut().next().expect("one listener").error = Some(Error::Other);
        let mut referee = Referee::new(
            7,
            vec![Expect::ListenFails { at: 0, serve: "listener", errors: &[Error::Busy], by: Time::from_nanos(10) }],
        );
        let finding = referee.watch(Time::ZERO, 0, &owner).expect_err("unexpected listener failure");
        assert_eq!(finding, "seed 7: listener failed with Other, not [Busy]");
        assert!(!referee.passed());
    }

    #[test]
    fn an_idle_judge_returns_the_overdue_finding_with_its_trace() {
        let now = Time::from_nanos(10);
        let mut judge =
            Judge::new(Referee::new(7, vec![Expect::Closed { at: 0, conn: "required", by: now, optional: false }]));
        let mut proc = Proc::new(limits(1), Owner::new(7, vec![], vec![]));
        proc.log.push("owner trace".into());
        let procs = [Hosted { proc, directory: Rc::new(RefCell::new(Directory::new())) }];
        judge.watch(now, &procs).expect("waiting is not a safety failure");
        judge.idle(now, &procs, false);
        let finding = judge.overdue(now).expect("the required close is overdue");
        assert!(finding.contains("seed 7: at 10 ns, unmet:"), "{finding}");
        assert!(finding.contains("required@0 closed by 10 ns (optional: false)"), "{finding}");
        assert!(finding.contains("still pending:"), "{finding}");
        assert!(finding.contains("owner trace"), "{finding}");
        assert!(!judge.passed());
    }

    #[test]
    fn common_keeps_matching_prefixes_and_the_first_mismatch_offset() {
        for (received, expected, same) in [
            (b"".as_slice(), b"abc".as_slice(), 0),
            (b"abc", b"abcd", 3),
            (b"abc", b"abc", 3),
            (b"abc", b"abX", 2),
            (b"abcd", b"abc", 3),
            (b"abc", b"Xbc", 0),
        ] {
            assert_eq!(super::common(received, expected), same, "{received:?} against {expected:?}");
        }
    }
}
