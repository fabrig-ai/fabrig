//! The root world's quiet-point invariants and bounded-wait referee
//! (temper/docs/design/testing/engine-checks.md, 3).

use skein_lib::Time;
use skein_world::inspect::{Inspection, Moment, check_tables, check_waits};

/// One run's last bounded waits and first finding, shared by every root driver.
#[derive(Debug, Hash)]
pub struct Checks {
    seed: u64,
    bounded: Vec<(String, String, Time)>,
    finding: Option<String>,
}

impl Checks {
    #[must_use]
    pub fn new(seed: u64) -> Checks {
        Checks { seed, bounded: Vec::new(), finding: None }
    }

    /// Inspect at the driver's quiet point, replacing the previous bounded waits.
    pub fn quiet(&mut self, root: &temper_engine_domain::Domain, at: Time, iteration: u64) -> Result<(), String> {
        self.quiet_with(root, at, iteration, &[])
    }

    /// Check a composed world's other roots alongside the same engine snapshot.
    pub fn quiet_with(
        &mut self,
        root: &temper_engine_domain::Domain,
        at: Time,
        iteration: u64,
        others: &[&Inspection],
    ) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        temper_engine_inspect::inspect(root, &mut inspection);
        let moment = Moment { seed: self.seed, at, iteration };
        let mut inspections = Vec::with_capacity(others.len() + 1);
        inspections.push(&inspection);
        inspections.extend_from_slice(others);
        let result = check_waits(&inspections, moment).and_then(|()| check_tables(&inspections, moment));
        self.bounded.clear();
        self.bounded.extend(
            inspection
                .waits()
                .iter()
                .filter_map(|wait| wait.bound.map(|bound| (wait.waiter.clone(), wait.kind.clone(), bound))),
        );
        if let Err(finding) = &result {
            self.finding = Some(finding.clone());
        }
        result
    }

    /// The next moment a driver must visit before it can move past it.
    #[must_use]
    pub fn next_bound(&self) -> Option<Time> {
        self.bounded.iter().map(|(_, _, bound)| *bound).min()
    }

    /// A bounded wait cannot be starved past its bound between quiet points.
    pub fn step(&mut self, at: Time, iteration: u64) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        if let Some((waiter, kind, bound)) = self.bounded.iter().find(|(_, _, bound)| *bound < at) {
            let inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
            let finding = inspection.finding(
                Moment { seed: self.seed, at, iteration },
                waiter,
                &format!("waits on {kind} past bound {} ns with no quiet point since", bound.as_nanos()),
            );
            self.finding = Some(finding.clone());
            return Err(finding);
        }
        Ok(())
    }
}
