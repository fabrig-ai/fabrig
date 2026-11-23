//! Rehearse the forge's half of Link over a private store image at a committed batch.
use super::{Queue, StoreChecks, World, top};

// Most runs do no rehearsal; even the 128-commit sample story stays sparse.
pub(super) const ONE_IN: u64 = 1024;

impl World {
    /// Choose checks before the story's commits; the report counts only this policy's commits.
    pub fn store_checks(&mut self, checks: StoreChecks) {
        self.store_checks = checks;
    }

    #[must_use]
    pub fn store_report(&self) -> String {
        self.store_checks.report()
    }

    #[must_use]
    pub fn store_counts(&self) -> (u64, u64) {
        (self.store_checks.commits(), self.store_checks.checked())
    }

    /// A scripted writer's output batch, including deliberate writer defects.
    pub fn commit(&mut self, requests: Box<[top::Request]>) -> Result<(), String> {
        assert!(self.top_out.is_empty(), "previous output batch drained");
        for request in requests {
            self.top_out.push(request);
        }
        self.try_take_top()
    }

    pub(super) fn check_commit(&mut self) -> Result<(), String> {
        self.commits = self.commits.checked_add(1).expect("world commit count fits u64");
        if self.store_checks.due(self.commits) {
            let stored = self.stored.clone();
            let (mut restored, _) = self.restored(&stored);
            let mut output = Queue::uncapped();
            restored.start_links(&self.tasks, &self.env, &mut output).map_err(|refusal| {
                jig_conformance::store_check::finding(self.commits, "Link", &format!("{refusal:?}"))
            })?;
        }
        Ok(())
    }
}

impl Drop for World {
    fn drop(&mut self) {
        eprintln!("{}", self.store_report());
    }
}
