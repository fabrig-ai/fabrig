//! Top-owned landing rows, coherent reads, receipts and effects (domain/landing.md, 2–4, and domain/landing-prepare.md, 1, and domain/landing-proof.md, 1–2, and domain/landing-turn.md, 1–2, and domain/landing.md, 10–15).
//! Landing targets use the adoption (temper/docs/design/domain/forge/adoption.md, 12).
use crate::domain::read_line::{self, Waiter};
use crate::domain::writes::{
    self,
    outbox::{self, Pin},
};
use crate::domain::{child, emit, keep_entry, take_entry};
use crate::{
    Check, Domain, Key, LandingEntry, LandingHeard, LandingRow, Limits, Operation, Receipt, ReceiptJob, ReceiptKey,
    ReceiptReport, ReceiptRun, RepositoryGates, Request, Stored, runs,
};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Env, List, Map, Queue, Token, Wall};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as policy;
pub use temper_engine_domain_forge_vocabulary::landing::*;
use temper_engine_domain_forge_vocabulary::writes::{Actor, Seen};

#[path = "landing_read_wait.rs"]
pub(crate) mod read_wait;
pub(crate) use read_wait::unavailable as read_unavailable;

#[path = "landing_follow.rs"]
mod saved_follow;
pub(crate) use saved_follow::rebuild;
pub(crate) mod members;
mod temper_dir;

#[derive(Clone, Copy, Hash, Debug)]
pub(crate) enum GateProblem {
    Runner { fault: policy::Fault, job: Option<u64> },
    Broke(ReportProblem),
}

#[derive(Clone, Copy, Hash, Debug)]
pub(crate) enum ReportProblem {
    Missing,
    Unread,
    Empty,
    NoVerdict,
}

