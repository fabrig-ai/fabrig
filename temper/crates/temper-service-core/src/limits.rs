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
    /// Preparations active at once on a worker; the rest wait in arrival order.
    pub preparations: u32,
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

const CHECKOUT: Checkout = Checkout {
    // Cached or held checkout workspaces, from the checkout world fixture.
    workspaces: 3,
    // One preparation at a time per worker: redeploys.md, 6.4.
    preparations: 1,
    // Held to the agent's two repository directories.
    repositories: 2,
    // Bytes in a checkout key, name, remote or branch.
    name_bytes: 256,
    // Holds the agent's largest commit-message argument.
    message_bytes: 4096,
    // Conflicted paths per repository, for a merge of a few hundred paths.
    conflicts: 512,
    // Bytes per conflicted path, with its NUL in the git component's stdout.
    path_bytes: 255,
    // Time for an operation reaching a forge, from the checkout world fixture.
    remote_timeout: Duration::from_secs(60),
    // Time for a local checkout operation, from the checkout world fixture.
    local_timeout: Duration::from_secs(10),
    // Facts awaiting drain, from the checkout world fixture.
    facts: 256,
};
const GIT: GitComponent = GitComponent {
    // One operation per cached workspace, plus the component's startup and sweep.
    operations: 3,
    // At least every conflict path including its terminating NUL.
    stdout_bytes: 131_072,
    // The tail retained for classification and the checkout's diagnostic.
    stderr_bytes: 512,
    // Each stdout/stderr demand, bounded further by io's read size.
    read_bytes: 4096,
    // One page of trash entries held while sweeping.
    sweep_entries: 16,
    // Every name in that page at the kernel's longest name.
    sweep_bytes: 4080,
};

