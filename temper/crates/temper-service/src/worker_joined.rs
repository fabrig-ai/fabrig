//! The worker seam in the composition (temper/docs/design/protocol/services.md, 6).
impl crate::Service {
    /// The worker domain, for a world to inspect before requesting shutdown.
    #[must_use]
    pub fn worker_domain(&self) -> Option<&temper_worker_domain::Domain> {
        match &self.worker {
            Some(worker) => Some(worker.domain()),
            None => None,
        }
    }
}