type Position = (u64, u16, u16);
type RepositoryGate = (api::Repository, Box<[u8]>);
type KeyAt = (api::Commit, policy::Key);
type ProjectLanding = (
    change::Way,
    Box<[RepositoryGates]>,
    crate::AfterLanding,
    Option<Box<[u8]>>,
    Option<Box<[u8]>>,
    policy::Project,
    u16,
);
#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<u64, LandingRow>,
    pub(crate) unmade: Map<u64, crate::unmade::Readback>,
    memberships: Map<u64, u64>,
    memberships_unbuilt: bool,
    claims: Map<u64, u64>,
    pub(crate) due: Map<u64, Wall>,
    pub(crate) receipts: Map<ReceiptKey, Receipt>,
    pub(crate) projects: Map<u32, ProjectLanding>,
    pub(crate) settings: Map<u32, crate::settings::ProjectLanding>,
    repairs: Map<u32, Result<crate::settings::Repairs, &'static [u8]>>,
    pub(crate) held: bool,
    pub(crate) pending: Map<Token, Pending>,
    decisions: Map<u64, Ready>,
    progress: Map<u64, read_wait::Progress>,
    pub(crate) keys: Map<Position, (api::Commit, Option<api::Commit>, policy::Key)>,
    pub(crate) main_keys: Map<RepositoryGate, KeyAt>,
    pub(crate) flights: Map<ReceiptKey, runs::Follow>,
    pub(crate) faults: Map<Position, runs::Follow>,
    /// Effect status and whether its outcome is a known refusal (domain/landing-turn.md, 1).
    pub(crate) results: Map<(u64, u16, u16, u8), (policy::Status, bool)>,
    pub(crate) limits: Limits,
}
#[derive(Hash, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the common coherent gather moves into its bounded handoff slot without allocating"
)]
enum Ready {
    Gathered { owner: Token, pending: Pending },
    Read(ReadStep),
    Claimed { released: bool },
}
#[derive(Hash, Debug)]
enum ReadStep {
    Failed(read_wait::Failure),
    Stalled,
    RoomExpired(Box<[u8]>),
    Held { released: bool },
    CleanupFailed,
}
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    pub(crate) task: u64,
    repository: api::Repository,
    resource: Option<GatherRead>,
    read: api::Read,
    retry: Retry,
    cached: bool,
    pub(crate) entry: u64,
    pub(crate) heard: LandingHeard,
    facts: Box<[policy::MemberFacts]>,
    head_trailers: List<crate::LandingHeadTrailers>,
    stage: Stage,
    polled: List<ReceiptKey>,
    flight_entry: Option<u64>,
    trees: List<policy::Tree>,
    base_trees: List<policy::Tree>,
    entries: List<policy::Entry>,
    woken: bool,
    evidence: Box<[crate::LandingEvidence]>,
    opened: Option<u64>,
    closed: policy::Status,
    round_pull: policy::RoundPull,
    turn_tip: Seen,
    cleaned: bool,
    upstream: Option<api::Commit>,
    paths: Box<[Paths]>,
    temper_dir: Option<Box<temper_dir::Check>>,
    reads: Map<(api::Repository, GatherRead), Result<api::Answer, api::Error>>,
    stall: Option<crate::stall::Report>,
}
#[cfg(test)]
impl Pending {
    #[expect(clippy::match_like_matches_macro, reason = "the step's fixture uses the stage pattern")]
    pub(crate) fn checking_temper_dir(&self) -> bool {
        if let Stage::TemperDir = self.stage { true } else { false }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Retry {
    None,
    Waiting,
    Disconnected,
}

#[derive(Hash, Debug)]
struct Paths {
    files: Box<[Box<[u8]>]>,
    whole: bool,
    comparison: Option<temper_dir::Comparison>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum GatherRead {
    Branch(Box<[u8]>),
    Pull(u64),
}
fn gather_resource(read: &api::Read) -> Option<GatherRead> {
    if let api::Read::Branch { branch } = read {
        Some(GatherRead::Branch(branch.clone()))
    } else if let api::Read::Pull { number, .. } = read {
        Some(GatherRead::Pull(*number))
    } else {
        None
    }
}
#[derive(Clone, Hash, Debug)]
enum Stage {
    Branch(u16),
    Main(u16),
    Ancestry(u16),
    ContainsCandidate(u16),
    TurnPull,
    Pull(u16),
    Reviews(u16, u32),
    Paths(u16),
    TemperDir,
    Collect,
    CleanupPull,
    CleanupBranch(u16),
    StallRun,
    StallJobs,
    Round {
        member: u16,
        stage: RoundRead,
        candidate: Option<api::Commit>,
    },
    Run {
        key: ReceiptKey,
        pages: u32,
    },
    FailedReport {
        key: ReceiptKey,
        run: api::Run,
        report: policy::GateReport,
    },
    FailedJobs {
        key: ReceiptKey,
        ended: GateEnd,
        next: u32,
        words: Box<[u8]>,
    },
    Tree {
        member: u16,
        gate: u16,
        candidate: api::Commit,
        object: api::Commit,
        page: u32,
        main: bool,
        reading_base: bool,
    },
}
#[derive(Clone, Hash, Debug)]
struct GateEnd {
    run: api::Run,
    report: ReceiptReport,
    failures: Option<temper_engine_domain_forge_tiers::Failures>,
}

#[derive(Clone, Copy, Hash, Debug)]
enum RoundRead {
    Branch,
    Prefix(u32),
    Head,
    Contained,
    Pull,
    Upstream,
}

impl Domain {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_landing_read_waiters(&self) -> Box<[u64]> {
        let mut tasks = List::with_capacity(self.landing.pending.len());
        for (_, pending) in &self.landing.pending {
            if pending.retry == Retry::Waiting {
                tasks.push(pending.task).expect("counted landing readers");
            }
        }
        tasks.into_boxed()
    }

    /// The read-admission retry that wakes this landing's retained gather.
    /// For its worlds' inspection adapter (testing/engine-checks.md, 2.2).
    #[must_use]
    pub fn inspection_landing_read_retry(&self, task: u64) -> Option<skein_lib::Time> {
        for (owner, pending) in &self.landing.pending {
            if pending.task == task && pending.retry == Retry::Waiting {
                return self.read_line.retry_at(Waiter::Landing(*owner));
            }
        }
        None
    }
}

/// The repositories whose receipts and heads a train judges.
pub(crate) fn repositories(row: &LandingRow, capacity: u32) -> Box<[api::Repository]> {
    let mut repositories = List::with_capacity(capacity);
    for member in &row.landing.members {
        let repository = repository(member);
        if !repositories.as_slice().contains(&repository) {
            repositories.push(repository).expect("train repositories are adopted");
        }
    }
    repositories.into_boxed()
}

impl State {
    pub(crate) fn reading_in(&self, repository: api::Repository) -> bool {
        for (_, pending) in &self.pending {
            if pending.repository == repository {
                return true;
            }
            if let Some(row) = self.rows.get(&pending.task) {
                for member in &row.landing.members {
                    if self::repository(member) == repository {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn gathers_owed(&self, forge: u16) -> u32 {
        let mut owed = 0_u32;
        for (_, pending) in &self.pending {
            if pending.repository.forge == forge {
                owed = owed.checked_add(1).expect("gathers fit their limit");
            }
        }
        owed
    }
    pub fn new(l: &Limits) -> Self {
        let slots = slots(l).expect("landing bounds checked");
        Self {
            rows: Map::with_capacity(l.landings),
            unmade: Map::with_capacity(l.landings),
            memberships: Map::with_capacity(l.changes),
            memberships_unbuilt: false,
            claims: Map::with_capacity(l.landings),
            due: Map::with_capacity(l.landings),
            receipts: Map::with_capacity(receipt_slots(l).expect("receipt bounds")),
            projects: Map::with_capacity(l.judge_projects),
            settings: Map::with_capacity(l.judge_projects),
            repairs: Map::with_capacity(l.judge_projects),
            held: false,
            pending: Map::with_capacity(l.landings),
            decisions: Map::with_capacity(l.landings),
            progress: Map::with_capacity(l.landings),
            keys: Map::with_capacity(slots),
            main_keys: Map::with_capacity(l.repositories.checked_mul(l.gates).expect("gate bounds")),
            flights: Map::with_capacity(slots),
            faults: Map::with_capacity(slots),
            results: Map::with_capacity(result_slots(l).expect("effect bounds")),
            limits: *l,
        }
    }
    pub fn way(&self, project: u32) -> change::Way {
        match self.projects.get(&project) {
            Some((way, _, _, _, _, _, _)) => *way,
            None => change::Way::Queue,
        }
    }
    pub(crate) fn gates(&self, project: u32, repository: api::Repository) -> &[policy::Gate] {
        if let Some((_, repositories, _, _, _, _, _)) = self.projects.get(&project) {
            for row in repositories {
                if row.repository == repository {
                    return &row.gates;
                }
            }
        }
        &[]
    }
}
fn valid_tier(tier: &crate::Tier, limits: &Limits) -> bool {
    if tier.name.is_empty()
        || tier.name.len() > policy::CHECK_NAME_BYTES
        || tier.description.len() > crate::DESCRIPTION_BYTES
        || tier.workflow.is_empty()
        || tier.workflow.len() > policy::WORKFLOW_BYTES
        || tier.workflow.contains(&b'/')
        || tier.artifact.is_empty()
        || tier.artifact.len() > policy::CHECK_NAME_BYTES
        || tier.artifact.contains(&b'/')
        || tier.ignored.len() > usize::try_from(limits.ignored_paths).expect("u32 fits")
    {
        return false;
    }
    match &tier.follows {
        crate::Follows::Branch(branch) if branch.is_empty() => return false,
        crate::Follows::Branch(_) | crate::Follows::Landing | crate::Follows::Goals => {}
    }
    for byte in &tier.name {
        if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
            return false;
        }
    }
    for path in &tier.ignored {
        if path.is_empty()
            || path.len() > usize::try_from(limits.name_bytes).expect("u32 fits")
            || path.first() == Some(&b'/')
            || path.last() == Some(&b'/')
        {
            return false;
        }
    }
    true
}
impl Domain {
    /// The operator's immutable start fact (domain/landing-settings.md, 1.3).
    pub fn hold_landings(&mut self, held: bool) {
        self.landing.held = held;
    }
    #[must_use]
    pub fn landing_held(&self) -> bool {
        self.landing.held
    }
    /// The landing built at restore, never the current proposed or committed setting.
    #[must_use]
    pub fn landing_settings(&self, project: u32) -> Option<&crate::settings::ProjectLanding> {
        self.landing.settings.get(&project)
    }
    #[must_use]
    pub fn project_checks_can_flake(&self, project: u32) -> bool {
        let Some(settings) = self.landing_settings(project) else { return true };
        match settings.flakes {
            Some(crate::settings::Flakes::Never) => false,
            Some(crate::settings::Flakes::Rerun(_)) | None => true,
        }
    }
    /// One start's resolution, supplied by the root with jig's current facts (red-repairs.md, 7).
    pub fn take_project_repairs(&mut self, project: u32, repairs: Result<crate::settings::Repairs, &'static [u8]>) {
        self.landing.repairs.insert(project, repairs).expect("restored project repairs");
    }
    /// Repairs available for this start alone (red-repairs.md, 4.2).
    pub fn project_repairs(&self, project: u32) -> Result<&crate::settings::Repairs, Box<[u8]>> {
        let field = match self.landing.repairs.get(&project) {
            Some(Ok(repairs)) => return Ok(repairs),
            Some(Err(field)) => Some(*field),
            None => None,
        };
        let number = skein_lib::Decimal::of(u64::from(project));
        let suffix = match field {
            Some(field) => skein_lib::bytes::concat(&[b": ", field]),
            None => Box::new([]),
        };
        Err(skein_lib::bytes::concat(&[b"repairs are off in project ", number.as_bytes(), &suffix]))
    }
    pub(crate) fn restore_landing_settings(&mut self) {
        use crate::settings::{SettingSlot, Value};
        self.landing.repairs = Map::with_capacity(self.landing.limits.judge_projects);
        let mut projects = Map::uncapped();
        for (key, _) in &self.setting_rows {
            if (key.kind == 4 || key.kind == 5) && (key.slot == SettingSlot::Change || key.slot == SettingSlot::Default)
            {
                projects.insert(key.scope, ()).expect("restored projects");
            }
        }
        if let Some(stock) = &self.setting_stock {
            for item in stock {
                if item.kind == 4 || item.kind == 5 {
                    projects.insert(item.scope, ()).expect("stock projects");
                }
            }
        }
        for (&project, ()) in &projects {
            let settings = match self.setting(4, project, 1) {
                Some(value) => match &value.value {
                    Value::Landing(v) => v.clone(),
                    Value::Gates(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::Goal(_)
                    | Value::Follow(_)
                    | Value::Dependency(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::Limits(_)
                    | Value::AgentFigures(_)
                    | Value::Welcome(_) => unreachable!("kind 4 has landing value"),
                },
                None => crate::settings::ProjectLanding::default(),
            };
            let gates = match self.setting(5, project, 1) {
                Some(value) => match &value.value {
                    Value::Gates(v) => v.repositories.clone(),
                    Value::Landing(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::Goal(_)
                    | Value::Follow(_)
                    | Value::Dependency(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::Limits(_)
                    | Value::AgentFigures(_)
                    | Value::Welcome(_) => unreachable!("kind 5 has gates value"),
                },
                None => Box::new([]),
            };
            assert!(
                self.project_landing(
                    project,
                    settings.by,
                    gates,
                    policy::Project {
                        gate_retries: u32::from(settings.gate_retries),
                        gate_timeout: skein_lib::Duration::from_millis(settings.gate_timeout_ms)
                    },
                    crate::AfterLanding {
                        wait: settings.wait,
                        verdict: temper_engine_domain_forge_tiers::Settings {
                            retries: u32::from(settings.tier_retries),
                            suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
                        },
                        bound: skein_lib::Duration::from_millis(settings.verification_bound_ms)
                    }
                ),
                "judged restored landing settings"
            );
            if settings.batch != 1 {
                assert!(self.project_batch(project, settings.batch), "judged restored batch");
            }
            if let Some(command) = &settings.merge_job {
                assert!(self.project_merge_job(project, command.clone()), "restored merge job");
            }
            if let Some(command) = &settings.close_job {
                assert!(self.project_close_job(project, command.clone()), "restored close job");
            }
            self.landing.settings.insert(project, settings).expect("restored project settings");
        }
    }
    /// A project's way and repository test gates (domain/landing-settings.md, 1.1).
    pub fn project_landing(
        &mut self,
        project: u32,
        way: change::Way,
        gates: Box<[RepositoryGates]>,
        gates_policy: policy::Project,
        after: crate::AfterLanding,
    ) -> bool {
        let l = &self.landing.limits;
        if gates.len() > usize::try_from(l.repositories).expect("u32 fits") {
            return false;
        }
        for (index, repository) in gates.iter().enumerate() {
            if way == change::Way::Turn
                && let Some(forge) = self.configured.get(&repository.repository.forge)
                && forge.kind == crate::Kind::GitHub
            {
                return false;
            }
            for prior in gates.get(..index).expect("preceding positions") {
                if prior.repository == repository.repository {
                    return false;
                }
            }
            if repository.gates.len() > usize::try_from(l.gates).expect("u32 fits") {
                return false;
            }
            if repository.tiers.len() > usize::try_from(l.tiers.tiers).expect("u32 fits") {
                return false;
            }
            for (index, tier) in repository.tiers.iter().enumerate() {
                if !valid_tier(tier, l) {
                    return false;
                }
                for gate in &repository.gates {
                    if gate.name == tier.name {
                        return false;
                    }
                }
                for prior in repository.tiers.get(..index).expect("preceding tiers") {
                    if prior.name == tier.name {
                        return false;
                    }
                }
            }
            for (index, gate) in repository.gates.iter().enumerate() {
                if gate.name.is_empty()
                    || gate.name.len() > policy::CHECK_NAME_BYTES
                    || gate.workflow.is_empty()
                    || gate.workflow.len() > policy::WORKFLOW_BYTES
                    || gate.ignored.len() > usize::try_from(l.ignored_paths).expect("u32 fits")
                {
                    return false;
                }
                if let Some(report) = &gate.report
                    && (report.artifact.is_empty()
                        || report.artifact.len() > policy::CHECK_NAME_BYTES
                        || report.artifact.contains(&b'/'))
                {
                    return false;
                }
                for byte in &gate.name {
                    if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
                        return false;
                    }
                }
                for prior in repository.gates.get(..index).expect("preceding positions") {
                    if prior.name == gate.name {
                        return false;
                    }
                }
                for path in &gate.ignored {
                    if path.is_empty()
                        || path.len() > usize::try_from(l.name_bytes).expect("u32 fits")
                        || path.first() == Some(&b'/')
                        || path.last() == Some(&b'/')
                    {
                        return false;
                    }
                }
            }
        }
        let mut has_tiers = false;
        if let Some((_, old, _, _, _, _, _)) = self.landing.projects.get(&project) {
            for repository in old {
                has_tiers |= !repository.tiers.is_empty();
            }
        }
        for repository in &gates {
            has_tiers |= !repository.tiers.is_empty();
        }
        let saved = self.landing.projects.insert(project, (way, gates, after, None, None, gates_policy, 1)).is_ok();
        if saved && has_tiers {
            self.tier_ops.dirty = true;
        }
        saved
    }
    /// The project's forming cap, checked against its deployment (domain/landing-settings.md, 1.1).
    pub fn project_batch(&mut self, project: u32, batch: u16) -> bool {
        if batch == 0 || u32::from(batch) > self.landing.limits.landing_members {
            return false;
        }
        let Some((_, _, _, _, _, _, cap)) = self.landing.projects.get_mut(&project) else { return false };
        *cap = batch;
        true
    }
    /// Main takes the project's batch; another train takes one whole unit (domain/landing-turn.md, 2).
    #[must_use]
    pub fn train_batch(&self, repository: api::Repository, branch: &[u8]) -> u16 {
        let Some(repo) = self.repository(repository) else { return 1 };
        if repo.landing.as_ref() != branch {
            return 1;
        }
        match self.landing.projects.get(&repo.project) {
            Some((_, _, _, _, _, _, batch)) => *batch,
            None => 1,
        }
    }
    /// A configured project's merge command (temper/docs/design/protocol/configuration/engine.md).
    pub fn project_merge_job(&mut self, project: u32, merge_job: Box<[u8]>) -> bool {
        let Some((_, _, _, command, _, _, _)) = self.landing.projects.get_mut(&project) else { return false };
        *command = Some(merge_job);
        true
    }
    /// A configured project's close command (temper/docs/design/domain/forge/change/README.md, 1).
    pub fn project_close_job(&mut self, project: u32, close_job: Box<[u8]>) -> bool {
        let Some((_, _, _, _, command, _, _)) = self.landing.projects.get_mut(&project) else { return false };
        *command = Some(close_job);
        true
    }
    /// The project's optional close fallback (temper/docs/design/domain/forge/change/README.md, 1).
    #[must_use]
    pub fn close_job(&self, project: u32) -> Option<&[u8]> {
        let (_, _, _, _, command, _, _) = self.landing.projects.get(&project)?;
        command.as_deref()
    }
    /// A project's landing way: queue where its configuration gives none (domain/landing-settings.md, 1.1).
    #[must_use]
    pub fn landing_way(&self, project: u32) -> change::Way {
        self.landing.way(project)
    }
    /// A repository's configured gates and tiers in its project, if it has any.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the step subset searches without iterator closures")]
    pub fn repository_gates(&self, project: u32, repository: api::Repository) -> Option<&RepositoryGates> {
        let (_, repositories, _, _, _, _, _) = self.landing.projects.get(&project)?;
        for row in repositories {
            if row.repository == repository {
                return Some(row);
            }
        }
        None
    }
    /// The project's optional merge fallback (temper/docs/design/protocol/configuration/engine.md).
    #[must_use]
    pub fn merge_job(&self, project: u32) -> Option<&[u8]> {
        let (_, _, _, command, _, _, _) = self.landing.projects.get(&project)?;
        command.as_deref()
    }
    /// The project's retained verification settings (domain/after-landing.md, 11).
    #[must_use]
    pub fn after_landing(&self, project: u32) -> Option<crate::AfterLanding> {
        match self.landing.projects.get(&project) {
            Some((_, _, after, _, _, _, _)) => Some(*after),
            None => None,
        }
    }
    /// The connected changes of a pending cross-repository dependency, upstream
    /// first. The root supplies jig's current graph on each decision; no graph is
    /// stored here (domain/landing.md, 4.1–4.2).
    #[must_use]
    pub fn joined_changes(&self, project: u32, task: u64, graph: &[crate::ChangeDependencies]) -> Box<[u64]> {
        let mut joined = List::with_capacity(self.landing.limits.changes);
        joined.push(task).expect("one change admitted");
        if self.landing.way(project) != change::Way::Turn {
            return joined.into_boxed();
        }
        // Every pass adds at least one vertex or has reached the fixed point.
        for _ in 0..self.landing.limits.changes {
            let before = joined.len();
            for edge in graph {
                if !joinable(self, project, edge.change) {
                    continue;
                }
                for upstream in &edge.pending {
                    if !joinable(self, project, *upstream) {
                        continue;
                    }
                    let down = self.changes.get(&edge.change).expect("joinable change");
                    let up = self.changes.get(upstream).expect("joinable upstream");
                    if down.repository == up.repository {
                        continue;
                    }
                    if joined.as_slice().contains(&edge.change) && !joined.as_slice().contains(upstream) {
                        joined.push(*upstream).expect("bounded changes");
                    } else if joined.as_slice().contains(upstream) && !joined.as_slice().contains(&edge.change) {
                        joined.push(edge.change).expect("bounded changes");
                    }
                }
            }
            if joined.len() == before {
                break;
            }
        }
        ordered_changes(self, joined.as_slice(), graph)
    }
    /// Check admission before the root allocates its deployment-funded task.
    pub fn landing_admits(&self, project: u32, changes: &[u64]) -> Result<(), crate::LandingRefusal> {
        self.landing_members_admit(project, changes)?;
        if !self.landing_room() {
            return Err(crate::LandingRefusal::Landings);
        }
        Ok(())
    }

    pub(crate) fn landing_members_admit(&self, project: u32, changes: &[u64]) -> Result<(), crate::LandingRefusal> {
        use crate::LandingRefusal;
        if self.landing.way(project) != change::Way::Turn {
            return Err(LandingRefusal::Way);
        }
        if changes.is_empty() || changes.len() > skein_lib::usize_of(self.landing.limits.landing_members) {
            return Err(LandingRefusal::Members);
        }
        let mut train = None;
        for (index, task) in changes.iter().enumerate() {
            if changes.get(..index).expect("preceding changes").contains(task) {
                return Err(LandingRefusal::Members);
            }
            let row = self.changes.get(task).ok_or(LandingRefusal::Repository)?;
            let repo = self.repositories.get(&row.repository).ok_or(LandingRefusal::Repository)?;
            if repo.project != project || self.landing_of(row.task).is_some() || !row.asks.is_empty() {
                return Err(LandingRefusal::Repository);
            }
            if let Some((repository, branch)) = &train {
                if *repository != row.repository || *branch != row.base {
                    return Err(LandingRefusal::Repository);
                }
            } else {
                train = Some((row.repository, row.base.clone()));
            }
            if !self.train_free(row.repository, &row.base) {
                return Err(LandingRefusal::Train);
            }
        }
        Ok(())
    }

    fn landing_room(&self) -> bool {
        if self.landing.rows.len() < self.landing.limits.landings {
            return true;
        }
        for (_, row) in &self.landing.rows {
            if reclaimable(self, row) {
                return true;
            }
        }
        false
    }

    /// Rows that cannot yield a place, in the order their task numbers were made.
    #[must_use]
    pub fn landing_room_holders(&self) -> Box<[u64]> {
        let mut holders = List::with_capacity(self.landing.rows.len());
        for (task, row) in &self.landing.rows {
            if !reclaimable(self, row) {
                holders.push(*task).expect("counted landing rows");
            }
        }
        holders.into_boxed()
    }

    pub(crate) fn oldest_landing_holder(&self, holders: &[u64]) -> Option<u64> {
        for task in holders {
            if let Some(row) = self.landing.rows.get(task)
                && !matches_terminal(&row.landing.phase)
            {
                return Some(*task);
            }
        }
        holders.first().copied()
    }
    /// The hold account for a unit beyond this milestone (domain/landing.md, 4.1).
    #[must_use]
    pub fn unit_words(&self, joined: &[u64]) -> Box<[u8]> {
        let mut words = List::with_capacity(jig_domain_tasks::HOLD_ACCOUNT_BYTES);
        append_words(&mut words, b"unit of ");
        append_words(&mut words, skein_lib::Decimal::of(u64::try_from(joined.len()).expect("change count")).as_bytes());
        append_words(&mut words, b" changes: ");
        let mut repository = None;
        let mut across = false;
        for (index, task) in joined.iter().enumerate() {
            if index != 0 {
                append_words(&mut words, b", ");
            }
            append_words(&mut words, b"C");
            append_words(&mut words, skein_lib::Decimal::of(*task).as_bytes());
            if let Some(row) = self.change(*task) {
                if let Some(prior) = repository {
                    across |= prior != row.repository;
                }
                repository = Some(row.repository);
            }
        }
        append_words(
            &mut words,
            if across {
                b": a unit across repositories lands with the merge queue's later batches (temper/docs/design/draft/merge-queue.md, 13)"
            } else {
                b": unit passes forge.landing_members"
            },
        );
        words.into_boxed()
    }
    /// Trains and their waiting changes are derived, never reserved (domain/landing-turn.md, 2).
    #[must_use]
    pub fn train_free(&self, repository: api::Repository, branch: &[u8]) -> bool {
        self.train_holder(repository, branch).is_none()
    }

    /// The live landing keeping this train (temper/docs/design/domain/landing-turn.md, 2).
    #[must_use]
    pub fn train_holder(&self, repository: api::Repository, branch: &[u8]) -> Option<u64> {
        for (task, row) in &self.landing.rows {
            if row.landing.branch.as_ref() != branch || matches_terminal(&row.landing.phase) {
                continue;
            }
            for member in &row.landing.members {
                if self::repository(member) == repository {
                    return Some(*task);
                }
            }
        }
        None
    }
    /// Each ready turn change names a train to judge; the root filters kept tasks.
    #[must_use]
    pub fn waiting_trains(&self) -> Box<[(api::Repository, Box<[u8]>)]> {
        let mut trains = List::with_capacity(self.landing.limits.changes);
        for (_, row) in &self.changes {
            if waiting(self, row) && !trains.as_slice().contains(&(row.repository, row.base.clone())) {
                trains.push((row.repository, row.base.clone())).expect("bounded changes");
            }
        }
        trains.into_boxed()
    }
    /// The first eligible waiting change of a free train with room (domain/landing-turn.md, 2).
    #[must_use]
    pub fn first_waiting(&self, repository: api::Repository, branch: &[u8], kept: &[u64], now: Wall) -> Option<u64> {
        if !self.landing_room() {
            return None;
        }
        let units = self.waiting_units(repository, branch, kept, &[], now);
        let selection = self.select_waiting(repository, branch, &units);
        let unit = units.get(skein_lib::usize_of(*selection.take.first()?))?;
        unit.changes.first().copied()
    }
    /// Queue candidates for one train, before the root filters task holds and dependencies.
    #[must_use]
    pub fn waiting_changes(&self, repository: api::Repository, branch: &[u8]) -> Box<[change::Ready]> {
        let mut ready = List::with_capacity(self.landing.limits.changes);
        for (task, row) in &self.changes {
            if row.repository == repository
                && row.base.as_ref() == branch
                && waiting(self, row)
                && let change::State::Ready { ready_since, .. } = row.change.state
            {
                ready
                    .push(change::Ready {
                        task: *task,
                        since: row.change.ready_since.unwrap_or(ready_since),
                        priority: row.priority,
                    })
                    .expect("bounded changes");
            }
        }
        ready.into_boxed()
    }
    /// The top owns each landing's bounded retry deadline (domain/landing.md, 4.2).
    pub fn wait_landing(&mut self, env: &Env<Limits>, task: u64, until: Option<Wall>) {
        if let Some(row) = self.landing.rows.get(&task)
            && row.room.is_some()
        {
            self.landing.due.remove(&task);
            return;
        }
        let until = match until {
            Some(until) => until,
            None => poll_at(self, env, task),
        };
        self.landing.due.insert(task, until).expect("one deadline per admitted landing");
    }
    pub fn offered_landing(&mut self, task: u64) {
        self.landing.due.remove(&task);
    }
    /// Wall time of the last advance and forge reads issued since it.
    #[must_use]
    pub fn landing_advance(&self, task: u64) -> Option<(Wall, u64)> {
        read_wait::last_advance(self, task)
    }

    pub(crate) fn landing_gate_timeout(&self, project: u32) -> skein_lib::Duration {
        match self.landing.projects.get(&project) {
            Some((_, _, _, _, _, settings, _)) => settings.gate_timeout,
            None => policy::Project::DEFAULT.gate_timeout,
        }
    }

    /// The stall of the current forge wait, exposed to the world referee.
    #[must_use]
    pub fn inspection_landing_bound(&self, task: u64) -> Option<skein_lib::Time> {
        read_wait::bound(self, task).or(read_wait::armed(self, task))
    }

    /// The engine's armed stall wake, separate from the poll.
    #[must_use]
    pub fn inspection_landing_stall_wake(&self, task: u64) -> Option<skein_lib::Time> {
        read_wait::armed(self, task)
    }

    #[must_use]
    pub fn landing(&self, task: u64) -> Option<&LandingRow> {
        self.landing.rows.get(&task)
    }
    #[must_use]
    pub fn receipt(&self, key: &ReceiptKey) -> Option<&Receipt> {
        let receipt = self.landing.receipts.get(key)?;
        if empty_gate_receipt(key, receipt) { None } else { Some(receipt) }
    }
}
pub(crate) fn append_words(words: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if words.push(*byte).is_err() {
            break;
        }
    }
}
/// A change with queued asks cannot wait for a new landing (domain/landing-members.md, 3.2).
#[expect(clippy::match_like_matches_macro, reason = "the foundation requires explicit matching")]
pub(crate) fn waiting(d: &Domain, row: &crate::ChangeRow) -> bool {
    row.change.way == change::Way::Turn
        && d.landing_of(row.task).is_none()
        && row.asks.is_empty()
        && if let change::State::Ready { .. } = row.change.state { true } else { false }
}
fn ordered_changes(d: &Domain, joined: &[u64], graph: &[crate::ChangeDependencies]) -> Box<[u64]> {
    let mut ordered = List::with_capacity(d.landing.limits.changes);
    for _ in 0..joined.len() {
        for member in joined {
            if ordered.as_slice().contains(member) {
                continue;
            }
            let mut blocked = false;
            for edge in graph {
                if edge.change == *member {
                    for upstream in &edge.pending {
                        if joined.contains(upstream) && !ordered.as_slice().contains(upstream) {
                            blocked = true;
                        }
                    }
                }
            }
            if !blocked {
                ordered.push(*member).expect("bounded changes");
            }
        }
    }
    ordered.into_boxed()
}

fn joinable(d: &Domain, project: u32, task: u64) -> bool {
    let Some(row) = d.changes.get(&task) else { return false };
    let Some(repository) = d.repositories.get(&row.repository) else { return false };
    if repository.project != project {
        return false;
    }
    if d.landing_of(task).is_some() || !row.asks.is_empty() {
        return false;
    }
    match row.change.state {
        change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::CancellingGates { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Held { .. } => true,
    }
}

/// Formed landings are sealed (domain/landing.md, 4.1).
pub(crate) fn order(_d: &mut Domain, _task: u64, _changes: &[u64], _out: &mut Queue<Request>) {}

fn member_position(row: &LandingRow, prior: &LandingRow, old: u16) -> Option<u16> {
    // A retained entry or follow may not link to a position in a restored row.
    let before = prior.landing.members.get(usize::from(old))?;
    let change = before.change;
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.change == change {
            return Some(u16::try_from(index).expect("member position"));
        }
    }
    assert!(
        prior.landing.round.branch.is_empty() && before.left.is_none() && before.landed.is_none(),
        "change {change} must be active in single landing {} to leave on cancellation and lose its position",
        row.landing.task,
    );
    None
}

// Positional state follows each retained change when a single landing's member leaves.
fn remap_members(d: &mut Domain, row: &mut LandingRow, prior: &LandingRow) {
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for entry in &prior.entries {
        if let Some(member) = member_position(row, prior, entry.member) {
            entries.push(LandingEntry { member, ..*entry }).expect("bounded entries");
        }
    }
    row.entries = entries.into_boxed();
    let mut follows = List::with_capacity(follow_slots(&d.landing.limits).expect("landing follow bound"));
    for old in prior.follows() {
        if let Some(member) = member_position(row, prior, old.entry.member) {
            let mut follow = old.clone();
            follow.entry.member = member;
            follows.push(follow).expect("bounded follows and marks");
        }
    }
    row.keep_follows(follows.into_boxed());
    // Move only this landing's values out before assigning new positions.
    // The scratch lists live for this step and are bounded by its members.
    let members = u32::try_from(prior.landing.members.len()).expect("member count");
    let gates = d.landing.limits.gates;
    let slots = members.checked_mul(gates).expect("gate slots");
    let mut keys = List::with_capacity(slots);
    let mut faults = List::with_capacity(slots);
    let result_slots = slots.checked_mul(4).expect("gate result slots").checked_add(members).expect("result slots");
    let mut results = List::with_capacity(result_slots);
    for old in 0..members {
        let old = u16::try_from(old).expect("member position");
        for gate in 0..gates {
            let gate = u16::try_from(gate).expect("gate position");
            let key = (row.landing.task, old, gate);
            if let Some(value) = d.landing.keys.remove(&key) {
                keys.push((old, gate, value)).expect("member keys");
            }
            if let Some(value) = d.landing.faults.remove(&key) {
                faults.push((old, gate, value)).expect("member faults");
            }
            for operation in 1..=4 {
                if let Some(value) = d.landing.results.remove(&(row.landing.task, old, gate, operation)) {
                    results.push((old, gate, operation, value)).expect("member results");
                }
            }
        }
        if let Some(value) = d.landing.results.remove(&(row.landing.task, old, 0, 5)) {
            results.push((old, 0, 5, value)).expect("member merge result");
        }
    }
    for (old, gate, value) in keys.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.keys.insert((row.landing.task, member, gate), value).expect("bounded keys");
        }
    }
    for (old, gate, operation, value) in results.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.results.insert((row.landing.task, member, gate, operation), value).expect("bounded results");
        }
    }
    for (old, gate, value) in faults.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.faults.insert((row.landing.task, member, gate), value).expect("bounded faults");
        }
    }
}

pub(crate) fn repository(member: &policy::Member) -> api::Repository {
    api::Repository { forge: member.repository.forge, repository: member.repository.repository }
}
fn gate_key(row: &LandingRow, member: u16, _gate: u16, name: &[u8], key: policy::Key) -> ReceiptKey {
    let m = row.landing.members.get(usize::from(member)).expect("member");
    ReceiptKey { project: row.landing.project, repository: repository(m), check: Check::Gate(Box::from(name)), key }
}
fn proof_key(proof: policy::Proof) -> Option<policy::Key> {
    match proof {
        policy::Proof::Unkeyed => None,
        policy::Proof::Keyed { key }
        | policy::Proof::Dispatched { key, .. }
        | policy::Proof::Passed { key }
        | policy::Proof::Failed { key } => Some(key),
    }
}
pub(crate) fn save(d: &mut Domain, env: &Env<Limits>, row: LandingRow, out: &mut Queue<Request>) {
    let advances = match d.landing.rows.get(&row.landing.task) {
        Some(prior) => read_wait::changed(prior, &row),
        None => true,
    };
    if advances {
        read_wait::advanced(d, env, &row);
    }
    if row.waiting.is_some() {
        crate::waits::changed(d, row.landing.task);
    }
    save_row(d, row, out);
}

// History only writes a timestamp, never an advance.
pub(crate) fn save_row(d: &mut Domain, mut row: LandingRow, out: &mut Queue<Request>) {
    if matches_terminal(&row.landing.phase) {
        leave_room(d, &mut row);
    }
    let project = row.landing.project;
    let task = row.landing.task;
    let was_live =
        if let Some(prior) = d.landing.rows.get(&task) { !matches_terminal(&prior.landing.phase) } else { false };
    let live = !matches_terminal(&row.landing.phase);
    if was_live != live {
        for (task, change) in &d.changes {
            if waiting(d, change) && change.base == row.landing.branch {
                for member in &row.landing.members {
                    if repository(member) == change.repository {
                        d.wait_reports.changed(*task);
                        break;
                    }
                }
            }
        }
    }
    let was_turn = match d.landing.rows.get(&task) {
        Some(prior) => crate::views::matches_turn(&prior.landing.phase),
        None => false,
    };
    let turn_changed = was_turn != crate::views::matches_turn(&row.landing.phase);
    if !crate::views::matches_turn(&row.landing.phase) {
        row.turn_since = None;
    }
    if let Some(prior) = d.landing.rows.get(&task) {
        row.last_event = prior.last_event.or(row.last_event);
    }
    let prior = d.landing.remember(row.clone());
    if was_live && !live {
        members::end(d, &row, out);
    }
    if let Some(prior) = &prior
        && was_live
        && !members::same_members(prior, &row)
    {
        for member in &prior.landing.members {
            if member.left.is_none() && !members::holds(&row, member.change) {
                crate::waits::changed(d, member.change);
                emit(out, Request::Marked { tasks: Box::new([member.change]) });
            }
        }
    }
    if !was_live && live {
        for member in &row.landing.members {
            crate::waits::changed(d, member.change);
            emit(out, Request::Marked { tasks: Box::new([member.change]) });
        }
    }
    emit(out, Request::Save { record: Stored::Landing(row), release: None, read_afresh: None });
    emit(
        out,
        Request::Marked {
            tasks: Box::new([
                task,
                crate::landings_subject(project, false),
                crate::landings_subject(project, true),
                crate::changes_subject(project),
            ]),
        },
    );
    // Every project's landing document names the turn's holder.
    if turn_changed {
        let mut tasks = List::with_capacity(crate::MARKED);
        for (other, saved) in &d.landing.rows {
            if *other != task && saved.landing.project == project {
                if saved.waiting == Some(policy::Awaited::Turn) {
                    d.wait_reports.changed(*other);
                }
                tasks.push(*other).expect("mark room");
                if tasks.room() == 0 {
                    emit(out, Request::Marked { tasks: tasks.into_boxed() });
                    tasks = List::with_capacity(crate::MARKED);
                }
            }
        }
        if !tasks.is_empty() {
            emit(out, Request::Marked { tasks: tasks.into_boxed() });
        }
    }
}
pub(crate) fn register(d: &mut Domain, env: &Env<Limits>, row: LandingRow, out: &mut Queue<Request>) {
    let task = row.landing.task;
    let mut changes = List::with_capacity(env.limits.landing_members);
    for member in &row.landing.members {
        let change = member.change;
        assert!(
            !changes.as_slice().contains(&change),
            "change {change} joining landing {task} must be new to that row",
        );
        let owner = d.landing_of(change);
        assert!(
            owner.is_none(),
            "change {change} joining landing {task} must be in no live landing; held by {owner:?}",
        );
        let added = changes.push(change);
        assert!(
            added.is_ok(),
            "change {change} joining landing {task} must fit its {} member slots",
            env.limits.landing_members,
        );
        let Some(saved) = d.changes.get(&change) else {
            unreachable!("change {change} joining landing {task} must be a preflighted change");
        };
        assert_eq!(saved.repository, repository(member), "change {change} joins landing {task} in its repository");
        assert_eq!(row.landing.branch, saved.base, "change {change} joins landing {task} on its train's branch");
        assert_eq!(saved.pull.unwrap_or(0), member.pull, "change {change} joins landing {task} with its pull");
        assert_eq!(
            member.gates.len(),
            d.train_gates(&row, repository(member)).len(),
            "change {change} joins landing {task} with its configured gates",
        );
    }
    let admission = d.landing_admits(row.landing.project, changes.as_slice());
    assert!(admission.is_ok(), "root preflighted landing {task} with changes {changes:?}: {admission:?}");
    assert!(!d.landing.rows.contains_key(&task), "fresh landing task");
    assert!(row.entries.is_empty(), "new landing has no effects");
    if d.landing.rows.len() == env.limits.landings {
        assert!(retire_finished(d, out), "preflight found a reclaimable terminal landing");
    }
    let count = row.landing.members.len();
    let first = row.landing.members.first().expect("preflighted members");
    crate::train_refusal::replace(d, out, repository(first), row.landing.branch.clone(), None, env.wall);
    save(d, env, row, out);
    for index in 0..count {
        emit(
            out,
            Request::LandingNews {
                task,
                member: u16::try_from(index).expect("bounded members"),
                step: crate::LandingNews::Joined,
            },
        );
    }
}
impl Domain {
    fn train_gates(&self, row: &LandingRow, repo: api::Repository) -> &[policy::Gate] {
        if let Some(repository) = self.repository(repo)
            && repository.landing != row.landing.branch
        {
            return &[];
        }
        self.landing.gates(row.landing.project, repo)
    }
}

fn fresh(gates: &[policy::Gate]) -> policy::MemberFacts {
    let mut facts = List::with_capacity(u32::try_from(gates.len()).expect("bounded gates"));
    let mut failed = List::with_capacity(u32::try_from(gates.len()).expect("bounded gates"));
    for gate in gates {
        failed.push(None).expect("one per gate");
        facts
            .push(policy::GateFacts {
                blocking: gate.blocking,
                key: None,
                receipt: policy::Status::Unknown,
                flight: policy::Flight::None,
                posting: policy::Status::Unknown,
                expired: None,
                run: 0,
            })
            .expect("bounded gates");
    }
    policy::MemberFacts {
        outcome: None,
        dependencies: Box::new([]),
        deleted: policy::Status::Unknown,
        ready: false,
        cancelled: false,
        head: [0; 32],
        main: [0; 32],
        contains_main: policy::Status::Unknown,
        contains_candidate: policy::Status::Unknown,
        asked: false,
        merge: policy::Status::Unknown,
        merged: None,
        gates: facts.into_boxed(),
        failed: failed.into_boxed(),
        contained: policy::Status::Unknown,
        candidate: None,
        preparation: policy::Preparation::Idle,
        record: policy::Status::Unknown,
        held: false,
        judgement: policy::Status::Passed,
        refused: None,
    }
}
fn pending_for(d: &Domain, task: u64) -> Option<Token> {
    for (token, pending) in &d.landing.pending {
        if pending.task == task {
            return Some(*token);
        }
    }
    None
}
// A procedure consumes its result even when a prefix read is busy (domain/landing-prepare.md, 1.2).
fn remember_preparation(d: &mut Domain, env: &Env<Limits>, task: u64, heard: &LandingHeard, out: &mut Queue<Request>) {
    let preparation = heard.preparation.unwrap_or(policy::Preparation::Running);
    if heard.outcomes.is_empty()
        && preparation != policy::Preparation::Failed
        && preparation != policy::Preparation::Conflict
    {
        return;
    }
    let Some(row) = d.landing.rows.get(&task) else { return };
    let failure = crate::preparation::remember(&row.landing.round.failure, &heard.failure, env.limits.landing_members);
    if row.landing.round.failure != failure {
        let mut row = row.clone();
        row.landing.round.failure = failure;
        save(d, env, row, out);
    }
}

pub(crate) fn job_woken(d: &mut Domain, task: u64) {
    if let Some(token) = pending_for(d, task) {
        d.landing.pending.get_mut(&token).expect("pending landing gather").woken = true;
    }
}
/// A member's evidence before its reads: only what its change row already holds.
fn unread_evidence(member: u64, change: &crate::ChangeRow) -> crate::LandingEvidence {
    crate::LandingEvidence {
        change: member,
        facts: crate::ChangeEvidence {
            behind: None,
            readback_owed: false,
            nearest_write: None,
            touched: crate::Touched::UNKNOWN,
            temper_dir: crate::TemperDirEvidence::Uncovered,
            prior: change.change.clone(),
            head: None,
            base_tip: [0; 32],
            contains_base: change::Status::Unknown,
            ci: change::Status::Unknown,
            gates: change.verdicts.clone(),
            reviews: Box::new([]),
            reviews_complete: false,
            size: None,
        },
    }
}

// No prepare can belong to a train round before its base is initialized.
// Retire repeated results before prefix reads can wait on a deleted branch
// (temper/docs/design/domain/landing-prepare.md, 1.1–1.2, and temper/docs/design/domain/landing.md, 15).
fn retire_preparation(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    heard: &mut LandingHeard,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.landing.rows.get(&task) else { return };
    if row.landing.round.branch.is_empty() {
        return;
    }
    let mut uninitialized = false;
    for member in &row.landing.members {
        if member.left.is_none() && member.landed.is_none() && member.base.is_none() {
            uninitialized = true;
        }
    }
    if !uninitialized {
        return;
    }
    heard.outcomes = Box::new([]);
    heard.failure = Box::new([]);
    let failure = crate::preparation::next_round(&row.landing.round.failure, env.limits.landing_members);
    if row.landing.round.failure != failure {
        let mut row = row.clone();
        row.landing.round.failure = failure;
        save(d, env, row, out);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one landing step keeps its admission and operator hold with the state handoff"
)]
pub(crate) fn step(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    mut heard: LandingHeard,
    out: &mut Queue<Request>,
) {
    if d.stopping || d.landing.unmade.contains_key(&task) {
        return;
    }
    crate::room::stepping(d, task);
    if d.landing.claims.contains_key(&task) {
        claim_step(d, task, entry, heard.released, out);
        return;
    }
    if d.landing.held {
        emit_decision(
            out,
            Request::LandingDecision {
                task,
                decision: policy::Decision::Wait { until: None, on: policy::Awaited::Release },
                entry: None,
                evidence: None,
                head_trailers: Box::new([]),
            },
            d,
            env,
        );
        return;
    }
    if let Some(row) = d.landing.rows.get(&task)
        && let policy::Phase::Held { why: policy::Hold::Claimed { .. }, .. } = row.landing.phase
    {
        claim_step(d, task, entry, heard.released, out);
        return;
    }
    let pending = pending_for(d, task);
    // The start leaves a real step's entry and hearing untouched (landing-preparing.md, 3).
    if entry == 0 && pending.is_some() {
        return;
    }
    if heard.released
        && let Some(row) = d.landing.rows.get(&task).cloned()
    {
        read_wait::released(d, env, &row);
    }
    if let Some(row) = d.landing.rows.get(&task)
        && let Some(local) = local_step(row, &heard)
    {
        if entry == 0 {
            emit(out, Request::WakeProcedure { task });
        } else {
            if let Some(owner) = pending {
                d.landing.pending.remove(&owner);
                d.read_line.leave(Waiter::Landing(owner));
            }
            read_ready(d, task, local, out);
        }
        return;
    }
    if let Some(job) = heard.job.take() {
        crate::job_history::heard(d, env, task, job, out);
    }
    retire_preparation(d, env, task, &mut heard, out);
    remember_preparation(d, env, task, &heard, out);
    if let Some(row) = d.landing.rows.get(&task)
        && row.landing.phase == policy::Phase::Cancelled
    {
        if entry == 0 {
            emit(out, Request::WakeProcedure { task });
        } else {
            emit_decision(
                out,
                Request::LandingDecision {
                    task,
                    decision: policy::Decision::Cancel,
                    entry: None,
                    evidence: None,
                    head_trailers: Box::new([]),
                },
                d,
                env,
            );
        }
        return;
    }
    if entry != 0
        && !heard.cancelled
        && !heard.released
        && let Some(why) = crate::room::expired_landing(d, env, task)
    {
        read_ready(d, task, ReadStep::RoomExpired(why), out);
        return;
    }
    if let Some(token) = pending {
        let pending = d.landing.pending.get_mut(&token).expect("pending step");
        pending.entry = entry;
        pending.heard = heard;
        pending.woken = true;
        return;
    }
    let Some(mut row) = d.landing.rows.get(&task).cloned() else {
        emit(out, Request::Refused { task });
        return;
    };
    for member in &mut row.landing.members {
        let Some(change) = d.changes.get(&member.change) else { return };
        let Some(pull) = change.pull else {
            let failure = read_wait::Failure::missing_pull(change.repository, member.change);
            wait(d, task, entry, failure, out);
            return;
        };
        member.pull = pull;
    }
    if d.landing.rows.get(&task) != Some(&row) {
        save(d, env, row.clone(), out);
    }
    if entry != 0
        && pending.is_none()
        && !heard.cancelled
        && let Some(until) = read_wait::bound(d, task)
        && until <= env.now
    {
        read_ready(d, task, ReadStep::Stalled, out);
        return;
    }
    let mut facts = List::with_capacity(env.limits.landing_members);
    let mut evidence = List::with_capacity(env.limits.landing_members);
    let mut paths = List::with_capacity(env.limits.landing_members);
    for member in &row.landing.members {
        let change = d.changes.get(&member.change).expect("member change");
        evidence.push(unread_evidence(member.change, change)).expect("bounded evidence");
        facts.push(fresh(d.train_gates(&row, repository(member)))).expect("admitted members");
        paths.push(Paths { files: Box::new([]), whole: false, comparison: None }).expect("admitted members");
    }
    let Some(first) = row.landing.members.first() else {
        return;
    };
    let Some(change) = d.changes.get(&first.change) else {
        return;
    };
    let repo = change.repository;
    let read = api::Read::Branch { branch: change.branch.clone() };
    d.sequence = d.sequence.checked_add(1).expect("read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 62));
    d.landing
        .pending
        .insert(
            owner,
            Pending {
                task,
                repository: repo,
                resource: gather_resource(&read),
                read: read.clone(),
                retry: Retry::None,
                cached: false,
                entry,
                heard,
                facts: facts.into_boxed(),
                head_trailers: List::with_capacity(env.limits.landing_members),
                stage: Stage::Branch(0),
                polled: List::with_capacity(slots(&env.limits).expect("bounded gates")),
                flight_entry: None,
                trees: List::with_capacity(env.limits.trees_in_flight),
                base_trees: List::with_capacity(env.limits.trees_in_flight),
                entries: List::with_capacity(env.limits.key_entries),
                woken: false,
                evidence: evidence.into_boxed(),
                paths: paths.into_boxed(),
                temper_dir: None,
                opened: None,
                closed: policy::Status::Unknown,
                round_pull: policy::RoundPull::Unread,
                turn_tip: Seen::Current,
                cleaned: false,
                upstream: None,
                stall: None,
                reads: Map::with_capacity(env.limits.landing_members.checked_mul(5).expect("u16 members fit")),
            },
        )
        .expect("one chain per landing");
    read_wait::issued(d, env, owner);
    child(d, env, skein_forge_client::Event::Read { owner, repository: repo, read }, out);
}
fn ask(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: Pending,
    repository: api::Repository,
    read: api::Read,
    out: &mut Queue<Request>,
) {
    pending.repository = repository;
    pending.resource = gather_resource(&read);
    pending.read = read.clone();
    pending.retry = Retry::None;
    pending.cached = match pending.resource.clone() {
        Some(resource) => pending.reads.contains_key(&(repository, resource)),
        None => false,
    };
    let cached = pending.cached;
    d.landing.pending.insert(owner, pending).expect("replaces pending chain");
    if !cached {
        read_wait::issued(d, env, owner);
        child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
    }
}
pub(crate) fn gather_read(d: &Domain, owner: Token) -> bool {
    match d.landing.pending.get(&owner) {
        Some(pending) => !matches_stall(&pending.stage),
        None => false,
    }
}

pub(crate) fn wait_in_line(d: &mut Domain, owner: Token, now: skein_lib::Time, out: &mut Queue<Request>) {
    d.landing.pending.get_mut(&owner).expect("waiting gather").retry = Retry::Waiting;
    read_line::join(d, Waiter::Landing(owner), now, out);
}

pub(crate) fn ask_again(d: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let Some(pending) = d.landing.pending.get(&owner) else { return };
    let event = skein_forge_client::Event::Read { owner, repository: pending.repository, read: pending.read.clone() };
    asking(d, owner);
    child(d, env, event, out);
}

pub(crate) fn asking(d: &mut Domain, owner: Token) {
    d.landing.pending.get_mut(&owner).expect("pending gather").retry = Retry::None;
}

pub(crate) fn connected(d: &mut Domain, now: skein_lib::Time, forge: u16, out: &mut Queue<Request>) {
    let mut owners = List::with_capacity(d.landing.pending.len());
    for (owner, pending) in &d.landing.pending {
        if pending.repository.forge == forge && pending.retry == Retry::Disconnected {
            owners.push(*owner).expect("one wait per landing");
        }
    }
    for owner in owners.into_boxed() {
        wait_in_line(d, owner, now, out);
    }
}

#[cfg(test)]
pub(crate) fn read_waiting(d: &Domain) -> bool {
    if let Some(Waiter::Landing(_)) = d.read_line.ready_head() {
        return true;
    }
    false
}

#[cfg(test)]
pub(crate) fn retry(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    read_line::serve(d, env, out);
}

pub(crate) fn settle(d: &mut Domain) {
    let mut waiting = List::with_capacity(d.landing.pending.len());
    for (owner, pending) in &d.landing.pending {
        if pending.retry != Retry::None {
            waiting.push(*owner).expect("one retry per landing");
        }
    }
    for owner in &waiting {
        d.landing.pending.remove(owner);
        d.read_line.leave(Waiter::Landing(*owner));
    }
}

fn poll_at(d: &Domain, env: &Env<Limits>, task: u64) -> Wall {
    let mut at = None;
    if let Some(row) = d.landing.rows.get(&task) {
        for member in &row.landing.members {
            let due = d.poll_at(env, repository(member));
            at = Some(match at {
                Some(prior) => due.min(prior),
                None => due,
            });
        }
    }
    let at = at.unwrap_or(env.now.saturating_add(env.limits.client.poll));
    wall_at(env, at)
}
fn wall_at(env: &Env<Limits>, at: skein_lib::Time) -> Wall {
    Wall::from_nanos(env.wall.as_nanos().saturating_add(at.as_nanos().saturating_sub(env.now.as_nanos())))
}
fn wait(d: &mut Domain, task: u64, entry: u64, failure: read_wait::Failure, out: &mut Queue<Request>) {
    // Every end of the start's read is a wake, never a decision (landing-preparing.md, 3).
    if entry == 0 {
        emit(out, Request::WakeProcedure { task });
    } else {
        read_ready(d, task, ReadStep::Failed(failure), out);
    }
}

fn read_failed(d: &mut Domain, row: &LandingRow, p: &Pending, error: Option<api::Error>, out: &mut Queue<Request>) {
    let failure = read_wait::Failure::pending(row, p, read_wait::Cause::answered(error));
    wait(d, p.task, p.entry, failure, out);
}

fn local_step(row: &LandingRow, heard: &LandingHeard) -> Option<ReadStep> {
    match row.landing.phase {
        policy::Phase::Held { why: policy::Hold::Read | policy::Hold::Cleanup, .. } => {
            Some(ReadStep::Held { released: heard.released })
        }
        policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
            if row.landing.round.cleanup.is_some() && !heard.released && !heard.cancelled =>
        {
            Some(ReadStep::CleanupFailed)
        }
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Held { .. }
        | policy::Phase::Landed
        | policy::Phase::Cancelled => None,
    }
}

fn claim_step(d: &mut Domain, task: u64, entry: u64, released: bool, out: &mut Queue<Request>) {
    if entry == 0 {
        emit(out, Request::WakeProcedure { task });
    } else {
        ready(d, task, Ready::Claimed { released }, out);
    }
}

fn claim_decision(d: &mut Domain, env: &Env<Limits>, task: u64, released: bool, out: &mut Queue<Request>) {
    let Some(mut row) = d.landing.rows.get(&task).cloned() else { return };
    let prior = row.clone();
    // The first decision observes the start's claim. Only release checks again.
    let conflict = match d.landing.claims.remove(&task) {
        Some(change) => Some(change),
        None if released => members::conflict(d, &row),
        None => None,
    };
    let decision = if let Some(change) = conflict {
        leave_room(d, &mut row);
        row.read_hold = None;
        let stepped = policy::claim(row.landing, change);
        row.landing = stepped.landing;
        stepped.decision
    } else {
        let stepped = policy::held(row.landing, released, env.wall);
        row.landing = stepped.landing;
        stepped.decision
    };
    if row != prior {
        crate::landing_history::transition(d, env.wall, &prior, &row, out);
        save(d, env, row, out);
    }
    if let policy::Decision::Hold(policy::Hold::Claimed { .. }) = decision {
        held_news(task, &prior, out);
    }
    emit_decision(
        out,
        Request::LandingDecision { task, decision, entry: None, evidence: None, head_trailers: Box::new([]) },
        d,
        env,
    );
    crate::waits::changed(d, task);
}

fn held_news(task: u64, row: &LandingRow, out: &mut Queue<Request>) {
    for (member, _) in row.landing.members.iter().enumerate() {
        emit(
            out,
            Request::LandingNews {
                task,
                member: u16::try_from(member).expect("bounded members"),
                step: crate::LandingNews::Held,
            },
        );
    }
}

fn read_ready(d: &mut Domain, task: u64, step: ReadStep, out: &mut Queue<Request>) {
    ready(d, task, Ready::Read(step), out);
}

fn read_decision(d: &mut Domain, env: &Env<Limits>, task: u64, step: ReadStep, out: &mut Queue<Request>) {
    let Some(mut row) = d.landing.rows.get(&task).cloned() else { return };
    let prior = row.clone();
    // A hold must keep the room order, even if the holding task steps again.
    if let ReadStep::Stalled | ReadStep::Failed(_) | ReadStep::CleanupFailed = &step {
        leave_room(d, &mut row);
    }
    let decision = match step {
        ReadStep::RoomExpired(why) => {
            if let Some(owner) = pending_for(d, task) {
                d.landing.pending.remove(&owner);
                d.read_line.leave(Waiter::Landing(owner));
            }
            d.landing.due.remove(&task);
            crate::room::pass(d, task);
            read_wait::room_held(d, env, &mut row, &why, out);
            row.landing.phase = policy::Phase::Held { was: Box::new(row.landing.phase), why: policy::Hold::Read };
            policy::Decision::Hold(policy::Hold::Read)
        }
        ReadStep::Held { released } => {
            let stepped = policy::held(row.landing, released, env.wall);
            row.landing = stepped.landing;
            if released {
                clear_cleanup_results(d, &row);
                row.read_hold = None;
                row.gate_hold = None;
                crate::room::release_place(d, crate::room::Waiter::Landing(task), env.wall);
            }
            stepped.decision
        }
        ReadStep::CleanupFailed => {
            let stepped = policy::cleanup_hold(row.landing);
            row.landing = stepped.landing;
            stepped.decision
        }
        ReadStep::Failed(_) | ReadStep::Stalled => {
            let repository = if let ReadStep::Failed(failure) = step {
                read_wait::failed(d, env, &row, failure);
                read_wait::repository_waiting(d, task).expect("failed wait's provider")
            } else {
                let Some(repository) = read_wait::repository_waiting(d, task) else {
                    emit(out, Request::WakeProcedure { task });
                    return;
                };
                repository
            };
            if let Some(until) = read_wait::bound(d, task)
                && until <= env.now
            {
                read_wait::held(d, env, &mut row, out);
                row.landing.phase = policy::Phase::Held { was: Box::new(row.landing.phase), why: policy::Hold::Read };
                policy::Decision::Hold(policy::Hold::Read)
            } else {
                let poll = d.poll_at(env, repository);
                let at = match read_wait::bound(d, task) {
                    Some(until) => until.min(poll),
                    None => poll,
                };
                policy::Decision::Wait { until: Some(wall_at(env, at)), on: policy::Awaited::Read }
            }
        }
    };
    if row != prior {
        crate::landing_history::transition(d, env.wall, &prior, &row, out);
        if row.landing.round.ending.is_some() && prior.landing.round.ending.is_none() {
            end_runs(d, env, &prior, &mut row, crate::history::GateVerdict::Superseded, None, out);
        }
        save(d, env, row, out);
    }
    if let policy::Decision::Hold(policy::Hold::Read | policy::Hold::Cleanup) = decision {
        held_news(task, &prior, out);
    }
    emit_decision(
        out,
        Request::LandingDecision { task, decision, entry: None, evidence: None, head_trailers: Box::new([]) },
        d,
        env,
    );
    crate::waits::changed(d, task);
}

pub(crate) fn read_deadline(d: &Domain) -> Option<skein_lib::Time> {
    read_wait::deadline(d)
}

pub(crate) fn fire_reads(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    read_wait::fire(d, env, out);
}

pub(crate) fn read_connection(d: &mut Domain, env: &Env<Limits>, forge: u16, connected: bool) {
    read_wait::connection(d, env, forge, connected);
}

pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    advance(d, env, owner, answer, out);
    while let Some((owner, answer)) = cached_answer(d) {
        advance(d, env, owner, answer, out);
    }
}
fn cached_answer(d: &Domain) -> Option<(Token, Result<api::Answer, api::Error>)> {
    for (owner, pending) in &d.landing.pending {
        if pending.cached {
            let resource = pending.resource.clone().expect("only a resource read is cached");
            let answer = pending.reads.get(&(pending.repository, resource)).expect("cached read remains");
            return Some((*owner, answer.clone()));
        }
    }
    None
}
#[expect(clippy::too_many_lines, reason = "one exhaustive bounded landing read chain")]
#[expect(clippy::match_like_matches_macro, reason = "the foundation forbids matches macros in step code")]
fn advance(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut p) = d.landing.pending.remove(&owner) else {
        return;
    };
    if !d.repositories.contains_key(&p.repository) {
        return;
    }
    let cached = p.cached;
    p.cached = false;
    if d.stopping {
        return;
    }
    if !matches_stall(&p.stage) {
        match answer {
            Err(api::Error::Busy) => {
                // Admission made no call. Keep the coherent prefix and wait for this read.
                d.landing.pending.insert(owner, p).expect("keeps the gather's slot");
                wait_in_line(d, owner, env.now, out);
                return;
            }
            Err(api::Error::NotConnected) => {
                p.retry = Retry::Disconnected;
                read_wait::pause(d, p.task);
                d.landing.pending.insert(owner, p).expect("keeps the gather's slot");
                return;
            }
            Ok(_) | Err(_) => {}
        }
    }
    let error = answer.as_ref().err().copied();
    if !cached {
        read_wait::resume(d, env, p.task, p.repository);
        read_wait::answered(d, &p, error);
    }
    let Some(row) = d.landing.rows.get(&p.task).cloned() else {
        return;
    };
    if row.landing.phase == policy::Phase::Cancelled || local_step(&row, &p.heard).is_some() {
        step(d, env, p.task, p.entry, p.heard, out);
        return;
    }
    // A reader of a shared key may outlive a cancellation and its replacement
    // dispatch. Its answers belong only to the write it was following.
    if let Stage::Run { key, .. } | Stage::FailedJobs { key, .. } | Stage::FailedReport { key, .. } = &p.stage {
        let current = match d.landing.flights.get(key) {
            Some(flight) => Some(flight.entry),
            None => None,
        };
        if current.is_none() || current != p.flight_entry {
            collect(d, env, owner, p, out);
            return;
        }
    }
    if p.evidence.len() != row.landing.members.len() {
        step(d, env, p.task, p.entry, p.heard, out);
        return;
    }
    for (member, evidence) in row.landing.members.iter().zip(&p.evidence) {
        if member.change != evidence.change {
            step(d, env, p.task, p.entry, p.heard, out);
            return;
        }
    }
    if let Some(resource) = p.resource.take() {
        p.reads.insert((p.repository, resource), answer.clone()).expect("five resources per member");
    }
    match p.stage.clone() {
        Stage::StallRun | Stage::StallJobs => {
            if let Stage::StallRun = p.stage {
                p.stall = Some(match answer {
                    Ok(api::Answer::Run(run)) => crate::stall::Report::run(&run),
                    Ok(_) | Err(_) => crate::stall::Report::checks(&[]),
                });
            } else {
                p.stall.as_mut().expect("report").answered(answer);
            }
            if let Some(read) = p.stall.as_mut().expect("report").next() {
                p.stage = Stage::StallJobs;
                let repository = p.repository;
                ask(d, env, owner, p, repository, read, out);
            } else {
                decide(d, owner, p, out);
            }
        }
        Stage::Branch(index)
        | Stage::Main(index)
        | Stage::Ancestry(index)
        | Stage::ContainsCandidate(index)
        | Stage::Pull(index) => {
            let member = row.landing.members.get(usize::from(index)).expect("member");
            let Some(change) = d.changes.get(&member.change) else {
                return;
            };
            let repo = change.repository;
            let fact = p.facts.get_mut(usize::from(index)).expect("member fact");
            let read = match p.stage {
                Stage::Branch(_) => {
                    let head = match answer {
                        Ok(api::Answer::Commit(head)) => head,
                        Ok(_) | Err(_) => {
                            // Its change can finish and release the branch as soon as
                            // the merge is read back, before this landing's next read.
                            let landed = match change.change.state {
                                change::State::Landed { merge, .. }
                                | change::State::Red { merge, .. }
                                | change::State::Cleared { merge }
                                | change::State::Uncleared { merge } => Some(merge),
                                change::State::Taking { .. }
                                | change::State::Producing { .. }
                                | change::State::Opening { .. }
                                | change::State::Recreating { .. }
                                | change::State::Reopening { .. }
                                | change::State::Checking { .. }
                                | change::State::Judging { .. }
                                | change::State::Gating { .. }
                                | change::State::Ready { .. }
                                | change::State::Preparing { .. }
                                | change::State::CancellingGates { .. }
                                | change::State::Queued { .. }
                                | change::State::First { .. }
                                | change::State::Updating { .. }
                                | change::State::WaitingProducer { .. }
                                | change::State::Resolving { .. }
                                | change::State::Repairing { .. }
                                | change::State::Landing { .. }
                                | change::State::NoChange { .. }
                                | change::State::Ended
                                | change::State::Held { .. } => {
                                    if member.left.is_some() {
                                        member.head
                                    } else {
                                        member.landed
                                    }
                                }
                            };
                            let Some(head) = landed else {
                                read_failed(d, &row, &p, error, out);
                                return;
                            };
                            if member.left.is_none() && member.landed.is_none() && Some(head) != member.candidate {
                                read_failed(d, &row, &p, error, out);
                                return;
                            }
                            head
                        }
                    };
                    fact.head = head;
                    p.stage = Stage::Main(index);
                    let branch = if turn_top(&row) == Some(index) { &row.landing.branch } else { &change.base };
                    api::Read::Branch { branch: branch.clone() }
                }
                Stage::Main(_) => {
                    let Ok(api::Answer::Commit(main)) = answer else {
                        read_failed(d, &row, &p, error, out);
                        return;
                    };
                    if turn_top(&row) == Some(index) {
                        p.turn_tip = observe_tip(d, &env.limits, repo, &row.landing.branch, Some(main), out);
                        if wait_for_record(d, &row, &p, p.turn_tip, out) {
                            return;
                        }
                        if member.candidate == Some(main) {
                            settle_turn_merge(d, &env.limits, &row, index, true, out);
                        }
                    }
                    let fact = p.facts.get_mut(usize::from(index)).expect("member fact");
                    fact.main = main;
                    if let Some(candidate) = member.candidate
                        && !row.landing.round.branch.is_empty()
                    {
                        if let policy::Phase::Turn { .. } = row.landing.phase
                            && main != candidate
                            && Some(main) != member.base
                        {
                            p.stage = Stage::ContainsCandidate(index);
                            api::Read::Commits { before: main, after: candidate, page: 1 }
                        } else {
                            p.stage = Stage::Pull(index);
                            api::Read::Pull { number: member.pull, ci: false }
                        }
                    } else {
                        p.stage = Stage::Ancestry(index);
                        api::Read::Commits { before: fact.head, after: main, page: 1 }
                    }
                }
                Stage::Ancestry(_) => {
                    fact.contains_main = match crate::domain::ancestry_status(answer) {
                        change::Status::Passed => policy::Status::Passed,
                        change::Status::Failed => policy::Status::Failed,
                        change::Status::Unknown | change::Status::Pending => policy::Status::Unknown,
                    };
                    p.stage = Stage::Pull(index);
                    api::Read::Pull { number: member.pull, ci: false }
                }
                Stage::ContainsCandidate(_) => {
                    fact.contains_candidate = match crate::domain::ancestry_status(answer) {
                        change::Status::Passed => policy::Status::Passed,
                        change::Status::Failed => policy::Status::Failed,
                        change::Status::Unknown | change::Status::Pending => {
                            read_failed(d, &row, &p, error, out);
                            return;
                        }
                    };
                    if fact.contains_candidate == policy::Status::Passed && turn_top(&row) == Some(index) {
                        settle_turn_merge(d, &env.limits, &row, index, true, out);
                    }
                    p.stage = Stage::Pull(index);
                    api::Read::Pull { number: member.pull, ci: false }
                }
                Stage::Pull(_) => {
                    let Ok(api::Answer::Pull(pull)) = answer else {
                        read_failed(d, &row, &p, error, out);
                        return;
                    };
                    let evidence = &mut p.evidence.get_mut(usize::from(index)).expect("member evidence").facts;
                    evidence.head = Some(pull.commit);
                    evidence.size = pull.size;
                    fact.merged = pull.merged;
                    fact.ready =
                        if let change::State::Ready { .. } | change::State::Judging { .. } = change.change.state {
                            true
                        } else {
                            false
                        };
                    fact.held = p.heard.held.contains(&member.change)
                        || if let change::State::Held { .. } = change.change.state { true } else { false };
                    fact.cancelled = change.change.state == change::State::Ended
                        || p.heard.ended.contains(&member.change)
                        || p.heard.cancelled;
                    p.stage = Stage::Reviews(index, 1);
                    api::Read::Reviews { number: member.pull, page: 1, skip: 0 }
                }
                Stage::Reviews(_, _)
                | Stage::Paths(_)
                | Stage::TemperDir
                | Stage::Collect
                | Stage::Round { .. }
                | Stage::StallRun
                | Stage::StallJobs
                | Stage::Run { .. }
                | Stage::CleanupPull
                | Stage::TurnPull
                | Stage::CleanupBranch(_)
                | Stage::FailedJobs { .. }
                | Stage::FailedReport { .. }
                | Stage::Tree { .. } => {
                    unreachable!("member stage")
                }
            };
            ask(d, env, owner, p, repo, read, out);
        }
        Stage::Reviews(index, page) => {
            let evidence = &mut p.evidence.get_mut(usize::from(index)).expect("member evidence").facts;
            if let Ok(api::Answer::Reviews { reviews, more, .. }) = answer {
                let mut gathered = List::with_capacity(env.limits.client.inbox);
                for review in &evidence.reviews {
                    gathered.push(review.clone()).expect("bounded reviews");
                }
                let mut fits = true;
                for review in reviews {
                    if gathered.push(review).is_err() {
                        fits = false;
                        break;
                    }
                }
                evidence.reviews = gathered.into_boxed();
                if fits && more && page < env.limits.client.rows {
                    let member = row.landing.members.get(usize::from(index)).expect("member");
                    let page = page.checked_add(1).expect("bounded review pages");
                    p.stage = Stage::Reviews(index, page);
                    ask(
                        d,
                        env,
                        owner,
                        p,
                        repository(member),
                        api::Read::Reviews { number: member.pull, page, skip: 0 },
                        out,
                    );
                    return;
                }
                evidence.reviews_complete = fits && !more;
            }
            let member = row.landing.members.get(usize::from(index)).expect("member");
            let change = d.changes.get(&member.change).expect("member's change");
            let after = path_candidate(&row, index);
            if let Some((after, repository)) = after.zip(d.repositories.get(&change.repository)) {
                for held in &d.landing_rules(repository.project) {
                    if !held.rule.sets.is_empty() && crate::covers_branch(&held.rule.pattern, repository, &change.base)
                    {
                        let before = p.facts.get(usize::from(index)).expect("member fact").main;
                        let repo = change.repository;
                        p.stage = Stage::Paths(index);
                        ask(d, env, owner, p, repo, api::Read::Compare { before, after }, out);
                        return;
                    }
                }
            }
            next_member(d, env, owner, p, &row, index, out);
        }
        Stage::Paths(index) => {
            let fact = p.facts.get(usize::from(index)).expect("member fact");
            let paths = p.paths.get_mut(usize::from(index)).expect("member paths");
            if let api::Read::Compare { before, after } = p.read {
                paths.comparison = Some(temper_dir::Comparison::new(before, after, &answer));
            }
            if let Ok(api::Answer::Compare { before, after, files, changed, .. }) = answer {
                paths.whole = before == fact.main
                    && Some(after) == path_candidate(&row, index)
                    && skein_lib::len_u64(files.len()) == changed;
                paths.files = files;
            }
            next_member(d, env, owner, p, &row, index, out);
        }
        Stage::TemperDir => temper_dir::answered(d, env, owner, p, &row, &answer, out),
        Stage::TurnPull => {
            let Ok(api::Answer::Pull(pull)) = answer else {
                read_failed(d, &row, &p, error, out);
                return;
            };
            if !judge_turn_pull(d, &env.limits, &row, &mut p, &pull, out) {
                return;
            }
            collect(d, env, owner, p, out);
        }
        Stage::CleanupPull => {
            let Ok(api::Answer::Pull(pull)) = answer else {
                read_failed(d, &row, &p, error, out);
                return;
            };
            if pull.state == api::State::Closed || pull.merged.is_some() {
                p.closed = policy::Status::Passed;
            }
            cleanup_branch(d, env, owner, p, &row, 0, out);
        }
        Stage::CleanupBranch(member) => {
            match answer {
                Ok(api::Answer::Commit(_)) => {}
                Err(api::Error::Missing) => {
                    p.facts.get_mut(usize::from(member)).expect("cleanup member").deleted = policy::Status::Passed;
                }
                Ok(_) | Err(_) => {
                    read_failed(d, &row, &p, error, out);
                    return;
                }
            }
            cleanup_branch(d, env, owner, p, &row, member.checked_add(1).expect("member bound"), out);
        }
        Stage::FailedReport { key, run, report } => {
            let missing = answer == Err(api::Error::Missing);
            let failures = match &answer {
                Ok(api::Answer::File { truncated: true, .. }) => Some(temper_engine_domain_forge_tiers::Failures::Many),
                Ok(api::Answer::File { bytes, .. }) => temper_engine_domain_forge_tiers::selection(
                    report.format,
                    bytes,
                    &temper_engine_domain_forge_tiers::Limits {
                        failing_tests: env.limits.tiers.failing_tests,
                        test_id_bytes: env.limits.tiers.test_id_bytes,
                    },
                ),
                Ok(_) | Err(_) => None,
            };
            let report = receipt_report(&report, env, answer);
            let problem = if missing { ReportProblem::Missing } else { report_problem(&report) };
            judge_end(d, env, owner, p, key, GateEnd { run, report, failures }, problem, out);
        }
        Stage::FailedJobs { key, ended, next, words } => {
            let mut log = List::with_capacity(8192);
            append_words(&mut log, &words);
            match answer {
                Ok(api::Answer::Job { step, log: tail, truncated, .. }) => {
                    if let Some(step) = step {
                        append_words(&mut log, b"Failed step: ");
                        append_words(&mut log, &step.name);
                        append_words(&mut log, b"\n");
                    }
                    append_words(&mut log, b"End of its log:\n");
                    if truncated {
                        append_words(&mut log, b"[log prefix omitted]\n");
                    }
                    append_words(&mut log, &tail);
                    append_words(&mut log, b"\n");
                }
                Ok(_) | Err(_) => append_words(&mut log, b"[job log could not be read]\n"),
            }
            failed_jobs(d, env, owner, p, key, ended, next, log.into_boxed(), out);
        }
        Stage::Round { member, stage, candidate } => {
            round_read(d, env, owner, p, &row, member, stage, candidate, answer, out);
        }
        Stage::Run { key, pages } => {
            if old_flake_attempt(d, &key, &answer) {
                p.polled.push(key).expect("one poll per flight");
                p.stage = Stage::Collect;
                collect(d, env, owner, p, out);
                return;
            }
            let writer = d.writers.get(&key.repository.forge).copied().unwrap_or(0);
            // Another landing can have completed this key while this read was in flight.
            let Some(flight) = d.landing.flights.get_mut(&key) else {
                p.stage = Stage::Collect;
                collect(d, env, owner, p, out);
                return;
            };
            let previous_run = match &flight.run {
                Some(run) => Some(run.id),
                None => None,
            };
            let listing = if let Ok(api::Answer::Runs { .. }) = &answer { true } else { false };
            let ended = flight.answered(answer, writer, env.now, env.wall, env.limits.client.lifetime);
            let next = if listing && pages < env.limits.client.rows { flight.read() } else { None };
            let candidate = flight.commit;
            let current_run = match &flight.run {
                Some(run) => Some(run.id),
                None => None,
            };
            let flight = d.landing.flights.get(&key).expect("observed flight").clone();
            let mut verdict = match &ended {
                Some(ended) => match ended.outcome {
                    policy::Verdict::Passed => Some(crate::history::GateVerdict::Passed),
                    policy::Verdict::Failed | policy::Verdict::Running | policy::Verdict::Fault(_) => None,
                },
                None => None,
            };
            let dispatching = d.landing.rows.get(&flight.task).expect("dispatching landing").clone();
            if flight.task == p.task
                && let Some(entry) = dispatching.run_entry(flight.entry)
            {
                let member = row.landing.members.get(usize::from(entry.member)).expect("flight member");
                let fresh = p.facts.get(usize::from(entry.member)).expect("fresh flight member");
                if member.head != Some(fresh.head) {
                    verdict = Some(crate::history::GateVerdict::Superseded);
                }
            }
            let mut observed = dispatching.clone();
            crate::landing_proofs::run(d, env, &mut observed, &flight, verdict, out);
            if observed.follows != dispatching.follows || observed.entries != dispatching.entries {
                save(d, env, observed, out);
            }
            if previous_run != current_run {
                mark_key(d, &key, out);
            }
            if let Some(ended) = ended {
                match ended.outcome {
                    policy::Verdict::Passed => {
                        let run = ended.run.expect("a passed verdict has a run");
                        let receipt = Receipt {
                            report: ReceiptReport::Unset,
                            base: None,
                            jobs: Box::new([]),
                            logs: Box::new([]),
                            failures: None,
                            passed: true,
                            candidate,
                            ended: env.wall,
                            run: ReceiptRun { id: run.id, number: run.number, url: run.url },
                        };
                        count_gate_flakes(d, env, flight.task, &key, out);
                        if tests_value(&flight.inputs).is_none() {
                            put_gate_receipt(d, env, &key, receipt, out);
                        }
                        finish_flight(d, env, &key, Some((run.number, true)), out);
                    }
                    policy::Verdict::Failed => {
                        let run = ended.run.expect("a failed verdict has a run");
                        judge_run(d, env, owner, p, key, run, out);
                        return;
                    }
                    policy::Verdict::Fault(fault) => {
                        let mut problem = GateProblem::Runner { fault, job: None };
                        if let Some(run) = ended.run
                            && run.commit == candidate
                            && run.state == api::RunState::Completed
                        {
                            let (workflow_failed, job_fault) = runs::gate_jobs(&run);
                            if workflow_failed {
                                judge_run(d, env, owner, p, key, run, out);
                                return;
                            }
                            if let Some((fault, job)) = job_fault {
                                problem = GateProblem::Runner { fault, job: Some(job) };
                            }
                        }
                        unjudged_end(d, env, &key, problem, out);
                    }
                    policy::Verdict::Running => {}
                }
            } else if let Some(read) = next {
                p.stage = Stage::Run { key: key.clone(), pages: pages.checked_add(1).expect("bounded pages") };
                ask(d, env, owner, p, key.repository, read, out);
                return;
            }
            p.polled.push(key).expect("one poll per flight");
            p.stage = Stage::Collect;
            collect(d, env, owner, p, out);
        }
        Stage::Tree { member, gate, candidate, object, page, main, reading_base } => {
            let Ok(api::Answer::Tree { entries, more }) = answer else {
                read_failed(d, &row, &p, error, out);
                return;
            };
            let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
            for entry in entries {
                if p.entries
                    .push(policy::Entry {
                        name: entry.name,
                        mode: entry.mode,
                        object: entry.object,
                        kind: match entry.kind {
                            api::TreeEntryKind::Blob => policy::EntryKind::Blob,
                            api::TreeEntryKind::Tree => policy::EntryKind::Tree,
                            api::TreeEntryKind::Commit => policy::EntryKind::Commit,
                        },
                    })
                    .is_err()
                {
                    crate::refusal::limit(
                        out,
                        p.task,
                        repo,
                        "forge.key_entries",
                        env.limits.key_entries,
                        u64::from(p.entries.len()).saturating_add(1),
                    );
                    return;
                }
            }
            if more {
                let Some(page) = page.checked_add(1) else {
                    emit(out, Request::Refused { task: p.task });
                    return;
                };
                p.stage = Stage::Tree { member, gate, candidate, object, page, main, reading_base };
                ask(d, env, owner, p, repo, api::Read::Tree { object, page }, out);
                return;
            }
            let entries = core::mem::replace(&mut p.entries, List::with_capacity(env.limits.key_entries));
            let count = p.trees.len().saturating_add(p.base_trees.len());
            if count == env.limits.trees_in_flight {
                crate::refusal::limit(
                    out,
                    p.task,
                    repo,
                    "forge.trees_in_flight",
                    env.limits.trees_in_flight,
                    u64::from(count).saturating_add(1),
                );
                return;
            }
            let trees = if reading_base { &mut p.base_trees } else { &mut p.trees };
            if trees.push(policy::Tree { object, entries: entries.into_boxed() }).is_err() {
                crate::refusal::limit(
                    out,
                    p.task,
                    repo,
                    "forge.trees_in_flight",
                    env.limits.trees_in_flight,
                    u64::from(count).saturating_add(1),
                );
                return;
            }
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let mut keyed = policy::key(
                policy::Repository { forge: repo.forge, repository: repo.repository },
                &configured.name,
                &policy::definition(&configured),
                &configured.ignored,
                candidate,
                p.trees.as_slice(),
                None,
                &policy::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
            );
            let mut reading_base = false;
            if configured.base
                && let policy::Keyed::Key(_) = keyed
            {
                let base = if main {
                    candidate
                } else {
                    row.landing.members.get(usize::from(member)).expect("member").base.expect("prepared base")
                };
                keyed = policy::key(
                    policy::Repository { forge: repo.forge, repository: repo.repository },
                    &configured.name,
                    &policy::definition(&configured),
                    &configured.ignored,
                    candidate,
                    p.trees.as_slice(),
                    Some((base, if base == candidate { p.trees.as_slice() } else { p.base_trees.as_slice() })),
                    &policy::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
                );
                reading_base = true;
            }
            match keyed {
                policy::Keyed::Need(object) => {
                    p.stage = Stage::Tree { member, gate, candidate, object, page: 1, main, reading_base };
                    ask(d, env, owner, p, repo, api::Read::Tree { object, page: 1 }, out);
                }
                policy::Keyed::Key(key) => {
                    if main {
                        d.landing
                            .main_keys
                            .insert((repo, configured.name.clone()), (candidate, key))
                            .expect("bounded repository gates");
                        p.trees = List::with_capacity(env.limits.trees_in_flight);
                        p.base_trees = List::with_capacity(env.limits.trees_in_flight);
                        p.stage = Stage::Collect;
                        collect(d, env, owner, p, out);
                    } else {
                        d.landing
                            .keys
                            .insert(
                                (p.task, member, gate),
                                (
                                    candidate,
                                    if configured.base {
                                        row.landing.members.get(usize::from(member)).expect("member").base
                                    } else {
                                        None
                                    },
                                    key,
                                ),
                            )
                            .expect("bounded landing gates");
                        emit_decision(
                            out,
                            Request::LandingDecision {
                                task: p.task,
                                decision: policy::Decision::Key { member, gate, candidate },
                                entry: None,
                                evidence: None,
                                head_trailers: Box::new([]),
                            },
                            d,
                            env,
                        );
                    }
                }
                policy::Keyed::TooLarge { bound: None, .. } => emit(out, Request::Refused { task: p.task }),
                policy::Keyed::TooLarge { bound: Some(bound), needed } => {
                    let (member, configured) = match bound {
                        policy::KeyBound::Entries => ("forge.key_entries", env.limits.key_entries),
                        policy::KeyBound::PathBytes => ("forge.key_path_bytes", env.limits.key_path_bytes),
                    };
                    crate::refusal::limit(out, p.task, repo, member, configured, needed);
                }
            }
        }
        Stage::Collect => unreachable!("collect emits the next read itself"),
    }
}
fn next_member(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    index: u16,
    out: &mut Queue<Request>,
) {
    let next = index.checked_add(1).expect("member index");
    if let Some(member) = row.landing.members.get(usize::from(next)) {
        let change = d.changes.get(&member.change).expect("member's change");
        let repo = change.repository;
        let read = api::Read::Branch { branch: change.branch.clone() };
        p.stage = Stage::Branch(next);
        ask(d, env, owner, p, repo, read, out);
        return;
    }
    gathered(d, env, owner, p, row, out);
}
fn gathered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    out: &mut Queue<Request>,
) {
    if row.landing.round.branch.is_empty() {
        p.stage = Stage::Collect;
        collect(d, env, owner, p, out);
        return;
    }
    for (member, fact) in row.landing.members.iter().zip(p.facts.iter_mut()) {
        if let Some((prefix, combined)) = member.candidate.zip(member.combined) {
            fact.candidate = Some((prefix, combined));
        }
    }
    if row.landing.round.ending.is_some() {
        if row.landing.round.ending == Some(policy::RoundEnd::Stale)
            && let Some(preparation) = p.heard.preparation
            && preparation != policy::Preparation::Running
        {
            round_member(d, env, owner, p, row, 0, out);
        } else {
            collect(d, env, owner, p, out);
        }
        return;
    }
    if row.landing.phase != policy::Phase::Waiting && row.landing.phase != policy::Phase::Preparing {
        round_pull(d, env, owner, p, row, out);
        return;
    }
    if let Some(upstream) = upstream_branch(d, row) {
        p.stage = Stage::Round { member: 0, stage: RoundRead::Upstream, candidate: None };
        let repo = p.repository;
        ask(d, env, owner, p, repo, api::Read::Branch { branch: upstream }, out);
    } else {
        round_member(d, env, owner, p, row, 0, out);
    }
}
pub(crate) fn round_top(row: &LandingRow) -> Option<u16> {
    round_top_in(&row.landing)
}

fn round_top_in(landing: &policy::Landing) -> Option<u16> {
    if let Some(search) = &landing.round.search
        && policy::probe(landing).is_none()
        && search.green != 0
        && landing.round.ending != Some(policy::RoundEnd::Cut)
    {
        return search.green.checked_sub(1);
    }
    let mut top = None;
    for (index, member) in landing.members.iter().enumerate() {
        if member.left.is_none() && member.landed.is_none() && member.candidate.is_some() {
            top = Some(u16::try_from(index).expect("member position"));
        }
    }
    top
}
fn round_member(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    from: u16,
    out: &mut Queue<Request>,
) {
    for (index, member) in row.landing.members.iter().enumerate().skip(usize::from(from)) {
        if member.left.is_some()
            || member.landed.is_some()
            || (row.landing.round.ending.is_some() && member.candidate.is_some())
        {
            continue;
        }
        p.stage =
            Stage::Round { member: u16::try_from(index).expect("member"), stage: RoundRead::Branch, candidate: None };
        let repo = repository(member);
        ask(d, env, owner, p, repo, api::Read::Branch { branch: member.branch.clone() }, out);
        return;
    }
    round_pull(d, env, owner, p, row, out);
}
fn round_pull(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    out: &mut Queue<Request>,
) {
    let Some(index) = round_top(row) else {
        collect(d, env, owner, p, out);
        return;
    };
    if row.landing.round.pull.is_some() {
        collect(d, env, owner, p, out);
        return;
    }
    let member = row.landing.members.get(usize::from(index)).expect("top");
    p.stage = Stage::Round { member: index, stage: RoundRead::Pull, candidate: member.candidate };
    let repo = repository(member);
    ask(
        d,
        env,
        owner,
        p,
        repo,
        api::Read::PullFor { head: member.branch.clone(), base: row.landing.branch.clone() },
        out,
    );
}
#[expect(clippy::too_many_arguments, reason = "one stage of the coherent round read")]
#[expect(clippy::too_many_lines, reason = "the bounded prefix read keeps each validation beside its next read")]
fn round_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    index: u16,
    stage: RoundRead,
    mut candidate: Option<api::Commit>,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let error = answer.as_ref().err().copied();
    let member = row.landing.members.get(usize::from(index)).expect("member");
    let fresh = p.facts.get(usize::from(index)).expect("member fact");
    let base = member.base.unwrap_or(fresh.main);
    let head = member.head.unwrap_or(fresh.head);
    let mut below = base;
    let outcomes = if p.heard.outcomes.is_empty() {
        crate::preparation::outcomes(&row.landing.round.failure, d.landing.limits.landing_members)
            .unwrap_or(Box::new([]))
    } else {
        p.heard.outcomes.clone()
    };
    let mut chain = true;
    for (position, (prior, fact)) in row.landing.members.iter().zip(&p.facts).enumerate().take(usize::from(index)) {
        if prior.left.is_some() || prior.landed.is_some() {
            continue;
        }
        if let Some((prefix, _)) = fact.candidate {
            below = prefix;
            continue;
        }
        let mut skipped = false;
        for outcome in &outcomes {
            if usize::from(outcome.member) == position && outcome.outcome != policy::Outcome::Combined {
                skipped = true;
            }
        }
        chain &= skipped;
    }
    let fact = p.facts.get_mut(usize::from(index)).expect("member fact");
    let (stage, read) = match stage {
        RoundRead::Upstream => {
            let Ok(api::Answer::Commit(tip)) = answer else {
                read_failed(d, row, &p, error, out);
                return;
            };
            p.upstream = Some(tip);
            round_member(d, env, owner, p, row, 0, out);
            return;
        }
        RoundRead::Branch => {
            candidate = match answer {
                Ok(api::Answer::Commit(tip)) => Some(tip),
                Err(api::Error::Missing) => None,
                Ok(_) | Err(_) => {
                    read_failed(d, row, &p, error, out);
                    return;
                }
            };
            let combined = crate::preparation::combined(&row.landing.round.failure, index);
            if chain && let Some(combined) = combined {
                let Some(tip) = candidate else {
                    read_failed(d, row, &p, error, out);
                    return;
                };
                fact.candidate = Some((tip, combined));
                round_member(d, env, owner, p, row, index.checked_add(1).expect("member bound"), out);
                return;
            }
            match candidate {
                Some(tip) => (RoundRead::Prefix(1), api::Read::Commits { before: below, after: tip, page: 1 }),
                None => (RoundRead::Contained, api::Read::Commits { before: base, after: head, page: 1 }),
            }
        }
        RoundRead::Prefix(page) => {
            if answer.is_err() {
                read_failed(d, row, &p, error, out);
                return;
            }
            let mut merge = false;
            fact.candidate = None;
            if let Some(tip) = candidate
                && let Ok(api::Answer::Commits { commits, more }) = answer
            {
                let mut seen = false;
                for commit in &commits {
                    if commit.id == tip {
                        seen = true;
                    }
                    if !chain || commit.id != tip || commit.parents.first() != Some(&below) {
                        continue;
                    }
                    if commit.parents.as_ref() == [below] && squash_head(&commit.message, head) {
                        fact.candidate = Some((tip, policy::Combine::Squash));
                    } else {
                        merge = true;
                    }
                }
                if !seen && more {
                    let Some(page) = page.checked_add(1) else {
                        read_failed(d, row, &p, None, out);
                        return;
                    };
                    p.stage = Stage::Round { member: index, stage: RoundRead::Prefix(page), candidate };
                    let repo = repository(member);
                    ask(d, env, owner, p, repo, api::Read::Commits { before: below, after: tip, page }, out);
                    return;
                }
            }
            if merge {
                (RoundRead::Head, api::Read::Commits { before: candidate.expect("prefix"), after: head, page: 1 })
            } else {
                (RoundRead::Contained, api::Read::Commits { before: base, after: head, page: 1 })
            }
        }
        RoundRead::Head => {
            // Read::Commits may expose only a merge's first parent; containment checks its head.
            match crate::domain::ancestry_status(answer) {
                change::Status::Passed => fact.candidate = Some((candidate.expect("prefix"), policy::Combine::Merge)),
                change::Status::Failed => {}
                change::Status::Unknown | change::Status::Pending => {
                    read_failed(d, row, &p, error, out);
                    return;
                }
            }
            (RoundRead::Contained, api::Read::Commits { before: base, after: head, page: 1 })
        }
        RoundRead::Contained => {
            let trailers = match &answer {
                Ok(api::Answer::Commits { commits, .. }) => head_commit_trailers(commits, head),
                Err(api::Error::TooLarge(_)) => None,
                Ok(_) | Err(_) => {
                    read_failed(d, row, &p, error, out);
                    return;
                }
            };
            fact.contained = if crate::domain::ancestry_status(answer) == change::Status::Passed {
                policy::Status::Passed
            } else {
                policy::Status::Failed
            };
            if let Some(block) = trailers
                && !block.is_empty()
            {
                p.head_trailers
                    .push(crate::LandingHeadTrailers { member: index, block })
                    .expect("one block per member");
            }
            round_member(d, env, owner, p, row, index.checked_add(1).expect("member bound"), out);
            return;
        }
        RoundRead::Pull => {
            if let Ok(api::Answer::Pull(pull)) = answer
                && (pull.state == api::State::Open || pull.merged == member.candidate)
                && Some(pull.commit) == member.candidate
            {
                p.opened = Some(pull.number);
            }
            collect(d, env, owner, p, out);
            return;
        }
    };
    p.stage = Stage::Round { member: index, stage, candidate };
    let repo = repository(member);
    ask(d, env, owner, p, repo, read, out);
}
fn head_commit_trailers(commits: &[api::CommitMessage], head: api::Commit) -> Option<Box<[u8]>> {
    for commit in commits {
        if commit.id == head {
            return Some(temper_protocol_member_message::trailers(&commit.message));
        }
    }
    None
}
fn path_candidate(row: &LandingRow, member: u16) -> Option<api::Commit> {
    if row.landing.round.branch.is_empty() {
        return row.landing.members.get(usize::from(member))?.candidate;
    }
    stack_candidate(row)
}
fn proof_member(row: &LandingRow, member: u16) -> u16 {
    if row.landing.round.branch.is_empty() { member } else { round_effect_top(row) }
}
fn stack_candidate(row: &LandingRow) -> Option<api::Commit> {
    if let Some(search) = &row.landing.round.search
        && policy::probe(&row.landing).is_none()
        && search.green != 0
    {
        return row.landing.members.get(usize::from(search.green.checked_sub(1)?))?.candidate;
    }
    let mut top = None;
    for member in &row.landing.members {
        if member.left.is_none() {
            top = member.candidate.or(top);
        }
    }
    top
}
pub(crate) fn round_effect_top(row: &LandingRow) -> u16 {
    round_effect_top_in(&row.landing)
}

