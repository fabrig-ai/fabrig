//! The fault ledger (`skein/docs/design/testing/sweeps.md`, 3): each fault
//! recorded as it falls, with its closed interval, identities and counters.
//! One ledger belongs to one run and only grows; unlike [`crate::domain::Ledger`],
//! it keeps faults, not requests in flight awaiting their terminals.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{format, string::String, vec::Vec};

use skein_lib::Time;

/// A fault's family and its name within that family.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Kind {
    pub family: &'static str,
    pub name: &'static str,
}

/// The identities a fault strikes; a field it doesn't strike is `None`.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Struck {
    pub account: Option<String>,
    pub runner: Option<String>,
    pub gate_run: Option<String>,
    pub candidate: Option<String>,
    pub base: Option<String>,
    pub request: Option<String>,
}

impl Struck {
    fn shares_identity(&self, other: &Struck) -> bool {
        [
            (&self.account, &other.account),
            (&self.runner, &other.runner),
            (&self.gate_run, &other.gate_run),
            (&self.candidate, &other.candidate),
            (&self.base, &other.base),
            (&self.request, &other.request),
        ]
        .into_iter()
        .any(|(left, right)| left.is_some() && left == right)
    }
}

/// The six counters of `temper/docs/design/draft/failure-worlds.md`, 3.3,
/// each counted independently.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Counter {
    ProviderRetries,
    SessionRetries,
    TaskTries,
    GateRetries,
    Repairs,
    Asks,
}

/// A fault's entry in its ledger; use only with the ledger that issued it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Injection(usize);

/// A recorded fault, its closed interval and the transitions attributed to it.
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the specified record exposes facts but reads counters through count")]
pub struct Injected {
    pub kind: Kind,
    pub from: Time,
    /// When it stopped; `None` while it is in force, including at the run's end.
    pub until: Option<Time>,
    pub struck: Struck,
    counts: BTreeMap<Counter, u64>,
}

impl Injected {
    /// How many transitions of this counter the world attributed to the fault.
    #[must_use]
    pub fn count(&self, counter: Counter) -> u64 {
        self.counts.get(&counter).copied().unwrap_or(0)
    }

    /// `from <= at`, and `at <= until` where it stopped.
    #[must_use]
    pub fn in_force(&self, at: Time) -> bool {
        self.from <= at && self.until.is_none_or(|until| at <= until)
    }

    fn finding(&self, why: &str) -> String {
        format!(
            "fault {:?}/{:?} [{:?}, {:?}] strikes {:?}: {why}",
            self.kind.family, self.kind.name, self.from, self.until, self.struck
        )
    }
}

/// One run's faults in the order they fell, retained after they stop.
#[derive(Default, Debug)]
pub struct FaultLedger {
    entries: Vec<Injected>,
    left_out: Vec<(&'static str, Kind, Time)>,
}

impl FaultLedger {
    /// Starts one run's empty fault ledger.
    #[must_use]
    pub fn new() -> FaultLedger {
        FaultLedger::default()
    }

    /// Records a draw left out before injection by its named exclusion
    /// (`skein/docs/design/testing/sweeps.md`, 4 and 5).
    pub fn leave_out(&mut self, scenario: &'static str, kind: Kind, at: Time) {
        self.left_out.push((scenario, kind, at));
    }

    /// Left-out draws in draw order, separate from injected faults.
    pub(crate) fn left_out_draws(&self) -> &[(&'static str, Kind, Time)] {
        &self.left_out
    }

    /// Records a fault as it starts.
    pub fn inject(&mut self, kind: Kind, from: Time, struck: Struck) -> Injection {
        let entry = Injection(self.entries.len());
        self.entries.push(Injected { kind, from, until: None, struck, counts: BTreeMap::new() });
        entry
    }

    /// Records when it stops; an instant's fault stops at its start.
    /// Returns a finding if already stopped or if `at` precedes its start.
    pub fn stop(&mut self, entry: Injection, at: Time) -> Result<(), String> {
        let injected = self.entries.get_mut(entry.0).expect("injection belongs to this ledger");
        if injected.until.is_some() {
            return Err(injected.finding("already stopped"));
        }
        if at < injected.from {
            return Err(injected.finding(&format!("stop at {} ns precedes its start", at.as_nanos())));
        }
        injected.until = Some(at);
        Ok(())
    }

    /// Counts one transition of a counter the world attributes to the fault.
    pub fn count(&mut self, entry: Injection, counter: Counter) {
        let injected = self.entries.get_mut(entry.0).expect("injection belongs to this ledger");
        let count = injected.counts.entry(counter).or_default();
        *count = count.checked_add(1).expect("a run's fault counter fits u64");
    }

    /// Reads a fault from this ledger, including after it stops.
    #[must_use]
    pub fn get(&self, entry: Injection) -> &Injected {
        self.entries.get(entry.0).expect("injection belongs to this ledger")
    }

    /// Every fault in the order it fell.
    #[must_use]
    pub fn entries(&self) -> &[Injected] {
        &self.entries
    }

    /// The faults in force at `at` that share a named identity in the same
    /// field of `struck`, in the order they fell. Absent identities don't match.
    #[must_use]
    pub fn striking(&self, at: Time, struck: &Struck) -> Vec<Injection> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, injected)| injected.in_force(at) && injected.struck.shares_identity(struck))
            .map(|(index, _)| Injection(index))
            .collect()
    }

    /// The kinds that fell, each once, whether or not still in force.
    #[must_use]
    pub fn fell(&self) -> BTreeSet<Kind> {
        self.entries.iter().map(|injected| injected.kind).collect()
    }
}