/// Shared profile parts and peer scalars (protocol/services.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    Small,
    Standard,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Peer {
    pub forges: u32,
    pub installations: u32,
    pub token_bytes: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shared {
    pub checkout: Checkout,
    pub git_component: GitComponent,
    pub peer: Peer,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
}
pub const FORGE_INSTALLATIONS: u32 = 8;
pub const FORGE_TOKEN_BYTES: u32 = 512;
#[must_use]
pub const fn profile(profile: Profile) -> Shared {
    match profile {
        Profile::Small | Profile::Standard => Shared {
            checkout: CHECKOUT,
            git_component: GIT,
            peer: Peer { forges: 3, installations: FORGE_INSTALLATIONS, token_bytes: FORGE_TOKEN_BYTES },
        },
    }
}
/// Every override known to a document, including the other owners' names.
pub const NAMES: &[&str] = &[
    "sign_in.members",
    "sign_in.pending",
    "sign_in.deadline_ms",
    "sign_in.fetches",
    "sign_in.body_bytes",
    "sign_in.url_bytes",
    "sign_in.secret_bytes",
    "picker.page",
    "routes.query_bytes",
    "root.journal.commits",
    "root.journal.held",
    "root.journal.writes",
    "root.journal.deliveries",
    "root.journal.transcript_bytes",
    "root.journal.result_bytes",
    "root.journal.run_bytes",
    "root.loads.loads",
    "root.loads.rows",
    "root.loads.bytes",
    "root.loads.read_bytes",
    "root.loads.reply_bytes",
    "root.loads.transcript_bytes",
    "root.brief.briefs",
    "root.brief.sections",
    "root.brief.parts",
    "root.brief.read_bytes",
    "root.brief.brief_bytes",
    "root.brief.gather_ms",
    "root.brief.budgets.task",
    "root.brief.budgets.dependencies",
    "root.brief.budgets.document",
    "root.brief.budgets.ci",
    "root.brief.budgets.reviews",
    "root.brief.budgets.pull",
    "root.brief.budgets.attempts",
    "root.brief.budgets.plan",
    "root.brief.budgets.notes",
    "root.brief.budgets.preferences",
    "root.brief.budgets.now",
    "forge.forges",
    "forge.repositories",
    "forge.tasks",
    "forge.holds",
    "forge.subscriptions",
    "forge.follows",
    "forge.follow_paths",
    "forge.follow_queue",
    "forge.follow_merge",
    "forge.follow_commits",
    "forge.follow_files",
    "forge.follow_message_bytes",
    "forge.follow_skips",
    "forge.follow_reason_bytes",
    "forge.entries",
    "forge.named_calls",
    "forge.changes",
    "forge.issues",
    "forge.resources_per_task",
    "forge.paths_per_subscription",
    "forge.name_bytes",
    "forge.output",
    "forge.facts",
    "forge.makings",
    "forge.adoptions",
    "forge.applications",
    "forge.collaborators",
    "forge.dependencies",
    "forge.changes_rows",
    "forge.changes_landed",
    "forge.landings_kept",
    "forge.landings",
    "forge.decided_per_project",
    "forge.judge_projects",
    "forge.judge_criteria",
    "forge.landing_rules",
    "forge.brief_sections",
    "forge.brief_bytes",
    "forge.tiers.tiers",
    "forge.tiers.failing_tests",
    "forge.tiers.test_id_bytes",
    "forge.sitting_ms",
    "forge.queue_window_ms",
    "forge.client.pending",
    "forge.client.calls",
    "forge.client.rate",
    "forge.client.reserve",
    "forge.client.window_ms",
    "forge.client.op_bytes",
    "forge.client.answer_bytes",
    "forge.client.rows",
    "forge.client.inbox",
    "forge.client.read_attempts",
    "forge.client.backoff_ms",
    "forge.client.backoff_max_ms",
    "forge.client.entries",
    "forge.client.write_attempts",
    "forge.client.lifetime_ms",
    "forge.client.clock_margin_ms",
    "forge.client.resources",
    "forge.client.repositories",
    "forge.client.poll_ms",
    "forge.client.poll_max_ms",
    "forge.client.hinted_ms",
    "forge.client.slow_ms",
    "forge.client.settle_ms",
    "forge.client.fallback_ms",
    "forge.client.facts",
    "forge.change_policy.repairs",
    "forge.change_policy.resolutions",
    "forge.change_policy.reruns",
    "forge.change_policy.updates",
    "forge.change_policy.questions",
    "forge.change_policy.tasks",
    "forge.change_policy.stall_ms",
    "forge.change_policy.gates",
    "forge.change_policy.clean_heads",
    "forge.issue_policy.plan_items",
    "forge.issue_policy.milestones",
    "forge.issue_policy.title_bytes",
    "forge.issue_policy.body_bytes",
    "forge.issue_policy.comment_bytes",
    "forge.issue_policy.interval_ms",
    "checkout.workspaces",
    "checkout.preparations",
    "checkout.repositories",
    "checkout.name_bytes",
    "checkout.message_bytes",
    "checkout.conflicts",
    "checkout.path_bytes",
    "checkout.remote_timeout_ms",
    "checkout.local_timeout_ms",
    "checkout.facts",
    "git_component.operations",
    "git_component.stdout_bytes",
    "git_component.stderr_bytes",
    "git_component.read_bytes",
    "git_component.sweep_entries",
    "git_component.sweep_bytes",
    "clients.bundle_files",
    "clients.bundle_bytes",
    "records.value_bytes",
    "forges.installations",
    "forges.log_origins",
    "forge_component.log_origins",
    "forge_component.log_poll_ms",
    "forge_component.log_catch_up_ms",
    "forge_component.log_pages",
    "forge_component.delivery_bytes",
    "forge_component.installations",
    "forge_component.key_bytes",
    "forge_component.renew_before_ms",
    "forge_component.forges",
    "forge_component.token_bytes",
    "forge_component.connections",
    "forge_component.head_bytes",
    "forge_component.headers",
    "forge_component.body_bytes_in",
    "forge_component.depth",
    "forge_component.chunk",
    "forge_component.string_bytes",
    "forge_component.body_bytes",
    "forge_component.inner",
    "forge_component.target_bytes",
    "forge_component.repositories",
    "forge_component.users",
    "forge_component.tls.read",
    "forge_component.tls.send",
    "forge_component.tls.records",
    "forge_component.hooks",
    "forge_component.hook_head_bytes",
    "forge_component.hook_body_bytes",
    "forge_component.hook_secret_bytes",
    "forge_component.hook_deadline_ms",
    "forge_component.deadline_ms",
    "forge_component.idle_ms",
    "forge_component.start_backoff_ms",
    "forge_component.start_backoff_max_ms",
    "box.backoff_ms",
    "box.backoff_max_ms",
    "box.restarts",
    "box.window_ms",
    "box.grace_ms",
    "box.line_bytes",
    "box.log_bytes",
    "box.tail_lines",
    "box.groups",
    "setup.deadline_ms",
    "setup.backoff_ms",
    "setup.backoff_max_ms",
    "setup.runs",
    "setup.states",
    "setup.state_life_ms",
    "setup.fetch_bytes",
];
pub const SHARED_NAMES: &[&str] = &[
    "forge.forges",
    "forges.installations",
    "forge_component.token_bytes",
    "checkout.workspaces",
    "checkout.preparations",
    "checkout.repositories",
    "checkout.name_bytes",
    "checkout.message_bytes",
    "checkout.conflicts",
    "checkout.path_bytes",
    "checkout.remote_timeout_ms",
    "checkout.local_timeout_ms",
    "checkout.facts",
    "git_component.operations",
    "git_component.stdout_bytes",
    "git_component.stderr_bytes",
    "git_component.read_bytes",
    "git_component.sweep_entries",
    "git_component.sweep_bytes",
];
fn duration(value: u64) -> Result<Duration, Refused> {
    match value.checked_mul(1_000_000) {
        Some(nanos) => Ok(Duration::from_nanos(nanos)),
        None => Err(Refused::Range),
    }
}
/// Apply shared overrides; another known name's integer is taken unread.
pub fn apply(limits: &mut Shared, name: &[u8], value: u64) -> Result<(), Refused> {
    match name {
        b"forge.forges" => limits.peer.forges = u32::try_from(value).or(Err(Refused::Range))?,
        b"forges.installations" => limits.peer.installations = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.token_bytes" => {
            limits.peer.token_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"checkout.workspaces" => limits.checkout.workspaces = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.preparations" => limits.checkout.preparations = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.repositories" => limits.checkout.repositories = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.name_bytes" => limits.checkout.name_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.message_bytes" => limits.checkout.message_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.conflicts" => limits.checkout.conflicts = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.path_bytes" => limits.checkout.path_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"checkout.remote_timeout_ms" => limits.checkout.remote_timeout = duration(value)?,
        b"checkout.local_timeout_ms" => limits.checkout.local_timeout = duration(value)?,
        b"checkout.facts" => limits.checkout.facts = u32::try_from(value).or(Err(Refused::Range))?,
        b"git_component.operations" => {
            limits.git_component.operations = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"git_component.stdout_bytes" => {
            limits.git_component.stdout_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"git_component.stderr_bytes" => {
            limits.git_component.stderr_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"git_component.read_bytes" => {
            limits.git_component.read_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"git_component.sweep_entries" => {
            limits.git_component.sweep_entries = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"git_component.sweep_bytes" => {
            limits.git_component.sweep_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        _ => {
            if !known(name) {
                return Err(Refused::Unknown);
            }
        }
    }
    Ok(())
}
#[must_use]
pub fn known(name: &[u8]) -> bool {
    for known in NAMES {
        if known.as_bytes() == name {
            return true;
        }
    }
    false
}
#[must_use]
pub fn shared_name(name: &[u8]) -> bool {
    for known in SHARED_NAMES {
        if known.as_bytes() == name {
            return true;
        }
    }
    false
}