fn round_effect_top_in(landing: &policy::Landing) -> u16 {
    if let Some(search) = &landing.round.search
        && policy::probe(landing).is_none()
        && landing.round.ending != Some(policy::RoundEnd::Cut)
        && search.green != 0
    {
        return search.green.checked_sub(1).expect("green member");
    }
    let mut top = 0;
    for (index, member) in landing.members.iter().enumerate() {
        if member.candidate.is_some() {
            top = u16::try_from(index).expect("member");
        }
    }
    top
}
fn round_closed(d: &Domain, row: &LandingRow) -> policy::Status {
    result(d, row.landing.task, round_effect_top(row), 0, 7)
}
fn conflict_words(row: &LandingRow, member: u16) -> Box<[u8]> {
    if let Some(outcomes) = crate::preparation::outcomes(
        &row.landing.round.failure,
        u32::try_from(row.landing.members.len()).expect("members"),
    ) {
        for outcome in outcomes {
            if outcome.member == member {
                return outcome.files;
            }
        }
    }
    row.landing.round.failure.clone()
}
fn whole_again(search: Option<&policy::Search>) -> bool {
    match search {
        Some(policy::Search { tests: policy::SearchTests::WholeAgain, .. }) => true,
        Some(policy::Search { tests: policy::SearchTests::Whole | policy::SearchTests::ByTest { .. }, .. }) | None => {
            false
        }
    }
}
fn round_words(d: &Domain, row: &LandingRow) -> (Box<[u8]>, Box<[u8]>) {
    let mut count = 0_u64;
    for member in &row.landing.members {
        if member.left.is_none() && member.landed.is_none() && member.candidate.is_some() {
            count = count.saturating_add(1);
        }
    }
    let mut title = List::with_capacity(d.landing.limits.issue_policy.title_bytes);
    let body_bytes = u32::try_from(row.landing.members.len())
        .expect("members")
        .checked_mul(256)
        .expect("bounded members")
        .checked_add(128)
        .expect("bounded round body");
    let mut body = List::with_capacity(body_bytes);
    append_words(&mut body, b"Round ");
    append_words(&mut body, skein_lib::Decimal::of(u64::from(row.landing.preparation)).as_bytes());
    append_words(&mut body, b", base ");
    let top = row.landing.members.get(usize::from(round_top(row).expect("top"))).expect("member");
    append_words(&mut body, &commit_hex(top.base.expect("round base")));
    append_words(&mut body, b".\n");
    if count > 1 {
        append_words(&mut title, b"Land ");
        append_words(&mut title, skein_lib::Decimal::of(count).as_bytes());
        append_words(&mut title, b" changes: ");
    }
    let mut first = true;
    for member in &row.landing.members {
        if member.left.is_some() || member.landed.is_some() || member.candidate.is_none() {
            continue;
        }
        let pull = skein_lib::Decimal::of(member.pull);
        if count == 1 {
            append_words(&mut title, b"Land #");
            append_words(&mut title, pull.as_bytes());
            append_words(&mut title, b": ");
            let change = d.change(member.change).expect("member change");
            let room = skein_lib::usize_of(title.capacity().saturating_sub(title.len()));
            append_words(&mut title, &crate::text::fit(&change.title, room, b"..."));
        } else {
            let number = concat(&[if first { b"#" } else { b", #" }, pull.as_bytes()]);
            if number.len() <= skein_lib::usize_of(title.capacity().saturating_sub(title.len())) {
                append_words(&mut title, &number);
            }
        }
        first = false;
        for part in [
            b"Change ".as_slice(),
            skein_lib::Decimal::of(member.change).as_bytes(),
            b", pull request #",
            pull.as_bytes(),
            b", head ",
            &commit_hex(member.head.expect("head")),
            b", prefix ",
            &commit_hex(member.candidate.expect("prefix")),
            b".\n",
        ] {
            append_words(&mut body, part);
        }
    }
    if let Some(search) = &row.landing.round.search {
        let red = row.landing.members.get(usize::from(search.red.checked_sub(1).expect("red"))).expect("red member");
        let Some(policy::Leave::Blamed { red: prefix, green, .. }) = red.left else { unreachable!("blamed boundary") };
        append_words(&mut body, b"Search: red at ");
        append_words(&mut body, &commit_hex(prefix));
        append_words(&mut body, b", change ");
        let mut separator = b"".as_slice();
        for member in &row.landing.members {
            if member.unit == red.unit {
                append_words(&mut body, separator);
                append_words(&mut body, skein_lib::Decimal::of(member.change).as_bytes());
                separator = b", ";
            }
        }
        append_words(&mut body, b" blamed; green at ");
        append_words(&mut body, &commit_hex(green));
        append_words(&mut body, b".\n");
    }
    if let Some(searched) = &row.landing.searched {
        append_words(&mut body, b"Search by test: red at ");
        append_words(&mut body, &commit_hex(searched.red));
        append_words(&mut body, b", change ");
        let mut separator = b"".as_slice();
        for change in &searched.changes {
            append_words(&mut body, separator);
            append_words(&mut body, skein_lib::Decimal::of(*change).as_bytes());
            separator = b", ";
        }
        append_words(&mut body, b" blamed; green at ");
        if let Some(green) = searched.green {
            append_words(&mut body, &commit_hex(green));
        } else {
            append_words(&mut body, b"the base");
        }
        append_words(&mut body, b".\n");
    }
    (title.into_boxed(), body.into_boxed())
}
fn upstream_branch(d: &Domain, row: &LandingRow) -> Option<Box<[u8]>> {
    if row.landing.members.first()?.base.is_some() {
        return None;
    }
    let repo = repository(row.landing.members.first()?);
    let adopted = d.repository(repo)?;
    let resource = skein_forge_client::Resource {
        repository: repo,
        what: skein_forge_client::What::Branch(row.landing.branch.clone()),
    };
    let name = super::domain::from_client(&resource, &d.landing.limits)?;
    if !d.holds.contains_key(&name) || crate::items::goal_branch(&name, &adopted.prefix).is_none() {
        return None;
    }
    Some(adopted.landing.clone())
}

