//! Gate blame and tries (`temper/docs/design/testing/blame.md`, 2–5).
//! Worlds supply observed reactions, not engine state; this judge attributes
//! them to a run's retained faults and returns findings in observation order.

use std::collections::BTreeMap;

use skein_lib::Time;
use skein_world::faults::{Counter, FaultLedger, Injection, Kind, Struck};

/// A gate job's setup failed.
pub const SET_UP: Kind = Kind { family: "ci", name: "set_up" };
/// A gate job was lost.
pub const LOST: Kind = Kind { family: "ci", name: "lost" };
/// A gate job waited for a runner.
pub const RUNNER_WAIT: Kind = Kind { family: "ci", name: "runner_wait" };
/// A gate or context never reported.
pub const SILENT: Kind = Kind { family: "ci", name: "silent" };
/// Someone other than temper cancelled the gate job.
pub const CANCELLED: Kind = Kind { family: "ci", name: "cancelled" };
/// A completed gate did not upload its report.
pub const MISSING_REPORT: Kind = Kind { family: "ci", name: "missing_report" };
/// A gate ran out of resources with its tests unobserved.
pub const STARVED: Kind = Kind { family: "ci", name: "starved" };
/// A candidate contains a wrong tree.
pub const WRONG_TREE: Kind = Kind { family: "ci", name: "wrong_tree" };
/// A base's own CI is red.
pub const BASE_RED: Kind = Kind { family: "ci", name: "base_red" };

/// The cause of a gate failure, as the table of blame.md, 2 classifies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Runner,
    Starved,
    Change,
    Base,
}

/// The class of a kind a gate row covers; `None` for any other.
#[must_use]
pub fn class(kind: Kind) -> Option<Class> {
    match kind {
        SET_UP | LOST | RUNNER_WAIT | SILENT | CANCELLED | MISSING_REPORT => Some(Class::Runner),
        STARVED => Some(Class::Starved),
        WRONG_TREE => Some(Class::Change),
        BASE_RED => Some(Class::Base),
        _ => None,
    }
}

/// What a hold's kind blames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Names {
    Runner,
    Change,
    Base,
}

/// An engine reaction observed by a world (blame.md, 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reaction {
    /// A gate run again after `run` ended, bounded per gate key.
    GateRetry { gate: String, run: String, bound: u64 },
    /// A repair asked of `change` for its red at `candidate`, on `base`.
    Repair { change: String, candidate: String, base: String },
    /// The branch's own repair, asked once for its red at `base`.
    BaseRepair { base: String },
    /// A hold or pause of `object`, blaming what `names` says.
    Hold { object: String, names: Names, run: Option<String>, candidate: Option<String>, base: Option<String> },
}

/// A reaction and the world's moment at which it was observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    pub at: Time,
    pub reaction: Reaction,
}

impl Reaction {
    fn struck(&self) -> Struck {
        match self {
            Self::GateRetry { run, .. } => Struck { gate_run: Some(run.clone()), ..Struck::default() },
            Self::Repair { candidate, base, .. } => {
                Struck { candidate: Some(candidate.clone()), base: Some(base.clone()), ..Struck::default() }
            }
            Self::BaseRepair { base } => Struck { base: Some(base.clone()), ..Struck::default() },
            Self::Hold { run, candidate, base, .. } => {
                Struck { gate_run: run.clone(), candidate: candidate.clone(), base: base.clone(), ..Struck::default() }
            }
        }
    }

    fn description(&self) -> (&str, &str) {
        match self {
            Self::GateRetry { gate, .. } => ("gate retry", gate),
            Self::Repair { change, .. } => ("repair", change),
            Self::BaseRepair { base } => ("base repair", base),
            Self::Hold { object, .. } => ("hold", object),
        }
    }
}

/// Judges each reaction in order, counting what it attributes on the ledger
/// (`Counter::GateRetries`, `Counter::Repairs`); returns one line per finding.
#[must_use]
pub fn judge(ledger: &mut FaultLedger, observed: &[Observed], seed: u64) -> Vec<String> {
    let mut retries = BTreeMap::<&str, u64>::new();
    let mut findings = Vec::new();
    for observation in observed {
        let struck = observation.reaction.struck();
        let fallen = ledger.fallen(observation.at, &struck);
        let why = match &observation.reaction {
            Reaction::GateRetry { gate, bound, .. } => {
                let count = retries.entry(gate).or_default();
                *count = count.checked_add(1).expect("a run's gate retries fit u64");
                let mut why = count_faults(ledger, &fallen, Counter::GateRetries, Class::Runner);
                if *count > *bound {
                    why.push("gate retries exceed bound");
                }
                why
            }
            Reaction::Repair { candidate, .. } => {
                let faults = fallen
                    .iter()
                    .copied()
                    .filter(|entry| ledger.get(*entry).struck.candidate.as_ref() == Some(candidate))
                    .collect::<Vec<_>>();
                count_faults(ledger, &faults, Counter::Repairs, Class::Change)
            }
            Reaction::BaseRepair { .. } => count_faults(ledger, &fallen, Counter::Repairs, Class::Base),
            Reaction::Hold { names, .. } => {
                if fallen.iter().any(|entry| names_class(*names, class(ledger.get(*entry).kind))) {
                    Vec::new()
                } else {
                    vec!["no fault explains the named blame"]
                }
            }
        };
        if !why.is_empty() {
            let kinds = fallen
                .iter()
                .map(|entry| {
                    let kind = ledger.get(*entry).kind;
                    format!("{}/{}", kind.family, kind.name)
                })
                .collect::<Vec<_>>();
            let kinds = if kinds.is_empty() { "nothing".into() } else { kinds.join(", ") };
            let (reaction, object) = observation.reaction.description();
            findings.push(format!(
                "blame: {reaction} of {object} at {:?}, seed {seed}: {}; struck by {kinds}",
                observation.at,
                why.join("; ")
            ));
        }
    }
    findings
}

fn names_class(names: Names, cause: Option<Class>) -> bool {
    match cause {
        Some(Class::Runner | Class::Starved) => names == Names::Runner,
        Some(Class::Change) => names == Names::Change,
        Some(Class::Base) => names == Names::Base,
        None => false,
    }
}

fn count_faults(
    ledger: &mut FaultLedger,
    fallen: &[Injection],
    counter: Counter,
    expected: Class,
) -> Vec<&'static str> {
    let mut explained = false;
    let mut repeated = false;
    for entry in fallen {
        let cause = class(ledger.get(*entry).kind);
        let eligible = cause == Some(expected) || (expected == Class::Runner && cause == Some(Class::Starved));
        if eligible {
            explained = true;
            ledger.count(*entry, counter);
            repeated |= ledger.get(*entry).count(counter) > 1;
        }
    }
    let mut why = Vec::new();
    if !explained {
        why.push("no fault explains the reaction");
    }
    if repeated {
        why.push("fault received more than one reaction");
    }
    why
}
