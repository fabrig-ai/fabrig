//! Startup vocabulary shared with the component (protocol/services.md, 4.2).
use alloc::boxed::Box;
pub mod application;
pub mod checks;
mod shared;
pub use application::sign_in::url as sign_in_url;
pub use shared::{AGENT_WORDS, admission, admission_words};

/// The box supplies its in-process worker; an engine alone supplies none
/// (temper/docs/design/protocol/cluster.md, 1).
#[must_use]
pub fn core(engine: &jig_protocol_config::Engine, deployment: [u8; 16], seed: u64) -> jig_domain_core::Config {
    let mut core = jig_protocol_config::core(engine, deployment, seed);
    if core.workers_in_process {
        core.defaults.workers = Box::new([jig_domain_core::settings::WorkerSetting {
            number: 1,
            name: skein_lib::bytes::copy_of(b"local"),
            places: 4,
            digest: None,
            withdrawn: false,
        }]);
    }
    core
}
