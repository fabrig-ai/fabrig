//! Shared tooling and job bounds (protocol/services.md, 3).
use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
use skein_lib::Duration;

/// The checkout's configured bounds (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Checkout {
    /// Workspaces on the worker's disk: the cache's bound, and the most that
    /// may be held at once. A prepare that finds every one held is refused.
    pub workspaces: u32,
    /// Repositories a spec names: at least one.
    pub repositories: u32,
    /// The longest workstream key, repository name or remote, or branch.
    pub name_bytes: u32,
    /// The longest commit message, its title and body together.
    pub message_bytes: u32,
    /// Maximum originally or still conflicted paths per repository. Zero
    /// disables merge starts; other workspace preparation remains available.
    pub conflicts: u32,
    /// Maximum bytes of one full relative conflicted path.
    pub path_bytes: u32,
    /// How long an operation that reaches the forge may take: a clone, a
    /// fetch, a branch's creation, a push.
    pub remote_timeout: Duration,
    /// How long one on the worker's disk may take: making a workspace,
    /// checking out, merging, committing.
    pub local_timeout: Duration,
    /// Facts kept until the parent drains them. Beyond them, facts are dropped
    /// and counted.
    pub facts: u32,
}
/// The git component's configured bounds (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GitComponent {
    pub operations: u32,
    pub stdout_bytes: u32,
    pub stderr_bytes: u32,
    pub read_bytes: u32,
    pub sweep_entries: u32,
    pub sweep_bytes: u32,
}

/// Job files and error output (temper/docs/design/protocol/tooling.md, 6).
pub const TOOLING_FILE_BYTES: u32 = 1_048_576;
pub const TOOLING_LOG_BYTES: u32 = 65_536;
pub const TOOLING_PATH_BYTES: u32 = 4096;
pub const TOOLING_PATH: &[u8] = b"/usr/local/bin:/usr/bin:/bin";

/// The merge job's kind and bounds (temper/docs/design/domain/worker/jobs.md).
pub const MERGE_KIND: u32 = 3;
pub const MERGE_SPEC_BYTES: u32 = 4_096;
pub const MERGE_RESULT_BYTES: u32 = 512;
pub const MERGE_DEADLINE: Duration = Duration::from_secs(600);

/// The merge job's kind and bounds (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn merge_bound() -> JobBound {
    JobBound { kind: JobKind(MERGE_KIND), spec: MERGE_SPEC_BYTES, result: MERGE_RESULT_BYTES, deadline: MERGE_DEADLINE }
}

/// Declare merge only where the tasks child holds its bounds
/// (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn merge_jobs(jig: &jig_protocol_config::Limits) -> Box<[JobBound]> {
    let bound = merge_bound();
    if bound.spec <= jig.core.tasks.spec_bytes && bound.result <= jig.core.tasks.result_bytes {
        Box::new([bound])
    } else {
        Box::new([])
    }
}

/// The close job's bounds (temper/docs/design/domain/worker/jobs.md).
pub const CLOSE_SPEC_BYTES: u32 = 16_384;
pub const CLOSE_RESULT_BYTES: u32 = 512;
pub const CLOSE_DEADLINE: Duration = Duration::from_secs(600);
