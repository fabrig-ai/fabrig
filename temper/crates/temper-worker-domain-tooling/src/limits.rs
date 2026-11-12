//! The worker's job limits (temper/docs/design/domain/worker/jobs.md;
//! temper/docs/design/protocol/tooling.md, 6).

use alloc::boxed::Box;

/// Bounds handed to every step by the parent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Limits {
    /// The request and result file bound: `tooling_file_bytes`.
    pub file_bytes: u32,
    /// Error output retained as detail: `tooling_log_bytes`.
    pub tail_bytes: u32,
    /// The longest command path, job directory or generated file path.
    pub path_bytes: u32,
    /// Jobs in flight at once.
    pub jobs: u32,
    /// The complete `PATH` value given to commands.
    pub path: Box<[u8]>,
}

/// Whether every bound and the command search path are nonzero.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.file_bytes != 0
        && limits.tail_bytes != 0
        && limits.path_bytes != 0
        && limits.jobs != 0
        && !limits.path.is_empty()
}

/// Why the capability's entries or limits cannot be used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Invalid {
    Limits,
    Kinds,
    Verb,
    Spec,
    Result,
}
