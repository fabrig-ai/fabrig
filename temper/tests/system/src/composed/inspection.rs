//! Cross-root quiet-point checks (temper/docs/design/testing/checks.md, 6.1).
use super::World;
use skein_world::inspect::Inspection;

impl World {
    pub(super) fn composed_checks(&mut self) -> Result<(), String> {
        let mut jig = Inspection::new("jig", jig_inspect::KINDS);
        jig_inspect::tasks(self.root.tasks(), &mut jig);
        // Checkout tokens are local to each lane, so check each worker with the root and jig separately.
        for lane in &self.lanes {
            let mut worker = temper_worker_inspect::inspection();
            temper_worker_inspect::inspect(&lane.worker, &lane.worker_env.limits, &lane.disk, &mut worker);
            self.checks.quiet_with(&self.root, self.env.now, self.iteration, &[&jig, &worker])?;
        }
        self.workload_bounds();
        Ok(())
    }
}