fn squash_head(message: &[u8], head: api::Commit) -> bool {
    temper_protocol_member_message::names_head(message, head)
}

/// A receipt is shared by project, repository, gate and filtered tree, independent
/// of the landing that dispatched it (domain/landing-proof.md, 1 and 2).
fn reads_key(d: &Domain, row: &LandingRow, key: &ReceiptKey) -> bool {
    if row.landing.project != key.project {
        return false;
    }
    for member in &row.landing.members {
        if repository(member) != key.repository {
            continue;
        }
        for (index, proof) in member.gates.iter().enumerate() {
            if proof_key(proof.proof) == Some(key.key)
                && let Some(gate) = d.train_gates(row, key.repository).get(index)
                && key.check == Check::Gate(gate.name.clone())
            {
                return true;
            }
        }
    }
    false
}

fn cleanup_first(round: &policy::Round) -> Option<u16> {
    Some(match round.ending? {
        policy::RoundEnd::Stale => 0,
        policy::RoundEnd::Leave | policy::RoundEnd::Conflict | policy::RoundEnd::Cut => round.changed.unwrap_or(0),
    })
}

/// Cleanup is recovered from the forge without dropping the prefixes that describe the round.
fn cleanup_branch(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    from: u16,
    out: &mut Queue<Request>,
) {
    let first = cleanup_first(&row.landing.round).expect("ended round");
    for (index, member) in row.landing.members.iter().enumerate().skip(usize::from(from.max(first))) {
        let facts = p.facts.get(index).expect("cleanup member");
        if member.candidate.is_none() && facts.candidate.is_none() {
            continue;
        }
        p.stage = Stage::CleanupBranch(u16::try_from(index).expect("member bound"));
        ask(d, env, owner, p, repository(member), api::Read::Branch { branch: member.branch.clone() }, out);
        return;
    }
    p.cleaned = true;
    collect(d, env, owner, p, out);
}
/// Rebind a member's pass to the merge's target, not to that member's own prefix.
pub(crate) fn temper_dir_tree(row: &LandingRow, prefix: api::Commit, tip: api::Commit) -> Option<api::Commit> {
    if let policy::Phase::Turn { .. } = row.landing.phase
        && row.landing.round.ending.is_none()
    {
        if row.landing.round.branch.is_empty() {
            for member in &row.landing.members {
                if member.left.is_none()
                    && member.landed.is_none()
                    && member.candidate == Some(prefix)
                    && member.base == Some(tip)
                {
                    return Some(prefix);
                }
            }
        } else {
            let top = turn_top(row)?;
            for member in row.landing.members.iter().take(usize::from(top).saturating_add(1)) {
                if member.left.is_none() && member.landed.is_none() && member.candidate == Some(prefix) {
                    let target = row.landing.members.get(usize::from(top))?;
                    if target.base == Some(tip) {
                        return target.candidate;
                    }
                }
            }
        }
    }
    None
}

fn turn_top(row: &LandingRow) -> Option<u16> {
    if let policy::Phase::Turn { .. } = row.landing.phase
        && !row.landing.round.branch.is_empty()
        && row.landing.round.ending.is_none()
    {
        return policy::top(&row.landing);
    }
    None
}

pub(crate) fn observe_tip(
    d: &mut Domain,
    limits: &Limits,
    repository: api::Repository,
    branch: &[u8],
    tip: Option<api::Commit>,
    out: &mut Queue<Request>,
) -> Seen {
    let Some(name) = writes::branch(limits, repository, branch) else { return Seen::Current };
    d.observe(&name, tip, Actor::Unknown, out)
}

fn wait_for_record(d: &mut Domain, row: &LandingRow, p: &Pending, seen: Seen, out: &mut Queue<Request>) -> bool {
    let cause = match seen {
        Seen::Behind { order } => read_wait::Cause::Behind { order },
        Seen::Owed => read_wait::Cause::NotThere,
        Seen::Current | Seen::Explained { .. } | Seen::Drift { .. } => return false,
    };
    wait(d, p.task, p.entry, read_wait::Failure::pending(row, p, cause), out);
    true
}

pub(crate) fn settle_turn_merge(
    d: &mut Domain,
    limits: &Limits,
    row: &LandingRow,
    top: u16,
    made: bool,
    out: &mut Queue<Request>,
) {
    if row.landing.round.merges == 0 || result(d, row.landing.task, top, 0, 5) == policy::Status::Passed {
        return;
    }
    let member = row.landing.members.get(usize::from(top)).expect("turn merge member");
    if let Some(pull) = row.landing.round.pull
        && let Some(candidate) = member.candidate
        && let Some(train) = writes::branch(limits, repository(member), &row.landing.branch)
    {
        outbox::merge_readback(d, &train, row.landing.task, pull, candidate, made, out);
    }
}

fn judge_turn_pull(
    d: &mut Domain,
    limits: &Limits,
    row: &LandingRow,
    p: &mut Pending,
    pull: &api::Pull,
    out: &mut Queue<Request>,
) -> bool {
    let top = turn_top(row).expect("turn pull has a top");
    let member = row.landing.members.get(usize::from(top)).expect("top");
    let number = row.landing.round.pull.expect("turn pull");
    let name = writes::branch(limits, repository(member), &member.branch).expect("admitted prefix branch");
    let seen = d.observe_pull(&name, number, pull, out);
    if wait_for_record(d, row, p, seen, out) {
        return false;
    }
    p.round_pull = if pull.number != number || pull.head != member.branch || pull.base != row.landing.branch {
        policy::RoundPull::Changed
    } else if let Some(commit) = pull.merged {
        if Some(commit) == member.candidate {
            settle_turn_merge(d, limits, row, top, true, out);
        }
        policy::RoundPull::Merged(commit)
    } else if pull.state == api::State::Closed {
        // An outside close is drift, but it still ends this round unlanded.
        settle_turn_merge(d, limits, row, top, false, out);
        policy::RoundPull::Closed
    } else if Some(pull.commit) == member.candidate && (pull.base_commit.is_none() || pull.base_commit == member.base) {
        policy::RoundPull::Open
    } else {
        policy::RoundPull::Changed
    };
    true
}

fn turn_drift_words(row: &LandingRow, facts: &policy::Facts, top: u16, seen: Seen) -> Option<Box<[u8]>> {
    let Seen::Drift { nearest } = seen else { return None };
    let base = row.landing.members.get(usize::from(top))?.base?;
    let tip = facts.members.get(usize::from(top))?.main;
    if base == tip {
        return None;
    }
    let entry = match nearest {
        Some(order) => concat(&[b"; nearest write entry ", skein_lib::Decimal::of(order).as_bytes()]),
        None => Box::from(b"; no write entry".as_slice()),
    };
    Some(concat(&[
        b"drift: ",
        &row.landing.branch,
        b" reads ",
        &commit_hex(tip),
        b", round base ",
        &commit_hex(base),
        &entry,
    ]))
}

fn merge_readback_words(row: &LandingRow, facts: &policy::Facts, top: u16) -> Box<[u8]> {
    let number = skein_lib::Decimal::of(row.landing.round.pull.expect("turn pull"));
    let state = match facts.round_pull {
        policy::RoundPull::Open => Box::from(b"open".as_slice()),
        policy::RoundPull::Merged(commit) => concat(&[b"merged at ", &commit_hex(commit)]),
        policy::RoundPull::Unread | policy::RoundPull::Closed | policy::RoundPull::Changed => {
            unreachable!("a turn holds only a read, unchanged open or merged pull")
        }
    };
    concat(&[
        b"merge of #",
        number.as_bytes(),
        b": ",
        &crate::preparation::failure(&row.landing.round.failure),
        b"; read #",
        number.as_bytes(),
        b" ",
        &state,
        b", ",
        &row.landing.branch,
        b" at ",
        &commit_hex(facts.members.get(usize::from(top)).expect("top facts").main),
    ])
}

fn collect(d: &mut Domain, env: &Env<Limits>, owner: Token, mut p: Pending, out: &mut Queue<Request>) {
    let row = d.landing.rows.get(&p.task).expect("pending landing");
    if row.landing.round.ending.is_some() && !p.cleaned {
        let row = row.clone();
        if let Some(number) = row.landing.round.pull {
            p.stage = Stage::CleanupPull;
            let repo = repository(row.landing.members.get(usize::from(round_effect_top(&row))).expect("round top"));
            ask(d, env, owner, p, repo, api::Read::Pull { number, ci: false }, out);
        } else {
            cleanup_branch(d, env, owner, p, &row, 0, out);
        }
        return;
    }
    if p.round_pull == policy::RoundPull::Unread
        && let Some(number) = row.landing.round.pull
        && let Some(top) = turn_top(row)
        && let Some(member) = row.landing.members.get(usize::from(top))
        && let Some(fresh) = p.facts.get(usize::from(top))
        && member.base == Some(fresh.main)
        && member.candidate != Some(fresh.main)
        && let policy::Status::Unknown | policy::Status::Failed = result(d, p.task, top, 0, 5)
    {
        p.stage = Stage::TurnPull;
        let repo = repository(member);
        ask(d, env, owner, p, repo, api::Read::Pull { number, ci: false }, out);
        return;
    }
    for (index, member) in row.landing.members.iter().enumerate() {
        let repo = repository(member);
        for (number, gate) in d.train_gates(row, repo).iter().enumerate() {
            let main = p.facts.get(index).expect("member fact").main;
            if match d.landing.main_keys.get(&(repo, gate.name.clone())) {
                Some((tip, _)) => *tip != main,
                None => true,
            } {
                p.stage = Stage::Tree {
                    member: u16::try_from(index).expect("member position"),
                    gate: u16::try_from(number).expect("gate position"),
                    candidate: main,
                    object: main,
                    page: 1,
                    main: true,
                    reading_base: false,
                };
                ask(d, env, owner, p, repo, api::Read::Tree { object: main, page: 1 }, out);
                return;
            }
        }
    }
    for (key, flight) in &d.landing.flights {
        if (flight.task != p.task && !reads_key(d, row, key)) || p.polled.as_slice().contains(key) {
            continue;
        }
        let pending_status = if flight.task == p.task
            && let Some(entry) = row.run_entry(flight.entry)
        {
            let member = row.landing.members.get(usize::from(entry.member)).expect("flight member");
            let proof = member.gates.get(usize::from(entry.gate)).expect("flight gate");
            let current =
                if row.landing.round.ending.is_none() { policy::probe(&row.landing).or(round_top(row)) } else { None };
            tests_value(&flight.inputs).is_none()
                && current == Some(entry.member)
                && proof_key(proof.proof) == Some(key.key)
                && proof.posted == policy::Posted::Nothing
        } else {
            false
        };
        if pending_status {
            continue;
        }
        if let Some(read) = flight.read() {
            let key = key.clone();
            p.flight_entry = Some(flight.entry);
            p.stage = Stage::Run { key: key.clone(), pages: 1 };
            ask(d, env, owner, p, key.repository, read, out);
            return;
        }
    }
    if p.entry == 0 {
        emit(out, Request::WakeProcedure { task: p.task });
    } else if p.woken {
        step(d, env, p.task, p.entry, p.heard, out);
    } else {
        decide(d, owner, p, out);
    }
}
fn result(d: &Domain, task: u64, member: u16, gate: u16, operation: u8) -> policy::Status {
    if let Some(status) = d.landing.results.get(&(task, member, gate, operation)) {
        return status.0;
    }
    // Recovery keeps each effect's identity in the landing and outbox (landing.md, 12).
    if let Some(row) = d.landing.rows.get(&task) {
        for item in &row.entries {
            if item.member == member
                && item.gate == gate
                && self::operation(item.operation) == operation
                && (item.preparation.is_none() || item.preparation == Some(row.landing.preparation))
                && let Some(entry) = d.entries.get(&item.entry)
            {
                return if entry.held { policy::Status::Failed } else { policy::Status::Pending };
            }
        }
        if operation == 5 && row.landing.round.merges > 0 {
            // A terminal merge retired before a restart is unknown, not a new first send.
            return policy::Status::Failed;
        }
    }
    policy::Status::Unknown
}
fn decide(d: &mut Domain, owner: Token, p: Pending, out: &mut Queue<Request>) {
    let task = p.task;
    ready(d, task, Ready::Gathered { owner, pending: p }, out);
}

fn ready(d: &mut Domain, task: u64, ready: Ready, out: &mut Queue<Request>) {
    assert!(
        d.landing.decisions.insert(task, ready).expect("one decision per landing").is_none(),
        "the root settles each handoff before the next event"
    );
    emit(out, Request::LandingReady { task });
}

impl Domain {
    /// Admit gathered facts before any decision state or write is kept
    /// (domain/landing-waits.md, 3). A refused handoff gathers again in its next due step.
    pub fn decide_landing(&mut self, env: &Env<Limits>, task: u64, due: bool, out: &mut Queue<Request>) {
        let Some(ready) = self.landing.decisions.remove(&task) else { return };
        if due {
            match ready {
                Ready::Gathered { owner, pending } => {
                    commit_decision(self, env, owner, pending, out);
                    crate::waits::report(self, env, task, out);
                }
                Ready::Read(step) => read_decision(self, env, task, step, out),
                Ready::Claimed { released } => claim_decision(self, env, task, released, out),
            }
        } else {
            emit(out, Request::WakeProcedure { task });
        }
        crate::room::ask(self, env, out);
    }
}

fn clear_cleanup_results(d: &mut Domain, row: &LandingRow) {
    for index in 0..row.landing.members.len() {
        let member = u16::try_from(index).expect("member");
        d.landing.results.remove(&(row.landing.task, member, 0, 7));
        d.landing.results.remove(&(row.landing.task, member, 0, 8));
    }
}

/// Recheck ownership at the decision, after the last forge read in the gather.
fn cleanup_ownership(d: &Domain, row: &LandingRow, facts: &mut [policy::MemberFacts]) -> Option<Box<[u8]>> {
    let first = cleanup_first(&row.landing.round)?;
    for (member, fresh) in row.landing.members.iter().zip(facts).skip(usize::from(first)) {
        if member.candidate.is_none() && fresh.candidate.is_none() {
            continue;
        }
        let repository = repository(member);
        let resource =
            skein_forge_client::Resource { repository, what: skein_forge_client::What::Branch(member.branch.clone()) };
        let Some(name) = super::domain::from_client(&resource, &d.landing.limits) else { continue };
        if let Some(hold) = d.hold(&name)
            && hold.task != row.landing.task
        {
            fresh.deleted = policy::Status::Failed;
            let by = skein_lib::Decimal::of(hold.task);
            return Some(crate::effect_words::branch_account(
                b"delete branch",
                &name,
                d.repository(repository),
                &concat(&[b"held by task ", by.as_bytes()]),
            ));
        }
    }
    None
}

