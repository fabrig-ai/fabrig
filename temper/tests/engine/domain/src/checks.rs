//! The root world's quiet-point invariants and bounded-wait referee
//! (temper/docs/design/testing/engine-checks.md, 3).

use skein_lib::{Time, Wall};
use skein_world::inspect::{Inspection, Moment, check_tables, check_waits};

/// One run's last bounded waits and first finding, shared by every root driver.
#[derive(Debug, Hash)]
pub struct Checks {
    seed: u64,
    bounded: Vec<(String, String, Time)>,
    earliest: Option<Time>,
    // Wall corrections do not make an already observed wait start in the future.
    record_wall: Wall,
    finding: Option<String>,
}

impl Checks {
    #[must_use]
    pub fn new(seed: u64) -> Checks {
        Checks { seed, bounded: Vec::new(), earliest: None, record_wall: Wall::EPOCH, finding: None }
    }

    /// Remember a wall reading the core has seen, including before a cold restart.
    pub(crate) fn observe_wall(&mut self, wall: Wall) {
        self.record_wall = self.record_wall.max(wall);
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
        let mut inspection = Inspection::new("engine", temper_engine_inspect::KINDS);
        self.quiet_reusing(root, at, iteration, &mut inspection, others)
    }

    /// Refresh the engine inspection at the quiet point, leaving its tables for other checks.
    /// The labels of bounded waits move to the driver to report a bound between quiet points.
    pub fn quiet_reusing(
        &mut self,
        root: &temper_engine_domain::Domain,
        at: Time,
        iteration: u64,
        inspection: &mut Inspection,
        others: &[&Inspection],
    ) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        inspection.reset();
        temper_engine_inspect::inspect(root, inspection);
        let moment = Moment { seed: self.seed, at, iteration };
        let core = root.core();
        self.observe_wall(core.inspection_wall());
        let result = if let Some((task, why)) = root.forge().inspection_wait_refusal() {
            Err(inspection.finding(moment, &format!("task {task}"), &format!("wait report refused: {why:?}")))
        } else {
            let primary = [&*inspection];
            let combined;
            let inspections = if others.is_empty() {
                &primary[..]
            } else {
                combined = [&primary[..], others].concat();
                &combined
            };
            inspection
                .check_violation(moment)
                .and_then(|()| check_waits(inspections, moment))
                .and_then(|()| check_tables(inspections, moment))
                .and_then(|()| jig_inspect::records(core, moment, self.record_wall))
        };
        self.bounded.clear();
        self.earliest = None;
        for wait in inspection.waits_mut() {
            if let Some(bound) = wait.bound {
                self.earliest = Some(self.earliest.map_or(bound, |earliest| earliest.min(bound)));
                self.bounded.push((std::mem::take(&mut wait.waiter), std::mem::take(&mut wait.kind), bound));
            }
        }
        if let Err(finding) = &result {
            self.finding = Some(finding.clone());
        }
        result
    }

    /// The next moment a driver must visit before it can move past it.
    #[must_use]
    pub const fn next_bound(&self) -> Option<Time> {
        self.earliest
    }

    /// A bounded wait cannot be starved past its bound between quiet points.
    pub fn step(&mut self, at: Time, iteration: u64) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        if self.earliest.is_some_and(|bound| bound < at)
            && let Some((waiter, kind, bound)) = self.bounded.iter().find(|(_, _, bound)| *bound < at)
        {
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
