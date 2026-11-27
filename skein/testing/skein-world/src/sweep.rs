//! Sweeps and exclusions (`skein/docs/design/testing/sweeps.md`, 4 and 5).
//! An exclusion acts before a fault is injected, never on a finding.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use crate::faults::{FaultLedger, Kind};
use crate::test_env::{Mode, mode};

pub(crate) mod cases;
mod shapes;

pub use cases::{Case, Test, cases, gate, input};
pub use shapes::{SHAPES_KEPT, Shapes};

/// Judge each seed in a range, then record the completed sweep.
pub fn each_seed<F: FnMut(u64)>(seeds: core::ops::Range<u64>, mut check: F) {
    let count = seeds.end.saturating_sub(seeds.start);
    for seed in seeds {
        check(seed);
    }
    if count > 0 {
        skein_checks::count("seed", count);
    }
}

/// One ignored scenario's combination of fault, workload and state.
#[derive(Debug)]
pub struct Exclusion {
    /// The ignored scenario it names: its test function's name.
    pub scenario: &'static str,
    /// The work that fixes it, as the draft's section 7 names it.
    pub fixed_by: &'static str,
    /// The fault it leaves out.
    pub fault: Kind,
    /// Each condition of workload or state that must hold, in the world's words.
    pub when: &'static [&'static str],
}

impl Exclusion {
    #[must_use]
    pub const fn new(
        scenario: &'static str,
        fixed_by: &'static str,
        fault: Kind,
        when: &'static [&'static str],
    ) -> Exclusion {
        Exclusion { scenario, fixed_by, fault, when }
    }
}

/// Configured faults, exclusions and coverage folded over a sweep's seeds.
#[derive(Debug)]
pub struct Sweep {
    name: &'static str,
    configured: BTreeSet<Kind>,
    exclusions: Vec<Exclusion>,
    fell: BTreeMap<Kind, u64>,
    seeds: u64,
    left_out: BTreeMap<&'static str, u64>,
    ineligible: BTreeMap<Kind, u64>,
}

impl Sweep {
    #[must_use]
    pub fn new(name: &'static str) -> Sweep {
        Sweep {
            name,
            configured: BTreeSet::new(),
            exclusions: Vec::new(),
            fell: BTreeMap::new(),
            seeds: 0,
            left_out: BTreeMap::new(),
            ineligible: BTreeMap::new(),
        }
    }

    /// A fault the sweep configures: one that must fall in at least one seed.
    pub fn configure(&mut self, fault: Kind) {
        self.configured.insert(fault);
    }

    /// Refuses exclusions that name no scenario, work or combination, a fault
    /// not configured, or an already excluded combination for that scenario.
    pub fn exclude(&mut self, exclusion: Exclusion) -> Result<(), String> {
        let scenario = exclusion.scenario;
        let refusal = |why| format!("sweep {}: exclusion {scenario:?}: {why}", self.name);
        if !scenario.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            || !scenario.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(refusal("names no ignored scenario"));
        }
        if exclusion.fixed_by.is_empty() {
            return Err(refusal("names no work that fixes it"));
        }
        if exclusion.when.is_empty() {
            return Err(refusal("leaves out the whole fault, not a combination"));
        }
        if !self.configured.contains(&exclusion.fault) {
            return Err(refusal("fault is not configured"));
        }
        if self.exclusions.iter().any(|prior| {
            prior.scenario == scenario
                && prior.fault == exclusion.fault
                && prior.when.iter().all(|condition| exclusion.when.contains(condition))
                && exclusion.when.iter().all(|condition| prior.when.contains(condition))
        }) {
            return Err(refusal("duplicates the scenario, fault and conditions"));
        }
        self.exclusions.push(exclusion);
        Ok(())
    }

    /// The first exclusion whose fault matches and whose conditions all hold.
    #[must_use]
    pub fn leaves_out(&self, fault: Kind, holding: &[&str]) -> Option<&Exclusion> {
        self.exclusions.iter().find(|exclusion| {
            exclusion.fault == fault && exclusion.when.iter().all(|condition| holding.contains(condition))
        })
    }

    /// Folds one seed's ledger: the kinds that fell, the kinds without an eligible target,
    /// and what its exclusions left out.
    /// Counts a `seed` check only outside a `cases` check, which counts its own case.
    pub fn ran(&mut self, ledger: &FaultLedger) {
        self.ran_in(ledger, |n| skein_checks::count("seed", n));
    }

    pub(crate) fn ran_in(&mut self, ledger: &FaultLedger, count: impl FnOnce(u64)) {
        self.seeds = self.seeds.checked_add(1).expect("a sweep's seed count fits u64");
        for kind in ledger.fell() {
            let count = self.fell.entry(kind).or_default();
            *count = count.checked_add(1).expect("a fault's case count fits u64");
        }
        for (scenario, _, _) in ledger.left_out_draws() {
            let count = self.left_out.entry(scenario).or_default();
            *count = count.checked_add(1).expect("a sweep's left-out draw count fits u64");
        }
        for &kind in ledger.ineligible_kinds() {
            let count = self.ineligible.entry(kind).or_default();
            *count = count.checked_add(1).expect("a fault's ineligible case count fits u64");
        }
        if !cases::in_case() {
            count(1);
        }
    }

    /// At the gate, each configured fault fell in at least one seed folded.
    /// Outside it, passes and writes reach, followed by any ineligible case counts.
    pub fn check_fell(&self) -> Result<(), String> {
        self.check_fell_in(mode(), cases::stderr)
    }

    pub(crate) fn check_fell_in<R: FnMut(fmt::Arguments<'_>)>(&self, mode: Mode, mut report: R) -> Result<(), String> {
        let mut findings = Vec::new();
        for kind in &self.configured {
            let count = self.fell.get(kind).copied().unwrap_or(0);
            if !mode.gate() {
                report(format_args!("fuzzy reach: {} {}/{} {count}", self.name, kind.family, kind.name));
            } else if count == 0 {
                findings.push(format!(
                    "sweep {}: {}/{} never fell in {} seeds",
                    self.name, kind.family, kind.name, self.seeds
                ));
            }
        }
        if !mode.gate() {
            for kind in &self.configured {
                let count = self.ineligible(*kind);
                if count > 0 {
                    report(format_args!("fuzzy reach: {} ineligible/{}/{} {count}", self.name, kind.family, kind.name));
                }
            }
        }
        if findings.is_empty() { Ok(()) } else { Err(findings.join("\n")) }
    }

    /// How many draws an exclusion left out, over the seeds folded.
    #[must_use]
    pub fn left_out(&self, scenario: &str) -> u64 {
        self.left_out.get(scenario).copied().unwrap_or(0)
    }

    /// How many of the cases folded had no eligible target for `fault`.
    #[must_use]
    pub fn ineligible(&self, fault: Kind) -> u64 {
        self.ineligible.get(&fault).copied().unwrap_or(0)
    }
}