#[expect(clippy::too_many_lines, reason = "one policy decision and its atomic durable outputs")]
fn commit_decision(d: &mut Domain, env: &Env<Limits>, owner: Token, mut p: Pending, out: &mut Queue<Request>) {
    let mut row = d.landing.rows.get(&p.task).expect("pending landing").clone();
    if let Some(local) = local_step(&row, &p.heard) {
        read_decision(d, env, p.task, local, out);
        return;
    }
    if p.heard.released && !held_stuck(&row.landing.phase) {
        clear_cleanup_results(d, &row);
        row.landing.round.cleanup = None;
        for index in 0..row.landing.members.len() {
            d.landing.results.remove(&(p.task, u16::try_from(index).expect("member"), 0, 9));
        }
    }
    let unprepared = match row.landing.members.first() {
        Some(member) => member.base.is_none(),
        None => false,
    };
    if unprepared {
        row.landing.round.upstream = p.upstream;
    }
    if continue_flake_rerun(d, env, &mut row, p.entry, out) || continue_retry(d, env, &mut row, p.entry, out) {
        return;
    }
    // A hearing with none decides nothing about the prepare (landing-preparing.md, 3).
    let preparation = p.heard.preparation.unwrap_or(policy::Preparation::Running);
    let outcomes = crate::preparation::outcomes(&row.landing.round.failure, env.limits.landing_members)
        .unwrap_or(p.heard.outcomes.clone());
    let mut empty = List::with_capacity(env.limits.landing_members.saturating_mul(env.limits.gates));
    for (index, (member, fact)) in row.landing.members.iter().zip(p.facts.iter_mut()).enumerate() {
        let position = u16::try_from(index).expect("member position");
        fact.merge = result(d, p.task, position, 0, 5);
        fact.asked =
            if let Some(change) = d.changes.get(&member.change) { members::asked(change, &row) } else { false };
        if !row.landing.round.branch.is_empty()
            && let Some(prefix) = member.candidate
        {
            let change = d.changes.get(&member.change).expect("member change");
            fact.judgement = policy::Status::Passed;
            for gate in &change.change.gates {
                if !gate.blocking {
                    continue;
                }
                if !(if row.landing.round.branch.is_empty() { row.landing.round.judging } else { member.judging })
                    && (gate.kind == change::GateKind::Check || gate.freshness == change::Freshness::Exact)
                {
                    fact.judgement = policy::Status::Pending;
                    continue;
                }
                match change::prefix_verdict(
                    &change.change,
                    *gate,
                    &change.verdicts,
                    member.head.expect("round head"),
                    prefix,
                    change.spec_revision,
                ) {
                    change::Status::Failed => fact.refused = Some(gate.number),
                    change::Status::Unknown | change::Status::Pending => fact.judgement = policy::Status::Pending,
                    change::Status::Passed => {}
                }
            }
        }
        fact.preparation = preparation;
        if fact.deleted != policy::Status::Passed {
            fact.deleted = result(d, p.task, position, 0, 8);
        }
        for outcome in &outcomes {
            if outcome.member == position {
                fact.outcome = Some(outcome.outcome);
            }
        }
        let mut dependencies = List::with_capacity(env.limits.landing_members);
        for dependency in &p.heard.dependencies {
            if dependency.change != member.change {
                continue;
            }
            for (index, prior) in row.landing.members.iter().enumerate() {
                if dependency.pending.contains(&prior.change) {
                    dependencies.push(u16::try_from(index).expect("member")).expect("bounded dependencies");
                }
            }
        }
        fact.dependencies = dependencies.into_boxed();
        fact.record = if member.landed.is_some() && fact.merged == member.landed {
            policy::Status::Passed
        } else {
            result(d, p.task, position, 0, 9)
        };
        for (number, gate) in d.train_gates(&row, repository(member)).iter().enumerate() {
            let number = u16::try_from(number).expect("gate position");
            let failed = fact.failed.get_mut(usize::from(number)).expect("gate failures");
            *failed = None;
            let fact = fact.gates.get_mut(usize::from(number)).expect("gate fact");
            let candidate = member.candidate.unwrap_or([0; 32]);
            fact.key = match d.landing.keys.get(&(p.task, position, number)) {
                Some((tip, base, key)) if *tip == candidate && *base == if gate.base { member.base } else { None } => {
                    Some(*key)
                }
                Some(_) | None => None,
            };
            if let Some(key) = proof_key(member.gates.get(usize::from(number)).expect("gate proof").proof).or(fact.key)
            {
                let key = gate_key(&row, position, number, &gate.name, key);
                let receipt = d.receipt(&key);
                if receipt.is_none() && d.landing.receipts.contains_key(&key) {
                    empty.push((key.clone(), position, number)).expect("bounded gate positions");
                }
                fact.receipt = if superseded(d, &row, &key) || d.landing.flights.contains_key(&key) {
                    policy::Status::Unknown
                } else {
                    match receipt {
                        Some(receipt) if receipt.passed => policy::Status::Passed,
                        Some(_) => policy::Status::Failed,
                        None => policy::Status::Unknown,
                    }
                };
                if let Some(receipt) = receipt
                    && !receipt.passed
                    && let Some(temper_engine_domain_forge_tiers::Failures::Tests(ids)) = &receipt.failures
                {
                    *failed = Some(ids.clone());
                }
                fact.flight = flight_fact(&d.landing, &key, (p.task, position, number));
                if fact.flight == policy::Flight::None
                    && runs::busy(
                        d,
                        repository(member),
                        &gate.workflow,
                        candidate,
                        &probe_inputs(
                            &row,
                            position,
                            number,
                            candidate,
                            key.key,
                            if gate.base { member.base } else { None },
                        ),
                    )
                {
                    fact.flight = policy::Flight::Other;
                }
                if let Some(receipt) = receipt {
                    fact.run = receipt.run.number;
                }
                if let Some(flight) = d.landing.flights.get(&key) {
                    if let Some(run) = &flight.run {
                        fact.run = run.number;
                    }
                    let gate_timeout = d.landing_gate_timeout(row.landing.project);
                    if row.room.is_none()
                        && let Some(clock) = flight.clock
                        && (clock.ran(env.wall) > gate_timeout
                            || clock.waited(env.wall) >= env.limits.change_policy.runner_wait)
                        && flight.outcome.is_none()
                        && flight.run.is_some()
                    {
                        fact.expired = Some(fact.run);
                    }
                }
            }
            fact.posting = result(
                d,
                p.task,
                position,
                number,
                if member.gates.get(usize::from(number)).expect("gate proof").posted == policy::Posted::Verdict {
                    4
                } else {
                    3
                },
            );
        }
    }
    for (key, member, gate) in empty.into_boxed() {
        if d.landing.receipts.remove(&key).is_some() {
            mark_key(d, &key, out);
            emit(out, Request::Erase { key: Key::Receipt(key.clone()) });
        }
        let proof = row
            .landing
            .members
            .get_mut(usize::from(member))
            .expect("member")
            .gates
            .get_mut(usize::from(gate))
            .expect("gate");
        let reset = match proof.proof {
            policy::Proof::Failed { .. } => true,
            policy::Proof::Dispatched { .. } => {
                !d.landing.flights.contains_key(&key) && !d.landing.faults.contains_key(&(p.task, member, gate))
            }
            policy::Proof::Unkeyed | policy::Proof::Keyed { .. } | policy::Proof::Passed { .. } => false,
        };
        if reset {
            *proof = policy::GateProof {
                proof: policy::Proof::Keyed { key: key.key },
                posted: policy::Posted::Nothing,
                faults: 0,
            };
        }
    }
    let cleanup = cleanup_ownership(d, &row, &mut p.facts);
    let top = round_top(&row).unwrap_or(0);
    let facts = policy::Facts {
        members: p.facts.clone(),
        turn_free: if row.landing.round.branch.is_empty() { single_turn(d, &row, &p.heard.kept) } else { true },
        released: p.heard.released,
        opened: p.opened,
        opening: result(d, p.task, top, 0, 6),
        round_pull: p.round_pull,
        merge_refused: match d.landing.results.get(&(p.task, top, 0, 5)) {
            Some((_, refused)) => *refused,
            None => false,
        },
        closed: if p.closed == policy::Status::Passed { p.closed } else { round_closed(d, &row) },
        preparation,
        train_holder: other_train_holder(d, &row),
    };
    let project = match d.landing.projects.get(&row.landing.project) {
        Some((_, _, _, _, _, settings, _)) => *settings,
        None => policy::Project::DEFAULT,
    };
    let mut stepped = policy::step(
        &row.landing,
        &facts,
        &project,
        &policy::Limits {
            tests_bytes: tests_room(d, &row, env.limits.client.op_bytes),
            write_attempts: env.limits.client.write_attempts,
            members: env.limits.landing_members,
            gates: env.limits.gates,
            poll: skein_lib::Duration::from_nanos(
                poll_at(d, env, p.task).as_nanos().saturating_sub(env.wall.as_nanos()),
            ),
        },
        env.wall,
    );
    let mut candidate_fault = None;
    if let policy::Decision::Merge { member, candidate } = stepped.decision {
        let target = temper_dir::Target::new(&row, &p, member, candidate);
        if let Some(check) = temper_dir::checked(&p, target) {
            if let temper_dir::Judgement::Fault(fault) = check {
                stepped = policy::candidate_fault(row.landing.clone(), &facts, &fault.covered);
                candidate_fault = Some(temper_dir::take_fault(&mut p));
            }
        } else {
            temper_dir::start(d, env, owner, p, &row, target, out);
            return;
        }
    }
    let writing = write_member(stepped.decision);
    if let Some((member, on)) = writing
        && !admit_entry(
            d,
            env,
            p.task,
            p.entry,
            &row,
            (member, on, blocked_write(d, &env.limits, &stepped.landing, member, on)),
            out,
        )
    {
        return;
    }
    if writing.is_none() {
        leave_room(d, &mut row);
    }
    if p.stall.is_none()
        && let policy::Decision::Hold(policy::Hold::Timeout { member, gate, .. }) = stepped.decision
    {
        let m = stepped.landing.members.get(usize::from(member)).expect("gate member");
        let repository = repository(m);
        let proof = proof_key(m.gates.get(usize::from(gate)).expect("gate proof").proof).expect("expired gate key");
        let name = &d.train_gates(&row, repository).get(usize::from(gate)).expect("gate").name;
        let key = gate_key(&row, member, gate, name, proof);
        let flight = d.landing.flights.get(&key).expect("expired gate flight");
        let run = flight.run.as_ref().expect("expired gate run").id;
        p.stage = Stage::StallRun;
        d.stall_words.remove(&p.task);
        ask(d, env, owner, p, repository, api::Read::Run { run }, out);
        return;
    }
    if let Some(report) = p.stall.take() {
        d.stall_words.insert(p.task, report).expect("bounded tasks");
    }
    crate::job_history::prepared(d, env.wall, &mut row, &p.facts, out);
    let prior = row.clone();
    row.landing = stepped.landing;
    if let policy::Decision::Hold(policy::Hold::Faults { member, gate }) = stepped.decision {
        row.gate_hold = gate_hold(d, &row, member, gate);
    }
    if stepped.decision == policy::Decision::Hold(policy::Hold::Cleanup) {
        row.landing.round.cleanup = cleanup;
    }
    if let policy::Decision::Wait { on, .. } = stepped.decision {
        let shared = if let policy::Awaited::Receipt { member, gate } = on {
            facts
                .members
                .get(usize::from(member))
                .expect("receipt member")
                .gates
                .get(usize::from(gate))
                .expect("receipt gate")
                .flight
                == policy::Flight::Other
        } else {
            false
        };
        read_wait::policy_wait(d, env, &row, on, shared);
        if !read_wait::changed(d.landing.rows.get(&p.task).expect("pending landing"), &row)
            && let Some(until) = read_wait::bound(d, p.task)
            && until <= env.now
        {
            read_decision(d, env, p.task, ReadStep::Stalled, out);
            return;
        }
    }
    if stepped.decision == policy::Decision::Hold(policy::Hold::Turn)
        && facts.members.get(usize::from(top)).expect("top facts").merge == policy::Status::Failed
        && facts.round_pull != policy::RoundPull::Unread
    {
        let words = merge_readback_words(&row, &facts, top);
        row.landing.round.failure =
            crate::preparation::remember(&row.landing.round.failure, &words, env.limits.landing_members);
    }
    if prior.landing.round.ending.is_none()
        && row.landing.round.ending == Some(policy::RoundEnd::Stale)
        && let Some(words) = turn_drift_words(&prior, &facts, top, p.turn_tip)
    {
        row.landing.round.failure =
            crate::preparation::remember(&row.landing.round.failure, &words, env.limits.landing_members);
    }
    if prior.landing.round.ending.is_some() && row.landing.round.ending.is_none() {
        row.landing.round.failure =
            crate::preparation::next_round(&row.landing.round.failure, env.limits.landing_members);
    }
    if row.landing.preparation != prior.landing.preparation {
        for (member, old) in row.landing.members.iter_mut().zip(&prior.landing.members) {
            if member.left.is_none() && old.landed.is_none() {
                member.round = Some(row.landing.preparation);
            }
        }
    }
    if !row.landing.round.branch.is_empty() {
        for (member, old) in row.landing.members.iter().zip(&prior.landing.members) {
            if member.candidate == old.candidate {
                continue;
            }
            let Some(candidate) = member.candidate else { continue };
            let resource = skein_forge_client::Resource {
                repository: repository(member),
                what: skein_forge_client::What::Branch(member.branch.clone()),
            };
            if let Some(name) = super::domain::from_client(&resource, &env.limits) {
                super::domain::record_owned_head(d, env, &name, candidate, Some(p.task), out);
            }
        }
    }
    crate::landing_history::transition(d, env.wall, &prior, &row, out);
    let decision = stepped.decision;
    crate::landing_proofs::search(d, env.wall, &prior, &row, out);
    if (row.landing.round.ending.is_some()
        && prior.landing.round.ending.is_none()
        && row.landing.round.ending != Some(policy::RoundEnd::Cut))
        || (row.landing.phase == policy::Phase::Cancelled && prior.landing.phase != policy::Phase::Cancelled)
    {
        end_runs(d, env, &prior, &mut row, crate::history::GateVerdict::Superseded, None, out);
    }
    if let policy::Decision::Hold(policy::Hold::Timeout { member, gate, .. }) = decision {
        end_runs(d, env, &prior, &mut row, crate::history::GateVerdict::Timeout, Some((member, gate)), out);
    }
    for (member, old) in row.landing.members.iter().zip(&prior.landing.members) {
        if member.landed.is_some() && old.landed.is_none() {
            emit(out, Request::WakeProcedure { task: member.change });
        }
    }
    if row.landing.members.len() != prior.landing.members.len() {
        remap_members(d, &mut row, &prior);
    }
    let merge = match decision {
        policy::Decision::Merge { member, candidate } => Some((member, candidate, false)),
        policy::Decision::Record { member, prefix } => Some((member, prefix, true)),
        policy::Decision::None
        | policy::Decision::Wait { .. }
        | policy::Decision::Prepare { .. }
        | policy::Decision::Open
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Landed { .. }
        | policy::Decision::Leave { .. }
        | policy::Decision::Judge { .. }
        | policy::Decision::Resolve { .. }
        | policy::Decision::Key { .. }
        | policy::Decision::Dispatch { .. }
        | policy::Decision::Retry { .. }
        | policy::Decision::Post { .. }
        | policy::Decision::Repair { .. }
        | policy::Decision::Proved
        | policy::Decision::Stale
        | policy::Decision::Finish
        | policy::Decision::Cancel
        | policy::Decision::End(_)
        | policy::Decision::Hold(_) => None,
    };
    let evidence = if let Some((member, candidate, recording)) = merge {
        let mut all = List::with_capacity(env.limits.landing_members);
        for (index, m) in row.landing.members.iter().enumerate() {
            let position = u16::try_from(index).expect("member");
            if (recording || row.landing.round.branch.is_empty()) && position != member {
                continue;
            }
            if !recording && (m.left.is_some() || m.landed.is_some() || position > member) {
                continue;
            }
            let candidate =
                if row.landing.round.branch.is_empty() { candidate } else { m.candidate.expect("member prefix") };
            let fact = p.facts.get(usize::from(position)).expect("fresh member facts");
            let mut evidence = p.evidence.get(usize::from(position)).expect("member evidence").clone();
            if evidence.facts.head != Some(candidate) {
                evidence.facts.size = None;
            }
            evidence.facts.head = Some(candidate);
            if !recording {
                evidence.facts.temper_dir = temper_dir::evidence(&p, position);
            }
            let paths = p.paths.get(usize::from(position)).expect("member paths");
            let classes = d.project_classes(row.landing.project);
            evidence.facts.touched = crate::touched(&classes, &paths.files, paths.whole);
            evidence.facts.base_tip = fact.main;
            evidence.facts.contains_base =
                match if fact.candidate.is_some() { policy::Status::Passed } else { fact.contains_main } {
                    policy::Status::Unknown => change::Status::Unknown,
                    policy::Status::Pending => change::Status::Pending,
                    policy::Status::Passed => change::Status::Passed,
                    policy::Status::Failed => change::Status::Failed,
                };
            evidence.facts.ci = change::Status::Passed;
            for gate in &p.facts.get(usize::from(proof_member(&row, position))).expect("top facts").gates {
                if gate.blocking && gate.receipt != policy::Status::Passed {
                    evidence.facts.ci = if gate.receipt == policy::Status::Failed {
                        change::Status::Failed
                    } else {
                        change::Status::Unknown
                    };
                }
            }
            all.push(evidence).expect("bounded evidence");
        }
        Some(all.into_boxed())
    } else {
        None
    };
    let mut committed = None;
    let mut head_trailers: Box<[crate::LandingHeadTrailers]> = Box::new([]);
    match decision {
        policy::Decision::Open
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Record { .. } => {
            let position = match decision {
                policy::Decision::Delete { member } | policy::Decision::Record { member, .. } => member,
                policy::Decision::Open => round_top(&row).expect("round top"),
                policy::Decision::Close => round_effect_top(&row),
                policy::Decision::None
                | policy::Decision::Wait { .. }
                | policy::Decision::Prepare { .. }
                | policy::Decision::Landed { .. }
                | policy::Decision::Leave { .. }
                | policy::Decision::Judge { .. }
                | policy::Decision::Resolve { .. }
                | policy::Decision::Key { .. }
                | policy::Decision::Dispatch { .. }
                | policy::Decision::Retry { .. }
                | policy::Decision::Post { .. }
                | policy::Decision::Repair { .. }
                | policy::Decision::Proved
                | policy::Decision::Merge { .. }
                | policy::Decision::Stale
                | policy::Decision::Finish
                | policy::Decision::Cancel
                | policy::Decision::End(_)
                | policy::Decision::Hold(_) => unreachable!("round write"),
            };
            let member = row.landing.members.get(usize::from(position)).expect("member");
            let (op, write, condition) = match decision {
                policy::Decision::Open => {
                    let (title, body) = round_words(d, &row);
                    (
                        Operation::Open,
                        api::Write::OpenPull {
                            title,
                            body,
                            head: member.branch.clone(),
                            base: row.landing.branch.clone(),
                        },
                        skein_forge_client::Condition::None,
                    )
                }
                policy::Decision::Close => (
                    Operation::Close,
                    api::Write::Close { number: row.landing.round.pull.expect("round pull") },
                    skein_forge_client::Condition::None,
                ),
                policy::Decision::Delete { member: number } => {
                    assert_eq!(number, position, "deleted member");
                    (
                        Operation::Delete,
                        api::Write::DeleteBranch { branch: member.branch.clone() },
                        skein_forge_client::Condition::None,
                    )
                }
                policy::Decision::Record { prefix, .. } => (
                    Operation::Record,
                    api::Write::Merge {
                        number: member.pull,
                        head: member.head.expect("member head"),
                        how: api::MergeHow::Recorded { commit: prefix },
                    },
                    skein_forge_client::Condition::Merge { base: row.landing.branch.clone() },
                ),
                policy::Decision::None
                | policy::Decision::Wait { .. }
                | policy::Decision::Prepare { .. }
                | policy::Decision::Key { .. }
                | policy::Decision::Dispatch { .. }
                | policy::Decision::Retry { .. }
                | policy::Decision::Post { .. }
                | policy::Decision::Repair { .. }
                | policy::Decision::Proved
                | policy::Decision::Merge { .. }
                | policy::Decision::Stale
                | policy::Decision::Finish
                | policy::Decision::Cancel
                | policy::Decision::End(_)
                | policy::Decision::Hold(_)
                | policy::Decision::Landed { .. }
                | policy::Decision::Resolve { .. }
                | policy::Decision::Leave { .. }
                | policy::Decision::Judge { .. } => unreachable!("round write"),
            };
            keep(d, &env.limits, &mut row, p.entry, position, 0, op, write, condition, out);
            committed = Some(p.entry);
        }
        policy::Decision::Landed { member, prefix } => {
            hand_landed(d, env.wall, &row, member, prefix, out);
        }
        policy::Decision::Resolve { member, base } => {
            members::hand(d, &row, member, change::Ask::Resolve { base }, out);
        }
        policy::Decision::Judge { member } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            if let Some(change) = d.changes.get_mut(&m.change) {
                let mut verdicts = List::with_capacity(env.limits.change_policy.gates);
                for verdict in &change.verdicts {
                    let mut exact = false;
                    for gate in &change.change.gates {
                        if gate.number == verdict.number
                            && gate.blocking
                            && (gate.freshness == change::Freshness::Exact || gate.kind == change::GateKind::Check)
                        {
                            exact = true;
                        }
                    }
                    if !exact {
                        verdicts.push(*verdict).expect("bounded verdicts");
                    }
                }
                change.verdicts = verdicts.into_boxed();
            }
            let ask = change::Ask::Judge(change::Judgement {
                landing: p.task,
                round: row.landing.preparation,
                pull: row.landing.round.pull.expect("round pull"),
                branch: m.branch.clone(),
                base: row.landing.branch.clone(),
                prefix: m.candidate.expect("prefix"),
            });
            members::hand(d, &row, member, ask, out);
        }
        policy::Decision::Leave { member, why } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let ask = match why {
                policy::Leave::Conflict => Some(change::Ask::Resolve { base: m.base.expect("round base") }),
                policy::Leave::Failed { gate, .. } => {
                    Some(change::Ask::Repair { gate, candidate: failed_candidate(d, &row, member, gate), below: None })
                }
                policy::Leave::Cancelled
                | policy::Leave::Held
                | policy::Leave::HeadMoved { .. }
                | policy::Leave::Refused { .. }
                | policy::Leave::EarlierConflict
                | policy::Leave::Unit { .. }
                | policy::Leave::Dependency { .. } => None,
                policy::Leave::Blamed { gate, red, green, .. } => {
                    Some(change::Ask::Repair { gate, candidate: red, below: Some(repair_below(&row, green)) })
                }
            };
            let task = m.change;
            if let Some(ask) = ask {
                members::hand(d, &row, member, ask, out);
            } else {
                emit(out, Request::WakeProcedure { task });
            }
        }
        policy::Decision::Repair { member, gate } => {
            let candidate =
                row.landing.members.get(usize::from(member)).expect("member").candidate.expect("proved candidate");
            members::hand(d, &row, member, change::Ask::Repair { gate, below: None, candidate }, out);
        }
        policy::Decision::Key { member, gate, candidate } => {
            let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
            p.stage =
                Stage::Tree { member, gate, candidate, object: candidate, page: 1, main: false, reading_base: false };
            save(d, env, row, out);
            ask(d, env, owner, p, repo, api::Read::Tree { object: candidate, page: 1 }, out);
            return;
        }
        policy::Decision::Dispatch { member, gate, key, try_ }
        | policy::Decision::Retry { member, gate, key, try_ } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let repo = repository(m);
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let candidate = m.candidate.expect("candidate before dispatch");
            let key = gate_key(&row, member, gate, &configured.name, key);
            let purpose = purpose_identity(&row, member, gate, 1, try_);
            if let Some(mut fault) = d.landing.faults.remove(&(p.task, member, gate)) {
                retry_gate(&mut fault, try_);
                fault.purpose.clone_from(&purpose);
                d.landing.flights.insert(key.clone(), fault).expect("bounded key flights");
            } else if let Some(flight) = d.landing.flights.get_mut(&key) {
                retry_gate(flight, try_);
                flight.purpose = purpose;
            } else {
                let inputs =
                    probe_inputs(&row, member, gate, candidate, key.key, if configured.base { m.base } else { None });
                let flight = runs::follow(
                    p.task,
                    p.entry,
                    purpose,
                    repo,
                    configured.workflow.clone(),
                    dispatch_branch(d, &row, m),
                    candidate,
                    inputs,
                );
                d.landing.flights.insert(key.clone(), flight).expect("bounded key flights");
            }
            let flight = d.landing.flights.get_mut(&key).expect("new flight");
            let write = flight.write(p.entry).expect("dispatch or retry write");
            flight.outcome = None;
            let operation = if let api::Write::Rerun { .. } = write { Operation::Rerun } else { Operation::Dispatch };
            flight.purpose = purpose_identity(
                &row,
                member,
                gate,
                match operation {
                    Operation::Dispatch => 1,
                    Operation::Rerun => 2,
                    Operation::Pending => 3,
                    Operation::Verdict => 4,
                    Operation::Merge => 5,
                    Operation::Open => 6,
                    Operation::Close => 7,
                    Operation::Delete => 8,
                    Operation::Record => 9,
                },
                try_,
            );
            keep(
                d,
                &env.limits,
                &mut row,
                p.entry,
                member,
                gate,
                operation,
                write,
                skein_forge_client::Condition::None,
                out,
            );
            mark_key(d, &key, out);
            committed = Some(p.entry);
        }
        policy::Decision::Post { member, gate, posting } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let repo = repository(m);
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let key = gate_key(
                &row,
                member,
                gate,
                &configured.name,
                proof_key(m.gates.get(usize::from(gate)).expect("gate proof").proof).expect("keyed posting"),
            );
            let (write, operation) =
                posting_write(m.candidate.expect("candidate"), &configured.name, &key, posting, d.receipt(&key));
            keep(
                d,
                &env.limits,
                &mut row,
                p.entry,
                member,
                gate,
                operation,
                write,
                skein_forge_client::Condition::None,
                out,
            );
            committed = Some(p.entry);
        }
        policy::Decision::Merge { member, candidate } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let change = d.changes.get(&m.change).expect("member change");
            let condition = skein_forge_client::Condition::Merge { base: change.base.clone() };
            let write = api::Write::Merge {
                number: if row.landing.round.branch.is_empty() {
                    m.pull
                } else {
                    row.landing.round.pull.expect("round pull")
                },
                head: candidate,
                how: api::MergeHow::FastForwardOnly,
            };
            keep(d, &env.limits, &mut row, p.entry, member, 0, Operation::Merge, write, condition, out);
            committed = Some(p.entry);
        }
        policy::Decision::Proved => {
            crate::landing_proofs::proved(d, env.wall, &prior, &row, out);
            if row.landing.round.branch.is_empty() {
                save(d, env, row.clone(), out);
                if !single_turn(d, &row, &p.heard.kept) {
                    save(d, env, row, out);
                    emit_decision(
                        out,
                        Request::LandingDecision {
                            task: p.task,
                            decision: policy::Decision::None,
                            entry: None,
                            evidence: None,
                            head_trailers: Box::new([]),
                        },
                        d,
                        env,
                    );
                    return;
                }
            }
            if let policy::Phase::Proved { at } = row.landing.phase {
                row.landing.phase = policy::Phase::Turn { at };
            }
        }
        policy::Decision::Finish => {
            for member in &row.landing.members {
                emit(out, Request::WakeProcedure { task: member.change });
            }
        }
        policy::Decision::Wait { .. } => {
            // A member may still have been due when its ask was published.
            // Keep the ask level-triggered until its change consumes it.
            for member in &row.landing.members {
                if let Some(change) = d.changes.get(&member.change)
                    && members::queued(change, row.landing.task)
                    && let change::State::Ready { .. } = change.change.state
                {
                    emit(out, Request::WakeProcedure { task: member.change });
                }
            }
        }
        policy::Decision::Prepare { member, head, main } => {
            if row.landing.round.branch.is_empty() {
                members::hand(d, &row, member, change::Ask::Prepare { head, main }, out);
            } else {
                head_trailers = p.head_trailers.into_boxed();
            }
        }
        policy::Decision::None
        | policy::Decision::Stale
        | policy::Decision::Cancel
        | policy::Decision::End(_)
        | policy::Decision::Hold(_) => {}
    }
    if row.landing.preparation != prior.landing.preparation
        || row.landing.phase == policy::Phase::Landed
        || row.landing.phase == policy::Phase::Cancelled
    {
        let mut entries = List::with_capacity(d.landing.limits.entries);
        for entry in &row.entries {
            if d.entries.contains_key(&entry.entry) {
                entries.push(*entry).expect("live entries");
            }
        }
        row.entries = entries.into_boxed();
        saved_follow::prune(&mut row, &d.landing.limits);
    }
    reset_results(d, &mut row, &prior);
    if (prior.landing.round.ending.is_some() && row.landing.round.ending.is_none())
        || row.landing.preparation != prior.landing.preparation
    {
        clear_cleanup_results(d, &row);
    }
    if row.landing.preparation != prior.landing.preparation || decision == policy::Decision::Stale {
        for index in 0..row.landing.members.len() {
            let member = u16::try_from(index).expect("member");
            d.landing.results.remove(&(p.task, member, 0, 6));
        }
    }
    let news = match decision {
        policy::Decision::Leave { .. } => Some(crate::LandingNews::Left),
        policy::Decision::Hold(_) => Some(crate::LandingNews::Held),
        policy::Decision::End(cause) => Some(crate::LandingNews::Cancelled(cause)),
        policy::Decision::Landed { .. } => Some(crate::LandingNews::Landed),
        policy::Decision::None
        | policy::Decision::Wait { .. }
        | policy::Decision::Prepare { .. }
        | policy::Decision::Key { .. }
        | policy::Decision::Dispatch { .. }
        | policy::Decision::Retry { .. }
        | policy::Decision::Post { .. }
        | policy::Decision::Repair { .. }
        | policy::Decision::Proved
        | policy::Decision::Merge { .. }
        | policy::Decision::Stale
        | policy::Decision::Finish
        | policy::Decision::Cancel
        | policy::Decision::Open
        | policy::Decision::Record { .. }
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Resolve { .. }
        | policy::Decision::Judge { .. } => {
            if whole_again(row.landing.round.search.as_ref()) && !whole_again(prior.landing.round.search.as_ref()) {
                Some(crate::LandingNews::WholeAgain)
            } else if row.landing.round.search != prior.landing.round.search && policy::probe(&row.landing).is_some() {
                Some(crate::LandingNews::Probing)
            } else if row.landing.phase == policy::Phase::Proving && prior.landing.phase != policy::Phase::Proving {
                Some(crate::LandingNews::Proving)
            } else {
                None
            }
        }
    };
    let mut news_members = List::with_capacity(env.limits.landing_members);
    match decision {
        policy::Decision::Leave { member, .. } | policy::Decision::Landed { member, .. } => {
            news_members.push(member).expect("one member");
        }
        policy::Decision::None
        | policy::Decision::Wait { .. }
        | policy::Decision::Prepare { .. }
        | policy::Decision::Open
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Record { .. }
        | policy::Decision::Judge { .. }
        | policy::Decision::Resolve { .. }
        | policy::Decision::Key { .. }
        | policy::Decision::Dispatch { .. }
        | policy::Decision::Retry { .. }
        | policy::Decision::Post { .. }
        | policy::Decision::Repair { .. }
        | policy::Decision::Proved
        | policy::Decision::Merge { .. }
        | policy::Decision::Stale
        | policy::Decision::Finish
        | policy::Decision::Cancel
        | policy::Decision::End(_)
        | policy::Decision::Hold(_) => {
            for (index, m) in row.landing.members.iter().enumerate() {
                let everyone = match news {
                    Some(crate::LandingNews::Held | crate::LandingNews::Cancelled(_)) => true,
                    Some(
                        crate::LandingNews::Joined
                        | crate::LandingNews::Probing
                        | crate::LandingNews::Proving
                        | crate::LandingNews::Landed
                        | crate::LandingNews::Left
                        | crate::LandingNews::WholeAgain,
                    )
                    | None => false,
                };
                if everyone || (m.left.is_none() && m.landed.is_none()) {
                    news_members.push(u16::try_from(index).expect("member")).expect("bounded news");
                }
            }
        }
    }
    if crate::views::matches_turn(&row.landing.phase) && !crate::views::matches_turn(&prior.landing.phase) {
        row.turn_since = Some(env.wall);
    }
    if let Some(fault) = candidate_fault {
        temper_dir::hold_members(d, env, &row, fault, out);
    }
    save(d, env, row, out);
    if let Some(step) = news {
        for member in news_members.into_boxed() {
            emit(out, Request::LandingNews { task: p.task, member, step });
        }
    }
    let decision = if prior.landing.round.branch.is_empty()
        && let policy::Decision::Prepare { .. } = decision
    {
        policy::Decision::None
    } else {
        decision
    };
    // A landed landing still follows its dispatched runs for their receipts
    // (domain/landing-proof.md, 2.1), while its members are free to land again.
    let decision = if decision == policy::Decision::Finish
        && let Some(on) = following_wait(d, p.task)
    {
        policy::Decision::Wait { until: Some(poll_at(d, env, p.task)), on }
    } else {
        decision
    };
    emit_decision(
        out,
        Request::LandingDecision { task: p.task, decision, entry: committed, evidence, head_trailers },
        d,
        env,
    );
}
fn end_runs(
    d: &mut Domain,
    env: &Env<Limits>,
    prior: &LandingRow,
    row: &mut LandingRow,
    verdict: crate::history::GateVerdict,
    only: Option<(u16, u16)>,
    out: &mut Queue<Request>,
) {
    let mut flights = List::with_capacity(d.landing.flights.len());
    for (_, flight) in &d.landing.flights {
        if flight.task == row.landing.task {
            flights.push(flight.clone()).expect("bounded flights");
        }
    }
    let mut observed = prior.clone();
    for flight in flights.into_boxed() {
        let Some(entry) = observed.run_entry(flight.entry) else { continue };
        let concerns = match only {
            Some(position) => position == (entry.member, entry.gate),
            None => true,
        };
        if concerns && entry.run.is_some() {
            crate::landing_proofs::run(d, env, &mut observed, &flight, Some(verdict), out);
            if let Some(entry) = row.run_entry_mut(flight.entry) {
                entry.run = observed.run_entry(flight.entry).expect("observed mark").run;
            }
        }
    }
}
fn dispatch_branch(d: &Domain, row: &LandingRow, member: &policy::Member) -> Box<[u8]> {
    if row.landing.round.branch.is_empty() {
        d.changes.get(&member.change).expect("member change").branch.clone()
    } else {
        member.branch.clone()
    }
}
fn held_stuck(phase: &policy::Phase) -> bool {
    match phase {
        policy::Phase::Held { why: policy::Hold::Stuck(_), .. } => true,
        policy::Phase::Held {
            why:
                policy::Hold::Cleanup
                | policy::Hold::Claimed { .. }
                | policy::Hold::Faults { .. }
                | policy::Hold::Prepare
                | policy::Hold::Read
                | policy::Hold::Record { .. }
                | policy::Hold::Timeout { .. }
                | policy::Hold::Turn,
            ..
        }
        | policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Landed
        | policy::Phase::Cancelled => false,
    }
}

fn other_train_holder(d: &Domain, row: &LandingRow) -> Option<u64> {
    if row.landing.round.branch.is_empty() {
        return None;
    }
    let member = row.landing.members.first()?;
    let holder = d.train_holder(repository(member), &row.landing.branch)?;
    if holder == row.landing.task { None } else { Some(holder) }
}

