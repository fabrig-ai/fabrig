//! The worker seam in the composition (temper/docs/design/protocol/services.md, 6).
use alloc::boxed::Box;
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

impl crate::Service {
    pub fn configure_forge_runtime(&mut self, github: crate::ForgeRuntime, roots: skein_tls::Config) { self.engine.as_mut().expect("composition engine").configure_forge_runtime(github, roots); }
    #[must_use]
    #[expect(clippy::type_complexity, reason = "startup resolution returns numbered host and port tuples")]
    pub fn unresolved(&self) -> Box<[(u16, Box<[u8]>, u16)]> { self.engine.as_ref().expect("composition engine").unresolved() }
    pub fn resolved(&mut self, forge: u16, address: Option<core::net::SocketAddr>) { self.engine.as_mut().expect("composition engine").resolved(forge, address); }
}
