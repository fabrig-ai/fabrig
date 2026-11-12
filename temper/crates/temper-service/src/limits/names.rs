//! Dotted overrides and their reference rows (temper/docs/design/protocol/configuration/limits.md;
//! temper/docs/design/protocol/documentation.md, 8).
use super::{Limits, Refused};
use skein_lib::Duration;
fn duration(value: u64) -> Result<Duration, Refused> {
    match value.checked_mul(1_000_000) {
        Some(nanos) => Ok(Duration::from_nanos(nanos)),
        None => Err(Refused::Range),
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive table of scalar dotted field paths")]
pub(super) fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    if find(name).is_none() {
        return Err(Refused::Unknown);
    }
    if name.starts_with(b"setup.") {
        return match temper_forge_setup::limits::apply_host(&mut limits.setup, name, value) {
            Ok(()) => Ok(()),
            Err(temper_forge_setup::limits::Refused::Unknown) => Err(Refused::Unknown),
            Err(temper_forge_setup::limits::Refused::Range) => Err(Refused::Range),
        };
    }
    match name {
        b"sign_in.members" => limits.sign_in.members = u32::try_from(value).or(Err(Refused::Range))?,
        b"sign_in.pending" => limits.sign_in.pending = u32::try_from(value).or(Err(Refused::Range))?,
        b"sign_in.deadline_ms" => limits.sign_in.deadline = duration(value)?,
        b"sign_in.fetches" => limits.sign_in.fetches = u32::try_from(value).or(Err(Refused::Range))?,
        b"sign_in.body_bytes" => limits.sign_in.body_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"sign_in.url_bytes" => limits.sign_in.url_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"sign_in.secret_bytes" => limits.sign_in.secret_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"picker.page" => limits.picker.page = u32::try_from(value).or(Err(Refused::Range))?,
        b"routes.query_bytes" => limits.routes.query_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.journal.commits" => limits.root.journal.commits = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.journal.held" => limits.root.journal.held = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.journal.writes" => limits.root.journal.writes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.journal.deliveries" => limits.root.journal.deliveries = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.journal.transcript_bytes" => {
            limits.root.journal.transcript_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.journal.result_bytes" => {
            limits.root.journal.result_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.journal.run_bytes" => limits.root.journal.run_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.loads.loads" => limits.root.loads.loads = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.loads.rows" => limits.root.loads.rows = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.loads.bytes" => limits.root.loads.bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.loads.read_bytes" => limits.root.loads.read_bytes = value,
        b"root.loads.reply_bytes" => limits.root.loads.reply_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.loads.transcript_bytes" => {
            limits.root.loads.transcript_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.briefs" => limits.root.brief.briefs = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.sections" => limits.root.brief.sections = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.parts" => limits.root.brief.parts = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.read_bytes" => limits.root.brief.read_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.brief_bytes" => limits.root.brief.brief_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.gather_ms" => limits.root.brief.gather = duration(value)?,
        b"root.brief.budgets.task" => limits.root.brief.budgets.task = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.budgets.dependencies" => {
            limits.root.brief.budgets.dependencies = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.document" => {
            limits.root.brief.budgets.document = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.ci" => limits.root.brief.budgets.ci = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.budgets.reviews" => {
            limits.root.brief.budgets.reviews = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.pull" => limits.root.brief.budgets.pull = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.budgets.attempts" => {
            limits.root.brief.budgets.attempts = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.plan" => limits.root.brief.budgets.plan = u32::try_from(value).or(Err(Refused::Range))?,
        b"root.brief.budgets.notes" => {
            limits.root.brief.budgets.notes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.preferences" => {
            limits.root.brief.budgets.preferences = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"root.brief.budgets.now" => {
            limits.root.brief.budgets.now = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.forges" => limits.forge.forges = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.repositories" => limits.forge.repositories = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.tasks" => limits.forge.tasks = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.holds" => limits.forge.holds = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.subscriptions" => limits.forge.subscriptions = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.entries" => limits.forge.entries = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.named_calls" => limits.forge.named_calls = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.changes" => limits.forge.changes = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.issues" => limits.forge.issues = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.resources_per_task" => {
            limits.forge.resources_per_task = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.paths_per_subscription" => {
            limits.forge.paths_per_subscription = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.name_bytes" => limits.forge.name_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.output" => limits.forge.output = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.facts" => limits.forge.facts = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.makings" => {
            if value == 0 || value > 64 {
                return Err(Refused::Range);
            }
            limits.forge.makings = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.adoptions" => limits.forge.adoptions = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.applications" => limits.forge.applications = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.collaborators" => limits.forge.collaborators = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.changes_rows" => limits.forge.changes_rows = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.changes_landed" => limits.forge.changes_landed = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.landings_kept" => limits.forge.landings_kept = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.landings" => limits.forge.landings = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.decided_per_project" => {
            limits.forge.decided_per_project = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.judge_projects" => limits.forge.judge_projects = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.judge_criteria" => limits.forge.judge_criteria = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.dependencies" => limits.forge.dependencies = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follows" => limits.forge.follows = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_paths" => limits.forge.follow_paths = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_queue" => limits.forge.follow_queue = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_merge" => limits.forge.follow_merge = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_commits" => limits.forge.follow_commits = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_files" => limits.forge.follow_files = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_message_bytes" => {
            limits.forge.follow_message_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.follow_skips" => limits.forge.follow_skips = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.follow_reason_bytes" => {
            limits.forge.follow_reason_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.landing_rules" => limits.forge.landing_rules = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.brief_sections" => limits.forge.brief_sections = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.brief_bytes" => limits.forge.brief_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.tiers.tiers" => limits.forge.tiers.tiers = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.tiers.failing_tests" => {
            limits.forge.tiers.failing_tests = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.tiers.test_id_bytes" => {
            limits.forge.tiers.test_id_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.sitting_ms" => limits.forge.sitting = duration(value)?,
        b"forge.queue_window_ms" => limits.forge.queue_window = duration(value)?,
        b"forge.client.pending" => limits.forge.client.pending = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.calls" => limits.forge.client.calls = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.rate" => limits.forge.client.rate = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.reserve" => limits.forge.client.reserve = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.window_ms" => limits.forge.client.window = duration(value)?,
        b"forge.client.op_bytes" => limits.forge.client.op_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.answer_bytes" => {
            limits.forge.client.answer_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.client.rows" => limits.forge.client.rows = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.inbox" => limits.forge.client.inbox = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.read_attempts" => {
            limits.forge.client.read_attempts = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.client.backoff_ms" => limits.forge.client.backoff = duration(value)?,
        b"forge.client.backoff_max_ms" => limits.forge.client.backoff_max = duration(value)?,
        b"forge.client.entries" => limits.forge.client.entries = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.write_attempts" => {
            limits.forge.client.write_attempts = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.client.lifetime_ms" => limits.forge.client.lifetime = duration(value)?,
        b"forge.client.clock_margin_ms" => limits.forge.client.clock_margin = duration(value)?,
        b"forge.client.resources" => limits.forge.client.resources = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.client.repositories" => {
            limits.forge.client.repositories = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.client.poll_ms" => limits.forge.client.poll = duration(value)?,
        b"forge.client.poll_max_ms" => limits.forge.client.poll_max = duration(value)?,
        b"forge.client.hinted_ms" => limits.forge.client.hinted = duration(value)?,
        b"forge.client.slow_ms" => limits.forge.client.slow = duration(value)?,
        b"forge.client.settle_ms" => limits.forge.client.settle = duration(value)?,
        b"forge.client.fallback_ms" => limits.forge.client.fallback = duration(value)?,
        b"forge.client.facts" => limits.forge.client.facts = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge.change_policy.repairs" => {
            limits.forge.change_policy.repairs = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.resolutions" => {
            limits.forge.change_policy.resolutions = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.reruns" => {
            limits.forge.change_policy.reruns = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.updates" => {
            limits.forge.change_policy.updates = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.questions" => {
            limits.forge.change_policy.questions = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.tasks" => {
            if value == 0 {
                return Err(Refused::Range);
            }
            limits.forge.change_policy.tasks = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.stall_ms" => limits.forge.change_policy.stall = duration(value)?,
        b"forge.change_policy.gates" => {
            limits.forge.change_policy.gates = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.change_policy.clean_heads" => {
            limits.forge.change_policy.clean_heads = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.plan_items" => {
            limits.forge.issue_policy.plan_items = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.milestones" => {
            limits.forge.issue_policy.milestones = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.title_bytes" => {
            limits.forge.issue_policy.title_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.body_bytes" => {
            limits.forge.issue_policy.body_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.comment_bytes" => {
            limits.forge.issue_policy.comment_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge.issue_policy.interval_ms" => limits.forge.issue_policy.interval = duration(value)?,
        b"checkout.workspaces" => limits.checkout.workspaces = u32::try_from(value).or(Err(Refused::Range))?,
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
        b"clients.bundle_files" => limits.clients.bundle_files = u32::try_from(value).or(Err(Refused::Range))?,
        b"clients.bundle_bytes" => limits.clients.bundle_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"records.value_bytes" => limits.records.value_bytes = u32::try_from(value).or(Err(Refused::Range))?,
        b"forges.installations" => limits.forges.installations = u32::try_from(value).or(Err(Refused::Range))?,
        b"forges.log_origins" => limits.forges.log_origins = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.log_origins" => {
            limits.forge_component.log_origins = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.log_poll_ms" => limits.forge_component.log_poll = duration(value)?,
        b"forge_component.log_catch_up_ms" => limits.forge_component.log_catch_up = duration(value)?,
        b"forge_component.log_pages" => {
            limits.forge_component.log_pages = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.delivery_bytes" => {
            limits.forge_component.delivery_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.installations" => {
            limits.forge_component.installations = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.key_bytes" => {
            limits.forge_component.key_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.renew_before_ms" => limits.forge_component.renew_before = duration(value)?,
        b"forge_component.forges" => limits.forge_component.forges = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.token_bytes" => {
            limits.forge_component.token_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.connections" => {
            limits.forge_component.connections = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.head_bytes" => {
            limits.forge_component.head_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.headers" => limits.forge_component.headers = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.body_bytes_in" => {
            limits.forge_component.body_bytes_in = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.depth" => limits.forge_component.depth = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.chunk" => limits.forge_component.chunk = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.string_bytes" => {
            limits.forge_component.string_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.body_bytes" => {
            limits.forge_component.body_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.inner" => limits.forge_component.inner = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.target_bytes" => {
            limits.forge_component.target_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.repositories" => {
            limits.forge_component.repositories = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.users" => limits.forge_component.users = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.tls.read" => {
            limits.forge_component.tls.read = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.tls.send" => {
            limits.forge_component.tls.send = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.tls.records" => {
            limits.forge_component.tls.records = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.hooks" => limits.forge_component.hooks = u32::try_from(value).or(Err(Refused::Range))?,
        b"forge_component.hook_head_bytes" => {
            limits.forge_component.hook_head_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.hook_body_bytes" => {
            limits.forge_component.hook_body_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.hook_secret_bytes" => {
            limits.forge_component.hook_secret_bytes = u32::try_from(value).or(Err(Refused::Range))?;
        }
        b"forge_component.hook_deadline_ms" => limits.forge_component.hook_deadline = duration(value)?,
        b"forge_component.deadline_ms" => limits.forge_component.deadline = duration(value)?,
        b"forge_component.idle_ms" => limits.forge_component.idle = duration(value)?,
        b"forge_component.start_backoff_ms" => limits.forge_component.start_backoff = duration(value)?,
        b"forge_component.start_backoff_max_ms" => limits.forge_component.start_backoff_max = duration(value)?,
        _ => return Err(Refused::Unknown),
    }
    Ok(())
}

/// What a named limit counts (temper/docs/design/protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Count,
    Bytes,
    Milliseconds,
}

/// One override name (temper/docs/design/protocol/documentation.md, 8).
#[derive(Clone, Copy, Debug)]
pub struct Name {
    pub name: &'static str,
    pub unit: Unit,
    /// What it bounds, one sentence of at most 120 bytes.
    pub line: &'static str,
    pub value: fn(&Limits) -> u64,
}

/// Every name the service override parser takes, including delegated setup host names.
pub static NAMES: [Name; 182] = [
    Name {
        name: "sign_in.members",
        unit: Unit::Count,
        line: "Bounds configured members and the forge connector's retained member bindings.",
        value: |limits| u64::from(limits.sign_in.members),
    },
    Name {
        name: "sign_in.pending",
        unit: Unit::Count,
        line: "Bounds simultaneous sign-ins; a start past this count gets the busy page.",
        value: |limits| u64::from(limits.sign_in.pending),
    },
    Name {
        name: "sign_in.deadline_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a sign-in's life from start through its return's answer and sets its cookie's lifetime.",
        value: |limits| limits.sign_in.deadline.as_millis(),
    },
    Name {
        name: "sign_in.fetches",
        unit: Unit::Count,
        line: "Bounds simultaneous exchanges with Google for sign-ins.",
        value: |limits| u64::from(limits.sign_in.fetches),
    },
    Name {
        name: "sign_in.body_bytes",
        unit: Unit::Bytes,
        line: "Bounds an answer body from Google, for a key set or a token.",
        value: |limits| u64::from(limits.sign_in.body_bytes),
    },
    Name {
        name: "sign_in.url_bytes",
        unit: Unit::Bytes,
        line: "Bounds a configured endpoint and the authorization URL given to the browser.",
        value: |limits| u64::from(limits.sign_in.url_bytes),
    },
    Name {
        name: "sign_in.secret_bytes",
        unit: Unit::Bytes,
        line: "Bounds the Google client secret.",
        value: |limits| u64::from(limits.sign_in.secret_bytes),
    },
    Name {
        name: "picker.page",
        unit: Unit::Count,
        line: "Bounds repositories taken by one picker read, within the forge's page cap of at most 100.",
        value: |limits| u64::from(limits.picker.page),
    },
    Name {
        name: "routes.query_bytes",
        unit: Unit::Bytes,
        line: "Bounds a forge route's query.",
        value: |limits| u64::from(limits.routes.query_bytes),
    },
    Name {
        name: "root.journal.commits",
        unit: Unit::Count,
        line: "Bounds commits awaiting the store's answer.",
        value: |limits| u64::from(limits.root.journal.commits),
    },
    Name {
        name: "root.journal.held",
        unit: Unit::Count,
        line: "Sets the retained held-delivery count; the uncapped journal does not use it for admission.",
        value: |limits| u64::from(limits.root.journal.held),
    },
    Name {
        name: "root.journal.writes",
        unit: Unit::Count,
        line: "Bounds unique writes per commit, including the saved deployment header; a decision has one fewer slot.",
        value: |limits| u64::from(limits.root.journal.writes),
    },
    Name {
        name: "root.journal.deliveries",
        unit: Unit::Count,
        line: "Sets the door output count used in the default admission watermark, without capping a decision.",
        value: |limits| u64::from(limits.root.journal.deliveries),
    },
    Name {
        name: "root.journal.transcript_bytes",
        unit: Unit::Bytes,
        line: "Bounds transcript bytes or a wrapped child row's owned bytes admitted to the journal.",
        value: |limits| u64::from(limits.root.journal.transcript_bytes),
    },
    Name {
        name: "root.journal.result_bytes",
        unit: Unit::Bytes,
        line: "Bounds owned result text or assignment section text admitted to a held delivery.",
        value: |limits| u64::from(limits.root.journal.result_bytes),
    },
    Name {
        name: "root.journal.run_bytes",
        unit: Unit::Bytes,
        line: "Bounds a run charter's owned bytes in a held assignment.",
        value: |limits| u64::from(limits.root.journal.run_bytes),
    },
    Name {
        name: "root.loads.loads",
        unit: Unit::Count,
        line: "Bounds issued store loads, including abandoned reads awaiting their terminal answer.",
        value: |limits| u64::from(limits.root.loads.loads),
    },
    Name {
        name: "root.loads.rows",
        unit: Unit::Count,
        line: "Bounds rows requested and accepted in one incoming store page.",
        value: |limits| u64::from(limits.root.loads.rows),
    },
    Name {
        name: "root.loads.bytes",
        unit: Unit::Bytes,
        line: "Bounds a kept page prefix, counting record slots and their owned bytes.",
        value: |limits| u64::from(limits.root.loads.bytes),
    },
    Name {
        name: "root.loads.read_bytes",
        unit: Unit::Bytes,
        line: "Bounds owned bytes one authenticated store read brings for a person.",
        value: |limits| limits.root.loads.read_bytes,
    },
    Name {
        name: "root.loads.reply_bytes",
        unit: Unit::Bytes,
        line: "Bounds a decoded store answer before its kept prefix is cut.",
        value: |limits| u64::from(limits.root.loads.reply_bytes),
    },
    Name {
        name: "root.loads.transcript_bytes",
        unit: Unit::Bytes,
        line: "Bounds a row's owned bytes, including transcripts and wrapped child rows, before the prefix is cut.",
        value: |limits| u64::from(limits.root.loads.transcript_bytes),
    },
    Name {
        name: "root.brief.briefs",
        unit: Unit::Count,
        line: "Bounds briefs the root gathers at once.",
        value: |limits| u64::from(limits.root.brief.briefs),
    },
    Name {
        name: "root.brief.sections",
        unit: Unit::Count,
        line: "Bounds brief sections the root gathers at once.",
        value: |limits| u64::from(limits.root.brief.sections),
    },
    Name {
        name: "root.brief.parts",
        unit: Unit::Count,
        line: "Bounds parts in one gathered brief section.",
        value: |limits| u64::from(limits.root.brief.parts),
    },
    Name {
        name: "root.brief.read_bytes",
        unit: Unit::Bytes,
        line: "Bounds bytes read for one gathered brief section.",
        value: |limits| u64::from(limits.root.brief.read_bytes),
    },
    Name {
        name: "root.brief.brief_bytes",
        unit: Unit::Bytes,
        line: "Bounds the body of a gathered brief.",
        value: |limits| u64::from(limits.root.brief.brief_bytes),
    },
    Name {
        name: "root.brief.gather_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the time spent gathering a brief.",
        value: |limits| limits.root.brief.gather.as_millis(),
    },
    Name {
        name: "root.brief.budgets.task",
        unit: Unit::Bytes,
        line: "Bounds task section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.task),
    },
    Name {
        name: "root.brief.budgets.dependencies",
        unit: Unit::Bytes,
        line: "Bounds dependency section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.dependencies),
    },
    Name {
        name: "root.brief.budgets.document",
        unit: Unit::Bytes,
        line: "Bounds document section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.document),
    },
    Name {
        name: "root.brief.budgets.ci",
        unit: Unit::Bytes,
        line: "Bounds CI section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.ci),
    },
    Name {
        name: "root.brief.budgets.reviews",
        unit: Unit::Bytes,
        line: "Bounds review section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.reviews),
    },
    Name {
        name: "root.brief.budgets.pull",
        unit: Unit::Bytes,
        line: "Bounds pull request section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.pull),
    },
    Name {
        name: "root.brief.budgets.attempts",
        unit: Unit::Bytes,
        line: "Bounds attempt section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.attempts),
    },
    Name {
        name: "root.brief.budgets.plan",
        unit: Unit::Bytes,
        line: "Bounds plan section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.plan),
    },
    Name {
        name: "root.brief.budgets.notes",
        unit: Unit::Bytes,
        line: "Bounds note section bytes in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.notes),
    },
    Name {
        name: "root.brief.budgets.preferences",
        unit: Unit::Bytes,
        line: "Bounds whole requester preferences in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.preferences),
    },
    Name {
        name: "root.brief.budgets.now",
        unit: Unit::Bytes,
        line: "Bounds a standing task's Now body in a gathered brief.",
        value: |limits| u64::from(limits.root.brief.budgets.now),
    },
    Name {
        name: "forge.forges",
        unit: Unit::Count,
        line: "Bounds retained forges of every kind and mode, counting an ended forge until its removal is whole.",
        value: |limits| u64::from(limits.forge.forges),
    },
    Name {
        name: "forge.repositories",
        unit: Unit::Count,
        line: "Bounds repositories retained by the forge connector, shared by every retained forge.",
        value: |limits| u64::from(limits.forge.repositories),
    },
    Name {
        name: "forge.tasks",
        unit: Unit::Count,
        line: "Bounds tasks participating in the forge connector.",
        value: |limits| u64::from(limits.forge.tasks),
    },
    Name {
        name: "forge.holds",
        unit: Unit::Count,
        line: "Bounds held task resources in the forge connector.",
        value: |limits| u64::from(limits.forge.holds),
    },
    Name {
        name: "forge.subscriptions",
        unit: Unit::Count,
        line: "Bounds subscriptions to forge topics.",
        value: |limits| u64::from(limits.forge.subscriptions),
    },
    Name {
        name: "forge.entries",
        unit: Unit::Count,
        line: "Bounds durable outbox entries in the forge connector.",
        value: |limits| u64::from(limits.forge.entries),
    },
    Name {
        name: "forge.named_calls",
        unit: Unit::Count,
        line: "Bounds retained named calls and their connector payloads, independently of outbox entries.",
        value: |limits| u64::from(limits.forge.named_calls),
    },
    Name {
        name: "forge.changes",
        unit: Unit::Count,
        line: "Bounds live changes retained by the forge connector.",
        value: |limits| u64::from(limits.forge.changes),
    },
    Name {
        name: "forge.issues",
        unit: Unit::Count,
        line: "Bounds live issue projections retained by the forge connector.",
        value: |limits| u64::from(limits.forge.issues),
    },
    Name {
        name: "forge.resources_per_task",
        unit: Unit::Count,
        line: "Bounds distinct forge names watched by one task, including held resources and subscribed branches.",
        value: |limits| u64::from(limits.forge.resources_per_task),
    },
    Name {
        name: "forge.paths_per_subscription",
        unit: Unit::Count,
        line: "Bounds paths retained by one forge subscription.",
        value: |limits| u64::from(limits.forge.paths_per_subscription),
    },
    Name {
        name: "forge.name_bytes",
        unit: Unit::Bytes,
        line: "Bounds names retained by the forge connector.",
        value: |limits| u64::from(limits.forge.name_bytes),
    },
    Name {
        name: "forge.output",
        unit: Unit::Count,
        line: "Bounds output records reserved for one forge step.",
        value: |limits| u64::from(limits.forge.output),
    },
    Name {
        name: "forge.facts",
        unit: Unit::Count,
        line: "Bounds facts awaiting the forge connector's drain.",
        value: |limits| u64::from(limits.forge.facts),
    },
    Name {
        name: "forge.makings",
        unit: Unit::Count,
        line: "Bounds repositories being made or failed, from 1 to 64.",
        value: |limits| u64::from(limits.forge.makings),
    },
    Name {
        name: "forge.adoptions",
        unit: Unit::Count,
        line: "Bounds repository adoptions in flight.",
        value: |limits| u64::from(limits.forge.adoptions),
    },
    Name {
        name: "forge.applications",
        unit: Unit::Count,
        line: "Bounds keyed adoption answers retained for request retention.",
        value: |limits| u64::from(limits.forge.applications),
    },
    Name {
        name: "forge.collaborators",
        unit: Unit::Count,
        line: "Bounds collaborators retained by the forge connector.",
        value: |limits| u64::from(limits.forge.collaborators),
    },
    Name {
        name: "forge.changes_rows",
        unit: Unit::Count,
        line: "Bounds queued rows across the changes document's queues.",
        value: |limits| u64::from(limits.forge.changes_rows),
    },
    Name {
        name: "forge.changes_landed",
        unit: Unit::Count,
        line: "Bounds recent landed changes in the changes document.",
        value: |limits| u64::from(limits.forge.changes_landed),
    },
    Name {
        name: "forge.landings_kept",
        unit: Unit::Count,
        line: "Bounds retained landings per repository.",
        value: |limits| u64::from(limits.forge.landings_kept),
    },
    Name {
        name: "forge.landings",
        unit: Unit::Count,
        line: "Bounds live landing procedure rows.",
        value: |limits| u64::from(limits.forge.landings),
    },
    Name {
        name: "forge.decided_per_project",
        unit: Unit::Count,
        line: "Bounds retained decided and asked rows per project.",
        value: |limits| u64::from(limits.forge.decided_per_project),
    },
    Name {
        name: "forge.judge_projects",
        unit: Unit::Count,
        line: "Bounds project judge tables retained beside committed policy.",
        value: |limits| u64::from(limits.forge.judge_projects),
    },
    Name {
        name: "forge.judge_criteria",
        unit: Unit::Count,
        line: "Bounds criteria in each deployment or project judge table.",
        value: |limits| u64::from(limits.forge.judge_criteria),
    },
    Name {
        name: "forge.dependencies",
        unit: Unit::Count,
        line: "Dependencies per project.",
        value: |limits| u64::from(limits.forge.dependencies),
    },
    Name {
        name: "forge.follows",
        unit: Unit::Count,
        line: "Follows per project.",
        value: |limits| u64::from(limits.forge.follows),
    },
    Name {
        name: "forge.follow_paths",
        unit: Unit::Count,
        line: "Path prefixes per follow.",
        value: |limits| u64::from(limits.forge.follow_paths),
    },
    Name {
        name: "forge.follow_queue",
        unit: Unit::Count,
        line: "Queued landings per follow.",
        value: |limits| u64::from(limits.forge.follow_queue),
    },
    Name {
        name: "forge.follow_merge",
        unit: Unit::Count,
        line: "Landings per follow change.",
        value: |limits| u64::from(limits.forge.follow_merge),
    },
    Name {
        name: "forge.follow_commits",
        unit: Unit::Count,
        line: "Commits retained per follow entry.",
        value: |limits| u64::from(limits.forge.follow_commits),
    },
    Name {
        name: "forge.follow_files",
        unit: Unit::Count,
        line: "Files retained per follow entry.",
        value: |limits| u64::from(limits.forge.follow_files),
    },
    Name {
        name: "forge.follow_message_bytes",
        unit: Unit::Bytes,
        line: "Message bytes retained per follow commit.",
        value: |limits| u64::from(limits.forge.follow_message_bytes),
    },
    Name {
        name: "forge.follow_skips",
        unit: Unit::Count,
        line: "Skips retained per follow.",
        value: |limits| u64::from(limits.forge.follow_skips),
    },
    Name {
        name: "forge.follow_reason_bytes",
        unit: Unit::Bytes,
        line: "Bytes per follow skip reason.",
        value: |limits| u64::from(limits.forge.follow_reason_bytes),
    },
    Name {
        name: "forge.landing_rules",
        unit: Unit::Count,
        line: "Bounds landing rules per scope.",
        value: |limits| u64::from(limits.forge.landing_rules),
    },
    Name {
        name: "forge.brief_sections",
        unit: Unit::Count,
        line: "Bounds forge brief sections gathered at once.",
        value: |limits| u64::from(limits.forge.brief_sections),
    },
    Name {
        name: "forge.brief_bytes",
        unit: Unit::Bytes,
        line: "Bounds bytes retained for one gathered forge brief section.",
        value: |limits| u64::from(limits.forge.brief_bytes),
    },
    Name {
        name: "forge.tiers.tiers",
        unit: Unit::Count,
        line: "Bounds tiers retained per repository.",
        value: |limits| u64::from(limits.forge.tiers.tiers),
    },
    Name {
        name: "forge.tiers.failing_tests",
        unit: Unit::Count,
        line: "Bounds failing tests reported per tier run.",
        value: |limits| u64::from(limits.forge.tiers.failing_tests),
    },
    Name {
        name: "forge.tiers.test_id_bytes",
        unit: Unit::Bytes,
        line: "Bounds a failing test's identifier in a tier's report.",
        value: |limits| u64::from(limits.forge.tiers.test_id_bytes),
    },
    Name {
        name: "forge.sitting_ms",
        unit: Unit::Milliseconds,
        line: "Sets the gap between a person's sittings on Home.",
        value: |limits| limits.forge.sitting.as_millis(),
    },
    Name {
        name: "forge.queue_window_ms",
        unit: Unit::Milliseconds,
        line: "Sets the time allowed to gather a landing queue.",
        value: |limits| limits.forge.queue_window.as_millis(),
    },
    Name {
        name: "forge.client.pending",
        unit: Unit::Count,
        line: "Bounds logical forge calls retained, queued and in flight.",
        value: |limits| u64::from(limits.forge.client.pending),
    },
    Name {
        name: "forge.client.calls",
        unit: Unit::Count,
        line: "Bounds forge calls in flight.",
        value: |limits| u64::from(limits.forge.client.calls),
    },
    Name {
        name: "forge.client.rate",
        unit: Unit::Count,
        line: "Bounds requests allowed in one forge rate window.",
        value: |limits| u64::from(limits.forge.client.rate),
    },
    Name {
        name: "forge.client.reserve",
        unit: Unit::Count,
        line: "Reserves requests in the forge rate window for writes.",
        value: |limits| u64::from(limits.forge.client.reserve),
    },
    Name {
        name: "forge.client.window_ms",
        unit: Unit::Milliseconds,
        line: "Sets the forge request rate window.",
        value: |limits| limits.forge.client.window.as_millis(),
    },
    Name {
        name: "forge.client.op_bytes",
        unit: Unit::Bytes,
        line: "Bounds owned bytes of one forge operation.",
        value: |limits| u64::from(limits.forge.client.op_bytes),
    },
    Name {
        name: "forge.client.answer_bytes",
        unit: Unit::Bytes,
        line: "Bounds decoded bytes of one forge answer.",
        value: |limits| u64::from(limits.forge.client.answer_bytes),
    },
    Name {
        name: "forge.client.rows",
        unit: Unit::Count,
        line: "Bounds rows in one forge answer page.",
        value: |limits| u64::from(limits.forge.client.rows),
    },
    Name {
        name: "forge.client.inbox",
        unit: Unit::Count,
        line: "Bounds comment and review delivery stamps retained per resource.",
        value: |limits| u64::from(limits.forge.client.inbox),
    },
    Name {
        name: "forge.client.read_attempts",
        unit: Unit::Count,
        line: "Bounds attempts of one forge read.",
        value: |limits| u64::from(limits.forge.client.read_attempts),
    },
    Name {
        name: "forge.client.backoff_ms",
        unit: Unit::Milliseconds,
        line: "Sets the first delay before a forge read is tried again.",
        value: |limits| limits.forge.client.backoff.as_millis(),
    },
    Name {
        name: "forge.client.backoff_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the delay before a forge read is tried again.",
        value: |limits| limits.forge.client.backoff_max.as_millis(),
    },
    Name {
        name: "forge.client.entries",
        unit: Unit::Count,
        line: "Bounds durable outbox entries in the forge client.",
        value: |limits| u64::from(limits.forge.client.entries),
    },
    Name {
        name: "forge.client.write_attempts",
        unit: Unit::Count,
        line: "Bounds attempts of one forge write.",
        value: |limits| u64::from(limits.forge.client.write_attempts),
    },
    Name {
        name: "forge.client.lifetime_ms",
        unit: Unit::Milliseconds,
        line: "Bounds how long a sent forge write may arrive.",
        value: |limits| limits.forge.client.lifetime.as_millis(),
    },
    Name {
        name: "forge.client.clock_margin_ms",
        unit: Unit::Milliseconds,
        line: "Sets the allowance for clock movement when a forge write's lifetime is checked.",
        value: |limits| limits.forge.client.clock_margin.as_millis(),
    },
    Name {
        name: "forge.client.resources",
        unit: Unit::Count,
        line: "Bounds resources retained for forge polling.",
        value: |limits| u64::from(limits.forge.client.resources),
    },
    Name {
        name: "forge.client.repositories",
        unit: Unit::Count,
        line: "Bounds repositories retained by the forge client and read by a setup plan.",
        value: |limits| u64::from(limits.forge.client.repositories),
    },
    Name {
        name: "forge.client.poll_ms",
        unit: Unit::Milliseconds,
        line: "Sets the normal interval between forge resource polls.",
        value: |limits| limits.forge.client.poll.as_millis(),
    },
    Name {
        name: "forge.client.poll_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the interval between forge resource polls.",
        value: |limits| limits.forge.client.poll_max.as_millis(),
    },
    Name {
        name: "forge.client.hinted_ms",
        unit: Unit::Milliseconds,
        line: "Sets the delay before a hinted forge read.",
        value: |limits| limits.forge.client.hinted.as_millis(),
    },
    Name {
        name: "forge.client.slow_ms",
        unit: Unit::Milliseconds,
        line: "Sets the delay before a slow forge read.",
        value: |limits| limits.forge.client.slow.as_millis(),
    },
    Name {
        name: "forge.client.settle_ms",
        unit: Unit::Milliseconds,
        line: "Bounds settling after a pull request's head or base moves, while hinted CI and mergeability are read again.",
        value: |limits| limits.forge.client.settle.as_millis(),
    },
    Name {
        name: "forge.client.fallback_ms",
        unit: Unit::Milliseconds,
        line: "Timed reads fall back while repository deliveries arrive; 0 keeps the polling paces.",
        value: |limits| limits.forge.client.fallback.as_millis(),
    },
    Name {
        name: "forge.client.facts",
        unit: Unit::Count,
        line: "Bounds forge client facts awaiting drain.",
        value: |limits| u64::from(limits.forge.client.facts),
    },
    Name {
        name: "forge.change_policy.repairs",
        unit: Unit::Count,
        line: "Bounds repair attempts in a change.",
        value: |limits| u64::from(limits.forge.change_policy.repairs),
    },
    Name {
        name: "forge.change_policy.resolutions",
        unit: Unit::Count,
        line: "Bounds conflict resolution attempts in a change.",
        value: |limits| u64::from(limits.forge.change_policy.resolutions),
    },
    Name {
        name: "forge.change_policy.reruns",
        unit: Unit::Count,
        line: "Bounds re-runs of a runner-failed job at one head.",
        value: |limits| u64::from(limits.forge.change_policy.reruns),
    },
    Name {
        name: "forge.change_policy.updates",
        unit: Unit::Count,
        line: "Bounds update attempts in a change.",
        value: |limits| u64::from(limits.forge.change_policy.updates),
    },
    Name {
        name: "forge.change_policy.questions",
        unit: Unit::Count,
        line: "Bounds questions raised in a change.",
        value: |limits| u64::from(limits.forge.change_policy.questions),
    },
    Name {
        name: "forge.change_policy.tasks",
        unit: Unit::Count,
        line: "Bounds the producer, reviews and repairs a change may create.",
        value: |limits| u64::from(limits.forge.change_policy.tasks),
    },
    Name {
        name: "forge.change_policy.stall_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the wait for a silent forge during a change.",
        value: |limits| limits.forge.change_policy.stall.as_millis(),
    },
    Name {
        name: "forge.change_policy.gates",
        unit: Unit::Count,
        line: "Bounds gates retained by a change.",
        value: |limits| u64::from(limits.forge.change_policy.gates),
    },
    Name {
        name: "forge.change_policy.clean_heads",
        unit: Unit::Count,
        line: "Bounds clean branch heads retained by a change.",
        value: |limits| u64::from(limits.forge.change_policy.clean_heads),
    },
    Name {
        name: "forge.issue_policy.plan_items",
        unit: Unit::Count,
        line: "Bounds items in an issue plan.",
        value: |limits| u64::from(limits.forge.issue_policy.plan_items),
    },
    Name {
        name: "forge.issue_policy.milestones",
        unit: Unit::Count,
        line: "Bounds milestones in an issue projection.",
        value: |limits| u64::from(limits.forge.issue_policy.milestones),
    },
    Name {
        name: "forge.issue_policy.title_bytes",
        unit: Unit::Bytes,
        line: "Bounds an issue title.",
        value: |limits| u64::from(limits.forge.issue_policy.title_bytes),
    },
    Name {
        name: "forge.issue_policy.body_bytes",
        unit: Unit::Bytes,
        line: "Bounds an issue body.",
        value: |limits| u64::from(limits.forge.issue_policy.body_bytes),
    },
    Name {
        name: "forge.issue_policy.comment_bytes",
        unit: Unit::Bytes,
        line: "Bounds an issue comment.",
        value: |limits| u64::from(limits.forge.issue_policy.comment_bytes),
    },
    Name {
        name: "forge.issue_policy.interval_ms",
        unit: Unit::Milliseconds,
        line: "Sets the interval between issue projection checks.",
        value: |limits| limits.forge.issue_policy.interval.as_millis(),
    },
    Name {
        name: "checkout.workspaces",
        unit: Unit::Count,
        line: "Bounds workspaces cached on disk and held at once; at least the slots; preparation refuses when all are held.",
        value: |limits| u64::from(limits.checkout.workspaces),
    },
    Name {
        name: "checkout.repositories",
        unit: Unit::Count,
        line: "Bounds repositories named by one checkout specification.",
        value: |limits| u64::from(limits.checkout.repositories),
    },
    Name {
        name: "checkout.name_bytes",
        unit: Unit::Bytes,
        line: "Bounds a workstream key, repository name, remote or branch.",
        value: |limits| u64::from(limits.checkout.name_bytes),
    },
    Name {
        name: "checkout.message_bytes",
        unit: Unit::Bytes,
        line: "Bounds a commit message's title and body together.",
        value: |limits| u64::from(limits.checkout.message_bytes),
    },
    Name {
        name: "checkout.conflicts",
        unit: Unit::Count,
        line: "Bounds conflicted paths per repository; zero disables merge starts.",
        value: |limits| u64::from(limits.checkout.conflicts),
    },
    Name {
        name: "checkout.path_bytes",
        unit: Unit::Bytes,
        line: "Bounds one full relative conflicted path.",
        value: |limits| u64::from(limits.checkout.path_bytes),
    },
    Name {
        name: "checkout.remote_timeout_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a checkout operation that reaches the forge, including cloning, fetching and pushing.",
        value: |limits| limits.checkout.remote_timeout.as_millis(),
    },
    Name {
        name: "checkout.local_timeout_ms",
        unit: Unit::Milliseconds,
        line: "Bounds an operation on the worker's disk, including checking out, merging and committing.",
        value: |limits| limits.checkout.local_timeout.as_millis(),
    },
    Name {
        name: "checkout.facts",
        unit: Unit::Count,
        line: "Bounds facts kept until the parent drains them; later facts are dropped and counted.",
        value: |limits| u64::from(limits.checkout.facts),
    },
    Name {
        name: "git_component.operations",
        unit: Unit::Count,
        line: "Bounds simultaneous git component operations; at least the workspaces.",
        value: |limits| u64::from(limits.git_component.operations),
    },
    Name {
        name: "git_component.stdout_bytes",
        unit: Unit::Bytes,
        line: "Bounds standard output retained from a git operation.",
        value: |limits| u64::from(limits.git_component.stdout_bytes),
    },
    Name {
        name: "git_component.stderr_bytes",
        unit: Unit::Bytes,
        line: "Bounds standard error retained from a git operation.",
        value: |limits| u64::from(limits.git_component.stderr_bytes),
    },
    Name {
        name: "git_component.read_bytes",
        unit: Unit::Bytes,
        line: "Bounds one read from a git operation's output.",
        value: |limits| u64::from(limits.git_component.read_bytes),
    },
    Name {
        name: "git_component.sweep_entries",
        unit: Unit::Count,
        line: "Bounds entries in one page of the checkout trash sweep's listing.",
        value: |limits| u64::from(limits.git_component.sweep_entries),
    },
    Name {
        name: "git_component.sweep_bytes",
        unit: Unit::Bytes,
        line: "Bounds bytes of names in one page of the checkout trash sweep's listing.",
        value: |limits| u64::from(limits.git_component.sweep_bytes),
    },
    Name {
        name: "clients.bundle_files",
        unit: Unit::Count,
        line: "Bounds bundle files, including the browser's page.",
        value: |limits| u64::from(limits.clients.bundle_files),
    },
    Name {
        name: "clients.bundle_bytes",
        unit: Unit::Bytes,
        line: "Bounds all bundle file bytes held in memory while the service runs.",
        value: |limits| u64::from(limits.clients.bundle_bytes),
    },
    Name {
        name: "records.value_bytes",
        unit: Unit::Bytes,
        line: "Bounds one forge record or carried value of temper's in the secrets store.",
        value: |limits| u64::from(limits.records.value_bytes),
    },
    Name {
        name: "forges.installations",
        unit: Unit::Count,
        line: "Bounds installations accepted by one GitHub forge.",
        value: |limits| u64::from(limits.forges.installations),
    },
    Name {
        name: "forges.log_origins",
        unit: Unit::Count,
        line: "Bounds one forge's job log origins, at most 32.",
        value: |limits| u64::from(limits.forges.log_origins),
    },
    Name {
        name: "forge_component.log_origins",
        unit: Unit::Count,
        line: "Bounds job log redirect origins and remembered origins per forge.",
        value: |limits| u64::from(limits.forge_component.log_origins),
    },
    Name {
        name: "forge_component.log_poll_ms",
        unit: Unit::Milliseconds,
        line: "Sets the interval between reads of a GitHub App's delivery log.",
        value: |limits| limits.forge_component.log_poll.as_millis(),
    },
    Name {
        name: "forge_component.log_catch_up_ms",
        unit: Unit::Milliseconds,
        line: "Bounds how far back delivery logs are read when a forge restarts within one process, at most a day.",
        value: |limits| limits.forge_component.log_catch_up.as_millis(),
    },
    Name {
        name: "forge_component.log_pages",
        unit: Unit::Count,
        line: "Bounds pages taken by one delivery log read.",
        value: |limits| u64::from(limits.forge_component.log_pages),
    },
    Name {
        name: "forge_component.delivery_bytes",
        unit: Unit::Bytes,
        line: "Bounds a delivery read from the log.",
        value: |limits| u64::from(limits.forge_component.delivery_bytes),
    },
    Name {
        name: "forge_component.installations",
        unit: Unit::Count,
        line: "Bounds installations for which the component holds tokens, per GitHub forge.",
        value: |limits| u64::from(limits.forge_component.installations),
    },
    Name {
        name: "forge_component.key_bytes",
        unit: Unit::Bytes,
        line: "Bounds a GitHub App's key.",
        value: |limits| u64::from(limits.forge_component.key_bytes),
    },
    Name {
        name: "forge_component.renew_before_ms",
        unit: Unit::Milliseconds,
        line: "Sets how long before expiry an installation token is minted again.",
        value: |limits| limits.forge_component.renew_before.as_millis(),
    },
    Name {
        name: "forge_component.forges",
        unit: Unit::Count,
        line: "Bounds forges served by the forge component.",
        value: |limits| u64::from(limits.forge_component.forges),
    },
    Name {
        name: "forge_component.token_bytes",
        unit: Unit::Bytes,
        line: "Bounds a forge's API token, git token or site admin token.",
        value: |limits| u64::from(limits.forge_component.token_bytes),
    },
    Name {
        name: "forge_component.connections",
        unit: Unit::Count,
        line: "Bounds forge connections, equal to the forge client's calls in flight.",
        value: |limits| u64::from(limits.forge_component.connections),
    },
    Name {
        name: "forge_component.head_bytes",
        unit: Unit::Bytes,
        line: "Bounds a response head read from a forge.",
        value: |limits| u64::from(limits.forge_component.head_bytes),
    },
    Name {
        name: "forge_component.headers",
        unit: Unit::Count,
        line: "Bounds fields in a forge response head.",
        value: |limits| u64::from(limits.forge_component.headers),
    },
    Name {
        name: "forge_component.body_bytes_in",
        unit: Unit::Bytes,
        line: "Bounds a streamed response body read from a forge, counting time and bytes rather than held memory.",
        value: |limits| u64::from(limits.forge_component.body_bytes_in),
    },
    Name {
        name: "forge_component.depth",
        unit: Unit::Count,
        line: "Bounds JSON nesting followed in a forge response.",
        value: |limits| u64::from(limits.forge_component.depth),
    },
    Name {
        name: "forge_component.chunk",
        unit: Unit::Bytes,
        line: "Bounds bytes of a JSON string demanded at once.",
        value: |limits| u64::from(limits.forge_component.chunk),
    },
    Name {
        name: "forge_component.string_bytes",
        unit: Unit::Bytes,
        line: "Bounds a retained forge name, title, context, description, link or path.",
        value: |limits| u64::from(limits.forge_component.string_bytes),
    },
    Name {
        name: "forge_component.body_bytes",
        unit: Unit::Bytes,
        line: "Bounds a forge response body retained whole.",
        value: |limits| u64::from(limits.forge_component.body_bytes),
    },
    Name {
        name: "forge_component.inner",
        unit: Unit::Count,
        line: "Bounds elements retained from a list within one forge document.",
        value: |limits| u64::from(limits.forge_component.inner),
    },
    Name {
        name: "forge_component.target_bytes",
        unit: Unit::Bytes,
        line: "Bounds a forge request target.",
        value: |limits| u64::from(limits.forge_component.target_bytes),
    },
    Name {
        name: "forge_component.repositories",
        unit: Unit::Count,
        line: "Bounds repositories in the forge component's table.",
        value: |limits| u64::from(limits.forge_component.repositories),
    },
    Name {
        name: "forge_component.users",
        unit: Unit::Count,
        line: "Bounds users in the forge component's table.",
        value: |limits| u64::from(limits.forge_component.users),
    },
    Name {
        name: "forge_component.tls.read",
        unit: Unit::Bytes,
        line: "Bounds plaintext demanded at once from a forge TLS connection.",
        value: |limits| u64::from(limits.forge_component.tls.read),
    },
    Name {
        name: "forge_component.tls.send",
        unit: Unit::Bytes,
        line: "Bounds plaintext send room demanded from a forge TLS connection.",
        value: |limits| u64::from(limits.forge_component.tls.send),
    },
    Name {
        name: "forge_component.tls.records",
        unit: Unit::Bytes,
        line: "Bounds ciphertext retained by a forge TLS connection, including handshake messages spanning records.",
        value: |limits| u64::from(limits.forge_component.tls.records),
    },
    Name {
        name: "forge_component.hooks",
        unit: Unit::Count,
        line: "Bounds connections to the webhooks listener.",
        value: |limits| u64::from(limits.forge_component.hooks),
    },
    Name {
        name: "forge_component.hook_head_bytes",
        unit: Unit::Bytes,
        line: "Bounds a webhook delivery's request head.",
        value: |limits| u64::from(limits.forge_component.hook_head_bytes),
    },
    Name {
        name: "forge_component.hook_body_bytes",
        unit: Unit::Bytes,
        line: "Bounds a streamed webhook body; a delivery past it gets 413 and hints nothing.",
        value: |limits| u64::from(limits.forge_component.hook_body_bytes),
    },
    Name {
        name: "forge_component.hook_secret_bytes",
        unit: Unit::Bytes,
        line: "Bounds a Forgejo forge's webhook secret, at least 32 bytes.",
        value: |limits| u64::from(limits.forge_component.hook_secret_bytes),
    },
    Name {
        name: "forge_component.hook_deadline_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a webhook delivery's read and answer from accept or the previous answer.",
        value: |limits| limits.forge_component.hook_deadline.as_millis(),
    },
    Name {
        name: "forge_component.deadline_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a forge call and each startup read, at most the forge client's lifetime.",
        value: |limits| limits.forge_component.deadline.as_millis(),
    },
    Name {
        name: "forge_component.idle_ms",
        unit: Unit::Milliseconds,
        line: "Bounds how long an idle forge connection is kept.",
        value: |limits| limits.forge_component.idle.as_millis(),
    },
    Name {
        name: "forge_component.start_backoff_ms",
        unit: Unit::Milliseconds,
        line: "Sets the first delay before a forge's start is tried again.",
        value: |limits| limits.forge_component.start_backoff.as_millis(),
    },
    Name {
        name: "forge_component.start_backoff_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the doubling delay before a forge's start is tried again.",
        value: |limits| limits.forge_component.start_backoff_max.as_millis(),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[0].name,
        unit: Unit::Count,
        line: temper_forge_setup::limits::HOST_NAMES[0].line,
        value: |limits| u64::from(limits.setup.runs),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[1].name,
        unit: Unit::Count,
        line: temper_forge_setup::limits::HOST_NAMES[1].line,
        value: |limits| u64::from(limits.setup.states),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[2].name,
        unit: Unit::Milliseconds,
        line: temper_forge_setup::limits::HOST_NAMES[2].line,
        value: |limits| limits.setup.state_life.as_millis(),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[4].name,
        unit: Unit::Milliseconds,
        line: temper_forge_setup::limits::HOST_NAMES[4].line,
        value: |limits| limits.setup.deadline.as_millis(),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[5].name,
        unit: Unit::Milliseconds,
        line: temper_forge_setup::limits::HOST_NAMES[5].line,
        value: |limits| limits.setup.backoff.as_millis(),
    },
    Name {
        name: temper_forge_setup::limits::HOST_NAMES[6].name,
        unit: Unit::Milliseconds,
        line: temper_forge_setup::limits::HOST_NAMES[6].line,
        value: |limits| limits.setup.backoff_max.as_millis(),
    },
];

/// Looks up a whole dotted override name.
#[must_use]
#[expect(clippy::manual_find, reason = "the step subset searches slices with a bounded for loop")]
pub fn find(name: &[u8]) -> Option<&'static Name> {
    for row in &NAMES {
        if row.name.as_bytes() == name {
            return Some(row);
        }
    }
    None
}