fn single_turn(d: &Domain, row: &LandingRow, kept: &[u64]) -> bool {
    let mut first = None;
    for (_, other) in &d.landing.rows {
        if other.landing.project != row.landing.project {
            continue;
        }
        match other.landing.phase {
            policy::Phase::Turn { .. } => return false,
            policy::Phase::Proved { at } => {
                if !kept.contains(&other.landing.task) {
                    let candidate = (at, other.landing.task);
                    let earlier = match first {
                        Some(prior) => candidate < prior,
                        None => true,
                    };
                    if earlier {
                        first = Some(candidate);
                    }
                }
            }
            policy::Phase::Waiting
            | policy::Phase::Preparing
            | policy::Phase::Proving
            | policy::Phase::Landed
            | policy::Phase::Held { .. }
            | policy::Phase::Cancelled => {}
        }
    }
    match first {
        Some((_, task)) => task == row.landing.task,
        None => false,
    }
}
fn purpose_identity(row: &LandingRow, member: u16, gate: u16, operation: u8, attempt: u16) -> Box<[u8]> {
    crate::purpose::EffectPurpose::Landing {
        task: row.landing.task,
        member,
        gate,
        operation,
        preparation: row.landing.preparation,
        attempt,
    }
    .identity()
    .expect("fixed purpose")
}
/// Probe inputs and room use the same selection slot (domain/landing-search-by-test.md, 3–4).
fn probe_inputs(
    row: &LandingRow,
    member: u16,
    gate: u16,
    candidate: api::Commit,
    key: policy::Key,
    base: Option<api::Commit>,
) -> Box<[api::Input]> {
    let probe = policy::probe(&row.landing) == Some(member);
    let base = inputs(candidate, key, base, probe);
    if probe
        && let Some(policy::Search { tests: policy::SearchTests::ByTest { gates, .. }, .. }) = &row.landing.round.search
        && let Some(Some(tests)) = gates.get(usize::from(gate))
    {
        return with_tests(base, runs::tests_input(tests));
    }
    base
}
fn with_tests(base: Box<[api::Input]>, tests: api::Input) -> Box<[api::Input]> {
    let mut inputs = List::with_capacity(u32::try_from(api::INPUTS).expect("forge inputs"));
    for input in base {
        inputs.push(input).expect("probe inputs");
    }
    inputs.push(tests).expect("selection slot");
    inputs.into_boxed()
}
fn tests_room(d: &Domain, row: &LandingRow, op_bytes: u32) -> u32 {
    let mut branch: Box<[u8]> = Box::new([]);
    let mut workflow: Box<[u8]> = Box::new([]);
    let mut base = None;
    let mut candidate = [0; 32];
    for member in &row.landing.members {
        if let Some(prefix) = member.candidate {
            candidate = prefix;
        }
        if member.branch.len() > branch.len() {
            branch.clone_from(&member.branch);
        }
        for gate in d.train_gates(row, repository(member)) {
            if gate.workflow.len() > workflow.len() {
                workflow.clone_from(&gate.workflow);
            }
            if gate.base {
                base = member.base;
            }
        }
    }
    let inputs = with_tests(inputs(candidate, [0; 32], base, true), runs::tests_input(&[]));
    let op = api::Op::Write(api::Write::Dispatch { workflow, branch, inputs });
    let bytes = skein_forge::measure::op(&op).expect("bounded probe dispatch");
    u32::try_from(u64::from(op_bytes).saturating_sub(bytes)).expect("room within op bound")
}
fn selected_report(selected: &[u8], failures: Option<&temper_engine_domain_forge_tiers::Failures>) -> bool {
    match failures {
        Some(temper_engine_domain_forge_tiers::Failures::Tests(ids)) => selected_failure(selected, ids),
        Some(temper_engine_domain_forge_tiers::Failures::Many) | None => false,
    }
}
fn selected_failure(selected: &[u8], ids: &[Box<[u8]>]) -> bool {
    let mut start = 0;
    for (end, byte) in selected.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        for id in ids {
            if selected.get(start..end) == Some(id.as_ref()) {
                return true;
            }
        }
        start = end.checked_add(1).expect("bounded input");
    }
    false
}
fn tests_value(inputs: &[api::Input]) -> Option<&[u8]> {
    for input in inputs {
        if input.name.as_ref() == b"temper_tests" {
            return Some(&input.value);
        }
    }
    None
}
/// Only the owning live probe can keep a pass or no verdict; late red receipts stand independently.
fn record_probe(d: &Domain, row: &mut LandingRow, key: &ReceiptKey, flight: &runs::Follow, run: u64, passed: bool) {
    let Some(crate::purpose::EffectPurpose::Landing { preparation, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
    else {
        return;
    };
    if preparation != row.landing.preparation
        || row.landing.round.ending.is_some()
        || matches_terminal(&row.landing.phase)
    {
        return;
    }
    let Some(probe) = policy::probe(&row.landing) else { return };
    let member = row.landing.members.get(usize::from(probe)).expect("probe member");
    if member.candidate != input_commit(&flight.inputs, b"temper_commit") || repository(member) != key.repository {
        return;
    }
    let mut gate_number = None;
    for (index, gate) in d.train_gates(row, key.repository).iter().enumerate() {
        if key.check == Check::Gate(gate.name.clone()) {
            gate_number = Some(u16::try_from(index).expect("gate number"));
        }
    }
    let Some(gate) = gate_number else { return };
    let Some(policy::Search { tests: policy::SearchTests::ByTest { gates, probed }, .. }) =
        &mut row.landing.round.search
    else {
        return;
    };
    match gates.get(usize::from(gate)) {
        Some(Some(_)) => {}
        Some(None) | None => return,
    }
    let member = probe.checked_add(1).expect("member number");
    let mut kept =
        List::with_capacity(d.landing.limits.landing_members.checked_mul(d.landing.limits.gates).expect("probe bound"));
    for previous in probed.iter() {
        if previous.member == member && previous.gate == gate {
            return;
        }
        kept.push(*previous).expect("bounded probes");
    }
    kept.push(policy::Probed { member, gate, run, passed }).expect("one end per probe and gate");
    *probed = kept.into_boxed();
}

pub(crate) fn inputs(
    candidate: api::Commit,
    key: policy::Key,
    base: Option<api::Commit>,
    probe: bool,
) -> Box<[api::Input]> {
    let mut inputs = List::with_capacity(u32::try_from(api::INPUTS).expect("forge input bound"));
    inputs
        .push(api::Input { name: Box::from(b"temper_commit".as_slice()), value: commit_hex(candidate) })
        .expect("commit input");
    inputs.push(api::Input { name: Box::from(b"temper_key".as_slice()), value: hex(&key) }).expect("key input");
    if let Some(base) = base {
        inputs
            .push(api::Input { name: Box::from(b"temper_base".as_slice()), value: commit_hex(base) })
            .expect("base input");
    }
    if probe {
        inputs
            .push(api::Input { name: Box::from(b"temper_probe".as_slice()), value: Box::from(b"yes".as_slice()) })
            .expect("probe input");
    }
    inputs.into_boxed()
}
fn commit_hex(commit: api::Commit) -> Box<[u8]> {
    let mut length = 32;
    if commit[20..] == [0; 12] {
        length = 20;
    }
    hex(commit.get(..length).expect("commit width"))
}

fn concat(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded words");
    }
    let mut bytes = List::with_capacity(u32::try_from(length).expect("bounded words"));
    for part in parts {
        for byte in *part {
            bytes.push(*byte).expect("sized words");
        }
    }
    bytes.into_boxed()
}
fn posting_write(
    candidate: api::Commit,
    name: &[u8],
    key: &ReceiptKey,
    posting: policy::Posting,
    receipt: Option<&Receipt>,
) -> (api::Write, Operation) {
    let short = hex(&key.key[..8]);
    let (check, ending, operation) = match posting {
        policy::Posting::Pending => (api::Check::Pending, b"".as_slice(), Operation::Pending),
        policy::Posting::Ran { passed: true } => (api::Check::Passed, b" passed".as_slice(), Operation::Verdict),
        policy::Posting::Ran { passed: false } => (api::Check::Failed, b" failed".as_slice(), Operation::Verdict),
        policy::Posting::Received => (api::Check::Passed, b" passed by receipt".as_slice(), Operation::Verdict),
    };
    let link: Box<[u8]> = if posting == policy::Posting::Pending {
        Box::new([])
    } else {
        match receipt {
            Some(receipt) => receipt.run.url.clone(),
            None => Box::new([]),
        }
    };
    (
        api::Write::Status {
            commit: candidate,
            context: concat(&[b"temper/gate/", name]),
            check,
            description: concat(&[b"key ", &short, ending]),
            link,
        },
        operation,
    )
}
fn operation(op: Operation) -> u8 {
    match op {
        Operation::Dispatch => 1,
        Operation::Rerun => 2,
        Operation::Pending => 3,
        Operation::Verdict => 4,
        Operation::Merge => 5,
        Operation::Open => 6,
        Operation::Close => 7,
        Operation::Delete => 8,
        Operation::Record => 9,
    }
}
#[expect(clippy::too_many_arguments, reason = "one effect and its member/gate identity")]
fn keep(
    d: &mut Domain,
    limits: &Limits,
    row: &mut LandingRow,
    entry: u64,
    member: u16,
    gate: u16,
    op: Operation,
    write: api::Write,
    condition: skein_forge_client::Condition,
    out: &mut Queue<Request>,
) {
    leave_room(d, row);
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for old in &row.entries {
        entries.push(*old).expect("bounded entries");
    }
    let item = LandingEntry {
        entry,
        member,
        gate,
        operation: op,
        run: None,
        preparation: Some(row.landing.preparation),
        base: row.landing.members.get(usize::from(member)).expect("effect member").base,
    };
    entries.push(item).expect("entry room checked");
    row.entries = entries.into_boxed();
    if op == Operation::Dispatch || op == Operation::Rerun {
        saved_follow::keep(d, row, item, &write);
    }
    d.landing
        .results
        .insert((row.landing.task, member, gate, operation(op)), (policy::Status::Pending, false))
        .expect("bounded effect positions");
    let facts = row.landing.members.get(usize::from(member)).expect("member");
    let change_branch = match d.changes.get(&facts.change) {
        Some(change) => Some(change.branch.as_ref()),
        None => None,
    };
    let (path, head, base) = match op {
        Operation::Open | Operation::Close => (Some(facts.branch.as_ref()), facts.candidate, None),
        Operation::Record => (change_branch, facts.head, None),
        Operation::Merge if row.landing.round.branch.is_empty() => (change_branch, facts.head, facts.base),
        // The round is proved on the train tip, not on its separate upstream.
        Operation::Merge => (Some(facts.branch.as_ref()), facts.candidate, facts.base),
        Operation::Delete | Operation::Dispatch | Operation::Rerun | Operation::Pending | Operation::Verdict => {
            (None, None, None)
        }
    };
    let entry = skein_forge_client::Entry {
        number: entry,
        task: row.landing.task,
        repository: repository(facts),
        effect: skein_forge_client::Effect { write, condition },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    let plan = outbox::plan(d, limits, &entry, path, Pin { head, base }).expect("landing write names");
    plan.admit(d, &entry, out);
    keep_entry(d, entry, None, out);
}

/// A previous build could retain a failed gate's report without a verdict.
fn empty_gate_receipt(key: &ReceiptKey, receipt: &Receipt) -> bool {
    if receipt.passed {
        return false;
    }
    match key.check {
        Check::Tier(_) => return false,
        Check::Gate(_) => {}
    }
    match &receipt.report {
        ReceiptReport::Unread => true,
        ReceiptReport::Tests(tests) => tests.tests.is_empty(),
        ReceiptReport::Unset | ReceiptReport::Many { .. } => false,
    }
}

pub(crate) fn put_receipt(
    d: &mut Domain,
    env: &Env<Limits>,
    key: ReceiptKey,
    receipt: Receipt,
    out: &mut Queue<Request>,
) {
    let mut count = 0_u32;
    let mut oldest: Option<(Wall, ReceiptKey)> = None;
    let protected = match &key.check {
        Check::Gate(name) => d.landing.main_keys.get(&(key.repository, name.clone())).copied(),
        Check::Tier(name) => match d.tier_ops.tip_keys.get(&(key.repository, name.clone())) {
            Some(judged) => Some((judged.tip, judged.key)),
            None => None,
        },
    };
    for (saved, value) in &d.landing.receipts {
        if saved.project != key.project || saved.repository != key.repository || saved.check != key.check {
            continue;
        }
        count = count.checked_add(1).expect("receipt bound");
        if Some(saved.key)
            == match protected {
                Some((_, key)) => Some(key),
                None => None,
            }
            || saved == &key
        {
            continue;
        }
        if match &oldest {
            Some((ended, _)) => value.ended < *ended,
            None => true,
        } {
            oldest = Some((value.ended, saved.clone()));
        }
    }
    if !d.landing.receipts.contains_key(&key) && count >= env.limits.receipts_per_gate {
        if let Some((_, oldest)) = oldest {
            d.landing.receipts.remove(&oldest);
            mark_key(d, &oldest, out);
            emit(out, Request::Erase { key: Key::Receipt(oldest) });
        } else {
            return;
        }
    }
    d.landing.receipts.insert(key.clone(), receipt.clone()).expect("bounded receipts per gate");
    mark_key(d, &key, out);
    emit(out, Request::Save { record: Stored::Receipt { key, receipt }, read_afresh: None, release: None });
}
fn finish_flight(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &ReceiptKey,
    probed: Option<(u64, bool)>,
    out: &mut Queue<Request>,
) {
    let Some(flight) = d.landing.flights.remove(key) else {
        return;
    };
    let Some(mut row) = d.landing.rows.get(&flight.task).cloned() else {
        return;
    };
    if let Some((run, passed)) = probed
        && tests_value(&flight.inputs).is_some()
    {
        record_probe(d, &mut row, key, &flight, run, passed);
    }
    for rerun in &mut row.flake_reruns {
        if rerun.key == *key {
            rerun.ended = true;
        }
    }
    for saved in row.follows_mut() {
        if saved.entry.entry == flight.entry {
            saved_follow::stop(saved);
        }
    }
    saved_follow::prune(&mut row, &d.landing.limits);
    save(d, env, row, out);
    emit(out, Request::WakeProcedure { task: flight.task });
    saved_follow::wake_project(d, key.project, flight.task, out);
}
/// A turn's merge belongs to its member's change; its effect is the landing's
/// (domain/forge/topics.md, 1.3; domain/landing-turn.md, 1).
pub(crate) fn merged_owner(d: &Domain, task: u64, entry: u64) -> u64 {
    let Some(row) = d.landing.rows.get(&task) else {
        return task;
    };
    for item in &row.entries {
        if item.entry == entry {
            assert!(item.operation == Operation::Merge || item.operation == Operation::Record, "merge or record entry");
            return row.landing.members.get(usize::from(item.member)).expect("merge's saved member").change;
        }
    }
    unreachable!("landing merge has a saved member entry");
}

/// A merge refusal requires a fresh read of the pull and base (domain/landing-turn.md, 1).
fn merge_refused(outcome: skein_forge_client::Outcome) -> bool {
    match outcome {
        skein_forge_client::Outcome::Failed(error) => match error {
            api::Error::Timeout | api::Error::Unavailable | api::Error::InvalidAnswer => false,
            api::Error::Forbidden
            | api::Error::Missing
            | api::Error::MissingJob
            | api::Error::TooLarge(_)
            | api::Error::Empty
            | api::Error::Full
            | api::Error::Exists
            | api::Error::NothingToMerge
            | api::Error::Closed
            | api::Error::Stale
            | api::Error::BaseMoved { .. }
            | api::Error::Conflict
            | api::Error::Protected
            | api::Error::Refused
            | api::Error::Busy
            | api::Error::NotConnected
            | api::Error::NotInstalled
            | api::Error::RateLimited { .. } => true,
        },
        // A merge made but not verified is not a refusal. Its turn reads back the tip and pull.
        skein_forge_client::Outcome::Raced { made: skein_forge_client::Made::Merged(_), .. }
        | skein_forge_client::Outcome::Made { .. }
        | skein_forge_client::Outcome::Queued
        | skein_forge_client::Outcome::Uncertain
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => false,
        skein_forge_client::Outcome::Raced { .. } => true,
    }
}

#[expect(clippy::too_many_lines, reason = "one landing effect updates its round facts, flight and cleanup together")]
pub(crate) fn outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    cleanup_account: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let Some(kept) = d.entries.get(&entry) else { return };
    let task = kept.entry.task;
    let Some(mut row) = d.landing.rows.get(&task).cloned() else { return };
    let mut found = None;
    for item in &row.entries {
        if item.entry == entry {
            found = Some(*item);
            break;
        }
    }
    let Some(item) = found else { return };
    let cleanup = item.operation == Operation::Close || item.operation == Operation::Delete;
    let status = match outcome {
        skein_forge_client::Outcome::Made { .. } => policy::Status::Passed,
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => policy::Status::Pending,
        skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => policy::Status::Failed,
    };
    if item.preparation.is_none() || item.preparation == Some(row.landing.preparation) {
        let key = (task, item.member, item.gate, operation(item.operation));
        if cleanup && status == policy::Status::Failed && cleanup_account.is_none() {
            // A veto never made a forge write; authority and its wait belong to the root.
            d.landing.results.remove(&key);
        } else {
            d.landing.results.insert(key, (status, merge_refused(outcome))).expect("bounded effect positions");
        }
        if status == policy::Status::Failed
            && let Some(account) = cleanup_account
            && row.landing.round.ending.is_some()
        {
            row.landing.round.cleanup = Some(account);
        }
    }
    if item.operation == Operation::Delete && status == policy::Status::Passed {
        let row = d.landing.rows.get(&task).expect("round deletion");
        let resource = skein_forge_client::Resource {
            repository: repository(row.landing.members.get(usize::from(item.member)).expect("member")),
            what: skein_forge_client::What::Branch(
                row.landing.members.get(usize::from(item.member)).expect("member").branch.clone(),
            ),
        };
        if let Some(name) = super::domain::from_client(&resource, &d.landing.limits) {
            // The next round's new branch has no ancestry to the deleted candidate.
            if d.heads.remove(&name).is_some() {
                writes::remove(d, &name, out);
                emit(out, Request::Erase { key: Key::BranchHead(name) });
            }
        }
    }
    let mut owned_key = None;
    for (key, flight) in &d.landing.flights {
        if flight.entry == entry {
            owned_key = Some(key.clone());
        }
    }
    if let Some(key) = owned_key {
        let flight = d.landing.flights.get_mut(&key).expect("owned flight");
        flight.waiting = status == policy::Status::Pending;
        if let Some(entry) = d.entries.get(&entry)
            && let Some(attempt) = entry.entry.attempt
        {
            flight.since = Some(attempt.sent);
            flight.wall = Some(attempt.wall);
        }
        if item.operation == Operation::Dispatch
            && let Some(entry) = d.entries.get(&entry)
        {
            flight.start = match entry.entry.start {
                Some(start) => Some(start.at),
                None => flight.start,
            };
        }
        if status == policy::Status::Failed {
            flight.outcome = Some(policy::Verdict::Fault(policy::Fault::Lost));
            if item.operation == Operation::Rerun && outcome == skein_forge_client::Outcome::Failed(api::Error::Refused)
            {
                let mut flake = false;
                for rerun in &mut row.flake_reruns {
                    if rerun.key == key && !rerun.ended {
                        rerun.pending = Box::new([]);
                        rerun.redispatch = true;
                        flake = true;
                    }
                }
                if flake {
                    runs::retry(flight, 2);
                }
            }
        }
    }
    if status != policy::Status::Pending {
        for record in row.follows_mut() {
            if record.entry.entry == entry && record.dispatch.is_some() {
                let kept = &d.entries.get(&entry).expect("settling entry still kept").entry;
                saved_follow::progress(record, kept);
                record.failed = status == policy::Status::Failed;
            }
        }
        if outcome != skein_forge_client::Outcome::Held || item.operation == Operation::Merge || cleanup {
            remove_entry(&mut row, entry, &d.landing.limits);
        }
        if item.operation == Operation::Record
            || item.operation == Operation::Open
            || item.operation == Operation::Merge
        {
            if status == policy::Status::Passed {
                crate::landing_proofs::made(d, env.wall, &row, item, out);
            }
            let words = crate::effect_words::outcome_words(outcome).unwrap_or(Box::new([]));
            row.landing.round.failure =
                crate::preparation::remember(&row.landing.round.failure, &words, d.landing.limits.landing_members);
        }
        save(d, env, row, out);
    }
    if d.landing.rows.get(&task).expect("effect landing").landing.phase != policy::Phase::Cancelled {
        emit(out, Request::WakeProcedure { task });
    }
}

fn remove_entry(row: &mut LandingRow, entry: u64, limits: &Limits) -> bool {
    let mut entries = List::with_capacity(limits.entries);
    let mut removed = false;
    for old in &row.entries {
        if old.entry == entry {
            removed = true;
        } else {
            entries.push(*old).expect("bounded entries");
        }
    }
    row.entries = entries.into_boxed();
    removed
}

/// A held write leaves on explicit settlement rather than on its hold.
pub(crate) fn not_made(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    if let Some(mut row) = d.landing.rows.get(&task).cloned()
        && remove_entry(&mut row, entry, &d.landing.limits)
    {
        save(d, env, row, out);
    }
}
fn awaits_pull(row: &LandingRow, member: &policy::Member, provider: api::Repository, number: u64) -> bool {
    let policy::Phase::Turn { .. } = row.landing.phase else { return false };
    repository(member) == provider
        && member.pull == number
        && member.combined == Some(policy::Combine::Merge)
        && member.candidate.is_some()
        && !member.told
}

fn wake_member_pull(
    d: &mut Domain,
    provider: api::Repository,
    number: u64,
    merged: Option<api::Commit>,
    out: &mut Queue<Request>,
) {
    for (task, row) in &d.landing.rows {
        for (index, member) in row.landing.members.iter().enumerate() {
            if awaits_pull(row, member, provider, number) {
                if let Some(owner) = pending_for(d, *task) {
                    let pending = d.landing.pending.get_mut(&owner).expect("landing's pending gather");
                    let changed = match merged {
                        None => true,
                        Some(commit) => match pending.facts.get(index) {
                            Some(fact) => fact.merged != Some(commit),
                            None => false,
                        },
                    };
                    if changed {
                        pending.woken = true;
                    }
                }
                emit(out, Request::WakeProcedure { task: *task });
                break;
            }
        }
    }
}

/// Merge metadata can arrive after the hint's wake read the old cached pull.
pub(crate) fn pull_changed(d: &mut Domain, provider: api::Repository, pull: &api::Pull, out: &mut Queue<Request>) {
    if let Some(merged) = pull.merged {
        wake_member_pull(d, provider, pull.number, Some(merged), out);
    }
}

pub(crate) fn hinted(d: &mut Domain, hint: &api::Hint, out: &mut Queue<Request>) {
    if let api::Change::Item(number) = hint.change {
        // The hint can precede main readback and invalidates an older gather.
        wake_member_pull(d, hint.repository, number, None, out);
    }
    if let api::Change::Commit(commit) = hint.change {
        for (_, flight) in &d.landing.flights {
            if flight.repository == hint.repository && flight.commit == commit {
                emit(out, Request::WakeProcedure { task: flight.task });
            }
        }
    }
}
/// The boundary context of domain/landing-search.md, 7 and landing-search-by-test.md, 7.
fn repair_below(row: &LandingRow, green: api::Commit) -> change::Below {
    let search = row.landing.round.search.as_ref().expect("blamed boundary");
    let by_test = match &search.tests {
        policy::SearchTests::ByTest { .. } => true,
        policy::SearchTests::Whole | policy::SearchTests::WholeAgain => false,
    };
    let mut top = 0;
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.candidate.is_some() {
            top = u16::try_from(index).expect("bounded members").checked_add(1).expect("member number");
        }
    }
    let mut below = change::Below { green, changes: [None; 4], by_test, top: search.red == top };
    let mut slots = below.changes.iter_mut();
    let end = search.green;
    for member in row.landing.members.iter().take(usize::from(end)) {
        if member.candidate.is_some() {
            *slots.next().expect("at most four members below the blamed unit") =
                Some(change::Landed { change: member.change, pull: member.pull });
        }
    }
    below
}

pub(crate) fn change_advanced(d: &Domain, row: &crate::ChangeRow, prior: &change::Change, out: &mut Queue<Request>) {
    if let Some(task) = d.landing_of(row.task)
        && prior != &row.change
    {
        emit(out, Request::WakeProcedure { task });
    }
}
// Dispatch inputs retain their exact key and prefix after the round cut clears
// the member's proof (landing-search.md, 6).
pub(crate) fn input_commit(inputs: &[api::Input], name: &[u8]) -> Option<api::Commit> {
    for input in inputs {
        if input.name.as_ref() != name {
            continue;
        }
        if input.value.len() != 40 && input.value.len() != 64 {
            return None;
        }
        let mut commit = [0; 32];
        for (slot, pair) in commit.iter_mut().zip(input.value.as_chunks::<2>().0) {
            let high = skein_lib::bytes::hex_value(*pair.first()?)?;
            let low = skein_lib::bytes::hex_value(*pair.get(1)?)?;
            *slot = high.checked_mul(16)?.checked_add(low)?;
        }
        return Some(commit);
    }
    None
}

/// An owed regate proves the key afresh, regardless of the receipt it finds there.
fn superseded(d: &Domain, row: &LandingRow, key: &ReceiptKey) -> bool {
    for member in &row.landing.members {
        if let Some(change) = d.changes.get(&member.change)
            && let Some(regate) = &change.regate
            && regate.taken.is_none()
            && &regate.receipt == key
            && member.left.is_none()
            && member.head == change.change.last_head
        {
            return true;
        }
    }
    false
}

/// A gate's flight identifies the landing that made the receipt. Only members
/// there take it; other landings that see this receipt still owe their dispatch.
fn put_gate_receipt(d: &mut Domain, env: &Env<Limits>, key: &ReceiptKey, receipt: Receipt, out: &mut Queue<Request>) {
    let run = receipt.run.id;
    put_receipt(d, env, key.clone(), receipt, out);
    let Some(kept) = d.receipt(key) else { return };
    if kept.run.id != run {
        return;
    }
    let Some(flight) = d.landing.flights.get(key) else { return };
    let Some(row) = d.landing.rows.get(&flight.task) else { return };
    let mut taken = List::with_capacity(env.limits.landing_members);
    for member in &row.landing.members {
        if member.left.is_none()
            && let Some(change) = d.changes.get_mut(&member.change)
            && member.head == change.change.last_head
            && let Some(regate) = &mut change.regate
            && regate.taken.is_none()
            && &regate.receipt == key
            && regate.run != run
        {
            regate.taken = Some(run);
            taken.push(change.clone()).expect("the landing's bounded members");
        }
    }
    for row in taken.into_boxed() {
        crate::domain::save_change(d, out, row);
    }
}

/// The single-prefix path hands a failed proof without a `Leave` run number.
/// Its saved observation, not the run currently cached at the key, pins that ask.
fn followed_measurement(row: &LandingRow, round: u16, member: u16, gate: u16, run: u64) -> bool {
    for follow in row.follows() {
        let entry = follow.entry;
        if entry.preparation == Some(round)
            && entry.member == member
            && entry.gate == gate
            && let Some(mark) = entry.run
            && mark.ended
            && mark.id == run
        {
            return true;
        }
    }
    false
}

/// A successful rerun write may precede the forge's next attempt. Only a terminal
/// reading beyond every kept job attempt can make its receipt; runner faults keep
/// their retry path (landing-regate.md, 2; domain/landing-proof.md, 2.4).
fn old_flake_attempt(d: &Domain, key: &ReceiptKey, answer: &Result<api::Answer, api::Error>) -> bool {
    let Ok(api::Answer::Run(run)) = answer else { return false };
    if run.state != api::RunState::Completed {
        return false;
    }
    let Some(flight) = d.landing.flights.get(key) else { return false };
    match runs::verdict(run, flight.commit) {
        policy::Verdict::Passed | policy::Verdict::Failed => {}
        policy::Verdict::Running | policy::Verdict::Fault(_) => return false,
    }
    let row = d.landing.rows.get(&flight.task).expect("dispatching landing");
    let mut active = false;
    for rerun in &row.flake_reruns {
        active |= rerun.key == *key && !rerun.ended;
    }
    if !active {
        return false;
    }
    for record in row.follows() {
        if record.entry.entry == flight.entry {
            for attempt in &record.reruns {
                if attempt.run != run.id {
                    continue;
                }
                let mut advanced = false;
                for job in &run.jobs {
                    advanced |= job.id == attempt.job && job.attempt > attempt.attempt;
                }
                if !advanced {
                    return true;
                }
            }
        }
    }
    false
}

fn hex(bytes: &[u8]) -> Box<[u8]> {
    let mut out =
        List::with_capacity(u32::try_from(bytes.len().checked_mul(2).expect("hex width")).expect("bounded hex"));
    for byte in bytes {
        for digit in crate::hex(*byte) {
            out.push(digit).expect("sized hex");
        }
    }
    out.into_boxed()
}
fn reset_results(d: &mut Domain, row: &mut LandingRow, prior: &LandingRow) {
    let mut retired = List::with_capacity(d.landing.faults.len());
    for (index, member) in row.landing.members.iter().enumerate() {
        let mut unchanged = false;
        for old in &prior.landing.members {
            if old.change == member.change && old.candidate == member.candidate {
                unchanged = true;
            }
        }
        if member.landed.is_some() || unchanged {
            continue;
        }
        let member = u16::try_from(index).expect("member");
        d.landing.results.remove(&(row.landing.task, member, 0, 5));
        for gate in 0..d.landing.limits.gates {
            if let Some(fault) =
                d.landing.faults.remove(&(row.landing.task, member, u16::try_from(gate).expect("gate")))
            {
                retired.push(fault.entry).expect("bounded faults");
            }
            for operation in 1..=9 {
                d.landing.results.remove(&(row.landing.task, member, u16::try_from(gate).expect("gate"), operation));
            }
        }
    }
    if !retired.is_empty() {
        for follow in row.follows_mut() {
            if retired.as_slice().contains(&follow.entry.entry) {
                saved_follow::stop(follow);
            }
        }
        saved_follow::prune(row, &d.landing.limits);
    }
}
/// Keep the spent key before any write, including when another landing observed the red
/// (domain/landing-regate.md, 2, 4, 5).
fn start_flake_rerun(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &ReceiptKey,
    run: &api::Run,
    report: &ReceiptReport,
    out: &mut Queue<Request>,
) -> bool {
    if !d.project_checks_can_flake(key.project) {
        return false;
    }
    let tests = match report {
        ReceiptReport::Tests(tests) => &tests.tests,
        ReceiptReport::Unset | ReceiptReport::Unread | ReceiptReport::Many { .. } => return false,
    };
    let gate = match &key.check {
        Check::Gate(gate) => gate,
        Check::Tier(_) => return false,
    };
    let Some(findings) = d.gate_flakes(key.project, key.repository, gate, tests) else { return false };
    let task = d.landing.flights.get(key).expect("failed flight").task;
    let mut row = d.landing.rows.get(&task).expect("dispatching landing").clone();
    for rerun in &row.flake_reruns {
        if rerun.key == *key {
            return false;
        }
    }
    let mut jobs = 0_u32;
    for job in &run.jobs {
        if job.conclusion == Some(api::Conclusion::Failure) {
            jobs = jobs.saturating_add(1);
        }
    }
    if jobs == 0 || jobs > 16 || row.flake_reruns.len() >= skein_lib::usize_of(d.landing.limits.entries) {
        return false;
    }
    let flight = d.landing.flights.get_mut(key).expect("failed flight");
    runs::rerun_flake(flight);
    let mut reruns =
        List::with_capacity(u32::try_from(row.flake_reruns.len()).expect("bounded keys").saturating_add(1));
    for rerun in row.flake_reruns {
        reruns.push(rerun).expect("kept keys");
    }
    reruns
        .push(crate::FlakeRerun {
            key: key.clone(),
            run: run.id,
            findings,
            pending: flake_pending(flight),
            redispatch: false,
            ended: false,
        })
        .expect("one new key");
    row.flake_reruns = reruns.into_boxed();
    save(d, env, row, out);
    emit(out, Request::WakeProcedure { task });
    true
}

fn flake_pending(flight: &runs::Follow) -> Box<[api::JobAttempt]> {
    let mut pending = List::with_capacity(16);
    for write in &flight.writes {
        if let api::Write::Rerun { attempt } = write {
            pending.push(*attempt).expect("bounded failed jobs");
        }
    }
    pending.into_boxed()
}

fn count_gate_flakes(d: &mut Domain, env: &Env<Limits>, task: u64, key: &ReceiptKey, out: &mut Queue<Request>) {
    let mut numbers = List::with_capacity(64);
    for rerun in &d.landing.rows.get(&task).expect("dispatching landing").flake_reruns {
        if rerun.key == *key && !rerun.ended {
            for number in &rerun.findings {
                numbers.push(*number).expect("bounded matched findings");
            }
        }
    }
    for number in numbers.into_boxed() {
        if let Some(row) = d.finding(key.project, number) {
            let mut row = row.clone();
            row.seen = Some(row.seen.unwrap_or(0).saturating_add(1));
            d.commit_finding(&env.limits, crate::findings::FindingWrite { row, erase: None }, out);
        }
    }
}

