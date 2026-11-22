//! Sweeps and exclusions (`skein/docs/design/testing/sweeps.md`, 4 and 5).
//! An exclusion acts before a fault is injected, never on a finding.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{format, string::String, vec::Vec};

use crate::faults::{FaultLedger, Kind};

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
    fell: BTreeSet<Kind>,
    seeds: u64,
    left_out: BTreeMap<&'static str, u64>,
}

impl Sweep {
    #[must_use]
    pub fn new(name: &'static str) -> Sweep {
        Sweep {
            name,
            configured: BTreeSet::new(),
            exclusions: Vec::new(),
            fell: BTreeSet::new(),
            seeds: 0,
            left_out: BTreeMap::new(),
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

    /// Folds one seed's ledger: what fell, and what its exclusions left out.
    pub fn ran(&mut self, ledger: &FaultLedger) {
        self.seeds = self.seeds.checked_add(1).expect("a sweep's seed count fits u64");
        self.fell.extend(ledger.fell());
        for (scenario, _, _) in ledger.left_out_draws() {
            let count = self.left_out.entry(scenario).or_default();
            *count = count.checked_add(1).expect("a sweep's left-out draw count fits u64");
        }
    }

    /// Each configured fault fell in at least one seed folded.
    pub fn check_fell(&self) -> Result<(), String> {
        let mut findings = Vec::new();
        for kind in self.configured.difference(&self.fell) {
            findings
                .push(format!("sweep {}: {}/{} never fell in {} seeds", self.name, kind.family, kind.name, self.seeds));
        }
        if findings.is_empty() { Ok(()) } else { Err(findings.join("\n")) }
    }

    /// How many draws an exclusion left out, over the seeds folded.
    #[must_use]
    pub fn left_out(&self, scenario: &str) -> u64 {
        self.left_out.get(scenario).copied().unwrap_or(0)
    }
}
