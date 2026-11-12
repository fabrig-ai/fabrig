//! The worker settings (temper/docs/design/protocol/services.md, 5).
pub use crate::worker_settings::Agent;
use alloc::boxed::Box;
pub use temper_service_core::settings::{Adapter, Files, PROGRAM_PATH_BYTES, Program};
/// A worker's settings: jig's part and temper's (protocol/configuration/README.md,
/// section 3); 09 builds the worker from it.
#[derive(Debug)]
pub struct Worker {
    pub jig: jig_protocol_config::Worker,
    /// The worker's root directory: workspaces, trash and home under it.
    pub root: Box<[u8]>,
    /// The absolute path of the git program, or none on this machine.
    pub git: Option<Box<[u8]>>,
}