/// The root supplies one entry per decision. Persist the remaining jobs beside that
/// entry, draining them without spending `gate_retries` (domain/landing-regate.md, 2).
fn continue_flake_rerun(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut LandingRow,
    entry: u64,
    out: &mut Queue<Request>,
) -> bool {
    let mut pending = None;
    for rerun in &row.flake_reruns {
        if !rerun.ended && (!rerun.pending.is_empty() || rerun.redispatch) && d.landing.flights.contains_key(&rerun.key)
        {
            pending = Some(rerun.key.clone());
            break;
        }
    }
    let Some(key) = pending else { return false };
    let flight = d.landing.flights.get(&key).expect("pending flake rerun");
    let Some(crate::purpose::EffectPurpose::Landing { member, gate, preparation, attempt: try_, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
    else {
        return false;
    };
    let base = entry_base(row, flight.entry);
    if flight.waiting || entry == 0 {
        leave_room(d, row);
        save(d, env, row.clone(), out);
        emit_decision(
            out,
            Request::LandingDecision {
                task: row.landing.task,
                decision: policy::Decision::Wait {
                    until: Some(poll_at(d, env, row.landing.task)),
                    on: policy::Awaited::Run { member, gate },
                },
                entry: None,
                evidence: None,
                head_trailers: Box::new([]),
            },
            d,
            env,
        );
        return true;
    }
    if !admit_entry(d, env, row.landing.task, entry, row, (member, policy::Awaited::Run { member, gate }, None), out) {
        return true;
    }
    let flight = d.landing.flights.get_mut(&key).expect("pending flake rerun");
    let write = flight.write(entry).expect("kept pending write");
    for rerun in &mut row.flake_reruns {
        if rerun.key == key {
            rerun.pending = flake_pending(flight);
            rerun.redispatch = false;
        }
    }
    let operation = if let api::Write::Rerun { .. } = write { Operation::Rerun } else { Operation::Dispatch };
    flight.purpose = crate::purpose::EffectPurpose::Landing {
        task: row.landing.task,
        member,
        gate,
        operation: self::operation(operation),
        preparation,
        attempt: try_,
    }
    .identity()
    .expect("landing purpose");
    keep(d, &env.limits, row, entry, member, gate, operation, write, skein_forge_client::Condition::None, out);
    if let Some(kept) = row.run_entry_mut(entry) {
        kept.preparation = Some(preparation);
        kept.base = base;
    }
    for kept in &mut row.entries {
        if kept.entry == entry {
            kept.preparation = Some(preparation);
            kept.base = base;
        }
    }
    save(d, env, row.clone(), out);
    mark_key(d, &key, out);
    emit_decision(
        out,
        Request::LandingDecision {
            task: row.landing.task,
            decision: policy::Decision::Retry { member, gate, key: key.key, try_ },
            entry: Some(entry),
            evidence: None,
            head_trailers: Box::new([]),
        },
        d,
        env,
    );
    true
}

fn continue_retry(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut LandingRow,
    entry: u64,
    out: &mut Queue<Request>,
) -> bool {
    let mut pending = None;
    for (key, flight) in &d.landing.flights {
        if flight.task == row.landing.task && !flight.writes.is_empty() {
            pending = Some(key.clone());
            break;
        }
    }
    let Some(key) = pending else {
        return false;
    };
    let flight = d.landing.flights.get(&key).expect("pending flight");
    let Some(crate::purpose::EffectPurpose::Landing { member: owner, gate: check, preparation, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
    else {
        return false;
    };
    if preparation != row.landing.preparation {
        return false;
    }
    let mut position = None;
    for (member, m) in row.landing.members.iter().enumerate() {
        if repository(m) != key.repository {
            continue;
        }
        for (gate, proof) in m.gates.iter().enumerate() {
            if let policy::Proof::Dispatched { key: proof, try_ } = proof.proof
                && proof == key.key
                && usize::from(owner) == member
                && usize::from(check) == gate
            {
                position = Some((u16::try_from(member).expect("member"), u16::try_from(gate).expect("gate"), try_));
            }
        }
    }
    let Some((member, gate, try_)) = position else {
        return false;
    };
    if result(d, row.landing.task, member, gate, 2) == policy::Status::Pending
        || flight.waiting
        || match &flight.run {
            Some(run) => run.state != api::RunState::Completed,
            None => true,
        }
    {
        leave_room(d, row);
        save(d, env, row.clone(), out);
        emit_decision(
            out,
            Request::LandingDecision {
                task: row.landing.task,
                decision: policy::Decision::Wait {
                    until: Some(poll_at(d, env, row.landing.task)),
                    on: policy::Awaited::Run { member, gate },
                },
                entry: None,
                evidence: None,
                head_trailers: Box::new([]),
            },
            d,
            env,
        );
        return true;
    }
    if !admit_entry(d, env, row.landing.task, entry, row, (member, policy::Awaited::Run { member, gate }, None), out) {
        return true;
    }
    let flight = d.landing.flights.get_mut(&key).expect("admitted reruns");
    let write = flight.write(entry).expect("next failed job");
    keep(d, &env.limits, row, entry, member, gate, Operation::Rerun, write, skein_forge_client::Condition::None, out);
    save(d, env, row.clone(), out);
    emit_decision(
        out,
        Request::LandingDecision {
            task: row.landing.task,
            decision: policy::Decision::Retry { member, gate, key: key.key, try_ },
            entry: Some(entry),
            evidence: None,
            head_trailers: Box::new([]),
        },
        d,
        env,
    );
    true
}

fn following(d: &Domain, task: u64) -> bool {
    following_flight(d, task).is_some()
}
#[expect(clippy::manual_find, reason = "the strict subset uses no closures")]
fn following_flight(d: &Domain, task: u64) -> Option<&runs::Follow> {
    for (_, flight) in &d.landing.flights {
        if flight.task == task {
            return Some(flight);
        }
    }
    None
}
fn following_wait(d: &Domain, task: u64) -> Option<policy::Awaited> {
    let flight = following_flight(d, task)?;
    let Some(crate::purpose::EffectPurpose::Landing { member, gate, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
    else {
        unreachable!("landing flight names its member and gate");
    };
    Some(match &flight.run {
        Some(run) if run.state == api::RunState::Completed => policy::Awaited::Receipt { member, gate },
        Some(_) | None => policy::Awaited::Run { member, gate },
    })
}

/// Startup's fresh-read wake may precede task activation; rebuild the poll at `CatchUp`
/// (domain/landing.md, 12; domain/root-restart.md, 1.1).
pub(crate) fn restoring(d: &mut Domain, env: &Env<Limits>, row: &LandingRow) {
    read_wait::advanced(d, env, row);
}

pub(crate) fn restore_waits(d: &mut Domain, env: &Env<Limits>) {
    read_wait::restore_waits(d, env);
    for (task, row) in &d.landing.rows {
        if row.room.is_some() || d.landing.unmade.contains_key(task) {
            continue;
        }
        match row.landing.phase {
            policy::Phase::Waiting
            | policy::Phase::Preparing
            | policy::Phase::Proving
            | policy::Phase::Proved { .. }
            | policy::Phase::Turn { .. } => {
                let until = poll_at(d, env, *task);
                d.landing.due.insert(*task, until).expect("one poll per live landing");
            }
            policy::Phase::Landed if following(d, *task) => {
                let until = poll_at(d, env, *task);
                d.landing.due.insert(*task, until).expect("one poll per following landing");
            }
            policy::Phase::Held { .. } | policy::Phase::Landed | policy::Phase::Cancelled => {}
        }
    }
}

pub(crate) fn refresh(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(env.limits.landings);
    for (task, row) in &d.landing.rows {
        if (row.landing.phase != policy::Phase::Landed && row.landing.phase != policy::Phase::Cancelled)
            || (row.landing.phase == policy::Phase::Landed && following(d, *task))
        {
            tasks.push(*task).expect("bounded landings");
        }
    }
    for task in tasks.into_boxed() {
        step(
            d,
            env,
            task,
            0,
            LandingHeard {
                job: None,
                outcomes: Box::new([]),
                dependencies: Box::new([]),
                held: Box::new([]),
                ended: Box::new([]),
                failure: Box::new([]),
                preparation: None,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
            out,
        );
    }
}

pub(crate) fn judge(
    d: &Domain,
    row: &LandingRow,
    criterion: crate::Criterion,
    head: api::Commit,
    evidence: &crate::ChangeEvidence,
    reviewers: &[crate::Reviewer],
) -> Option<crate::JudgeVerdict> {
    for member in &row.landing.members {
        if !crate::domain::repository_fresh(d, repository(member)) {
            return Some(crate::JudgeVerdict::Wait);
        }
    }
    for (position, member) in row.landing.members.iter().enumerate() {
        if member.candidate != Some(head) {
            continue;
        }
        let repo = repository(member);
        if evidence.head != Some(head) {
            return Some(crate::JudgeVerdict::Wait);
        }
        if criterion == crate::Criterion::Ci {
            let gate_position =
                if row.landing.round.branch.is_empty() { u16::try_from(position).ok()? } else { round_effect_top(row) };
            let gate_member = row.landing.members.get(usize::from(gate_position))?;
            for (number, configured) in d.train_gates(row, repo).iter().enumerate() {
                if !configured.blocking {
                    continue;
                }
                let Some(key) = proof_key(gate_member.gates.get(number).expect("gate proof").proof) else {
                    return Some(crate::JudgeVerdict::Wait);
                };
                let key = gate_key(row, gate_position, u16::try_from(number).ok()?, &configured.name, key);
                match d.receipt(&key) {
                    Some(receipt) if receipt.passed => {}
                    Some(_) => return Some(crate::JudgeVerdict::Refuse),
                    None => return Some(crate::JudgeVerdict::Wait),
                }
            }
            return Some(crate::JudgeVerdict::Met);
        }
        let change = d.changes.get(&member.change)?;
        let mut clean =
            List::with_capacity(u32::try_from(change.change.clean.len()).expect("bounded clean").saturating_add(1));
        for head in &change.change.clean {
            clean.push(*head).expect("clean room");
        }
        if let Some(head) = member.head {
            clean.push(head).expect("head room");
        }
        return Some(crate::judge::judge(
            criterion,
            d.repositories.get(&repo)?,
            head,
            clean.as_slice(),
            evidence,
            reviewers,
            (&change.change.gates, change.spec_revision),
        ));
    }
    None
}

/// An active key's owner takes precedence over a previous owner's fault.
pub(crate) fn flight_fact(state: &State, key: &ReceiptKey, position: Position) -> policy::Flight {
    match state.flights.get(key) {
        Some(flight) if flight.task == position.0 => policy::Flight::Mine { run: flight.outcome },
        Some(_) => policy::Flight::Other,
        None => match state.faults.get(&position) {
            Some(fault) => match fault.gate_problem {
                Some(GateProblem::Broke(_)) => policy::Flight::Broke,
                Some(GateProblem::Runner { .. }) | None => policy::Flight::Mine { run: fault.outcome },
            },
            None => policy::Flight::None,
        },
    }
}

/// Terminal rows with no effects left surrender their live slot on admission
/// (domain/landing.md, 12); receipts have their independent retention.
fn retire_finished(d: &mut Domain, out: &mut Queue<Request>) -> bool {
    let mut finished = None;
    for (task, row) in &d.landing.rows {
        if reclaimable(d, row) {
            finished = Some(*task);
            break;
        }
    }
    let Some(task) = finished else {
        return false;
    };
    d.stall_words.remove(&task);
    read_wait::clear(d, task);
    crate::room::leave(d, crate::room::Waiter::Landing(task));
    let retired = d.landing.erase(task).expect("selected finished landing");
    emit(
        out,
        Request::Marked {
            tasks: Box::new([
                task,
                crate::landings_subject(retired.landing.project, false),
                crate::landings_subject(retired.landing.project, true),
            ]),
        },
    );
    d.landing.due.remove(&task);
    for member in 0..d.landing.limits.landing_members {
        d.landing.results.remove(&(task, u16::try_from(member).expect("member position"), 0, 5));
        for gate in 0..d.landing.limits.gates {
            let member = u16::try_from(member).expect("admitted member position");
            let gate = u16::try_from(gate).expect("admitted gate position");
            d.landing.keys.remove(&(task, member, gate));
            d.landing.faults.remove(&(task, member, gate));
            for operation in 1..=9 {
                d.landing.results.remove(&(task, member, gate, operation));
            }
        }
    }
    emit(out, Request::Erase { key: Key::Landing(task) });
    true
}

fn reclaimable(d: &Domain, row: &LandingRow) -> bool {
    if !matches_terminal(&row.landing.phase) || following(d, row.landing.task) {
        return false;
    }
    for (_, entry) in &d.entries {
        if entry.entry.task == row.landing.task {
            return false;
        }
    }
    true
}

pub(crate) fn matches_terminal(phase: &policy::Phase) -> bool {
    match phase {
        policy::Phase::Landed | policy::Phase::Cancelled => true,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Held { .. } => false,
    }
}

impl Domain {
    #[must_use]
    pub fn landing_hold_words(&self, task: u64) -> Option<Box<[u8]>> {
        let row = self.landing.rows.get(&task)?;
        self.landing_hold_reason(row)
    }
    /// Shared by the landing hold, the task's cancellation and the members' news.
    #[must_use]
    pub fn landing_stuck_words(&self, task: u64, cause: policy::Stuck) -> Option<Box<[u8]>> {
        Some(self.stuck_words(self.landing.rows.get(&task)?, cause))
    }

    fn stuck_words(&self, row: &LandingRow, cause: policy::Stuck) -> Box<[u8]> {
        use skein_lib::Decimal;
        let words = match cause {
            policy::Stuck::Member { change, held } => concat(&[
                b"cannot move: member change ",
                Decimal::of(change).as_bytes(),
                if held { b" is held" } else { b" has ended" },
            ]),
            policy::Stuck::Train { landing } => {
                concat(&[b"cannot move: landing ", Decimal::of(landing).as_bytes(), b" holds its train"])
            }
            policy::Stuck::Wake { on } => {
                let wait = match read_wait::wait_words(self, row, on) {
                    Some((_, words)) => words,
                    None => Box::from(b"a missing member".as_slice()),
                };
                concat(&[b"cannot move: its wait has no wake: ", &wait])
            }
        };
        crate::text::fit(&words, skein_lib::usize_of(crate::history::REASON_BYTES), b"...")
    }

    pub(crate) fn landing_hold_reason(&self, row: &LandingRow) -> Option<Box<[u8]>> {
        let failure = crate::preparation::failure(&row.landing.round.failure);
        let policy::Phase::Held { why, .. } = row.landing.phase else { return None };
        match why {
            policy::Hold::Claimed { change } => {
                let change_words = skein_lib::Decimal::of(change);
                Some(match members::older(self, row.landing.task, change) {
                    Some(owner) => concat(&[
                        b"change ",
                        change_words.as_bytes(),
                        b" is a member of landing ",
                        skein_lib::Decimal::of(owner).as_bytes(),
                        b" too, which keeps it",
                    ]),
                    None => concat(&[b"change ", change_words.as_bytes(), b" was claimed by an older landing"]),
                })
            }
            policy::Hold::Read => row.read_hold.clone(),
            policy::Hold::Stuck(cause) => Some(self.stuck_words(row, cause)),
            policy::Hold::Cleanup => Some(concat(&[b"cleanup: ", row.landing.round.cleanup.as_deref()?])),
            policy::Hold::Prepare => Some(concat(&[b"prepare job: ", &failure])),
            policy::Hold::Record { member } => Some(concat(&[
                b"turn: record of #",
                skein_lib::Decimal::of(row.landing.members.get(usize::from(member))?.pull).as_bytes(),
                b": ",
                &failure,
            ])),
            policy::Hold::Turn => Some(concat(&[b"turn: ", &failure])),
            policy::Hold::Faults { member, gate } | policy::Hold::Timeout { member, gate, .. } => {
                let m = row.landing.members.get(usize::from(member))?;
                let repo = self.repository(repository(m))?;
                let proof = proof_key(m.gates.get(usize::from(gate))?.proof);
                let gate = self.train_gates(row, repository(m)).get(usize::from(gate))?;
                let mut words = List::with_capacity(16384);
                for part in [b"gate ".as_slice(), &gate.name, b" of ", &repo.owner, b"/", &repo.name] {
                    append_words(&mut words, part);
                }
                match why {
                    policy::Hold::Faults { .. } => {
                        append_words(
                            &mut words,
                            row.gate_hold.as_deref().unwrap_or(b" ran past its retries without judging the change"),
                        );
                    }
                    policy::Hold::Timeout { run, .. } => {
                        append_words(&mut words, b": run ");
                        append_words(&mut words, skein_lib::Decimal::of(run).as_bytes());
                        let mut waiting = false;
                        if let Some(proof) = proof {
                            let key = ReceiptKey {
                                project: row.landing.project,
                                repository: repository(m),
                                check: Check::Gate(gate.name.clone()),
                                key: proof,
                            };
                            if let Some(flight) = self.landing.flights.get(&key)
                                && let Some(clock) = flight.clock
                            {
                                let timeout = match self.landing.projects.get(&row.landing.project) {
                                    Some((_, _, _, _, _, settings, _)) => settings.gate_timeout,
                                    None => policy::Project::DEFAULT.gate_timeout,
                                };
                                waiting = clock.ran(clock.at) <= timeout
                                    && (clock.waited(clock.at) >= self.landing.limits.change_policy.runner_wait
                                        || clock.seen == change::Execution::Waiting);
                            }
                        }
                        append_words(
                            &mut words,
                            if waiting { b" waiting for a runner past its bound" } else { b" running past its bound" },
                        );
                        return Some(self.ci_stall_words(
                            row.landing.task,
                            words.as_slice(),
                            jig_domain_tasks::HOLD_ACCOUNT_BYTES,
                        ));
                    }
                    policy::Hold::Cleanup
                    | policy::Hold::Prepare
                    | policy::Hold::Read
                    | policy::Hold::Record { .. }
                    | policy::Hold::Turn
                    | policy::Hold::Claimed { .. }
                    | policy::Hold::Stuck(_) => unreachable!("gate hold"),
                }
                Some(words.into_boxed())
            }
        }
    }
    /// The one source of a member's leaving words (domain/landing.md, 4.3).
    #[must_use]
    pub fn landing_leave_words(&self, task: u64, member: u16) -> Option<Box<[u8]>> {
        let row = self.landing.rows.get(&task)?;
        let m = row.landing.members.get(usize::from(member))?;
        let files = conflict_words(row, member);
        Some(match m.left? {
            policy::Leave::Conflict => concat(&[b"conflict with the branch: ", &files]),
            policy::Leave::EarlierConflict => concat(&[b"conflict with an earlier member: ", &files]),
            policy::Leave::Unit { change } => {
                concat(&[b"change ", skein_lib::Decimal::of(change).as_bytes(), b" of its unit left"])
            }
            policy::Leave::Dependency { change } => {
                concat(&[b"its dependency, change ", skein_lib::Decimal::of(change).as_bytes(), b", left"])
            }
            policy::Leave::Blamed { gate, run, red, green } => concat(&[
                b"gate ",
                &self.train_gates(row, repository(m)).get(usize::from(gate))?.name,
                b" failed in ",
                skein_lib::Decimal::of(run).as_bytes(),
                b" at ",
                &commit_hex(red),
                b", above ",
                &commit_hex(green),
            ]),
            policy::Leave::HeadMoved { head } => concat(&[b"head moved to ", &commit_hex(head)]),
            policy::Leave::Cancelled => Box::from(b"cancelled".as_slice()),
            policy::Leave::Held => Box::from(b"held".as_slice()),
            policy::Leave::Refused { gate } => {
                concat(&[b"gate ", skein_lib::Decimal::of(gate).as_bytes(), b" refused"])
            }
            policy::Leave::Failed { gate, run } => concat(&[
                b"gate ",
                &self.train_gates(row, repository(m)).get(usize::from(gate))?.name,
                b" failed in ",
                skein_lib::Decimal::of(run).as_bytes(),
            ]),
        })
    }
    /// The prefix refused by this gate, while its change repairs at its own head.
    #[must_use]
    pub fn landing_refused_at(&self, task: u64, gate: u64) -> Option<api::Commit> {
        let mut prefix = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change == task && member.left == Some(policy::Leave::Refused { gate }) {
                    prefix = member.candidate;
                }
            }
        }
        prefix
    }
    /// The candidate pins its failure snapshot even after the landing's live row retires.
    #[must_use]
    pub fn candidate_failure_words(&self, change: u64, gate: u16, candidate: api::Commit) -> Option<Box<[u8]>> {
        let (_, _, receipt) = self.failure_receipt(change, gate, Some(candidate))?;
        Some(receipt.logs.clone())
    }
    /// The receipt pins the gate run and its jobs for a repair's required words.
    #[must_use]
    pub fn landing_failure(&self, change: u64, why: change::Repair) -> Option<crate::LandingFailure> {
        let (_, gate, receipt) = self.repair_receipt(change, why)?;
        Some(crate::LandingFailure {
            gate: gate.into(),
            run: receipt.run.clone(),
            report: receipt.report.clone(),
            prefix: receipt.candidate,
            base: receipt.base,
            jobs: receipt.jobs.clone(),
        })
    }
    /// Only a change's current repair reads gate checks at its head or candidate.
    #[must_use]
    pub fn landing_failure_at(
        &self,
        parent: u64,
        task: u64,
        repository: api::Repository,
        commit: api::Commit,
    ) -> Option<crate::LandingFailure> {
        let row = self.change(parent)?;
        let (child, change::Delegate::Repair(why)) = row.delegate? else { return None };
        if row.terminal() || child != task || row.repository != repository {
            return None;
        }
        let failure = self.landing_failure(parent, why)?;
        if row.change.last_head != Some(commit) && failure.prefix != commit {
            return None;
        }
        Some(failure)
    }

    pub(super) fn landing_regate(&self, change: u64, why: change::Repair) -> Option<crate::Regate> {
        let (key, _, receipt) = self.repair_receipt(change, why)?;
        Some(crate::Regate { receipt: key.clone(), run: receipt.run.id, taken: None })
    }
    pub(super) fn release_regate(&self, row: &crate::ChangeRow, why: change::Repair) -> Option<crate::Regate> {
        if let Some(prior) = &row.regate {
            let run = prior.taken.unwrap_or(prior.run);
            let receipt = self.receipt(&prior.receipt)?;
            if receipt.passed || receipt.run.id != run {
                return None;
            }
            return Some(crate::Regate { receipt: prior.receipt.clone(), run, taken: None });
        }
        self.landing_regate(row.task, why)
    }

    pub(super) fn regate_failure(&self, row: &crate::ChangeRow, ask: &change::Handed) -> Option<&Receipt> {
        let regate = row.regate.as_ref()?;
        let taken = regate.taken?;
        let change::Ask::Repair { gate, candidate, .. } = ask.ask else { return None };
        let landing = self.landing.rows.get(&ask.landing)?;
        let receipt = self.receipt(&regate.receipt)?;
        if receipt.passed || receipt.run.id != taken || regate.receipt.project != landing.landing.project {
            return None;
        }
        for (position, member) in landing.landing.members.iter().enumerate() {
            if member.change != row.task || repository(member) != regate.receipt.repository {
                continue;
            }
            let name = &self.train_gates(landing, repository(member)).get(usize::from(gate))?.name;
            if regate.receipt.check != Check::Gate(name.clone()) {
                return None;
            }
            let measured = match member.left {
                Some(policy::Leave::Failed { gate: failed, run }) => failed == gate && run == receipt.run.number,
                Some(policy::Leave::Blamed { gate: failed, run, red, .. }) => {
                    failed == gate && red == candidate && run == receipt.run.number
                }
                // A single landing may hand the repair from the failed proof without leaving.
                None => {
                    let fact = member.gates.get(usize::from(gate))?;
                    fact.proof == (policy::Proof::Failed { key: regate.receipt.key })
                        && member.candidate == Some(candidate)
                        && followed_measurement(landing, ask.round, u16::try_from(position).ok()?, gate, taken)
                }
                Some(
                    policy::Leave::Conflict
                    | policy::Leave::EarlierConflict
                    | policy::Leave::Unit { .. }
                    | policy::Leave::Dependency { .. }
                    | policy::Leave::HeadMoved { .. }
                    | policy::Leave::Cancelled
                    | policy::Leave::Held
                    | policy::Leave::Refused { .. },
                ) => false,
            };
            if measured {
                return Some(receipt);
            }
        }
        None
    }
    fn repair_receipt(&self, change: u64, why: change::Repair) -> Option<(&ReceiptKey, &[u8], &Receipt)> {
        let (gate, candidate) = match why {
            change::Repair::LandingAt { gate, candidate, .. } => (gate, Some(candidate)),
            change::Repair::Landing { gate } => (gate, None),
            change::Repair::Ci
            | change::Repair::Semantic
            | change::Repair::Gate(_)
            | change::Repair::Review { .. }
            | change::Repair::TemperDir { .. } => {
                return None;
            }
        };
        self.failure_receipt(change, gate, candidate)
    }
    fn failure_receipt(
        &self,
        change: u64,
        gate: u16,
        candidate: Option<api::Commit>,
    ) -> Option<(&ReceiptKey, &[u8], &Receipt)> {
        let mut kept_run = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change != change {
                    continue;
                }
                match member.left {
                    Some(policy::Leave::Blamed { gate: failed, run, red, .. }) => {
                        if failed == gate && (candidate.is_none() || candidate == Some(red)) {
                            kept_run = Some(run);
                        }
                    }
                    Some(policy::Leave::Failed { gate: failed, run }) => {
                        let prefix = row.landing.members.get(usize::from(round_effect_top(row)));
                        if failed == gate && (candidate.is_none() || matches_candidate(prefix, candidate)) {
                            kept_run = Some(run);
                        }
                    }
                    None
                    | Some(
                        policy::Leave::Conflict
                        | policy::Leave::EarlierConflict
                        | policy::Leave::Unit { .. }
                        | policy::Leave::Dependency { .. }
                        | policy::Leave::HeadMoved { .. }
                        | policy::Leave::Cancelled
                        | policy::Leave::Held
                        | policy::Leave::Refused { .. },
                    ) => {}
                }
            }
        }
        let change = self.change(change)?;
        let project = self.repository(change.repository)?.project;
        let name = &self.landing.gates(project, change.repository).get(usize::from(gate))?.name;
        for (key, receipt) in &self.landing.receipts {
            if key.project == project
                && key.repository == change.repository
                && key.check == Check::Gate(name.clone())
                && match kept_run {
                    Some(run) => run == receipt.run.number,
                    None => candidate == Some(receipt.candidate),
                }
                && !receipt.passed
                && !empty_gate_receipt(key, receipt)
            {
                return Some((key, name, receipt));
            }
        }
        None
    }
    /// The failed jobs are read before the receipt is committed; repairs use that snapshot.
    #[must_use]
    pub fn landing_failure_words(&self, change: u64, gate: u16) -> Option<Box<[u8]>> {
        let mut words = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change == change && matches_failed(member.left, gate) {
                    words = Some(crate::preparation::failure(&row.landing.round.failure));
                }
            }
        }
        words
    }
    /// The news lines are formed from the committed landing (domain/landing.md, 4.2).
    #[must_use]
    pub fn landing_news_words(&self, task: u64, position: u16, step: crate::LandingNews) -> Option<(u64, Box<[u8]>)> {
        let row = self.landing.rows.get(&task)?;
        let member = row.landing.members.get(usize::from(position))?;
        let mut words = List::with_capacity(16384);
        append_words(&mut words, b"landing ");
        append_words(&mut words, skein_lib::Decimal::of(task).as_bytes());
        append_words(&mut words, b": ");
        if let crate::LandingNews::Joined
        | crate::LandingNews::Proving
        | crate::LandingNews::Probing
        | crate::LandingNews::Landed
        | crate::LandingNews::Left = step
        {
            append_words(&mut words, b"change ");
            append_words(&mut words, skein_lib::Decimal::of(member.change).as_bytes());
            append_words(&mut words, b" ");
        }
        match step {
            crate::LandingNews::Joined => {
                let repo = self.repository(repository(member))?;
                for part in [
                    b"joined, into ".as_slice(),
                    &repo.owner,
                    b"/",
                    &repo.name,
                    b" ",
                    &row.landing.branch,
                    b", round ",
                    skein_lib::Decimal::of(u64::from(row.landing.preparation.max(1))).as_bytes(),
                ] {
                    append_words(&mut words, part);
                }
                let mut separator = b", with changes ".as_slice();
                for peer in &row.landing.members {
                    if peer.change != member.change {
                        append_words(&mut words, separator);
                        append_words(&mut words, skein_lib::Decimal::of(peer.change).as_bytes());
                        separator = b", ";
                    }
                }
            }
            crate::LandingNews::Proving => {
                for part in
                    [b"proving at ".as_slice(), &commit_hex(member.candidate?), b" on ", &commit_hex(member.base?)]
                {
                    append_words(&mut words, part);
                }
                if let Some(top) = stack_candidate(row)
                    && Some(top) != member.candidate
                {
                    append_words(&mut words, b", under ");
                    append_words(&mut words, &commit_hex(top));
                }
            }
            crate::LandingNews::Probing => {
                let search = row.landing.round.search.as_ref()?;
                let probe = row.landing.members.get(usize::from(policy::probe(&row.landing)?))?;
                let red = row.landing.members.get(usize::from(search.red.checked_sub(1)?))?;
                append_words(&mut words, b"probing at ");
                append_words(&mut words, &commit_hex(probe.candidate?));
                match &search.tests {
                    policy::SearchTests::ByTest { gates, .. } => {
                        let mut count = 0_u64;
                        for cases in gates.iter().flatten() {
                            count = count
                                .checked_add(u64::try_from(cases.len()).expect("bounded selection"))
                                .expect("bounded cases");
                        }
                        append_words(&mut words, b" by ");
                        append_words(&mut words, skein_lib::Decimal::of(count).as_bytes());
                        append_words(&mut words, b" tests");
                    }
                    policy::SearchTests::Whole | policy::SearchTests::WholeAgain => {}
                }
                for part in [b", red at ".as_slice(), &commit_hex(red.candidate?), b", on ", &commit_hex(probe.base?)] {
                    append_words(&mut words, part);
                }
            }
            crate::LandingNews::WholeAgain => {
                append_words(&mut words, b"probing whole again from ");
                append_words(&mut words, &commit_hex(member.base?));
            }
            crate::LandingNews::Landed => {
                append_words(&mut words, b"landed at ");
                append_words(&mut words, &commit_hex(member.landed?));
            }
            crate::LandingNews::Left => {
                append_words(&mut words, b"left: ");
                append_words(&mut words, &self.landing_leave_words(task, position)?);
            }
            crate::LandingNews::Held => {
                append_words(&mut words, b"held: ");
                append_words(&mut words, &self.landing_hold_words(task)?);
            }
            crate::LandingNews::Cancelled(cause) => {
                append_words(&mut words, b"cancelled: ");
                append_words(&mut words, &self.stuck_words(row, cause));
            }
        }
        Some((member.change, words.into_boxed()))
    }
    /// Describe the exact saved landing write before core authorization.
    #[must_use]
    pub fn describe_landing_effect(
        &self,
        task: u64,
        entry: u64,
    ) -> Option<(api::Repository, crate::What, crate::AgentEffect)> {
        let row = self.landing.rows.get(&task)?;
        let mut item = None;
        for candidate in &row.entries {
            if candidate.entry == entry {
                item = Some(*candidate);
            }
        }
        let item = item?;
        let member = row.landing.members.get(usize::from(item.member))?;
        let kept = &self.entries.get(&entry)?.entry;
        if kept.task != task {
            return None;
        }
        let (kind, what, state) = match &kept.effect.write {
            api::Write::Dispatch { .. } => (16, crate::What::Repository, member.candidate?),
            api::Write::Rerun { .. } => (17, crate::What::Repository, member.candidate?),
            api::Write::Status { commit, .. } => (6, crate::What::Repository, *commit),
            api::Write::Merge { head, how, .. } => {
                let change = self.changes.get(&member.change)?;
                let mut parts = List::with_capacity(self.landing.limits.authority_segments);
                let mut start = 0;
                for (index, byte) in change.base.iter().enumerate() {
                    if *byte == b'/' {
                        parts.push(Box::from(change.base.get(start..index)?)).ok()?;
                        start = index.checked_add(1)?;
                    }
                }
                parts.push(Box::from(change.base.get(start..)?)).ok()?;
                (
                    4,
                    crate::What::Branch(parts.into_boxed()),
                    match how {
                        api::MergeHow::Recorded { commit } => *commit,
                        api::MergeHow::FastForwardOnly | api::MergeHow::Style => *head,
                    },
                )
            }
            api::Write::OpenPull { .. } => (3, crate::What::Repository, member.candidate?),
            api::Write::Close { number } => (12, crate::What::Pull(*number), member.candidate.unwrap_or([0; 32])),
            api::Write::DeleteBranch { branch } => {
                let resource = skein_forge_client::Resource {
                    repository: kept.repository,
                    what: skein_forge_client::What::Branch(branch.clone()),
                };
                (
                    15,
                    super::domain::from_client(&resource, &self.landing.limits)?.what,
                    member.candidate.unwrap_or([0; 32]),
                )
            }
            api::Write::CreateIssue { .. }
            | api::Write::Post { .. }
            | api::Write::Review { .. }
            | api::Write::Edit { .. }
            | api::Write::SetReviewers { .. }
            | api::Write::Reopen { .. }
            | api::Write::Update { .. }
            | api::Write::CreateBranch { .. } => return None,
        };
        crate::effect_shape(kind)?;
        Some((kept.repository, what, crate::AgentEffect { effect: kept.effect.clone(), kind, state }))
    }
}
/// Authority's wait is not a refused merge: it spent no send and keeps its turn
/// (domain/forge/catching-up.md, 4; domain/landing-turn.md, 1).
pub(crate) fn defer_merge(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    if !matches_entry(d, task, entry) {
        return;
    }
    let mut row = d.landing.rows.get(&task).expect("staged landing").clone();
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for item in &row.entries {
        if item.entry != entry {
            entries.push(*item).expect("bounded entries");
            continue;
        }
        match item.operation {
            Operation::Merge => {
                if !row.landing.round.branch.is_empty() {
                    row.landing.round.merges = row.landing.round.merges.checked_sub(1).expect("staged merge counted");
                }
            }
            Operation::Record => {}
            Operation::Dispatch
            | Operation::Rerun
            | Operation::Pending
            | Operation::Verdict
            | Operation::Open
            | Operation::Close
            | Operation::Delete => unreachable!("only merge authority is deferred"),
        }
        d.landing
            .results
            .insert((task, item.member, item.gate, operation(item.operation)), (policy::Status::Unknown, false))
            .expect("staged effect position");
    }
    row.entries = entries.into_boxed();
    outbox::outcome(d, task, entry, skein_forge_client::Outcome::Withdrawn, out);
    take_entry(d, entry, task);
    emit(out, Request::Erase { key: Key::Entry(entry) });
    save(d, env, row, out);
    let repositories = match d.landing.rows.get(&task) {
        Some(row) => repositories(row, d.repositories.capacity()),
        None => Box::new([]),
    };
    if super::domain::wait_fresh(d, task, &repositories) {
        d.landing.due.remove(&task);
    }
}

/// A refused entry never reaches the client and is observed as failed next step.
pub(crate) fn veto(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    if !matches_entry(d, task, entry) {
        return;
    }
    outbox::outcome(d, task, entry, skein_forge_client::Outcome::Withdrawn, out);
    outcome(d, env, entry, skein_forge_client::Outcome::Failed(api::Error::Refused), None, out);
    take_entry(d, entry, task);
    emit(out, Request::Erase { key: Key::Entry(entry) });
}
fn matches_entry(d: &Domain, task: u64, entry: u64) -> bool {
    match d.entries.get(&entry) {
        Some(row) => row.entry.task == task && d.landing.rows.contains_key(&task),
        None => false,
    }
}

/// Ending a landing leaves its changes live; cancelling a change removes it from its landing.
pub(crate) fn closing(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    ending: crate::ReleaseEnding,
    out: &mut Queue<Request>,
) {
    if d.landing.rows.contains_key(&task) {
        end_row(d, env, task, out);
        return;
    }
    if ending != crate::ReleaseEnding::Cancelled {
        return;
    }
    let Some(landing) = d.landing_of(task) else { return };
    let Some(mut row) = d.landing.rows.get(&landing).cloned() else {
        unreachable!("change {task} cancelling must have its indexed landing {landing}");
    };
    assert!(!matches_terminal(&row.landing.phase), "change {task} cancelling must have a live landing {landing}");
    let mut owned = None;
    for member in &row.landing.members {
        if member.change == task {
            owned = Some(member);
            break;
        }
    }
    let Some(member) = owned else {
        unreachable!("change {task} cancelling must be held by its indexed landing {landing}");
    };
    assert!(member.left.is_none(), "change {task} cancelling has not left its indexed landing {landing}");
    if member.landed.is_some() {
        return;
    }
    let prior = row.clone();
    row.landing = policy::leave(row.landing, task);
    crate::landing_history::transition(d, env.wall, &prior, &row, out);
    if row.landing.members.len() != prior.landing.members.len() {
        remap_members(d, &mut row, &prior);
    }
    save(d, env, row, out);
    emit(out, Request::WakeProcedure { task: landing });
}

/// Record a member's landed prefix and hand it to the change, as in the turn.
pub(crate) fn hand_landed(
    d: &mut Domain,
    wall: Wall,
    row: &LandingRow,
    member: u16,
    prefix: api::Commit,
    out: &mut Queue<Request>,
) {
    let change = row.landing.members.get(usize::from(member)).expect("member").change;
    let signed = match d.changes.get(&change) {
        Some(row) => row.signed.clone(),
        None => Box::new([]),
    };
    crate::landing_log::landed(d, wall, prefix, change, signed, out);
    crate::verification::landed(d, prefix, change, out);
    members::hand(d, row, member, change::Ask::Landed { prefix }, out);
}

/// Ends a live landing row as its task's closing does (landing-liveness.md, 5):
/// phase `Cancelled`, members told or left ready, index entries and own
/// asks gone, runs no longer followed, train woken, history closed. A row
/// already ended is left as it is.
pub(crate) fn end_row(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(row) = d.landing.rows.get(&task) else { return };
    if matches_terminal(&row.landing.phase) {
        return;
    }
    let prior = row.clone();
    let mut row = prior.clone();
    let mut told = List::with_capacity(env.limits.landing_members);
    for (index, member) in row.landing.members.iter_mut().enumerate() {
        if member.landed.is_some() && !member.told {
            member.told = true;
            told.push(u16::try_from(index).expect("member")).expect("bounded news");
        }
    }
    row.landing.round.told |= !told.is_empty();
    row.landing.phase = policy::Phase::Cancelled;
    row.read_hold = None;
    read_wait::clear(d, task);
    saved_follow::cancelled(d, &mut row, out);
    d.landing.due.remove(&task);
    for member in &row.landing.members {
        if member.left.is_none()
            && d.landing_of(member.change) == Some(task)
            && let Some(change) = d.changes.get_mut(&member.change)
        {
            let prior = change.change.clone();
            if let Some(merge) = member.landed {
                if let Some(change::Handed { ask: change::Ask::Landed { .. }, .. }) = members::head(change) {
                    // The change's step will record the landed prefix.
                } else if crate::domain::landed_commit(&prior.state).is_none() {
                    change.change.state = change::State::Landed { merge, since: env.wall };
                }
            } else if let change::State::Judging { head, ready_since, .. } = change.change.state {
                change.change.state = change::State::Ready { head, ready_since };
            }
            let mut changed = change.clone();
            if crate::domain::landed_commit(&prior.state).is_none()
                && crate::domain::landed_commit(&changed.change.state).is_some()
            {
                crate::domain::reds::landed(d, &env.limits, &changed, out);
            }
            crate::change_history::advanced(d, env.wall, &mut changed, &prior, out);
            d.changes.insert(member.change, changed.clone()).expect("retained member");
            crate::domain::save_change_parts(d, out, changed);
            emit(out, Request::WakeProcedure { task: member.change });
        }
    }
    crate::landing_history::transition(d, env.wall, &prior, &row, out);
    save(d, env, row, out);
    for member in told.into_boxed() {
        emit(out, Request::LandingNews { task, member, step: crate::LandingNews::Landed });
    }
    wake_train_ready(d, task, out);
}

// An ending landing frees only its repository and branch (domain/landing.md, 4.2, and domain/landing-turn.md, 2).
fn wake_train_ready(d: &Domain, task: u64, out: &mut Queue<Request>) {
    let Some(landing) = d.landing.rows.get(&task) else { return };
    let Some(member) = landing.landing.members.first() else { return };
    let repository = repository(member);
    for (task, row) in &d.changes {
        if d.landing_of(*task).is_none()
            && let change::State::Ready { .. } = row.change.state
            && row.repository == repository
            && row.base == landing.landing.branch
        {
            emit(out, Request::WakeProcedure { task: *task });
        }
    }
}

/// Receipt and run observations remake every landing reading this key.
fn mark_key(d: &Domain, key: &ReceiptKey, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(crate::MARKED);
    for (task, row) in &d.landing.rows {
        if row.landing.project != key.project {
            continue;
        }
        let mut reads = false;
        for member in &row.landing.members {
            if repository(member) != key.repository {
                continue;
            }
            for (index, proof) in member.gates.iter().enumerate() {
                if proof_key(proof.proof) == Some(key.key)
                    && let Some(gate) = d.landing.gates(key.project, key.repository).get(index)
                    && key.check == Check::Gate(gate.name.clone())
                {
                    reads = true;
                }
            }
        }
        if reads {
            tasks.push(*task).expect("mark room");
            if tasks.room() == 0 {
                emit(out, Request::Marked { tasks: tasks.into_boxed() });
                tasks = List::with_capacity(crate::MARKED);
            }
        }
    }
    if !tasks.is_empty() {
        emit(out, Request::Marked { tasks: tasks.into_boxed() });
    }
}

fn write_member(decision: policy::Decision) -> Option<(u16, policy::Awaited)> {
    match decision {
        // Every member of a round names its train's repository.
        policy::Decision::Open => Some((0, policy::Awaited::Open)),
        policy::Decision::Close => Some((0, policy::Awaited::Close)),
        policy::Decision::Delete { member } => Some((member, policy::Awaited::Delete { member })),
        policy::Decision::Record { member, .. } => Some((member, policy::Awaited::Record { member })),
        policy::Decision::Dispatch { member, gate, .. } | policy::Decision::Retry { member, gate, .. } => {
            Some((member, policy::Awaited::Run { member, gate }))
        }
        policy::Decision::Post { member, gate, .. } => Some((member, policy::Awaited::Post { member, gate })),
        policy::Decision::Merge { member, .. } => Some((member, policy::Awaited::Merge)),
        policy::Decision::None
        | policy::Decision::Wait { .. }
        | policy::Decision::Prepare { .. }
        | policy::Decision::Landed { .. }
        | policy::Decision::Leave { .. }
        | policy::Decision::Judge { .. }
        | policy::Decision::Resolve { .. }
        | policy::Decision::Key { .. }
        | policy::Decision::Repair { .. }
        | policy::Decision::Proved
        | policy::Decision::Stale
        | policy::Decision::Finish
        | policy::Decision::Cancel
        | policy::Decision::End(_)
        | policy::Decision::Hold(_) => None,
    }
}

fn leave_room(d: &mut Domain, row: &mut LandingRow) {
    row.room = None;
    crate::room::leave(d, crate::room::Waiter::Landing(row.landing.task));
}

/// Before keeping any policy progress, find whether this turn's write would
/// fit both of its branch records. The next turn decides the write afresh.
fn blocked_write(
    d: &Domain,
    limits: &Limits,
    landing: &policy::Landing,
    member: u16,
    on: policy::Awaited,
) -> Option<crate::Name> {
    use crate::domain::writes;
    let selected = match on {
        policy::Awaited::Open => round_top_in(landing)?,
        policy::Awaited::Close => round_effect_top_in(landing),
        policy::Awaited::Merge | policy::Awaited::Record { .. } | policy::Awaited::Delete { .. } => member,
        policy::Awaited::Release
        | policy::Awaited::Ready { .. }
        | policy::Awaited::Turn
        | policy::Awaited::Prepare
        | policy::Awaited::Prefixes
        | policy::Awaited::Post { .. }
        | policy::Awaited::Run { .. }
        | policy::Awaited::Receipt { .. }
        | policy::Awaited::Judgement { .. }
        | policy::Awaited::Ancestry { .. }
        | policy::Awaited::Read => return None,
    };
    let member = landing.members.get(usize::from(selected)).expect("member for write");
    let change = d.changes.get(&member.change);
    let base = match (on, change) {
        (policy::Awaited::Merge, Some(change)) => Some(change.base.as_ref()),
        _ => None,
    };
    let path = match (on, change) {
        (policy::Awaited::Record { .. }, Some(change)) => change.branch.as_ref(),
        (policy::Awaited::Merge, Some(change)) if landing.round.branch.is_empty() => change.branch.as_ref(),
        _ => member.branch.as_ref(),
    };
    for path in [Some(path), base].into_iter().flatten() {
        let name = writes::branch(limits, repository(member), path)?;
        if writes::full(d, &name) {
            return Some(name);
        }
    }
    None
}

fn admit_entry(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    row: &LandingRow,
    waiting: (u16, policy::Awaited, Option<crate::Name>),
    out: &mut Queue<Request>,
) -> bool {
    let (member, on, blocked) = waiting;
    if d.entries.contains_key(&entry) {
        let mut prior = d.landing.rows.get(&task).expect("pending landing").clone();
        if prior.room.is_some() {
            leave_room(d, &mut prior);
            save(d, env, prior, out);
        }
        refuse_entry(task, row, member, out);
        return false;
    }
    let waiter = crate::room::Waiter::Landing(task);
    if crate::room::check(d, &env.limits, Some(waiter)) && blocked.is_none() {
        return true;
    }
    crate::room::block_write(d, waiter, blocked);
    let mut prior = d.landing.rows.get(&task).expect("pending landing").clone();
    if prior.room.is_none() {
        prior.room = Some(crate::room::next_place(d));
        save(d, env, prior, out);
    }
    d.landing.due.remove(&task);
    emit_decision(
        out,
        Request::LandingDecision {
            task,
            decision: policy::Decision::Wait { until: None, on },
            entry: None,
            evidence: None,
            head_trailers: Box::new([]),
        },
        d,
        env,
    );
    false
}

fn refuse_entry(task: u64, row: &LandingRow, member: u16, out: &mut Queue<Request>) {
    let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
    crate::refusal::read(out, task, b"reserve landing effect", repo, &api::Read::Settings, api::Error::Exists);
}

fn matches_candidate(member: Option<&policy::Member>, candidate: Option<api::Commit>) -> bool {
    match member {
        Some(member) => member.candidate == candidate,
        None => false,
    }
}

fn matches_failed(left: Option<policy::Leave>, gate: u16) -> bool {
    if let Some(policy::Leave::Failed { gate: failed, .. } | policy::Leave::Blamed { gate: failed, .. }) = left {
        gate == failed
    } else {
        false
    }
}

/// A failed step's end waits until the gate's report decides its verdict (domain/landing-proof.md, 2.3–2.4).
fn reported_end(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &ReceiptKey,
    verdict: crate::history::GateVerdict,
    out: &mut Queue<Request>,
) {
    let flight = d.landing.flights.get(key).expect("reported flight").clone();
    let dispatching = d.landing.rows.get(&flight.task).expect("dispatching landing").clone();
    let mut observed = dispatching.clone();
    crate::landing_proofs::run(d, env, &mut observed, &flight, Some(verdict), out);
    if observed.follows != dispatching.follows || observed.entries != dispatching.entries {
        save(d, env, observed, out);
    }
}

fn retry_gate(flight: &mut runs::Follow, try_: u16) {
    if let Some(run) = &flight.run
        && run.conclusion == Some(api::Conclusion::Cancelled)
    {
        runs::retry_run(flight, try_, false);
    } else if let Some(GateProblem::Broke(_)) = flight.gate_problem {
        runs::retry_run(flight, try_, true);
    } else {
        runs::retry(flight, try_);
    }
}

fn gate_hold(d: &Domain, row: &LandingRow, member: u16, gate: u16) -> Option<Box<[u8]>> {
    let m = row.landing.members.get(usize::from(member))?;
    let proof = m.gates.get(usize::from(gate))?;
    let configured = d.train_gates(row, repository(m)).get(usize::from(gate))?;
    let key = gate_key(row, member, gate, &configured.name, proof_key(proof.proof)?);
    let flight = d.landing.faults.get(&(row.landing.task, member, gate)).or(d.landing.flights.get(&key))?;
    let problem = match flight.gate_problem {
        Some(problem) => problem,
        None => match flight.outcome {
            Some(policy::Verdict::Fault(fault)) => GateProblem::Runner { fault, job: None },
            Some(policy::Verdict::Passed | policy::Verdict::Failed | policy::Verdict::Running) | None => return None,
        },
    };
    let candidate = commit_hex(flight.commit);
    let tries = skein_lib::Decimal::of(u64::from(proof.faults));
    let detail = match problem {
        GateProblem::Broke(report) => concat(&[
            b": the check broke at ",
            &candidate,
            match report {
                ReportProblem::Missing => b": its report was missing",
                ReportProblem::Unread => b": its report could not be read",
                ReportProblem::Empty => b": its report names no failed test",
                ReportProblem::NoVerdict => b": it showed no verdict",
            },
        ]),
        GateProblem::Runner { fault, job } => {
            let name = job_words(flight.run.as_ref(), job);
            concat(&[b": the runner's fault at ", &candidate, b": ", &name, runner_words(fault, job.is_some())])
        }
    };
    Some(concat(&[&detail, b"; the change wasn't judged; retries: ", tries.as_bytes()]))
}

fn job_words(run: Option<&api::Run>, job: Option<u64>) -> Box<[u8]> {
    if let Some(run) = run
        && let Some(id) = job
    {
        for job in &run.jobs {
            if job.id == id {
                return concat(&[b"job ", &crate::text::fit(&job.name, 256, b"..."), b" "]);
            }
        }
    }
    Box::new([])
}

fn runner_words(fault: policy::Fault, job: bool) -> &'static [u8] {
    match fault {
        policy::Fault::SetUp => b"failed in set-up",
        policy::Fault::Completion => b"failed in completion",
        policy::Fault::NoStep => {
            if job {
                b"failed with no failed step"
            } else {
                b"the run failed with no failed step"
            }
        }
        policy::Fault::Cancelled => {
            if job {
                b"was cancelled by no one of temper's"
            } else {
                b"the run was interrupted"
            }
        }
        policy::Fault::OtherCommit => b"the run was at another commit",
        policy::Fault::Lost => b"the run could not be found",
        policy::Fault::MissingWorkflow => b"the workflow was missing",
        policy::Fault::Unreported => b"no report was uploaded",
    }
}

fn unjudged_end(d: &mut Domain, env: &Env<Limits>, key: &ReceiptKey, problem: GateProblem, out: &mut Queue<Request>) {
    let flight = d.landing.flights.get_mut(key).expect("unjudged flight");
    flight.gate_problem = Some(problem);
    if let GateProblem::Runner { fault, .. } = problem {
        flight.outcome = Some(policy::Verdict::Fault(fault));
    }
    reported_end(d, env, key, crate::history::GateVerdict::Fault, out);
    retain_fault(d, env, key, out);
}

/// Keep the failed run's job attempts for the policy's retry, never a receipt (domain/landing-proof.md, 2.4).
fn retain_fault(d: &mut Domain, env: &Env<Limits>, key: &ReceiptKey, out: &mut Queue<Request>) {
    let flight = d.landing.flights.get(key).expect("faulting flight");
    let dispatching = d.landing.rows.get(&flight.task).expect("dispatching landing");
    let mut position = None;
    if let Some(crate::purpose::EffectPurpose::Landing { member, gate, preparation, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
        && preparation == dispatching.landing.preparation
        && dispatching.landing.members.get(usize::from(member)).expect("flight member").candidate == Some(flight.commit)
    {
        position = Some((flight.task, member, gate));
    }
    if let Some(position) = position {
        if let Some(flight) = d.landing.flights.remove(key) {
            d.landing.faults.insert(position, flight).expect("bounded faults");
        }
    } else {
        finish_flight(d, env, key, None, out);
    }
    mark_key(d, key, out);
}

fn receipt_report(
    report: &policy::GateReport,
    env: &Env<Limits>,
    answer: Result<api::Answer, api::Error>,
) -> ReceiptReport {
    use temper_engine_domain_forge_tiers::{Limits, NamedFailures, named_report};
    let limits = Limits {
        failing_tests: env.limits.tiers.failing_tests.min(1024),
        test_id_bytes: env.limits.tiers.test_id_bytes.min(4096),
    };
    let parsed = match answer {
        Ok(api::Answer::File { truncated: true, .. }) => Ok(NamedFailures::Many),
        Ok(api::Answer::File { bytes, .. }) => named_report(report.format, &bytes, &limits),
        Ok(_) | Err(_) => return ReceiptReport::Unread,
    };
    match parsed {
        Err(_) => ReceiptReport::Unread,
        Ok(NamedFailures::Many) => ReceiptReport::Many { bound: limits.failing_tests },
        Ok(NamedFailures::Tests(tests)) => {
            let mut kept = List::with_capacity(limits.failing_tests);
            for test in tests {
                let matched = match &report.fuzzy {
                    Some(prefix) => test.name.starts_with(prefix),
                    None => false,
                };
                kept.push(crate::ReceiptTest { key: test.key, matched }).expect("bounded report");
            }
            ReceiptReport::Tests(crate::ReceiptTests {
                tests: kept.into_boxed(),
                matched: report.matched.clone(),
                rest: report.rest.clone(),
            })
        }
    }
}

/// A gate's promised report is read before any logs, which only a refusal keeps.
fn judge_run(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    key: ReceiptKey,
    run: api::Run,
    out: &mut Queue<Request>,
) {
    let mut report = None;
    for gate in d.landing.gates(key.project, key.repository) {
        if key.check == Check::Gate(gate.name.clone()) {
            report.clone_from(&gate.report);
            break;
        }
    }
    let evidence = if let Some(report) = report {
        if let Some(read) = crate::tiers::parts::artifact_read(run.id, report.artifact.clone(), report.format, env) {
            p.stage = Stage::FailedReport { key: key.clone(), run, report };
            ask(d, env, owner, p, key.repository, read, out);
            return;
        }
        ReceiptReport::Unread
    } else {
        ReceiptReport::Unset
    };
    let problem = report_problem(&evidence);
    judge_end(d, env, owner, p, key, GateEnd { run, report: evidence, failures: None }, problem, out);
}

/// Snapshot each failed job before publishing the failed receipt (domain/landing-proof.md, 2.3).
#[expect(clippy::too_many_arguments, reason = "one coherent failed run and its bounded log cursor")]
fn failed_jobs(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    key: ReceiptKey,
    ended: GateEnd,
    next: u32,
    words: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let run = &ended.run;
    let mut log = List::with_capacity(8192);
    append_words(&mut log, &words);
    let row = d.landing.rows.get(&p.task).expect("failed landing");
    let base = failed_run_base(d, row, &key, run).expect("run base");
    if next == 0 {
        let name = match &key.check {
            Check::Gate(name) | Check::Tier(name) => name,
        };
        for part in [
            b"Gate: ".as_slice(),
            name,
            b"\nRun: ",
            &run.url,
            b"\nCandidate: ",
            &commit_hex(run.commit),
            b"\nBase: ",
            &commit_hex(base),
            b"\n",
        ] {
            append_words(&mut log, part);
        }
    }
    let total = u32::try_from(run.jobs.len()).expect("bounded jobs");
    for index in next..total {
        let job = run.jobs.get(skein_lib::usize_of(index)).expect("job");
        if job.conclusion != Some(api::Conclusion::Failure) {
            continue;
        }
        append_words(&mut log, b"Failed job: ");
        append_words(&mut log, &job.name);
        append_words(&mut log, b"\n");
        let attempt = api::JobAttempt { head: run.commit, run: run.id, job: job.id, attempt: job.attempt };
        let share = log.room().checked_div(total.saturating_sub(index).max(1)).expect("positive job count");
        let read = api::Read::Job {
            part: api::LogPart::End,
            attempt,
            max_bytes: share.min(env.limits.client.answer_bytes).max(1),
        };
        p.stage = Stage::FailedJobs {
            key: key.clone(),
            ended,
            next: index.checked_add(1).expect("bounded jobs"),
            words: log.into_boxed(),
        };
        ask(d, env, owner, p, key.repository, read, out);
        return;
    }
    failed_receipt(d, env, owner, p, key, ended, log.into_boxed(), out);
}

fn report_problem(report: &ReceiptReport) -> ReportProblem {
    match report {
        ReceiptReport::Tests(_) => ReportProblem::Empty,
        ReceiptReport::Unread => ReportProblem::Unread,
        ReceiptReport::Unset | ReceiptReport::Many { .. } => ReportProblem::NoVerdict,
    }
}

/// A promised report binds, while a gate without one can refuse by its own failed step.
#[expect(clippy::too_many_arguments, reason = "a gate end carries the report result beside its run")]
fn judge_end(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    key: ReceiptKey,
    ended: GateEnd,
    problem: ReportProblem,
    out: &mut Queue<Request>,
) {
    let flight = d.landing.flights.get(&key).expect("ended flight");
    let (workflow_failed, job_fault) = runs::gate_jobs(&ended.run);
    let shown = match &ended.report {
        ReceiptReport::Unset if workflow_failed => policy::Shown::Failures,
        ReceiptReport::Many { .. } => policy::Shown::Failures,
        ReceiptReport::Tests(tests) if !tests.tests.is_empty() => policy::Shown::Failures,
        ReceiptReport::Unset | ReceiptReport::Unread | ReceiptReport::Tests(_) => policy::Shown::Nothing,
    };
    let (runner, job) = if let Some((fault, job)) = job_fault {
        (Some(fault), Some(job))
    } else {
        match flight.outcome {
            Some(policy::Verdict::Fault(fault)) => (Some(fault), None),
            Some(policy::Verdict::Failed | policy::Verdict::Passed | policy::Verdict::Running) | None => (None, None),
        }
    };
    let unjudged = match policy::judge(shown, runner) {
        policy::Judgement::Refused => {
            failed_jobs(d, env, owner, p, key, ended, 0, Box::new([]), out);
            return;
        }
        policy::Judgement::Runner(fault) => GateProblem::Runner { fault, job },
        policy::Judgement::Broke => GateProblem::Broke(problem),
        policy::Judgement::Passed => unreachable!("only runs without a pass need failure evidence"),
    };
    unjudged_end(d, env, &key, unjudged, out);
    p.polled.push(key).expect("one poll per flight");
    p.stage = Stage::Collect;
    collect(d, env, owner, p, out);
}

/// Only a refusal reaches here; the receipt and its repair carry the log snapshot.
#[expect(clippy::too_many_arguments, reason = "the failed run and its bounded log snapshot complete one landing read")]
fn failed_receipt(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    key: ReceiptKey,
    ended: GateEnd,
    words: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let GateEnd { run, report, failures } = ended;
    let flight = d.landing.flights.get(&key).expect("failed flight");
    let no_verdict = match tests_value(&flight.inputs) {
        Some(selected) => !selected_report(selected, failures.as_ref()),
        None => false,
    };
    reported_end(d, env, &key, crate::history::GateVerdict::Failed, out);
    let mut row = d.landing.rows.get(&p.task).expect("failed landing").clone();
    let base = failed_run_base(d, &row, &key, &run).expect("run base");
    row.landing.round.failure =
        crate::preparation::remember(&row.landing.round.failure, &words, env.limits.landing_members);
    save(d, env, row, out);
    p.stage = Stage::Collect;
    if !no_verdict && start_flake_rerun(d, env, &key, &run, &report, out) {
        p.polled.push(key).expect("one poll per flight");
        collect(d, env, owner, p, out);
        return;
    }
    let number = run.number;
    let receipt = Receipt {
        report,
        base: Some(base),
        jobs: receipt_jobs(&run.jobs),
        logs: words,
        failures,
        passed: false,
        candidate: run.commit,
        ended: env.wall,
        run: ReceiptRun { id: run.id, number: run.number, url: run.url },
    };
    if !no_verdict {
        put_gate_receipt(d, env, &key, receipt, out);
    }
    finish_flight(d, env, &key, if no_verdict { Some((number, false)) } else { None }, out);
    collect(d, env, owner, p, out);
}

pub(super) fn receipt_jobs(run_jobs: &[api::RunJob]) -> Box<[ReceiptJob]> {
    let mut jobs = List::with_capacity(16);
    for job in run_jobs {
        if job.conclusion == Some(api::Conclusion::Failure) {
            let name = job.name.get(..job.name.len().min(256)).expect("bounded name").into();
            if jobs.push(ReceiptJob { job: job.id, attempt: job.attempt, name }).is_err() {
                break;
            }
        }
    }
    jobs.into_boxed()
}

fn entry_base(row: &LandingRow, entry: u64) -> Option<api::Commit> {
    row.run_entry(entry)?.base
}

fn failed_run_base(d: &Domain, row: &LandingRow, key: &ReceiptKey, run: &api::Run) -> Option<api::Commit> {
    let mut base = None;
    if let Some(flight) = d.landing.flights.get(key) {
        base = input_commit(&flight.inputs, b"temper_base");
        base = entry_base(row, flight.entry).or(base);
    }
    if base.is_none() {
        for member in &row.landing.members {
            if member.candidate == Some(run.commit) {
                base = member.base;
            }
        }
    }
    if base.is_none() {
        base = row.landing.members.first().expect("round member").base;
    }
    base
}

fn failed_candidate(d: &Domain, row: &LandingRow, member: u16, gate: u16) -> api::Commit {
    if !row.landing.round.branch.is_empty() {
        return row
            .landing
            .members
            .get(usize::from(round_effect_top(row)))
            .expect("failed top")
            .candidate
            .expect("failed prefix");
    }
    let m = row.landing.members.get(usize::from(member)).expect("member");
    if let Some(key) = proof_key(m.gates.get(usize::from(gate)).expect("gate").proof) {
        let configured = d.train_gates(row, repository(m)).get(usize::from(gate)).expect("configured gate");
        let key = gate_key(row, member, gate, &configured.name, key);
        if let Some(receipt) = d.receipt(&key) {
            return receipt.candidate;
        }
    }
    m.candidate.expect("candidate")
}

fn matches_stall(stage: &Stage) -> bool {
    if let Stage::StallRun | Stage::StallJobs = stage {
        return true;
    }
    false
}

pub(crate) fn release_repository(
    d: &mut Domain,
    repository: api::Repository,
    out: &mut Queue<Request>,
    groups: &mut Queue<Box<[Key]>>,
) {
    let mut tasks = List::with_capacity(d.landing.rows.len());
    for (task, row) in &d.landing.rows {
        for member in &row.landing.members {
            if member.repository.forge == repository.forge && member.repository.repository == repository.repository {
                tasks.push(*task).expect("bounded landings");
                break;
            }
        }
    }
    for task in &tasks {
        crate::room::leave(d, crate::room::Waiter::Landing(*task));
        let row = d.landing.erase(*task).expect("selected repository landing");
        members::end(d, &row, out);
        d.landing.due.remove(task);
        read_wait::clear(d, *task);
        d.landing.decisions.remove(task);
        groups.push(Box::new([Key::Landing(*task)]));
        out.push(Request::Marked { tasks: Box::new([*task]) });
    }
    let mut keys = List::with_capacity(d.landing.pending.len());
    for (key, row) in &d.landing.pending {
        if row.repository == repository || tasks.as_slice().contains(&row.task) {
            keys.push(*key).expect("bounded landing.pending");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.pending.remove(&key).expect("collected landing.pending");
        d.read_line.leave(Waiter::Landing(key));
    }
    let mut keys = List::with_capacity(d.landing.keys.len());
    for (key, _row) in &d.landing.keys {
        if tasks.as_slice().contains(&key.0) {
            keys.push(*key).expect("bounded landing.keys");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.keys.remove(&key).expect("collected landing.keys");
    }
    let mut keys = List::with_capacity(d.landing.results.len());
    for (key, _row) in &d.landing.results {
        if tasks.as_slice().contains(&key.0) {
            keys.push(*key).expect("bounded landing.results");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.results.remove(&key).expect("collected landing.results");
    }
    let mut keys = List::with_capacity(d.landing.faults.len());
    for (key, _row) in &d.landing.faults {
        if tasks.as_slice().contains(&key.0) {
            keys.push(*key).expect("bounded landing.faults");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.faults.remove(&key).expect("collected landing.faults");
    }
    let mut keys = List::with_capacity(d.landing.main_keys.len());
    for (key, _row) in &d.landing.main_keys {
        if key.0 == repository {
            keys.push(key.clone()).expect("bounded landing.main_keys");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.main_keys.remove(&key).expect("collected landing.main_keys");
    }
    let mut keys = List::with_capacity(d.landing.flights.len());
    for (key, _row) in &d.landing.flights {
        if key.repository == repository {
            keys.push(key.clone()).expect("bounded landing.flights");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.flights.remove(&key).expect("collected landing.flights");
    }
    let mut projects = List::with_capacity(d.landing.projects.len());
    for (project, (_, repositories, _, _, _, _, _)) in &d.landing.projects {
        let mut kept = List::with_capacity(u32::try_from(repositories.len()).expect("bounded gate repositories"));
        for row in repositories {
            if row.repository != repository {
                kept.push(row.clone()).expect("fewer gate repositories");
            }
        }
        projects.push((*project, kept.into_boxed())).expect("bounded landing projects");
    }
    for (project, repositories) in projects.into_boxed() {
        d.landing.projects.get_mut(&project).expect("retained project").1 = repositories;
    }
}

pub(crate) fn erase_project_receipts(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut keys = List::with_capacity(d.landing.receipts.len());
    for (key, _) in &d.landing.receipts {
        if key.project == project {
            keys.push(key.clone()).expect("bounded receipts");
        }
    }
    for key in keys.into_boxed() {
        d.landing.receipts.remove(&key);
        d.landing.flights.remove(&key);
        out.push(Request::Erase { key: Key::Receipt(key) });
    }
}

pub(crate) fn erase_repository_receipts(d: &mut Domain, repository: api::Repository, groups: &mut Queue<Box<[Key]>>) {
    let mut keys = List::with_capacity(d.landing.receipts.len());
    for (key, _row) in &d.landing.receipts {
        if key.repository == repository {
            keys.push(key.clone()).expect("bounded landing.receipts");
        }
    }
    for key in keys.into_boxed() {
        let _row = d.landing.receipts.remove(&key).expect("collected landing.receipts");
        groups.push(Box::new([Key::Receipt(key)]));
    }
}

/// Keep the wait of the decision carried out by the top (domain/landing-waits.md, 4).
fn emit_decision(out: &mut Queue<Request>, request: Request, d: &mut Domain, env: &Env<Limits>) {
    if let Request::LandingDecision { task, decision, .. } = &request {
        if let policy::Decision::Wait { on: on @ policy::Awaited::Run { .. }, .. } = decision
            && let Some(row) = d.landing.rows.get(task).cloned()
        {
            read_wait::policy_wait(d, env, &row, *on, false);
        }
        read_wait::decided(d, env, *task, *decision, out);
        let waiting = if let policy::Decision::Wait { on, .. } = decision { Some(*on) } else { None };
        if let Some(mut row) = d.landing.rows.get(task).cloned()
            && row.waiting != waiting
        {
            row.waiting = waiting;
            save(d, env, row, out);
        }
    }
    emit(out, request);
}
