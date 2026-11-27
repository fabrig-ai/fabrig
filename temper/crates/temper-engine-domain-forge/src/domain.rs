//! Top-level forge routing and commit fence (jig's domain/connectors.md, sections 2–3, and domain/connectors-effects.md, section 1, and domain/connectors-facts.md, section 1
//! ). Durable rows are held here and mirrored by root store
//! writes.
//! A requester's confirmed branch moves become its change's new head
//! (temper/docs/design/domain/forge/change/drift.md, 1).
mod taking;
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Map, Queue, Set, Token};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;
pub use temper_engine_domain_forge_vocabulary::domain::*;

use crate::{
    BranchHead, ChangeRow, CiState, Class, Event, Hold, IssueRow, Key, Kinds, Limits, Name, News, PullState,
    RecordKind, ReleaseEnding, ReleaseRow, Repository, Request, RestartStage, Role, Stored, Subscriber, Topic, What,
    Writer,
};
use crate::{adoption, brief, held, topics};

mod admission;
#[path = "catching_up.rs"]
mod catching_up;
mod change_reads;
mod ci_news;
pub(crate) mod connection_wait;
#[cfg(test)]
mod frozen;
#[path = "full.rs"]
pub(crate) mod full;
#[path = "goal_branch.rs"]
pub(crate) mod goal_branch;
pub(crate) mod heads;
mod hints;
mod inspection;
mod landed;
#[cfg(test)]
mod limit_refusals;
#[path = "make.rs"]
pub(crate) mod make;
#[path = "outage.rs"]
pub(crate) mod outage;
pub(crate) mod read_line;
pub(crate) mod read_session;
mod rebase;
#[path = "reds.rs"]
pub(crate) mod reds;
pub(crate) mod release;
mod release_failure;
mod release_reads;
pub(crate) mod repository_reads;
mod restart_wait;
#[path = "restore.rs"]
mod restore;
mod room;
pub use restore::{RestoreFit, RestoreSet, StartLink};
#[path = "design.rs"]
mod design;
pub(crate) mod writes;
use design::{design_claim_permitted, goal_run, unpublished_branch};

/// One comparison in flight for a branch's observed tip move.
#[derive(Hash, Debug)]
pub(crate) struct PendingLanding {
    topic: Topic,
    before: skein_forge::api::Commit,
    after: skein_forge::api::Commit,
}
#[derive(Hash, Debug)]
pub(crate) struct PendingLost {
    task: u64,
    attempt: u64,
    name: Name,
    state: LostReadState,
    last_answer: Option<skein_forge::api::Error>,
    kind: LostWriterKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum LostWriterKind {
    Retired,
    Slot,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum LostReadState {
    Restoring,
    Asked,
    WaitingForRoom,
    WaitingForConnection,
    WaitingForRetry,
    WaitingForOutbox,
}
/// One fresh verdict read shared by commit news and live CI subscribers.
#[derive(Hash, Debug)]
pub(crate) struct PendingCi {
    repository: skein_forge::api::Repository,
    head: skein_forge::api::Commit,
    refresh: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum StepStage {
    Taking,
    Branch,
    Base,
    CloseLanding,
    Pull,
    Ci,
    ExecutionRun(u64),
    StallChecks,
    StallJobs,
    BaseCi,
    Run(u64),
    Compare,
    ContainsHead,
    CatchUp,
    MergeHooks(skein_forge::api::Commit),
    Reviews(u32),
    FollowCheck(crate::follow_check::ReadKind),
    Paths,
}

/// One sequence of fresh reads before a level-triggered change step.
#[expect(clippy::struct_excessive_bools, reason = "independent facts and progress of one fresh-read chain")]
#[derive(Hash, Debug)]
pub(crate) struct PendingStep {
    task: u64,
    entry: u64,
    follow_tree: List<temper_engine_domain_forge_landing::Entry>,
    stage: StepStage,
    heard: change::Heard,
    gates: Box<[change::GateReport]>,
    queue_repair_active: bool,
    kept: bool,
    merge_job: bool,
    pending: Box<[change::Pending]>,
    branch: Option<skein_forge::api::Commit>,
    branch_fresh: bool,
    drift: Option<change_reads::Drift>,
    behind: Option<crate::ReadBehind>,
    base_tip: Option<skein_forge::api::Commit>,
    base_read: Option<skein_forge::api::Commit>,
    close_landing: Option<skein_forge::api::Commit>,
    pull: Option<skein_forge::api::Pull>,
    pull_ci: bool,
    ci: change::Status,
    execution: Option<crate::execution::Gather>,
    failed: crate::change_ci::Failed,
    stall: Option<crate::stall::Report>,
    base_ci: change::Status,
    contains_base: change::Status,
    contains_head: change::Status,
    contains_catch_up: change::Status,
    reviews: Box<[skein_forge::api::Review]>,
    reviews_complete: bool,
    paths: Box<[Box<[u8]>]>,
    paths_whole: bool,
    woken: bool,
    retry_at: Option<skein_lib::Time>,
    read: Option<(skein_forge::api::Repository, skein_forge::api::Read)>,
    progress: List<(change::Change, change::Change)>,
}
/// Comparison at its head and the first time the last judged reviews were kept.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ReviewHead {
    head: Option<skein_forge::api::Commit>,
    touched: crate::Touched,
    since: Option<skein_lib::Wall>,
}

/// Volatile branch reads of an unpushed change (keeping-up.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct BranchRead {
    at: Option<skein_lib::Time>,
    due: bool,
    present: bool,
    producer: change::Status,
}

impl BranchRead {
    pub(crate) const NEW: Self = Self { at: None, due: false, present: false, producer: change::Status::Unknown };
}

/// Keep an offered step while it waits for restart observations (forge/catching-up.md, 4).
#[derive(Hash, Debug)]
enum FreshStep {
    Gather(Box<PendingStep>),
    Verified { merge: skein_forge::api::Commit, heard: Box<change::Heard> },
}

struct StepInput {
    task: u64,
    entry: u64,
    heard: change::Heard,
    gates: Box<[change::GateReport]>,
    queue_repair_active: bool,
    kept: bool,
    merge_job: bool,
    pending: Box<[change::Pending]>,
}

/// One committed outbox entry. A held one is out of the client's outbox and
/// keeps its room until a person makes it again or it is settled not made
/// (domain/forge/effects.md, 1; jig's `domain/connectors-effects.md`, 1.3).
#[derive(Hash, Debug)]
pub(crate) struct Outboxed {
    pub(crate) entry: skein_forge_client::Entry,
    pub(crate) owner_ended: Option<skein_lib::Wall>,
    pub(crate) held: bool,
    handoff: Handoff,
}

impl Outboxed {
    fn check_owner(&self, number: u64, task: u64) {
        assert!(
            self.entry.number == number && self.entry.task == task,
            "outbox entry {number} belongs to task {task}, found entry {} of task {}",
            self.entry.number,
            self.entry.task
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Handoff {
    Uncommitted,
    /// Restored entries enter the client first at `CatchUp`.
    Restored,
    Ready,
    Handed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum RestorePhase {
    Cold,
    Restoring,
    Active,
}

/// Volatile connection and the last failure's step.
pub(crate) type ConnectionState = (crate::Connection, Option<crate::FailedStep>);

type WaitingClaim = (u64, Box<[Name]>, Box<[Name]>, Box<[u64]>);

/// The bounded forge connector subtree.
/// Jig's live release authority, projected before a change reads or takes a result.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskHold {
    Change { revision: u64 },
    Blocked,
}

#[derive(Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the volatile full-outbox mark is independent of connector phases")]
pub struct Domain {
    retire_pending: bool,
    pub(crate) releasing: release::State,
    watch_due: Option<skein_lib::Time>,
    pub(crate) machine: Option<crate::Machine>,
    pub(crate) records: Map<u16, crate::ForgeRecord>,
    pub(crate) forge_number: u16,
    pub(crate) configured: Map<u16, crate::Configured>,
    /// The connector was built with `--box`; a closed pipe does not change it
    /// (temper/docs/design/domain/forge/adoption-person.md, 1).
    pub(crate) built_under_box: bool,
    pub(crate) under_box: bool,
    pub(crate) owner: Option<u64>,
    pub(crate) playground_config: Option<crate::PlaygroundConfig>,
    pub(crate) playground: Option<crate::Playground>,
    pub(crate) playground_pending: Map<Token, Token>,
    pub(crate) playground_ends: Map<Token, Token>,
    pub(crate) playground_answers: Map<Token, crate::Answered>,
    pub(crate) playground_due: Option<skein_lib::Time>,
    pub(crate) playground_request: Option<Token>,
    bindings: Map<Box<[u8]>, Box<[u8]>>,
    pub(crate) making_acts: Set<crate::MakingKey>,
    pub(crate) repository_makings: Map<crate::MakingKey, crate::RepositoryMaking>,
    pub(crate) milestones: Map<u64, crate::MilestoneRow>,
    pub(crate) person_briefs: Map<(u64, u32), crate::PersonBrief>,
    pub(crate) project_facts: Map<u32, crate::ProjectFacts>,
    pub(crate) history_reads: crate::history::read::State,
    pub(crate) history_sweep: crate::history::sweep::State,
    pub(crate) history_projects: Map<u32, crate::history::Project>,
    pub(crate) devices: Option<Box<crate::devices::State>>,
    pub(crate) drafts: Option<Box<crate::drafts::State>>,
    pub(crate) aways: Map<(u64, u32), crate::Away>,
    pub(crate) decisions: Map<(u32, u32), crate::Decided>,
    pub(crate) findings: Option<Box<Map<(u32, u32), crate::Finding>>>,
    pub(crate) repairs_asked: Set<u64>,
    pub(crate) reds: Map<(u64, Box<[u8]>), crate::Red>,
    pub(crate) red_number: Option<u64>,
    pub(crate) red_passes: Map<(u64, Box<[u8]>), crate::RedPass>,
    pub(crate) looked: Map<(u64, u32), crate::Looked>,
    pub(crate) dismissed: Map<(u64, crate::PersonFact), ()>,
    pub(crate) connections: Map<u16, ConnectionState>,
    /// One setup runs at a time (temper/docs/design/protocol/forge/engine.md, 8).
    pub(crate) fetched: Option<(u16, crate::SetUpFetched)>,
    pub(crate) first: Map<u16, crate::First>,
    pub(crate) ended: Map<u16, (Box<[u8]>, Option<crate::AdminStop>)>,
    pub(crate) apps: Map<u16, crate::AppWay>,
    setup_answers: Map<Token, (skein_lib::Wall, crate::SetUpAnswer)>,
    setup_pending: Map<Token, (Token, Duration)>,
    pub(crate) choosing: Map<Token, make::Choosing>,
    pub(crate) setup_people: Map<Token, Box<[u8]>>,
    pub(crate) owned_people: Map<u16, Box<[u8]>>,
    pub(crate) connect: crate::connect::State,
    pub(crate) installations: Map<(u16, u64), crate::Installation>,
    pub(crate) repositories: Map<skein_forge::api::Repository, Box<Repository>>,
    pub(crate) names: Map<u64, Box<[Name]>>,
    pub(crate) holds: Map<Name, Hold>,
    pub(crate) subscriptions: Map<(u64, Topic), Subscriber>,
    pub(crate) goal_files: Map<u64, topics::Files>,
    pub(crate) goal_file_reads: Map<Token, topics::Pending>,
    pub(crate) heads: Map<Name, BranchHead>,
    pub(crate) writes: Map<Name, crate::writes::Writes>,
    write_reads: writes::readback::State,
    // The tip before a queued advance, or None for a watch refresh.
    write_heads: Map<Name, Option<skein_forge::api::Commit>>,
    kept_moves: Map<Name, writes::kept::Movement>,
    pub(crate) pulls: Map<Name, PullState>,
    /// Each pull's last mergeable, unsaved: a restart reads it afresh.
    pull_mergeable: Map<Name, bool>,
    /// Last judged reviews per live change, unsaved: a restart reads them afresh.
    pub(crate) review_facts: Map<u64, crate::ReviewFacts>,
    /// Comparison and review time per live change, unsaved and bounded like review facts.
    pub(crate) review_heads: Map<u64, ReviewHead>,
    ci: Map<(skein_forge::api::Repository, skein_forge::api::Commit), CiState>,
    pub(crate) entries: Map<u64, Outboxed>,
    pub(crate) flights: Map<(u64, u64), ()>,
    named_calls: Map<crate::NamedCall, crate::CallPayload>,
    proposed_effects: Map<u64, crate::ProposedEffect>,
    pub(crate) landed: Map<skein_forge::api::Commit, u64>,
    landings: Map<Token, PendingLanding>,
    lost: Map<Token, PendingLost>,
    pending_ci: Map<Token, PendingCi>,
    ci_news: Map<(u64, skein_forge::api::Commit), ci_news::Watch>,
    restart_fresh: Option<bool>,
    restart_reads_started: bool,
    fresh_repositories: Set<skein_forge::api::Repository>,
    restart_settling: bool,
    catching_up_report: Option<crate::Report>,
    pub(crate) catching_up_seen: Map<u16, crate::CatchingUp>,
    /// Claims received while the restart reads are pending.
    waiting_claims: Map<u64, WaitingClaim>,
    projection_waits: Set<u64>,
    release_waits: Map<u64, Box<[skein_forge::api::Repository]>>,
    release_reads: Map<u64, release_reads::Clock>,
    waiting_release_outcomes: Map<u64, (skein_forge_client::Outcome, Option<crate::EffectAccount>)>,
    /// Restored procedures and entries awaiting their restart observations (forge/catching-up.md, 4).
    fresh_waits: Map<u64, Box<[skein_forge::api::Repository]>>,
    fresh_steps: Map<u64, FreshStep>,
    entry_looks: Set<u64>,
    pub(crate) fresh_branches: Set<Name>,
    pub(crate) sequence: u64,
    pub(crate) closing: crate::closer::State,
    pub(crate) changes: Map<u64, ChangeRow>,
    task_holds: Map<u64, TaskHold>,
    /// The cold part last put, also held while restart collects the two parts.
    pub(crate) change_cold: Map<u64, crate::ChangeCold>,
    /// Progress parts awaiting the restore fit barrier.
    pub(crate) change_hot: Map<u64, crate::ChangeHot>,
    pub(crate) steps: Map<Token, PendingStep>,
    /// Last-step queue merges withheld by the operator
    /// (temper/docs/design/testing/engine-checks.md, 2.2).
    merge_withheld: Set<u64>,
    pub(crate) stall_words: Map<u64, crate::stall::Report>,
    pub(crate) read_sessions: Map<u64, read_session::Session>,
    merge_hooks: Map<u64, (skein_forge::api::Commit, crate::Hooks)>,
    // Volatile, bounded by changes (temper/docs/design/domain/forge/adoption-connection.md, 1).
    change_connections: Option<Box<Map<u64, skein_lib::Wall>>>,
    start_conflicts: Map<u64, skein_forge::api::Commit>,
    // Volatile input for the next change step, never restored (domain/forge/change/gates.md, 1).
    review_requests: Map<u64, skein_forge::api::Commit>,
    pub(crate) makings: Map<Token, goal_branch::Making>,
    pub(crate) room: crate::room::State,
    pub(crate) wait_reports: crate::waits::State,
    pub(crate) full: full::State,
    pub(crate) panel_ci: Map<u64, change::Ci>,
    pub(crate) issues: Map<u64, IssueRow>,
    pub(crate) releases: Map<u64, ReleaseRow>,
    pub(crate) kept: Map<u64, Box<[Name]>>,
    pub(crate) namespace: Box<[u8]>,
    pub(crate) adoptions: Map<Token, adoption::PendingAdoption>,
    pub(crate) application: adoption::State,
    pub(crate) writers: Map<u16, u64>,
    /// Each forge's git identity, the application credential its runs push
    /// and fetch with (protocol/credentials.md).
    pub(crate) identities: Map<u16, u32>,
    pub(crate) workflows: crate::workflows::State,
    pub(crate) capabilities: Map<Token, crate::capabilities::Pending>,
    pub(crate) stopping: bool,
    pub(crate) landing: crate::landing::State,
    pub(crate) follow_retry: Map<(u32, u32), skein_lib::Time>,
    pub(crate) follow_queue: u32,
    pub(crate) follow_merge: u32,
    pub(crate) follow_reason_bytes: u32,
    pub(crate) follow_rows: Map<(u32, u32), crate::FollowRow>,
    pub(crate) follow_reads: Map<Token, crate::follows::Pending>,
    pub(crate) tier_ops: crate::tiers::State,
    pub(crate) tiers: Map<(skein_forge::api::Repository, Box<[u8]>), crate::TierRow>,
    pub(crate) refused_settings: Map<crate::settings::SettingKey, ()>,
    pub(crate) setting_rows: Map<crate::settings::SettingKey, crate::settings::Edit>,
    pub(crate) setting_stock: Option<Box<[crate::settings::Stock]>>,
    pub(crate) held_rules: crate::settings::HeldRules,
    pub(crate) held_charters: crate::ForgeCharters,
    pub(crate) landing_logs: Map<skein_forge::api::Repository, crate::LandingLog>,
    pub(crate) verifications: Map<(skein_forge::api::Repository, skein_forge::api::Commit), crate::Verification>,
    /// Restored timers stay passive until the scripted fresh-read stage (domain/root-restart.md, 1.1).
    restore_phase: RestorePhase,
    pub(crate) reconciliation: crate::reconcile::Reconciliation,
    pub(crate) restore_started: Option<skein_lib::Wall>,
    restore_fit: Option<Box<RestoreFit>>,
    outage: Option<Box<outage::State>>,
    pub(crate) client: skein_forge_client::Domain,
    client_out: Queue<skein_forge_client::Request>,
    // A refused brief read waits for the next client reclamation.
    pub(crate) read_line: read_line::Line,
    pub(crate) brief_fetches: Map<Token, brief::BriefFetch>,
    pub(crate) brief_planned: Map<Token, held::Planned>,
    pub(crate) brief_documents: Map<Token, Box<crate::BriefDocument>>,
    pub(crate) document_fetches: Map<Token, crate::brief_document::Fetch>,
    pub(crate) brief_pending: Map<Token, held::Pending>,
    pub(crate) brief_held: Map<Token, held::Held>,
    merge_kind: Option<(u32, u32)>,
    close_kind: Option<(u32, u32)>,
    projection_due: Map<u64, skein_lib::Wall>,
    pub(crate) call_room: Set<Token>,
    pub(crate) projection_room: Set<u64>,
    /// Volatile: a projection met the full outbox since the root's step began
    /// (temper/docs/design/domain/forge/issues.md). Never saved; false after a restore.
    full_outbox_met: bool,
    change_due: Map<u64, skein_lib::Wall>,
    /// True wakes the procedure too; false steps only its pending dependents.
    pub(crate) procedure_wakes: Map<u64, bool>,
    pub(crate) settling: Map<u64, crate::settle::Settle>,
}

impl Domain {
    pub(crate) fn poll_at(&self, env: &Env<Limits>, repository: skein_forge::api::Repository) -> skein_lib::Time {
        self.client.poll_at(&env.limits.client, env.now, repository, env.limits.client.poll)
    }
    /// The party's rows in key order (temper/docs/design/domain/forge/attention.md, 1).
    #[must_use]
    pub fn dismissals(&self, party: u64) -> Box<[crate::PersonFact]> {
        let mut rows = List::with_capacity(self.dismissed.len());
        for ((owner, what), ()) in self.dismissed.range((party, crate::PersonFact::ToBegin { project: 0 })..) {
            if *owner != party {
                break;
            }
            rows.push(*what).expect("party's rows fit the dismissal table");
        }
        rows.into_boxed()
    }

    /// The core's declared merge bounds (temper/docs/design/domain/worker/jobs.md).
    pub fn merge_kind(&mut self, spec: u32, result: u32) {
        self.merge_kind = Some((spec, result));
    }

    /// The core's declared close bounds (temper/docs/design/domain/worker/jobs.md).
    pub fn close_kind(&mut self, spec: u32, result: u32) {
        self.close_kind = Some((spec, result));
    }

    /// Before the first input, size the volatile routing tables by the connection component's bounds.
    /// The product of the connection bounds must fit the installation row count.
    pub fn connection_limits(&mut self, _forges: u32, installations: u32) {
        let forges = self.records.capacity();
        assert!(self.connections.is_empty() && self.installations.is_empty(), "connection limits before inputs");
        assert!(crate::limits::connection_valid(forges, installations), "connection bounds");
        self.connections = Map::with_capacity(forges);
        self.first = Map::with_capacity(forges);
        self.ended = Map::with_capacity(forges);
        self.apps = Map::with_capacity(forges);
        self.installations = Map::with_capacity(forges.checked_mul(installations).expect("connection bounds"));
    }
    /// Public forge configuration, with kept records taking precedence.
    #[must_use]
    pub fn configured(&self, forge: u16) -> Option<&crate::Configured> {
        self.configured.get(&forge)
    }
    /// The admitted requester's login is routing input, not document configuration.
    pub fn setup_person(&mut self, request: Token, login: Box<[u8]>) {
        self.setup_people.insert(request, login).expect("one person per admitted setup");
    }
    /// The machine's person login for an owned forge (domain/agent/agents.md, 1).
    #[must_use]
    pub fn owned_person(&self, forge: u16) -> Option<&[u8]> {
        let config = self.configured(forge)?;
        if config.mode != crate::Mode::Owned {
            return None;
        }
        match self.owned_people.get(&forge) {
            Some(login) if !login.is_empty() => Some(login),
            Some(_) | None => None,
        }
    }
    #[must_use]
    pub fn connection(&self, forge: u16) -> crate::Connection {
        if self.forge_ended(forge) {
            return crate::Connection::NotConnected;
        }
        match self.connections.get(&forge) {
            Some((state, _)) => state.clone(),
            None => crate::Connection::NotConnected,
        }
    }
    pub(crate) fn change_access_lost(&self, row: &ChangeRow) -> bool {
        if !self.repositories.contains_key(&row.repository) {
            return true;
        }
        if let crate::Connection::Connected { .. } = self.connection(row.repository.forge) {
            if connection_wait::contains(self, row.task) {
                return true;
            }
            if let Some(owner) = step_owner(self, row.task) {
                return outage::exempts_change(self, row.task, owner);
            }
            return false;
        }
        true
    }

    pub(crate) fn change_landing_withheld(&self, row: &ChangeRow) -> bool {
        if !self.landing.held {
            return false;
        }
        if self.merge_withheld.contains(&row.task) || crate::landing::waiting(self, row) {
            return true;
        }
        if let Some(entry) = row.pending
            && let Some(entry) = self.entries.get(&entry)
        {
            return held_landing(self, &entry.entry);
        }
        false
    }

    pub(crate) fn change_verification_settings(&self, row: &ChangeRow) -> (Duration, change::Wait) {
        if let Some(repository) = self.repositories.get(&row.repository)
            && let Some(after) = self.after_landing(repository.project)
        {
            return (after.bound, after.wait);
        }
        (Duration::from_secs(7200), change::Wait::Cleared)
    }

    /// Create's service tell; a forge without a setup intent ignores it.
    pub fn first_made(&mut self, forge: u16, id: u64) -> bool {
        if let Some(first) = self.first.get_mut(&forge) {
            first.state = crate::FirstState::Made { id };
            return true;
        }
        false
    }
    /// Create's stopped step is volatile; the first repository's intent is kept.
    pub fn first_failed(&mut self, forge: u16, step: crate::CreateStep) -> bool {
        if let Some(first) = self.first.get_mut(&forge) {
            first.state = crate::FirstState::Failed { step };
            return true;
        }
        false
    }
    #[must_use]
    pub fn forges(&self) -> crate::Forges {
        let mut rows = List::with_capacity(self.configured.len());
        for (forge, config) in &self.configured {
            if self.forge_ended(*forge) {
                continue;
            }
            let (ended, ended_stop) = match self.ended.get(forge) {
                Some((why, stop)) => (Some(why.clone()), *stop),
                None => (None, None),
            };
            let mut makings = List::with_capacity(self.repository_makings.len());
            for row in self.makings(*forge) {
                makings.push(row.clone()).expect("bounded makings");
            }
            rows.push(crate::Row {
                catching_up: self.catching_up(*forge),
                makings: makings.into_boxed(),
                read_only: config.kind == crate::Kind::GitHub && config.read_only,
                number: *forge,
                host: crate::host(&config.git),
                kind: config.kind,
                mode: config.mode,
                connection: self.connection(*forge),
                failed: match self.connections.get(forge) {
                    Some((_, failed)) => *failed,
                    None => None,
                },
                fetched: match self.fetched {
                    Some((number, fetched))
                        if number == *forge
                            && self.connection(*forge)
                                == (crate::Connection::SettingUp { step: Some(crate::SetUpStep::Fetch) }) =>
                    {
                        Some(fetched)
                    }
                    Some(_) | None => None,
                },
                offer: self.offer(*forge),
                first: self.first.get(forge).cloned(),
                starts_with: match match self.record(*forge) {
                    Some(record) => record.first.as_ref(),
                    None => None,
                } {
                    Some(crate::FirstIntent::Copies { addresses }) => {
                        crate::StartsWith::Copies { addresses: addresses.clone() }
                    }
                    Some(crate::FirstIntent::Nothing) => crate::StartsWith::Nothing,
                    Some(crate::FirstIntent::Empty(_)) | None => crate::StartsWith::Repository,
                },
                installations: self.forge_installations(*forge, config.kind),
                ended,
                app: match self.connection(*forge) {
                    crate::Connection::Connected { .. } if config.kind == crate::Kind::GitHub => {
                        self.apps.get(forge).copied()
                    }
                    crate::Connection::Connected { .. }
                    | crate::Connection::NotConnected
                    | crate::Connection::NotSetUp
                    | crate::Connection::SettingUp { .. }
                    | crate::Connection::SetUpFailed { .. }
                    | crate::Connection::Connecting
                    | crate::Connection::Stopped { .. } => None,
                },
                web: config.web.clone(),
                repositories: self.forge_repositories(*forge),
                owned: config.owned.clone(),
                admin: config.admin_connect,
                ended_stop,
            })
            .expect("configured forges");
        }
        crate::Forges { rows: rows.into_boxed(), full: self.full.document() }
    }
    fn forge_installations(&self, forge: u16, kind: crate::Kind) -> Box<[(u64, crate::Installation)]> {
        let mut rows = List::with_capacity(self.installations.len());
        if kind == crate::Kind::GitHub {
            for ((owner, id), installation) in &self.installations {
                if *owner == forge {
                    rows.push((*id, installation.clone())).expect("bounded installations");
                }
            }
        }
        rows.into_boxed()
    }
    fn forge_repositories(&self, forge: u16) -> Box<[crate::AdoptedRow]> {
        let mut rows = List::with_capacity(self.repositories.len());
        for (repository, adopted) in &self.repositories {
            if repository.forge == forge {
                rows.push(crate::AdoptedRow {
                    owner: adopted.owner.clone(),
                    name: adopted.name.clone(),
                    project: adopted.project,
                })
                .expect("bounded adopted repositories");
            }
        }
        rows.into_boxed()
    }
    /// One decision shared by the document and setup admission (domain/forge/adoption-connection.md, 1).
    #[must_use]
    pub fn offer(&self, forge: u16) -> crate::Offer {
        let Some(config) = self.configured.get(&forge) else { return crate::Offer::NotOffered };
        if self.forge_ended(forge) || config.mode != crate::Mode::Owned || !config.asked {
            return crate::Offer::NotOffered;
        }
        match self.connection(forge) {
            crate::Connection::NotSetUp
            | crate::Connection::SetUpFailed { .. }
            | crate::Connection::SettingUp { .. } => {
                if self.under_box {
                    crate::Offer::Offered
                } else {
                    crate::Offer::NotUnderTheBox
                }
            }
            crate::Connection::NotConnected
            | crate::Connection::Connecting
            | crate::Connection::Connected { .. }
            | crate::Connection::Stopped { .. } => crate::Offer::NotOffered,
        }
    }
    fn setup_answer(
        &mut self,
        env: &Env<Limits>,
        request: Token,
        answer: crate::SetUpAnswer,
        out: &mut Queue<Request>,
    ) {
        if let Some((token, retention)) = self.setup_pending.remove(&request) {
            let until = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(retention.as_nanos()));
            self.setup_answers.insert(token, (until, answer)).expect("reserved setup answer");
            out.push(Request::ApplicationDone { request, token });
        }
    }
    /// Volatile setup answer or retained application outcome for this request.
    #[must_use]
    pub fn application_answer(&self, token: Token) -> Option<crate::Answered> {
        if let Some(answer) = self.connect.answers.get(&token) {
            return Some(answer.clone());
        }
        if let Some(answer) = self.playground_answers.get(&token) {
            return Some(answer.clone());
        }
        if let Some((_, answer)) = self.setup_answers.get(&token) {
            return Some(crate::Answered::SetUp(*answer));
        }
        match self.application_outcome(token) {
            Some(crate::ApplicationOutcome::Follow(words)) => Some(crate::Answered::Follow(words.clone())),
            Some(crate::ApplicationOutcome::MakeKept) => Some(crate::Answered::Make(crate::MakeAnswer::Kept)),
            Some(crate::ApplicationOutcome::MakeRefused(why)) => {
                Some(crate::Answered::Make(crate::MakeAnswer::Refused(*why)))
            }
            Some(
                outcome @ (crate::ApplicationOutcome::Opened { .. }
                | crate::ApplicationOutcome::OpenRefused(_)
                | crate::ApplicationOutcome::Adopted(_)
                | crate::ApplicationOutcome::CoreRefused(_)
                | crate::ApplicationOutcome::ForgeRefused(_)),
            ) => Some(crate::Answered::Adopt(outcome.clone())),
            None => None,
        }
    }
    #[must_use]
    pub fn installation(&self, forge: u16, installation: u64) -> Option<&crate::Installation> {
        self.installations.get(&(forge, installation))
    }

    /// Read the landing branch's procedure waits (temper/docs/design/domain/forge/change/gates.md, 2).
    #[must_use]
    pub fn requires(&self, repository: &Repository, branch: &[u8], touched: crate::Touched) -> crate::Requires {
        self.held_requires(repository, branch, touched)
    }

    /// Read the connector's retained part for the exact supplied call identity.
    #[must_use]
    pub fn named_answer(&self, key: crate::NamedCall) -> Option<crate::CallPayload> {
        self.named_calls.get(&key).cloned()
    }

    /// Retain a bounded payload after the caller admits the named decision.
    pub fn keep_named_answer(
        &mut self,
        limits: &Limits,
        key: crate::NamedCall,
        answer: crate::CallPayload,
    ) -> Result<Option<crate::CallPayload>, crate::CallPayload> {
        if key.task == 0 || key.attempt == 0 {
            return Err(answer);
        }
        if restore::named_payload(limits, &answer).is_err() {
            return Err(answer);
        }
        match self.named_calls.insert(key, answer) {
            Ok(previous) => Ok(previous),
            Err((_, answer)) => Err(answer),
        }
    }

    /// Drop the payload when its owner retires the named call.
    pub fn forget_named_answer(&mut self, key: crate::NamedCall) -> Option<crate::CallPayload> {
        self.named_calls.remove(&key)
    }

    /// Identify the named effect whose connector outcome changed.
    #[must_use]
    pub fn named_effect(&self, entry: u64) -> Option<crate::NamedCall> {
        for (key, answer) in &self.named_calls {
            match answer {
                crate::CallPayload::Effect { entry: number, .. } if *number == entry => return Some(*key),
                crate::CallPayload::Effect { .. }
                | crate::CallPayload::Read(_)
                | crate::CallPayload::Refused(_)
                | crate::CallPayload::Follow { .. }
                | crate::CallPayload::Finding(_)
                | crate::CallPayload::Red(_)
                | crate::CallPayload::Decided { .. } => {}
            }
        }
        None
    }

    /// Update the connector's own part when its client reports an effect outcome.
    pub fn named_outcome(&mut self, entry: u64, outcome: skein_forge_client::Outcome) {
        if outcome == skein_forge_client::Outcome::Uncertain || outcome == skein_forge_client::Outcome::Queued {
            return;
        }
        let Some(key) = self.named_effect(entry) else { return };
        let Some(answer) = self.named_calls.get_mut(&key) else { unreachable!("named effect owner") };
        match answer {
            crate::CallPayload::Effect { outcome: kept, .. } => *kept = Some(outcome),
            crate::CallPayload::Read(_)
            | crate::CallPayload::Refused(_)
            | crate::CallPayload::Follow { .. }
            | crate::CallPayload::Finding(_)
            | crate::CallPayload::Red(_)
            | crate::CallPayload::Decided { .. } => {
                unreachable!("effect payload")
            }
        }
    }

    /// Last coverage at the change's current head, unknown until a step compares it.
    #[must_use]
    pub fn change_touched(&self, task: u64) -> crate::Touched {
        match self.changes.get(&task) {
            Some(row) => self.touched_at(row),
            None => crate::Touched::UNKNOWN,
        }
    }

    pub(crate) fn touched_at(&self, row: &ChangeRow) -> crate::Touched {
        match self.review_heads.get(&row.task) {
            Some(kept) if kept.head == row.change.last_head && !row.frozen() => kept.touched,
            Some(_) | None => crate::Touched::UNKNOWN,
        }
    }

    pub(crate) fn review_since(&self, task: u64) -> Option<skein_lib::Wall> {
        match self.review_heads.get(&task) {
            Some(kept) => kept.since,
            None => None,
        }
    }

    pub(crate) fn keep_touched(&mut self, task: u64, head: Option<skein_forge::api::Commit>, touched: crate::Touched) {
        match self.review_heads.get_mut(&task) {
            Some(kept) => {
                kept.head = head;
                kept.touched = touched;
            }
            None => {
                self.review_heads
                    .insert(task, ReviewHead { head, touched, since: None })
                    .expect("one head per bounded live change");
            }
        }
    }

    /// Retain the last judged reviews for a live change without writing its record.
    /// The same facts answer false; facts at the kept head keep their first time,
    /// and facts at another head take `wall` (domain/forge/change/gates.md, 2, and domain/forge/change/questions.md, 2).
    /// Returns whether a watched panel needs a new document.
    pub fn keep_review_facts(&mut self, wall: skein_lib::Wall, task: u64, facts: crate::ReviewFacts) -> bool {
        let Some(row) = self.changes.get(&task) else { return false };
        if row.frozen() || self.review_facts.get(&task) == Some(&facts) {
            return false;
        }
        let same_head = match self.review_facts.get(&task) {
            Some(kept) => kept.head == facts.head,
            None => false,
        };
        if !same_head {
            match self.review_heads.get_mut(&task) {
                Some(kept) => kept.since = Some(wall),
                None => {
                    self.review_heads
                        .insert(
                            task,
                            ReviewHead { head: facts.head, touched: crate::Touched::UNKNOWN, since: Some(wall) },
                        )
                        .expect("one head per bounded live change");
                }
            }
        }
        self.review_facts.insert(task, facts).expect("one review read per bounded live change");
        true
    }

    /// Keep the connector's retry deadline; the root carries no timer policy.
    pub fn wait_change(&mut self, env: &Env<Limits>, task: u64, until: Option<skein_lib::Wall>) {
        if connection_wait::contains(self, task) || self.waiting_for_room(task) {
            self.change_due.remove(&task);
            return;
        }
        let until = match until {
            Some(until) => until,
            None => {
                let unpushed = match self.changes.get(&task) {
                    Some(row) => !hints::has_head(self, row),
                    None => false,
                };
                let interval = if unpushed && env.limits.client.fallback > Duration::ZERO {
                    env.limits.client.fallback
                } else {
                    env.limits.queue_window
                };
                skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(interval.as_nanos()))
            }
        };
        self.change_due.insert(task, until).expect("one bounded change deadline");
    }

    /// A place in the shared outbox line (domain/forge/room.md, 3).
    #[must_use]
    pub fn waiting_for_room(&self, task: u64) -> bool {
        match self.changes.get(&task) {
            Some(row) => row.room.is_some(),
            None => false,
        }
    }

    /// A failed making still holds its resource, and a passed-over making
    /// keeps its outbox place (domain/forge/branches.md, 1; domain/forge/room.md, 3).
    #[must_use]
    pub fn making_retry_candidates(&self) -> Set<u64> {
        let mut tasks = Set::with_capacity(self.holds.len().saturating_add(self.makings.len()));
        for (_, hold) in &self.holds {
            tasks.insert(hold.task).expect("one candidate per hold");
        }
        for (_, making) in &self.makings {
            if making.outbox_wait.is_some() {
                tasks.insert(making.task).expect("one candidate per making");
            }
        }
        tasks
    }

    /// A making keeps its place while its task is held (domain/forge/room.md, 3).
    #[must_use]
    pub fn making_waits(&self, task: u64) -> bool {
        for (_, pending) in &self.makings {
            if pending.task == task && pending.outbox_wait.is_some() {
                return true;
            }
        }
        false
    }

    /// An ask holds room until this procedure can decide afresh.
    #[must_use]
    pub fn room_asked(&self, task: u64) -> bool {
        if crate::room::asked(self, crate::room::Waiter::Change(task))
            || crate::room::asked(self, crate::room::Waiter::Landing(task))
            || crate::room::asked(self, crate::room::Waiter::Release(task))
        {
            return true;
        }
        for (owner, pending) in &self.makings {
            if pending.task == task && crate::room::asked(self, crate::room::Waiter::Making(*owner)) {
                return true;
            }
        }
        false
    }

    /// A new offered procedure step replaces its previous waiting deadline.
    pub fn offered_change(&mut self, task: u64) {
        self.change_due.remove(&task);
    }

    /// The issue procedure owns the interval reported by its policy child.
    pub fn projection_after(&mut self, goal: u64, when: skein_lib::Wall) {
        self.projection_due.insert(goal, when).expect("one bounded projection deadline");
    }

    /// The core has finished the closing projection handshake.
    pub fn projection_settled(&mut self, goal: u64) {
        self.projection_due.remove(&goal);
    }

    /// Whether a projection met the full outbox since the root's step began
    /// (temper/docs/design/domain/forge/issues.md).
    #[must_use]
    pub fn full_outbox_met(&self) -> bool {
        self.full_outbox_met
    }

    /// The root's step begins: forget what the last one's projections met.
    pub fn begin_step(&mut self) {
        self.full_outbox_met = false;
    }

    /// Free outbox room behind the line (domain/forge/room.md, 2).
    #[must_use]
    pub fn effect_room(&self, limits: &Limits) -> bool {
        crate::room::check(self, limits, None)
    }

    /// Promise a described call's room until its entry or terminal ask (domain/forge/room.md, 5).
    pub fn reserve_call(&mut self, env: &Env<Limits>, owner: Token, task: u64) -> bool {
        assert!(!self.call_room.contains(&owner), "one outbox reservation for call flight {owner:?} of task {task}");
        let limits = &env.limits;
        if !crate::room::check(self, limits, None) {
            return false;
        }
        assert!(
            crate::room::taken(self) < u64::from(self.entries.capacity()),
            "outbox reservation for call flight {owner:?} of task {task} has room within forge.entries"
        );
        let inserted = self.call_room.insert(owner);
        assert!(inserted.is_ok(), "outbox reservation for call flight {owner:?} of task {task} fits its room");
        self.full.reserve(full::Reservation::Call(owner), task, None);
        full::filled(self, env);
        true
    }

    /// Check the forge end of the root's staged call flight (domain/forge/room.md, 2.2).
    pub fn assert_call_reservation(&self, owner: Token, task: u64) {
        assert!(self.call_room.contains(&owner), "call flight {owner:?} of task {task} holds an outbox reservation");
        self.full.assert_owner(full::Reservation::Call(owner), task);
    }

    fn take_call_room(&mut self, owner: Token, task: u64) {
        self.assert_call_reservation(owner, task);
        let removed = self.call_room.remove(&owner);
        assert!(removed, "return the held outbox reservation of call flight {owner:?} for task {task}");
    }

    /// A call ended without an entry; offer its room to the line's first waiter.
    pub fn unreserve_call(&mut self, env: &Env<Limits>, owner: Token, task: u64, out: &mut Queue<Request>) {
        self.take_call_room(owner, task);
        crate::room::ask(self, env, out);
    }

    /// Check the forge end of the root's staged goal projection (domain/forge/room.md, 2.2).
    pub fn assert_projection_reservation(&self, entry: u64, goal: u64) {
        assert!(
            self.projection_room.contains(&entry),
            "projection entry {entry} of goal {goal} holds an outbox reservation"
        );
        self.full.assert_owner(full::Reservation::Projection(entry), goal);
    }

    fn take_projection_room(&mut self, entry: u64, goal: u64) {
        self.assert_projection_reservation(entry, goal);
        let removed = self.projection_room.remove(&entry);
        assert!(removed, "return the held outbox reservation of projection entry {entry} for goal {goal}");
    }

    /// A connector's durable payload, read again before a holder accepts it.
    #[must_use]
    pub fn proposed_effect(&self, number: u64) -> Option<&crate::ProposedEffect> {
        self.proposed_effects.get(&number)
    }

    /// Bind agent and projection keys to the durable deployment ID before
    /// connector records are restored.
    pub fn bind_deployment(&mut self, id: [u8; 16], limits: &Limits) -> bool {
        if !self.client.bind_deployment(id, &limits.client) {
            return false;
        }
        self.namespace = Box::from(id);
        true
    }
    /// Construct a connector with the deployment namespace and any writers
    /// known at start; invalid limits or writer configuration are refused.
    #[expect(clippy::too_many_lines, reason = "the connector's bounded state is initialized together")]
    pub fn new(
        l: &Limits,
        seed: u64,
        config: crate::Config,
        identities: &[crate::GitIdentity],
        forges: &[crate::Configured],
        under_box: bool,
    ) -> Result<Domain, skein_forge::api::Error> {
        if !crate::valid(l)
            || (crate::purpose::EffectPurpose::Release { task: 0, resource: Box::from(b"reason".as_slice()) })
                .key(&config.namespace, l.client.op_bytes)
                .is_none()
        {
            return Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()));
        }
        let (configured, forge_number) = crate::forges::document(l.forges, forges)?;
        let mut credentials = Map::with_capacity(l.forges);
        for identity in identities {
            if credentials.insert(identity.forge, identity.credential).is_err() {
                return Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::at_least(
                    skein_forge::api::Bound::Forge,
                    u64::from(l.forges),
                    u64::from(credentials.len()).saturating_add(1),
                )));
            }
        }
        let mut writers = Map::with_capacity(l.client.repositories);
        for writer in &config.writers {
            if writers.insert(writer.forge, writer.author).is_err() {
                return Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::at_least(
                    skein_forge::api::Bound::Forge,
                    u64::from(l.client.repositories),
                    u64::from(writers.len()).saturating_add(1),
                )));
            }
        }
        Ok(Domain {
            outage: None,
            machine: None,
            retire_pending: false,
            releasing: release::State::new(l),
            watch_due: None,
            records: Map::with_capacity(l.forges),
            forge_number,
            configured,
            built_under_box: under_box,
            under_box,
            owner: None,
            playground_config: None,
            playground: None,
            playground_pending: Map::with_capacity(l.applications),
            playground_ends: Map::with_capacity(l.applications),
            playground_answers: Map::with_capacity(l.applications.saturating_mul(2)),
            playground_due: None,
            playground_request: None,
            bindings: Map::with_capacity(l.bindings),
            repository_makings: Map::with_capacity(l.makings),
            making_acts: Set::with_capacity(l.makings),
            milestones: Map::uncapped(),
            person_briefs: Map::with_capacity(l.bindings.saturating_mul(l.judge_projects)),
            project_facts: Map::with_capacity(l.judge_projects),
            history_reads: crate::history::read::State::new(l.judge_projects),
            history_sweep: crate::history::sweep::State::new(l.history_events),
            history_projects: Map::with_capacity(l.judge_projects),
            devices: match config.push {
                Some(push) => Some(Box::new(crate::devices::State::new(Some(push), l.devices()))),
                None => None,
            },
            drafts: None,
            aways: Map::with_capacity(l.bindings.saturating_mul(l.judge_projects)),
            decisions: Map::with_capacity(l.decided()),
            findings: None,
            repairs_asked: Set::with_capacity(l.findings()),
            reds: Map::with_capacity(l.branch_reds.checked_mul(l.holds).expect("checked red capacity")),
            red_number: None,
            red_passes: Map::with_capacity(l.branch_reds.checked_mul(l.changes).expect("checked red pass capacity")),
            looked: Map::with_capacity(l.bindings.saturating_mul(l.judge_projects)),
            dismissed: Map::with_capacity(l.dismissed()),
            connections: Map::with_capacity(l.forges),
            fetched: None,
            first: Map::with_capacity(l.forges),
            ended: Map::with_capacity(l.forges),
            apps: Map::with_capacity(l.forges),
            setup_answers: Map::with_capacity(l.applications),
            setup_pending: Map::with_capacity(l.applications),
            choosing: Map::with_capacity(l.applications),
            setup_people: Map::with_capacity(l.applications),
            owned_people: Map::with_capacity(l.forges),
            connect: crate::connect::State::new(l.applications),
            installations: Map::with_capacity(l.client.rows),
            repositories: Map::with_capacity(l.repositories),
            names: Map::with_capacity(
                l.tasks
                    .checked_add(l.repositories)
                    .ok_or(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()))?,
            ),
            holds: Map::with_capacity(l.holds),
            subscriptions: Map::with_capacity(l.subscriptions),
            goal_files: Map::with_capacity(l.changes),
            goal_file_reads: Map::with_capacity(l.changes),
            heads: Map::with_capacity(l.client.resources),
            writes: Map::uncapped(),
            write_reads: writes::readback::State::new(),
            write_heads: Map::uncapped(),
            kept_moves: Map::with_capacity(l.client.resources),
            pulls: Map::with_capacity(l.client.resources),
            pull_mergeable: Map::with_capacity(l.client.resources),
            review_facts: Map::with_capacity(l.changes),
            review_heads: Map::with_capacity(l.changes),
            ci: Map::with_capacity(l.subscriptions),
            entries: Map::with_capacity(l.entries),
            flights: Map::with_capacity(l.entries),
            named_calls: Map::with_capacity(l.named_calls),
            proposed_effects: Map::with_capacity(l.tasks),
            landed: Map::with_capacity(l.landings.max(l.changes)),
            landings: Map::with_capacity(l.landings.max(l.client.resources)),
            lost: Map::with_capacity(l.holds),
            pending_ci: Map::uncapped(),
            ci_news: Map::uncapped(),
            restart_fresh: None,
            restart_reads_started: false,
            fresh_repositories: Set::with_capacity(l.repositories),
            restart_settling: false,
            catching_up_report: None,
            catching_up_seen: Map::with_capacity(l.forges),
            waiting_claims: Map::with_capacity(l.tasks),
            projection_waits: Set::with_capacity(l.issues),
            release_waits: Map::with_capacity(l.tasks),
            release_reads: Map::with_capacity(l.tasks),
            waiting_release_outcomes: Map::with_capacity(l.tasks),
            fresh_waits: Map::with_capacity(
                l.changes
                    .checked_add(l.landings)
                    .expect("change and landing waiters")
                    .checked_add(l.repositories.checked_mul(l.tiers.tiers).expect("tier waiters"))
                    .expect("restart waiters"),
            ),
            fresh_steps: Map::with_capacity(l.changes),
            entry_looks: Set::with_capacity(l.entries),
            fresh_branches: Set::with_capacity(l.client.resources),
            sequence: 0,
            closing: crate::closer::State::new(l.tasks),
            changes: Map::with_capacity(l.changes),
            task_holds: Map::with_capacity(l.changes),
            change_cold: Map::with_capacity(l.changes),
            change_hot: Map::with_capacity(l.changes),
            steps: Map::with_capacity(l.changes),
            merge_withheld: Set::with_capacity(l.changes),
            stall_words: Map::with_capacity(
                l.changes
                    .checked_add(l.landings)
                    .expect("task bounds")
                    .checked_add(l.repositories.checked_mul(l.tiers.tiers).expect("tier bounds"))
                    .expect("held task bounds"),
            ),
            read_sessions: Map::with_capacity(l.changes),
            merge_hooks: Map::with_capacity(l.changes),
            change_connections: None,
            start_conflicts: Map::with_capacity(l.changes),
            review_requests: Map::with_capacity(l.changes),
            makings: Map::with_capacity(l.holds),
            room: crate::room::State::new(l),
            wait_reports: crate::waits::State::new(),
            full: full::State::new(l.entries),
            panel_ci: Map::with_capacity(l.changes),
            issues: Map::with_capacity(l.issues),
            releases: Map::with_capacity(l.tasks),
            kept: Map::with_capacity(l.repositories),
            namespace: config.namespace.clone(),
            adoptions: Map::with_capacity(l.adoptions),
            application: adoption::State::new(l),
            capabilities: Map::with_capacity(l.repositories),
            workflows: crate::workflows::State::new(l),
            writers,
            identities: credentials,
            stopping: false,
            landing: crate::landing::State::new(l),
            follow_retry: Map::with_capacity(crate::follows::capacity(l).expect("checked follow capacity")),
            follow_queue: l.follow_queue,
            follow_merge: l.follow_merge,
            follow_reason_bytes: l.follow_reason_bytes,
            follow_rows: Map::with_capacity(crate::follows::capacity(l).expect("checked follow capacity")),
            follow_reads: Map::with_capacity(crate::follows::capacity(l).expect("checked follow capacity")),
            tier_ops: crate::tiers::State::new(l),
            tiers: Map::with_capacity(l.repositories.checked_mul(l.tiers.tiers).expect("checked tier capacity")),
            setting_rows: Map::with_capacity(
                crate::settings::rows(l)
                    .ok_or(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()))?,
            ),
            refused_settings: Map::with_capacity(
                crate::settings::rows(l)
                    .ok_or(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()))?,
            ),
            setting_stock: None,
            held_rules: empty_rules(l),
            held_charters: crate::ForgeCharters { change: 0, review: 0 },
            verifications: Map::with_capacity(l.changes),
            landing_logs: Map::with_capacity(l.repositories),
            restore_phase: RestorePhase::Cold,
            reconciliation: crate::reconcile::Reconciliation::new(l),
            restore_started: None,
            restore_fit: None,
            client: skein_forge_client::Domain::configured(
                &l.client,
                seed,
                skein_forge_client::Config { namespace: config.namespace, writers: config.writers },
            )?,
            client_out: Queue::uncapped(),
            read_line: read_line::Line::new(l),
            brief_fetches: Map::with_capacity(l.brief_sections),
            brief_planned: Map::with_capacity(l.brief_sections),
            brief_documents: Map::with_capacity(l.brief_sections),
            document_fetches: Map::with_capacity(l.brief_sections),
            brief_pending: Map::with_capacity(l.brief_sections),
            brief_held: Map::with_capacity(l.brief_sections),
            merge_kind: None,
            close_kind: None,
            projection_due: Map::with_capacity(l.issues),
            call_room: Set::with_capacity(l.entries),
            projection_room: Set::with_capacity(l.entries),
            full_outbox_met: false,
            change_due: Map::with_capacity(
                l.changes
                    .checked_add(l.tasks)
                    .ok_or(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()))?,
            ),
            procedure_wakes: Map::with_capacity(l.changes),
            settling: Map::with_capacity(l.changes),
        })
    }

    /// Raise every holder of the at-once limits (configuration/limit-settings.md, 5).
    fn raise(&mut self, limits: &Limits) {
        self.catching_up_seen.raise(limits.forges);
        let before = self.landing.limits;
        assert!(
            limits.tasks >= before.tasks
                && limits.holds >= before.holds
                && limits.entries >= before.entries
                && limits.client.entries >= before.client.entries,
            "at-once limits only rise"
        );
        let mut raised = before;
        raised.tasks = limits.tasks;
        raised.holds = limits.holds;
        raised.entries = limits.entries;
        raised.client.entries = limits.client.entries;
        assert_eq!(raised, *limits, "other limits wait for the next start");
        self.names.raise(limits.tasks.checked_add(limits.repositories).expect("checked names capacity"));
        self.proposed_effects.raise(limits.tasks);
        self.releases.raise(limits.tasks);
        self.room.closing.raise(limits.tasks);
        self.waiting_claims.raise(limits.tasks);
        self.release_waits.raise(limits.tasks);
        self.release_reads.raise(limits.tasks);
        self.waiting_release_outcomes.raise(limits.tasks);
        self.change_due.raise(limits.changes.checked_add(limits.tasks).expect("checked task capacity"));
        self.closing.raise(limits.tasks);
        self.holds.raise(limits.holds);
        self.lost.raise(limits.holds);
        self.makings.raise(limits.holds);
        self.read_line.raise(limits);
        self.reds.raise(limits.branch_reds.checked_mul(limits.holds).expect("checked red capacity"));
        self.entries.raise(limits.entries);
        self.room.asked.raise(limits.entries);
        self.call_room.raise(limits.entries);
        self.projection_room.raise(limits.entries);
        self.entry_looks.raise(limits.entries);
        self.full.raise(limits.entries);
        self.flights.raise(limits.entries);
        if let Some(outage) = &mut self.outage {
            outage.raise(limits.entries);
        }
        self.reconciliation.raise(limits);
        self.client.raise(&limits.client);
        self.landing.limits = *limits;
    }

    /// Whether the landing effect's exact-head condition guards this judge.
    #[must_use]
    pub fn guards_landing(&self, _project: u32, requirement: u16, _parameters: u32) -> bool {
        requirement == 5
    }

    /// Judge one forge requirement for the exact head of a change effect.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "a judgement names its project, criterion and coherent change evidence"
    )]
    pub fn judge_landing(
        &self,
        _project: u32,
        requirement: u16,
        _parameters: u32,
        task: u64,
        head: skein_forge::api::Commit,
        evidence: &crate::ChangeEvidence,
        reviewers: &[crate::Reviewer],
    ) -> Option<crate::JudgeVerdict> {
        match requirement {
            5 => self.judge_held_landing(false, task, head, evidence, reviewers),
            6 => self.judge_held_landing(true, task, head, evidence, reviewers),
            // temper/docs/design/domain/forge/change/gates.md, 2: requirements 1 to 4 attached
            // by owners wait; their parameters no longer index a criteria table.
            1..=4 => Some(crate::JudgeVerdict::Wait),
            _ => None,
        }
    }

    /// The next child timer, if one is armed.
    #[must_use]
    pub fn next_deadline(&self) -> Option<skein_lib::Time> {
        if self.restore_phase == RestorePhase::Restoring {
            return None;
        }
        let mut next = self.client.next_deadline();
        for at in &[
            brief::next_deadline(self),
            self.watch_due,
            self.playground_due,
            self.full.deadline(),
            catching_up::deadline(self),
            crate::tiers::deadline(self),
            goal_branch::deadline(self),
            self.read_line.deadline(),
        ] {
            next = earlier_time(next, *at);
        }
        if !self.stopping {
            for at in &[
                ci_news::deadline(self),
                crate::follows::deadline(self),
                outage::deadline(self),
                crate::unmade::deadline(self),
                crate::landing::read_deadline(self),
                retry_deadline(self),
            ] {
                next = earlier_time(next, *at);
            }
        }
        if !self.stopping && fresh_reads_in(self) {
            next = earlier_time(next, self.write_reads.deadline());
        }
        next
    }

    /// Earliest procedure or projection wait, reported to the root's clock.
    #[must_use]
    pub fn next_wall_deadline(&self) -> Option<skein_lib::Wall> {
        if self.stopping {
            return None;
        }
        if release::pending(self) {
            return Some(skein_lib::Wall::from_nanos(0));
        }
        if self.restore_phase == RestorePhase::Restoring {
            return None;
        }
        let mut next = read_session::deadline(self);
        for at in &[
            crate::history::sweep::deadline(self),
            crate::findings::judging::deadline(self),
            self.room.deadline,
            crate::closer::deadline(self),
            release_reads::deadline(self),
            goal_branch::read_deadline(self),
        ] {
            next = match (next, *at) {
                (Some(previous), Some(at)) => Some(previous.min(at)),
                (previous, None) => previous,
                (None, at) => at,
            };
        }
        for (goal, deadline) in &self.projection_due {
            if repository_reads::projection_fresh(self, *goal) {
                next = Some(match next {
                    Some(previous) => previous.min(*deadline),
                    None => *deadline,
                });
            }
        }
        for deadlines in [&self.change_due, &self.landing.due, &self.tier_ops.due] {
            for (_, deadline) in deadlines {
                next = Some(match next {
                    Some(previous) => previous.min(*deadline),
                    None => *deadline,
                });
            }
        }
        next
    }

    /// Whether a child call or a newly committed entry may be sent.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        if self.restore_phase == RestorePhase::Restoring {
            return false;
        }
        if self.stopping {
            return self.full.dirty || self.client.is_ready();
        }
        self.full.dirty
            || self.tier_ops.dirty
            || crate::follows::waiting(self)
            || self.next_handoff().is_some()
            || !self.procedure_wakes.is_empty()
            || crate::room::waiting(self)
            || self.client.is_ready()
            || read_line::askable(self)
    }

    fn next_handoff(&self) -> Option<u64> {
        for (number, row) in &self.entries {
            if row.handoff == Handoff::Handed || held_landing(self, &row.entry) {
                continue;
            }
            if row.handoff != Handoff::Ready || goal_branch::reserved_before(self, *number) {
                return None;
            }
            for (_, pending) in &self.steps {
                if pending.entry != 0 && pending.entry < *number {
                    return None;
                }
            }
            for (_, pending) in &self.landing.pending {
                if pending.entry != 0 && pending.entry < *number {
                    return None;
                }
            }
            return Some(*number);
        }
        None
    }

    /// Whether no brief section is being gathered by the connector.
    #[must_use]
    pub fn briefs_idle(&self) -> bool {
        self.brief_fetches.is_empty()
            && self.document_fetches.is_empty()
            && self.brief_documents.is_empty()
            && self.brief_planned.is_empty()
            && self.brief_pending.is_empty()
            && self.brief_held.is_empty()
    }

    /// Snapshot the provider ids restored for one forge, within the client's capacity.
    #[must_use]
    pub fn adopted_ids(&self, forge: u16) -> Box<[u64]> {
        let mut ids = List::with_capacity(self.repositories.len());
        for (repository, _) in &self.repositories {
            if repository.forge == forge && !self.forge_ended(forge) {
                ids.push(u64::from(repository.repository)).expect("bounded adopted repositories");
            }
        }
        ids.into_boxed()
    }

    /// Transfer one completed section to the parent assembling an assignment.
    pub fn take_brief(&mut self, section: Token) -> Option<Box<[u8]>> {
        match self.brief_held.remove(&section) {
            Some(held) => Some(held.words),
            None => None,
        }
    }

    /// Reclaim transient child buffers after the current decision.
    pub fn reclaim(&mut self) {
        let had_room = self.client.has_room();
        self.client.reclaim();
        if self.client.has_room() != had_room {
            self.full.dirty = true;
        }
        self.read_line.wake();
    }

    /// Check one typed task interest before the root adds its matching durable task row.
    #[must_use]
    pub fn can_subscribe(&self, limits: &Limits, subscription: &Subscriber) -> bool {
        self.subscription_problem(limits, subscription).is_none()
    }

    /// Name the failed subscription check and its configured limit, before any task row is added.
    #[must_use]
    pub fn subscription_problem(
        &self,
        limits: &Limits,
        subscription: &Subscriber,
    ) -> Option<(&'static [u8], Option<u32>)> {
        let problem = admission::subscription_problem(self, limits, subscription)?;
        let configured = match problem.failure {
            Some((_, crate::RefusalCause::Limit { configured, .. })) => {
                Some(u32::try_from(configured).expect("u32 limit"))
            }
            Some((
                _,
                crate::RefusalCause::Unadopted | crate::RefusalCause::InvalidName | crate::RefusalCause::Backend(_),
            ))
            | None => None,
        };
        Some((problem.words, configured))
    }

    /// Current subscriber for an automatic goal topic, preserving its durable number.
    #[must_use]
    pub fn topic_subscriber(&self, task: u64, topic: &Topic) -> Option<&Subscriber> {
        self.subscriptions.get(&(task, topic.clone()))
    }

    /// Find the connector half of one durable task subscription.
    #[must_use]
    pub fn subscription(&self, task: u64, number: u64) -> Option<Topic> {
        for (_, row) in &self.subscriptions {
            if row.task == task && row.number == number {
                return Some(row.topic.clone());
            }
        }
        None
    }

    /// Current live resource names of one task, for root-side name additions.
    #[must_use]
    pub fn names(&self, task: u64) -> Option<&[Name]> {
        match self.names.get(&task) {
            Some(names) => Some(names),
            None => None,
        }
    }

    /// A released task's repositories (temper/docs/design/domain/forge/adoption-connection.md, 1).
    #[must_use]
    pub fn kept_names(&self, task: u64) -> Option<&[Name]> {
        match self.kept.get(&task) {
            Some(names) => Some(names),
            None => None,
        }
    }

    /// Split restored live and kept names (temper/docs/design/domain/forge/effects.md, 2).
    pub fn link_names(&mut self, tasks: &jig_domain_tasks::Domain, limits: &Limits) {
        if self.restore_fit.is_some() {
            return;
        }
        let mut absent = List::with_capacity(self.names.capacity());
        let mut live = 0_u32;
        for (task, _) in &self.names {
            if tasks.task(*task).is_none() {
                absent.push(*task).expect("bounded restored names");
            } else {
                if let Err(failure) =
                    restore::room(RecordKind::Names, RestoreSet::Rows(RecordKind::Names), live, limits.tasks, false)
                {
                    self.restore_fit = Some(Box::new(failure));
                    return;
                }
                live = live.checked_add(1).expect("bounded live names");
            }
        }
        for task in absent.into_boxed() {
            if let Err(failure) = restore::room(
                RecordKind::Names,
                RestoreSet::Rows(RecordKind::Names),
                self.kept.len(),
                limits.repositories,
                self.kept.contains_key(&task),
            ) {
                self.restore_fit = Some(Box::new(failure));
                return;
            }
            let resources = self.names.remove(&task).expect("restored names row");
            self.kept.insert(task, resources).expect("kept names fit");
        }
    }

    /// Resolve worker head reports against this attempt's durable forge writers.
    /// Design writers read their branch afresh instead (domain/forge/branches.md, 3).
    /// Prepared assignments leave the root when delivered; their temporary
    /// lifetime must not determine whether a finished writer can be released.
    #[must_use]
    pub fn reported_heads(
        &self,
        limits: &Limits,
        task: u64,
        attempt: u64,
        pushed: &[crate::Pushed],
    ) -> Box<[(Name, skein_forge::api::Commit, bool)]> {
        let mut names = List::with_capacity(limits.resources_per_task);
        for (name, hold) in &self.holds {
            if hold.writer != Some(Writer::Run { task, attempt }) {
                continue;
            }
            for reported in pushed {
                if untag(reported.tag) == Some((name.forge, name.repository)) {
                    names
                        .push((name.clone(), reported.commit, reported.checked))
                        .expect("claimed writer destinations are bounded");
                    break;
                }
            }
        }
        names.into_boxed()
    }

    /// Whether this branch's own readers must wait for its making (domain/forge/branches.md, 1).
    #[must_use]
    pub fn goal_branch_waiting(&self, name: &Name) -> bool {
        goal_branch::waiting(self, name)
    }

    pub(crate) fn goal_branch_read_waiting(&self, repository: skein_forge::api::Repository, branch: &[u8]) -> bool {
        goal_branch::base_wait(self, repository, branch)
    }

    /// Occupancy at a refused entrance (protocol/configuration/limit-settings.md, 6).
    #[must_use]
    pub fn limit_usage(&self, set: &crate::LimitSet) -> Option<u64> {
        use crate::LimitSet;
        Some(match set {
            LimitSet::Operation => return None,
            LimitSet::Entries => u64::from(self.entries.len()),
            LimitSet::Holds => u64::from(self.holds.len()),
            LimitSet::Tasks => u64::from(self.names.len()),
            LimitSet::Changes => u64::from(self.changes.len()),
            LimitSet::Steps => u64::from(self.steps.len()),
            LimitSet::Subscriptions => u64::from(self.subscriptions.len()),
            LimitSet::Watches => skein_lib::len_u64(self.client.live_keys().len()),
            LimitSet::Heads => u64::from(self.heads.len()),
            LimitSet::Repositories => {
                let resources = self.client.live_keys();
                let mut repositories = Map::with_capacity(u32::try_from(resources.len()).expect("bounded watches"));
                for resource in resources {
                    repositories.insert(resource.repository, ()).expect("repositories bounded by watches");
                }
                u64::from(repositories.len())
            }
            LimitSet::DesignWriters(name) => match self.holds.get(name) {
                Some(hold) => skein_lib::len_u64(hold.design_writers.len()),
                None => 0,
            },
        })
    }

    /// Current holder and writer of one named resource for root claim checks.
    #[must_use]
    pub fn hold(&self, name: &Name) -> Option<&Hold> {
        self.holds.get(name)
    }

    /// The last durable head observed for a named branch.
    #[must_use]
    pub fn branch_head(&self, name: &Name) -> Option<skein_forge::api::Commit> {
        match self.heads.get(name) {
            Some(row) => Some(row.commit),
            None => None,
        }
    }

    /// A producer's refused early start, retained until that delegate ends (domain/forge/change/questions.md, 1).
    #[must_use]
    pub fn start_conflict(&self, task: u64) -> Option<skein_forge::api::Commit> {
        self.start_conflicts.get(&task).copied()
    }

    /// Adopted repository facts used by root authority and workspace translation.
    #[must_use]
    pub fn repository(&self, provider: skein_forge::api::Repository) -> Option<&Repository> {
        match self.repositories.get(&provider) {
            Some(row) => Some(row),
            None => None,
        }
    }

    /// The project's role on a repository (temper/docs/design/domain/forge/adoption-connection.md, 1).
    #[must_use]
    pub fn repository_role(&self, project: u32, repository: skein_forge::api::Repository) -> Option<Role> {
        let row = self.repository(repository)?;
        Some(row.for_project(project)?.role)
    }

    /// Resolve a deployment-wide saved repository tag within its project.
    #[must_use]
    pub fn repository_tag(&self, project: u32, tag: u64) -> Option<Repository> {
        let (forge, repository) = untag(tag)?;
        let adopted = self.repository(skein_forge::api::Repository { forge, repository })?;
        adopted.for_project(project)
    }

    /// All adopted repositories of this project on forges that have not ended.
    #[must_use]
    pub fn readable_repositories(&self, project: u32) -> Box<[Repository]> {
        let mut repositories = List::with_capacity(self.repositories.capacity());
        for (_, repository) in &self.repositories {
            if !self.forge_ended(repository.provider.forge)
                && let Some(row) = repository.for_project(project)
            {
                repositories.push(row).expect("bounded adopted repositories");
            }
        }
        repositories.into_boxed()
    }

    /// Repositories a person's change may land into
    /// (temper/docs/design/domain/forge/change/opened.md, 2).
    #[must_use]
    pub fn opening_repositories(&self, project: u32) -> Box<[&Repository]> {
        let mut repositories = List::with_capacity(self.repositories.capacity());
        for (_, repository) in &self.repositories {
            if repository.role_for(project).is_some()
                && repository.role_for(project) != Some(Role::Context)
                && repository.kinds.open
                && !self.forge_ended(repository.provider.forge)
            {
                repositories.push(repository.as_ref()).expect("bounded adopted repositories");
            }
        }
        repositories.into_boxed()
    }

    /// Adopted repositories in this project that still permit a workspace push.
    #[must_use]
    pub fn writable_repositories(&self, project: u32) -> Box<[&Repository]> {
        let mut repositories = List::with_capacity(self.repositories.capacity());
        for (_, repository) in &self.repositories {
            if repository.role_for(project).is_some()
                && repository.role_for(project) != Some(Role::Context)
                && repository.kinds.push
                && !self.forge_ended(repository.provider.forge)
            {
                repositories.push(repository.as_ref()).expect("bounded adopted repositories");
            }
        }
        repositories.into_boxed()
    }

    /// This project's explicitly selected home repository, when adopted.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset uses explicit iteration instead of iterator closures")]
    pub fn home(&self, project: u32) -> Option<&Repository> {
        for (_, repository) in &self.repositories {
            if repository.project == project && repository.home && !self.forge_ended(repository.provider.forge) {
                return Some(repository);
            }
        }
        None
    }

    /// Retained changes in task order (temper/docs/design/domain/forge/branches.md, 2).
    #[must_use]
    pub fn change_tasks(&self) -> Box<[u64]> {
        let mut tasks = List::with_capacity(self.changes.len());
        for (task, _) in &self.changes {
            tasks.push(*task).expect("retained change count");
        }
        tasks.into_boxed()
    }

    /// Keep the current hold alongside spec revisions while a gather is in flight.
    pub fn refresh_task_hold(&mut self, task: u64, hold: Option<TaskHold>) {
        if !self.changes.contains_key(&task) {
            return;
        }
        match hold {
            Some(hold) => {
                self.task_holds.insert(task, hold).expect("one projection per retained change");
            }
            None => {
                self.task_holds.remove(&task);
            }
        }
    }

    /// Refresh jig's revision projection before reading verdicts. The forge never advances it
    /// (temper/docs/design/domain/forge/change/gates.md, 1, Valid at a spec revision).
    pub fn refresh_spec_revision(&mut self, task: u64, revision: u64, out: &mut Queue<Request>) {
        let Some(row) = self.changes.get_mut(&task) else { return };
        if row.frozen() || row.spec_revision == revision {
            return;
        }
        row.spec_revision = revision;
        let saved = row.clone();
        let mut reviewing = false;
        for gate in &row.change.gates {
            reviewing |= gate.kind == change::GateKind::Agent;
        }
        if reviewing {
            if let Some(landing) = self.landing_of(task) {
                emit(out, Request::WakeProcedure { task: landing });
            }
            emit(out, Request::WakeProcedure { task });
        }
        save_change(self, out, saved);
    }

    /// Current durable change state used by the root's procedure route.
    #[must_use]
    pub fn change(&self, task: u64) -> Option<&ChangeRow> {
        self.changes.get(&task)
    }

    /// Complete the account at the root, where the read and delegate evidence meet.
    /// A later save of the same hold must not replace it.
    pub fn keep_accounted_hold(&mut self, task: u64, accounted: &change::Hold) {
        if let Some(row) = self.changes.get_mut(&task)
            && let change::State::Held { hold, .. } = &mut row.change.state
            && hold.cause == accounted.cause
            && hold.since == accounted.since
            && hold.judged == accounted.judged
            && hold.words.is_empty()
        {
            hold.clone_from(accounted);
        }
    }

    pub fn save_hold_words(&mut self, task: u64, out: &mut Queue<Request>) {
        if let Some(row) = self.changes.get(&task) {
            save_change_parts(self, out, row.clone());
        }
    }

    /// A change's committed pull identity for bounded root brief sections.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset uses explicit iteration instead of iterator closures")]
    pub fn change_for_pull(&self, repository: u32, number: u64) -> Option<&ChangeRow> {
        for (_, row) in &self.changes {
            if row.repository.repository == repository && row.pull == Some(number) {
                return Some(row);
            }
        }
        None
    }

    /// Lookup for a brief includes the round pull only while its member judges it.
    #[must_use]
    pub fn change_for_brief(&self, repository: u32, number: u64) -> Option<&ChangeRow> {
        for (_, row) in &self.changes {
            if row.repository.repository == repository {
                if row.pull == Some(number) {
                    return Some(row);
                }
                if let change::State::Judging { target, .. } = &row.change.state
                    && target.pull == number
                {
                    return Some(row);
                }
            }
        }
        None
    }

    /// Repair owner and task for one landing queue, retained through restart.
    #[must_use]
    pub fn queue_repair(&self, repository: skein_forge::api::Repository, base: &[u8]) -> Option<(u64, u64)> {
        let mut newest = None;
        for (_, row) in &self.changes {
            if row.repository == repository
                && row.base.as_ref() == base
                && let Some(repair) = row.queue_repair
                && match newest {
                    Some((_, prior)) => repair > prior,
                    None => true,
                }
            {
                newest = Some((row.task, repair));
            }
        }
        newest
    }

    #[must_use]
    pub fn queue_repairs(&self, repository: skein_forge::api::Repository, base: &[u8]) -> u32 {
        let mut count = 0_u32;
        for (_, row) in &self.changes {
            if row.repository == repository && row.base.as_ref() == base {
                count = count.saturating_add(row.queue_repairs);
            }
        }
        count
    }

    #[must_use]
    pub fn queue_repair_owner(&self, repair: u64) -> Option<u64> {
        for (_, row) in &self.changes {
            if row.queue_repair == Some(repair) {
                return Some(row.task);
            }
        }
        None
    }

    /// Committed goal projection for root subscription and retry routing.
    #[must_use]
    pub fn issue(&self, goal: u64) -> Option<&IssueRow> {
        self.issues.get(&goal)
    }

    /// Save a changed goal priority and invalidate the affected queue panels (domain/forge/queues.md).
    pub fn priority(&mut self, env: &Env<Limits>, task: u64, priority: i32, out: &mut Queue<Request>) {
        let Some(row) = self.changes.get_mut(&task) else { return };
        if row.priority == priority {
            return;
        }
        let before = crate::panel::ready(row);
        row.priority = priority;
        let saved = row.clone();
        save_queue_panel(self, env, before, saved, out);
    }

    /// Live change procedures in one repository, bounded by the connector's change table.
    #[must_use]
    pub fn changes_for(&self, repository: skein_forge::api::Repository, limit: u32) -> Box<[u64]> {
        let mut tasks = List::with_capacity(limit);
        for (task, row) in &self.changes {
            if row.repository == repository && !row.frozen() {
                tasks.push(*task).expect("change table bounds this list");
            }
        }
        tasks.into_boxed()
    }
}

/// Decide one parent event and collect its durable records and outputs.
pub fn step(d: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if let Event::WaitsRefused { task, why } = event {
        crate::waits::refused(d, task, why);
        return;
    }
    decide(d, env, event, out);
    read_afresh_complete(d, env, out);
    if d.restore_phase == RestorePhase::Active {
        crate::unmade::advance(d, env, out);
    }
    catching_up::advance(d, env, out);
    catching_up::mark(d, out);
}

#[expect(clippy::too_many_lines, reason = "one exhaustive connector input dispatcher")]
fn decide(d: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if d.restore_fit.is_some() {
        return;
    }
    match event {
        Event::LimitsRaised => d.raise(&env.limits),
        Event::ReadHistory { read, query } => crate::history::read::begin(d, read, query, out),
        Event::HistoryLoaded { load, rows, next } => crate::history::read::loaded(d, env, load, rows, next, out),
        Event::HistoryUnloaded { load } => crate::history::read::unloaded(d, load, out),
        Event::Signed { task, signed } => {
            if let Some(row) = d.changes.get_mut(&task) {
                row.signed = signed;
                let saved = row.clone();
                save_change(d, out, saved);
            }
        }
        Event::Pushed { party, device, endpoint, answer } => {
            crate::devices::pushed(d, party, device, &endpoint, answer, out);
        }
        Event::Closer { row } => crate::closer::save(d, env, row, out),
        Event::CloserRelayed { task, relay } => {
            if let Some(mut row) = d.closer(task).cloned() {
                row.relay = relay;
                crate::closer::save(d, env, row, out);
            }
        }
        Event::ParkCloser { task, wait } => crate::closer::park(d, task, wait, out),
        Event::ReadCloserSelection { task, repository } => crate::closer::read_selection(d, env, task, repository, out),
        Event::CheckCloser { task, step, repository, branch, base } => {
            crate::closer::check(d, env, task, step, repository, branch, crate::closer::Check::Landing(base), out);
        }
        Event::ReadCloserClose { task, step, dependencies, repository, branch } => {
            crate::closer::check(
                d,
                env,
                task,
                step,
                repository,
                branch,
                crate::closer::Check::Close(dependencies),
                out,
            );
            crate::waits::changed(d, task);
        }
        Event::ChangeEnding { task, ending } => {
            crate::room::owner_ended(d, task, env.wall);
            crate::room::leave(d, crate::room::Waiter::Change(task));
            if let Some(row) = d.changes.get_mut(&task) {
                let verified = row.ending.is_none()
                    && ending == crate::Ending::Landed
                    && crate::change_history::unfollowed(&row.change.state);
                let fresh = row.ending.is_none() && !row.terminal();
                let owned = row.change.owns_turn;
                row.change.owns_turn = false;
                row.room = None;
                row.ending = Some(ending);
                let mut row = row.clone();
                if verified {
                    let mut event = crate::change_history::step(
                        d,
                        &row,
                        crate::history::What::After { verdict: crate::history::AfterVerdict::Verified },
                    );
                    event.head = landed_commit(&row.change.state);
                    crate::change_history::write(d, env.wall, &mut row, event, out);
                }
                if fresh && let Some(ending) = crate::readiness::ending_kind(row.ending.as_ref().expect("ending set")) {
                    crate::readiness::ending(d, env, &mut row, ending, out);
                }
                wake_after_turn(d, &row, owned);
                save_change(d, out, row);
            }
        }
        Event::TierWords { task, words } => crate::tiers::words::projected(d, task, &words, out),
        Event::StatusTask { project, party, task } => crate::status::task(d, env.wall, party, project, Some(task), out),
        Event::StatusRefused { project, party } => crate::status::task(d, env.wall, party, project, None, out),
        Event::StatusHeard { task, report } => crate::status::heard(d, env.wall, task, report, out),
        Event::Follow { row } => crate::follows::register(d, row, out),
        Event::TierTask { repository, tier, number, task } => {
            crate::tiers::task(d, repository, tier, number, Some(task), out);
        }
        Event::TierRefused { repository, tier, number } => crate::tiers::task(d, repository, tier, number, None, out),
        Event::StepTier { task, entry, heard } => crate::tiers::step(d, env, task, entry, heard, out),
        Event::VetoTier { task, entry } => crate::tiers::veto(d, task, entry, out),
        Event::OrderLanding { task, changes } => crate::landing::order(d, task, &changes, out),
        Event::Landing { row } => crate::landing::register(d, env, row, out),
        Event::TrainRefusal { repository, branch, cause } => {
            crate::train_refusal::replace(d, out, repository, branch, cause, env.wall);
        }
        Event::VetoLanding { task, entry } => crate::landing::veto(d, env, task, entry, out),
        Event::DeferLandingMerge { task, entry } => crate::landing::defer_merge(d, env, task, entry, out),
        Event::StepLanding { task, entry, heard } => {
            crate::landing::step(d, env, task, entry, heard, out);
            crate::waits::changed(d, task);
        }
        Event::Settle => {
            d.stopping = true;
            d.restart_fresh = None;
            d.watch_due = None;
            crate::landing::settle(d);
            d.read_line.clear();
            child(d, env, skein_forge_client::Event::Settle, out);
        }
        Event::Bind { request, address, subject } => bind(d, request, address, subject, out),
        Event::PlaygroundMade { project, account } => crate::playground::made(d, project, account, out),
        Event::PlaygroundFailed { why } => crate::playground::failed(d, why, out),
        Event::ResetActAnswered { key, keep } => crate::playground::act_answered(d, key, keep, out),
        Event::BoxClosed => {
            d.under_box = false;
            d.fetched = None;
            for (forge, _) in &d.configured {
                if d.forge_ended(*forge) {
                    continue;
                }
                if let Some((state, failed)) = d.connections.get_mut(forge)
                    && let crate::Connection::SettingUp { .. } = state
                {
                    *failed = failed_at(state, *failed);
                    *state = crate::Connection::SetUpFailed { why: Box::from(b"the box closed the pipe".as_slice()) };
                }
            }
            out.push(Request::MarkedForges);
        }
        Event::AppRecorded { forge, app } => {
            if d.configured.contains_key(&forge) && !d.forge_ended(forge) {
                d.apps.insert(forge, app).expect("configured forge App way");
                out.push(Request::MarkedForges);
            }
        }
        Event::ConnectAnswered { forge, request, answer } => crate::connect::connected(d, forge, request, answer, out),
        Event::AccessAnswered { forge: _, request, answer } => adoption::accessed(d, env, request, answer, out),
        Event::ReachableAnswered { forge, request, answer } => crate::connect::reached(d, forge, request, answer, out),
        Event::AcceptAnswered { forge, request, answer, stopped } => {
            crate::connect::accepted(d, forge, request, answer, stopped, out);
        }
        Event::InstallAnswered { forge, request, answer, stopped } => {
            crate::connect::installed(d, forge, request, answer, stopped, out);
        }
        Event::WayEnded { forge, why } => crate::connect::way_ended(d, forge, &why, out),
        Event::Owner { party } => crate::link::owner(d, party, out),
        Event::PersonOf { forge, answer } => crate::link::person_of(d, forge, answer, out),
        Event::BotLogin { forge, login } => crate::connect::bot_login(d, forge, login, out),
        Event::ConnectEnded { forge, request, connected, why, stop } => {
            crate::connect::ended(d, forge, request, connected, why, stop, out);
        }
        Event::SetUpAnswered { forge: _, request, answer } => {
            d.setup_answer(env, request, answer, out);
        }
        Event::FirstMade { forge, id } => {
            if d.first_made(forge, id) {
                out.push(Request::MarkedForges);
            }
        }
        Event::FirstFailed { forge, step } => {
            if d.first_failed(forge, step) {
                out.push(Request::MarkedForges);
            }
        }
        Event::Connection { forge, state } => {
            let was_connected = !connection_wait::disconnected(&d.connection(forge));
            let connected = !connection_wait::disconnected(&state);
            if d.forge_ended(forge) {
                return;
            }
            match &state {
                crate::Connection::Connected { id, .. } => {
                    d.ended.remove(&forge);
                    retire_ended(d, env, out);
                    assert!(
                        writer(d, env, skein_forge_client::Writer { forge, author: *id }, out),
                        "same forge writer"
                    );
                    // A restart's NotConnected narrows nothing (domain/forge/effects.md,
                    // 2). Resume these reads as client call room becomes free.
                    let mut waiting = List::with_capacity(env.limits.holds);
                    for (owner, pending) in &d.lost {
                        if pending.name.forge == forge && pending.state == LostReadState::WaitingForConnection {
                            waiting.push(*owner).expect("lost reads bounded by holds");
                        }
                    }
                    let line_was_empty = !d.read_line.waiting();
                    goal_branch::connected(d, env.now, forge, out);
                    for owner in &waiting {
                        d.lost.get_mut(owner).expect("collected lost read").state = LostReadState::WaitingForRoom;
                        read_line::join(d, read_line::Waiter::Lost(*owner), env.now, out);
                    }
                    crate::landing::connected(d, env.now, forge, out);
                    if line_was_empty {
                        d.read_line.wake();
                    }
                }
                crate::Connection::NotConnected
                | crate::Connection::NotSetUp
                | crate::Connection::SettingUp { .. }
                | crate::Connection::SetUpFailed { .. }
                | crate::Connection::Connecting
                | crate::Connection::Stopped { .. } => {}
            }
            let failed = match &state {
                crate::Connection::SetUpFailed { .. } => match d.connections.get(&forge) {
                    Some((held, failed)) => failed_at(held, *failed),
                    None => None,
                },
                crate::Connection::NotConnected
                | crate::Connection::NotSetUp
                | crate::Connection::SettingUp { .. }
                | crate::Connection::Connecting
                | crate::Connection::Connected { .. }
                | crate::Connection::Stopped { .. } => None,
            };
            d.connections.insert(forge, (state, failed)).expect("configured forge connections");
            if was_connected != connected {
                read_session::connection(d, env, forge, connected, out);
                crate::landing::read_connection(d, env, forge, connected);
            }
            match d.connection(forge) {
                crate::Connection::Connected { .. } => connection_wait::connected(d, env, forge, out),
                crate::Connection::NotConnected
                | crate::Connection::NotSetUp
                | crate::Connection::SettingUp { .. }
                | crate::Connection::SetUpFailed { .. }
                | crate::Connection::Connecting
                | crate::Connection::Stopped { .. } => {}
            }
            if let Some((number, _)) = d.fetched
                && number == forge
            {
                d.fetched = None;
            }
            crate::playground::connected(d, out);
            crate::waits::change_forge(d, forge);
            out.push(Request::MarkedForges);
        }
        Event::Fetched { forge, fetched } => {
            if d.connection(forge) == (crate::Connection::SettingUp { step: Some(crate::SetUpStep::Fetch) }) {
                d.fetched = Some((forge, fetched));
                out.push(Request::MarkedForges);
            }
        }
        Event::Installation { forge, installation, state } => {
            match state {
                Some(state) => {
                    d.installations.insert((forge, installation), state).expect("bounded installations");
                }
                None => {
                    let _gone = d.installations.remove(&(forge, installation));
                }
            }
            out.push(Request::MarkedForges);
        }
        Event::SecretsErased { forge } => {
            crate::removal::secrets_erased(d, env, forge, out);
            crate::playground::removed(d, out);
        }
        Event::BoxRemoved { forge } => {
            crate::removal::box_removed(d, env, forge, out);
            crate::playground::removed(d, out);
        }
        Event::FollowChangeAmended { task, follows, words } => {
            if let Some(row) = d.changes.get_mut(&task) {
                row.follows = Some(follows);
                row.title = temper_engine_domain_forge_closer::first_line(&words).into();
                row.body = words;
                let row = row.clone();
                save_change(d, out, row);
            }
        }
        Event::TierAct { request, token, party, call } => {
            crate::tiers::hold::asked(d, env, request, token, party, call, out);
        }
        Event::FollowAct { request, token, retention, project, follow, act, authorized } => {
            crate::follows::asked(d, env, request, token, retention, project, follow, act, authorized, out);
        }
        Event::SetupPerson { forge, login } => {
            d.owned_people.insert(forge, login).expect("kept owned person");
        }
        Event::PlaceChosen { request, place, git_program } => make::chosen(d, env, request, place, git_program, out),
        Event::Application { request, token, retention, ask, role, carried } => match ask {
            crate::Ask::RemoveForge { forge } => {
                if let Some(value) = carried {
                    out.push(Request::CarriedUnused { number: value.number });
                }
                crate::removal::admit(d, request, token, role, forge, out);
            }
            ask @ (crate::Ask::Connect { .. }
            | crate::Ask::Replace { .. }
            | crate::Ask::Install { .. }
            | crate::Ask::Accept { .. }
            | crate::Ask::Reachable { .. }) => {
                crate::connect::start(d, request, token, ask, role, carried, out);
            }
            crate::Ask::StartPlayground => crate::playground::start(d, env.now, request, token, out),
            crate::Ask::EndPlayground => crate::playground::end(d, request, token, out),
            crate::Ask::DismissMaking { forge, org, name, into } => {
                if let Some(value) = carried {
                    out.push(Request::CarriedUnused { number: value.number });
                }
                crate::making::dismiss(
                    d,
                    env,
                    request,
                    token,
                    retention,
                    role,
                    crate::MakingKey { forge, org, name },
                    into,
                    out,
                );
            }
            ask @ crate::Ask::MakeRepository { .. } => {
                if let Some(value) = carried {
                    out.push(Request::CarriedUnused { number: value.number });
                }
                crate::making::admit(d, env, request, token, retention, role, ask, out);
            }
            crate::Ask::SetDraft(_) => unreachable!("draft admissions carry their party"),
            crate::Ask::Adopt(adoption) => adoption::start(d, env, request, token, retention, adoption, out),
            ask @ crate::Ask::SetUp { .. } => make::setup(d, env, request, token, retention, ask, carried, out),
        },
        Event::MakingStarted { key } => crate::making::started(d, key, out),
        Event::MakingCopying { key } => crate::making::copying(d, key, out),
        Event::MakingMade { key, id, branch } => crate::making::made(d, key, id, branch, out),
        Event::MakingStopped { key, failed } => crate::making::stopped(d, key, failed, out),
        Event::MakingAdopt { key } => crate::making::adopt(d, key, out),
        Event::MakingAsked { request, adoption, repository } => {
            adoption::making_asked(d, env, request, adoption, repository, out);
        }
        Event::MakingActed { key, outcome } => crate::making::acted(d, key, outcome, out),
        Event::MakingAgain { key } => {
            d.making_acts.remove(&key);
        }
        Event::AddDevice { request, token, party, person, call } => {
            if person {
                crate::devices::add(d, env, request, token, party, call, out);
            } else {
                d.connect.answer(
                    request,
                    token,
                    crate::Answered::Device(crate::DeviceAnswer::Refused(crate::DeviceRefusal::Person)),
                    out,
                );
            }
        }
        Event::RemoveDevice { request, token, party, number } => {
            crate::devices::remove(d, request, token, party, number, out);
        }
        Event::SetDraft { request, token, party, call } => {
            crate::drafts::start(d, env, request, token, party, call, out);
        }
        Event::SetAway { request, token, party, project, on, until } => {
            crate::away::set_away(d, env, request, token, party, project, on, until, out);
        }
        Event::Overturn { request, token, party, project, number } => {
            crate::away::overturn(d, request, token, party, project, number, out);
        }
        Event::Decided { to, key, party, project, task, always, call } => {
            crate::away::decided(d, env, to, key, party, project, task, always, call, out);
        }
        Event::Looked { request, token, party, project } => {
            crate::status::looked(d, env, request, token, party, project, out);
        }
        Event::Dismiss { request, token, party, what } => {
            crate::dismissed::dismiss(d, request, token, party, what, out);
        }
        Event::ApplicationAnswered { request, outcome } => adoption::answered(d, env, request, outcome, out),
        Event::ForgetProjection { goal } => {
            crate::closer::forget(d, goal, out);
            if let Some(row) = d.issues.get(&goal)
                && row.pending.is_none()
                && !row.failed
            {
                d.issues.remove(&goal);
                emit(out, Request::Erase { key: Key::Issue(goal) });
            }
        }
        Event::PlanDocument { section, document } => {
            assert!(
                d.brief_planned.insert(section, held::Planned { source: crate::BriefSource::Document, partner: None })
                    == Ok(None),
                "one bounded planned section"
            );
            assert!(d.brief_documents.insert(section, document) == Ok(None), "one bounded planned document");
        }
        Event::PlanBrief { section, source, partner } => {
            if let Some(partner) = partner {
                d.brief_planned.get_mut(&partner).expect("earlier paired section planned").partner = Some(section);
            }
            assert!(
                d.brief_planned.insert(section, held::Planned { source, partner }) == Ok(None),
                "one bounded planned section"
            );
        }
        Event::GatherPlanned { section, task, parts, bytes, ci_budget, deadline } => {
            let Some(planned) = d.brief_planned.remove(&section) else {
                out.push(Request::BriefSized { section, size: Err(crate::brief_words::bounds()) });
                return;
            };
            let source = planned.source;
            let max_job_bytes = match source {
                crate::BriefSource::Document => env.limits.client.answer_bytes,
                crate::BriefSource::Ci { .. } => env.limits.client.answer_bytes.min(ci_budget / 5).max(1),
                crate::BriefSource::Home { .. }
                | crate::BriefSource::Request { .. }
                | crate::BriefSource::Reviews { .. }
                | crate::BriefSource::Pull { .. } => 0,
            };
            held::gather(d, section, source, planned.partner, parts, bytes, max_job_bytes, env.limits.brief_bytes, out);
            d.brief_documents.remove(&section);
            brief::set_context(d, section, task, deadline, env.now);
        }
        Event::GatherBrief { owner, source, parts, bytes, max_job_bytes } => {
            brief::gather(d, owner, source, parts, bytes, max_job_bytes, env.limits.brief_bytes, out);
        }
        Event::GatherBriefHeld { section, source, parts, bytes, max_job_bytes } => {
            held::gather(d, section, source, None, parts, bytes, max_job_bytes, env.limits.brief_bytes, out);
        }
        Event::CutBrief { section, bytes } => held::cut(d, section, bytes, out),
        Event::TakeBrief { section } => held::take(d, section, out),
        Event::DropBrief { section } => {
            d.brief_planned.remove(&section);
            d.brief_documents.remove(&section);
            held::drop_section(d, section);
            brief::drop_section(d, section);
        }
        Event::ReleaseProject { project, request } => adoption::release(d, env, project, request, out),
        Event::Adopt { request } => adoption::adopt(d, env, request, out),
        Event::ForgetAdoption { repository, restore } => match restore {
            Some(mut previous) => {
                if let Some(current) = d.repository(repository)
                    && current.project == previous.project
                {
                    previous.refusals.clone_from(&current.refusals);
                }
                d.repositories.insert(repository, Box::new(previous.clone())).expect("restores an admitted repository");
                emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Repository(previous) });
                emit(out, Request::MarkedForges);
            }
            None => {
                release::repository(d, env, repository, out);
            }
        },
        Event::Names { task, resources } => {
            if !d.kept.contains_key(&task) {
                names(d, env, task, resources, out);
            }
        }
        Event::Rename { task, resources } => {
            if d.names.contains_key(&task) {
                names(d, env, task, resources, out);
            }
        }
        Event::Unnamed { task } => {
            d.closing.branch_waits.remove(&task);
            if d.kept.remove(&task).is_some() {
                emit(out, Request::Erase { key: Key::Names(task) });
                release::first_write(d);
                crate::removal::unnamed(d, env, out);
            } else {
                unnamed(d, env, task, false, out);
            }
        }
        Event::Hold { entry, task, resource, from } => hold(d, env, entry, task, resource, from, out),
        Event::Claim { task, attempt, writes, designs, holders } => {
            if repository_reads::claim_fresh(d, &writes, &designs) {
                claim(d, env, task, attempt, &writes, &designs, &holders, out);
            } else if !matches_later_claim(d, task, attempt) {
                d.waiting_claims.insert(task, (attempt, writes, designs, holders)).expect("one claim per live run");
            }
        }
        Event::Answered { task, attempt, pushed } => answered(d, env, task, attempt, &pushed, out),
        Event::Finished { task, attempt, lost } => d.finished(task, attempt, lost, out),
        Event::Lost { task, attempt } => lost(d, env, task, attempt, out),
        Event::KeepProposedEffect { row } => {
            assert!(
                d.proposed_effects.insert(row.number, row.clone()).is_ok(),
                "one bounded effect proposal per live task"
            );
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::ProposedEffect(row) });
        }
        Event::DropProposedEffect { number } => {
            if d.proposed_effects.remove(&number).is_some() {
                emit(out, Request::Erase { key: Key::ProposedEffect(number) });
            }
        }
        Event::Enqueue { entry } => enqueue(d, env, None, entry, out),
        Event::EnqueueCall { owner, entry } => enqueue(d, env, Some(owner), entry, out),
        Event::KeepProjection { row, entry } => {
            assert_eq!(entry.task, row.goal, "projection entry {} belongs to goal {}", entry.number, row.goal);
            assert_eq!(
                row.pending,
                Some(entry.number),
                "projection row of goal {} owns entry {}",
                row.goal,
                entry.number
            );
            d.take_projection_room(entry.number, row.goal);
            d.issues.insert(row.goal, row.clone()).expect("projection capacity admitted");
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(row) });
            keep_entry(d, entry, None, out);
        }
        Event::DropProjection { entry, goal } => d.take_projection_room(entry, goal),
        Event::Milestone { row } => crate::milestone_rows::project(d, row, out),
        Event::Project { entry, repository, view } => project(d, env, entry, repository, view, out),
        Event::FailProjection { goal, repository, closing } => {
            fail_projection(d, env, goal, Some(repository), closing, out);
        }
        Event::ProjectDesired { entry, goal } => {
            if let Some(row) = d.issues.get(&goal)
                && let Some(view) = &row.desired
            {
                project(d, env, entry, row.repository, view.clone(), out);
            }
        }
        Event::DropChange { task } => room::drop_change(d, env, task, out),
        Event::DropErased { task } => room::drop_erased(d, task, out),
        Event::Change { row } => register_change(d, env, row, out),
        Event::Priority { task, priority } => d.priority(env, task, priority, out),
        Event::QueueRepairStarted { owner, repair } => {
            if let Some(row) = d.changes.get_mut(&owner)
                && !row.base_repair
            {
                row.queue_repair = Some(repair);
                row.queue_repairs = row.queue_repairs.saturating_add(1);
                let saved = row.clone();
                save_change(d, out, saved);
                crate::readiness::repair_started(d, env, owner, out);
            }
        }
        Event::ProducedTitle { task, child, title, report } => produced_title(d, task, child, &title, report, out),
        Event::JobHistory { task, job } => {
            if job.outcome.is_none()
                && let Some(row) = d.changes.get(&task)
                && row.frozen()
            {
                return;
            }
            if job.outcome.is_some() {
                let mut reading = None;
                for (owner, pending) in &d.steps {
                    if pending.task == task {
                        reading = Some(*owner);
                    }
                }
                if let Some(owner) = reading {
                    d.steps.get_mut(&owner).expect("pending gather").woken = true;
                }
                crate::landing::job_woken(d, task);
            }
            crate::job_history::heard(d, env, task, job, out);
            if job.outcome.is_some()
                && let Some(mut row) = d.changes.get(&task).cloned()
                && row.frozen()
            {
                // An open attempt ends without gathering new branch facts after its change's end.
                crate::job_history::merged(d, env.wall, &mut row, None, out);
                d.changes.insert(task, row.clone()).expect("retained job owner");
                save_change(d, out, row);
            }
        }
        Event::JobSuspended { task, child, attempt } => {
            crate::job_history::suspended(d, env, task, child, attempt, out);
        }
        Event::Delegated { task, child, kind, close_at } => delegated(d, env, task, child, kind, close_at, out),
        Event::DelegateRefused { task, child } => delegate_refused(d, env, task, child, out),
        Event::DelegateAdmissionRefused { task, kind } => delegate_admission_refused(d, env, task, kind, out),
        Event::DelegateResult {
            task,
            child,
            ended_at,
            status,
            no_change,
            flake,
            no_test,
            failed,
            lost,
            cancelled,
            words,
        } => {
            delegate_result(
                d, env, task, child, ended_at, status, no_change, flake, no_test, failed, lost, cancelled, &words, out,
            );
        }
        Event::RetireFailedClose { task, child } => {
            if let Some(row) = d.changes.get_mut(&task)
                && row.delegate == Some((child, change::Delegate::Close))
                && row.delegate_status == change::Status::Failed
            {
                row.delegate = None;
                let saved = row.clone();
                save_change(d, out, saved);
            }
        }
        Event::DependenciesMet { task } => {
            crate::waits::change_dependencies_met(d, task);
            dependencies_met(d, env, task, out);
        }
        Event::RelayRefused { task, questions, words } => {
            if let Some(row) = d.changes.get_mut(&task) {
                let before = crate::panel::ready(row);
                let owned = row.change.owns_turn;
                row.questions = questions;
                let held = relay_hold(&mut row.change.state, change::Cause::Failed, env.wall);
                if held {
                    row.change.owns_turn = false;
                    row.producer_failure = Some(words);
                }
                let saved = row.clone();
                wake_after_turn(d, &saved, owned);
                save_queue_change(d, env, before, saved, out);
                if held {
                    emit(out, Request::RelayRefusal { task });
                }
            }
        }
        Event::Relayed { task, questions } => {
            if let Some(row) = d.changes.get_mut(&task) {
                let before = crate::panel::ready(row);
                let owned = row.change.owns_turn;
                let was_asking = has_agent_question(row);
                let hold = match questions.open.first() {
                    Some(question) => {
                        question.stands == change::Stands::Taken
                            && questions.taken >= env.limits.change_policy.questions
                    }
                    None => false,
                };
                row.questions = questions;
                let answered = was_asking && !has_agent_question(row);
                if hold && relay_hold(&mut row.change.state, change::Cause::Questions, env.wall) {
                    row.change.owns_turn = false;
                }
                let saved = row.clone();
                wake_after_turn(d, &saved, owned);
                save_queue_change(d, env, before, saved, out);
                if answered {
                    emit(out, Request::WakeProcedure { task });
                }
            }
        }
        Event::StartConflicted { child, tag, head } => {
            for (task, row) in &d.changes {
                if row.delegate == Some((child, change::Delegate::Produce))
                    && crate::tag(row.repository.forge, row.repository.repository) == tag
                {
                    d.start_conflicts.insert(*task, head).expect("one start conflict per change");
                    d.procedure_wakes.insert(*task, true).expect("one pending wake per change");
                    break;
                }
            }
        }
        Event::ReviewsChanged { task } => reviews_changed(d, env, task, out),
        Event::ReviewRequested { task, head } => {
            if let Some(row) = d.changes.get_mut(&task)
                && !row.frozen()
            {
                let owned = row.change.owns_turn;
                row.change.owns_turn = false;
                let saved = row.clone();
                d.review_requests.insert(task, head).expect("one review request per change");
                d.procedure_wakes.insert(task, true).expect("one pending wake per change");
                if owned {
                    wake_after_turn(d, &saved, owned);
                    save_change(d, out, saved);
                }
            }
        }
        Event::StepChange { task, entry, heard, gates, queue_repair_active, kept, merge_job, pending } => {
            crate::waits::change_standing(d, task, &pending, heard.delegate_live);
            step_change(
                d,
                env,
                StepInput { task, entry, heard, gates, queue_repair_active, kept, merge_job, pending },
                out,
            );
            crate::waits::changed(d, task);
        }
        Event::ReleaseChange { task, release } => {
            let take = match d.changes.get(&task) {
                Some(row) => match &release {
                    Some(release) => match &row.task_release {
                        Some(prior) => release.revision > prior.revision,
                        None => true,
                    },
                    None => matches_held(&row.change.state) && row.change.released.is_none(),
                },
                None => false,
            };
            if take {
                if let Some(row) = d.changes.get_mut(&task)
                    && release.is_some()
                {
                    row.task_release = release;
                }
                release_change(d, env, task, out);
            }
        }
        Event::GoalReleased { task } => goal_branch::accept_release(d, env, task, out),
        Event::SettleEffects { task, root, ending } => settle_effects(d, env, task, root, ending, out),
        Event::SavedWork { task, branches } => saved_work(d, env, task, &branches, out),
        Event::Release { task, root, ending, reason, entry } => release(d, env, task, root, ending, reason, entry, out),
        Event::ContinueRelease { task, entry } => continue_release(d, env, task, entry, out),
        Event::ContinueGoalBranch { task, entry } => goal_branch::continue_making(d, env, task, entry, out),
        Event::RetryGoalBranch { task } => goal_branch::retry_task(d, env, task, out),
        Event::ReleaseProjection { goal } => release_projection(d, goal, out),
        Event::Committed { entry } => committed(d, env, entry, out),
        Event::Resolve { entry, again } => resolve(d, env, entry, again, out),
        Event::Withdraw { entry } => withdraw(d, env, entry, out),
        Event::VetoChange { task, entry, prior, leaves_queue } => {
            veto_change(d, env, task, entry, prior, leaves_queue, out);
        }
        Event::Subscribe { subscription } => subscribe(d, env, subscription, out),
        Event::Unsubscribe { task, topic } => unsubscribe(d, env, task, topic, out),
        Event::Hint { hint } => {
            crate::landing::hinted(d, &hint, out);
            crate::follow_check::hinted(d, &hint, out);
            crate::tiers::hinted(d, &hint, out);
            let changes = d.changes_for(hint.repository, env.limits.changes);
            // Only a branch with no recognised owner retains a repository fallback.
            let mut matched = false;
            for task in &changes {
                if hints::concerns(d, d.changes.get(task).expect("repository's change"), &hint) {
                    matched = true;
                    break;
                }
            }
            crate::settle::hinted(d, env, &hint, &changes);
            for task in changes {
                let row = d.changes.get(&task).expect("repository's change");
                let branch_hint = match &hint.change {
                    skein_forge::api::Change::Branch(branch) | skein_forge::api::Change::BranchMoved { branch, .. } => {
                        branch.is_empty() || *branch == row.branch
                    }
                    skein_forge::api::Change::Commit(_) | skein_forge::api::Change::Item(_) => false,
                };
                let fallback = match hint.change {
                    skein_forge::api::Change::Branch(_) | skein_forge::api::Change::BranchMoved { .. } => {
                        !matched && hints::has_head(d, row)
                    }
                    skein_forge::api::Change::Commit(_) | skein_forge::api::Change::Item(_) => false,
                };
                if fallback || hints::concerns(d, row, &hint) {
                    if branch_hint {
                        branch_due(d, task);
                    }
                    let refreshing = match hint.change {
                        skein_forge::api::Change::Commit(head) => ci_news::hint(d, env, task, head),
                        skein_forge::api::Change::Item(_)
                        | skein_forge::api::Change::Branch(_)
                        | skein_forge::api::Change::BranchMoved { .. } => false,
                    };
                    if !refreshing {
                        d.procedure_wakes.insert(task, true).expect("one pending wake per change");
                    }
                }
            }
            ci_hint(d, env, &hint, out);
            child(d, env, skein_forge_client::Event::Hint { hint }, out);
        }
        Event::RoomUnavailable { task } => crate::room::pass(d, task),
        Event::Client(event) => child(d, env, event, out),
        Event::Restore { record } => restore(d, env, record),
        Event::Restored { clock } => restored(d, env, clock, out),
        Event::WaitsRefused { .. } => unreachable!("a refusal never steps the connector"),
        Event::CatchUp => {
            d.catching_up_report = Some(crate::Report::new(env.now));
            crate::room::catch_up(d, env.wall);
            read_afresh(d, env, out);
            settle_outbox(d, env, out);
            d.room.ready = true;
            crate::landing::restore_waits(d, env);
            crate::waits::begin_catch_up(d, d.restore_started.unwrap_or(env.wall));
        }
    }
    if !d.write_heads.is_empty() || !d.client_out.is_empty() {
        drain(d, env, out);
    }
    release::first_page(d, env, out);
    if d.retire_pending {
        d.retire_pending = false;
        retire_ended(d, env, out);
    }
    crate::room::ask(d, env, out);
}

/// Drive one child call after prior saves have committed.
pub fn resume(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    drive(d, env, out);
    read_afresh_complete(d, env, out);
    catching_up::advance(d, env, out);
    catching_up::mark(d, out);
}

fn drive(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    full::update(d, env, out);
    if d.restore_fit.is_some() {
        return;
    }
    if d.restore_phase == RestorePhase::Restoring && !d.stopping {
        return;
    }
    if d.stopping {
        skein_forge_client::resume(&mut d.client, &child_env(env), &mut d.client_out);
        drain(d, env, out);
        return;
    }
    if d.tier_ops.dirty {
        crate::tiers::watch(d, env, out);
        return;
    }
    crate::follows::resume(d, env, out);
    crate::playground::expire(d, env.now, out);
    brief::expire(d, env.now, out);
    crate::room::ask(d, env, out);
    let before = out.len();
    if handoff_one(d, env, out) || env.now.as_nanos() == u64::MAX {
        crate::room::ask(d, env, out);
    }
    if out.len() != before {
        // An admission refusal already spent this step's child output burst.
        return;
    }
    read_line::prune_makings(d);
    if read_line::serve(d, env, out) {
        // A successful admission takes this turn. Give any further places
        // to the line, too, before resuming the client's polls.
        return;
    }
    writes::readback::drive(d, env, out);
    let child_env = child_env(env);
    skein_forge_client::resume(&mut d.client, &child_env, &mut d.client_out);
    drain(d, env, out);
    crate::room::ask(d, env, out);
    if out.room() > 0
        && let Some((task, wake)) = d.procedure_wakes.iter().next()
    {
        let (task, wake) = (*task, *wake);
        d.procedure_wakes.remove(&task);
        emit(out, if wake { Request::WakeProcedure { task } } else { Request::StepDependents { task } });
    }
}

/// Expire one client timer.
pub fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    expire(d, env, out);
    if d.restore_phase == RestorePhase::Active {
        crate::unmade::advance(d, env, out);
    }
    catching_up::advance(d, env, out);
    catching_up::mark(d, out);
}

fn expire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    full::update(d, env, out);
    if d.restore_fit.is_some() {
        return;
    }
    if d.stopping {
        skein_forge_client::fire(&mut d.client, &child_env(env), &mut d.client_out);
        drain(d, env, out);
        return;
    }
    if release::pending(d) {
        release::page(d, env, out);
        return;
    }
    if d.restore_phase == RestorePhase::Restoring {
        return;
    }
    crate::findings::judging::fire(d, env.wall, out);
    crate::history::sweep::fire(d, env.wall, out);
    crate::playground::expire(d, env.now, out);
    brief::expire(d, env.now, out);
    crate::follows::fire(d, env.now);
    read_line::prune_makings(d);
    crate::closer::fire(d, env, out);
    release_reads::fire(d, env, out);
    read_line::fire(d, env.now, out);
    goal_branch::fire(d, env, out);
    ci_news::fire(d, env, out);
    admission::fire(d, env, out);
    outage::fire(d, env, out);
    crate::landing::fire_reads(d, env, out);
    crate::tiers::fire(d, env, out);
    let mut goals = List::with_capacity(env.limits.issues);
    for (goal, when) in &d.projection_due {
        if *when <= env.wall && goals.room() > 0 {
            goals.push(*goal).expect("bounded ready projection");
        }
    }
    for goal in &goals {
        if repository_reads::projection_fresh(d, *goal) {
            d.projection_due.remove(goal);
            emit(out, Request::ProjectAfter { goal: *goal, when: None });
        } else {
            d.projection_waits.insert(*goal).expect("one wait per issue");
        }
    }
    let mut changes = List::with_capacity(env.limits.changes);
    for (task, when) in &d.change_due {
        if *when <= env.wall && changes.room() > 0 {
            changes.push(*task).expect("bounded ready change");
        }
    }
    for task in &changes {
        d.change_due.remove(task);
        emit(out, Request::WakeProcedure { task: *task });
    }
    read_session::fire(d, env, out);
    retry_gathers(d, env, out);
    let mut landings = List::with_capacity(env.limits.landings);
    for (task, when) in &d.landing.due {
        if *when <= env.wall && landings.room() > 0 {
            landings.push(*task).expect("bounded due landings");
        }
    }
    for task in &landings {
        d.landing.due.remove(task);
        emit(out, Request::WakeProcedure { task: *task });
    }
    writes::readback::drive(d, env, out);
    let child_env = child_env(env);
    skein_forge_client::fire(&mut d.client, &child_env, &mut d.client_out);
    drain(d, env, out);
    crate::room::ask(d, env, out);
    if crate::room::fire(d, env, out) {
        crate::room::ask(d, env, out);
    }
}

/// Give a forge its writer once access is known (domain/forge/adoption-connection.md, 1).
/// A refusal keeps neither a writer nor a durable row.
pub fn writer(d: &mut Domain, env: &Env<Limits>, writer: skein_forge_client::Writer, out: &mut Queue<Request>) -> bool {
    if !skein_forge_client::writer(&mut d.client, &child_env(env), writer, &mut d.client_out) {
        return false;
    }
    d.writers.insert(writer.forge, writer.author).expect("client writer fits the connector's table");
    drain(d, env, out);
    true
}

fn child_env(env: &Env<Limits>) -> Env<skein_forge_client::Limits> {
    env.with(env.limits.client)
}

pub(crate) fn child(d: &mut Domain, env: &Env<Limits>, event: skein_forge_client::Event, out: &mut Queue<Request>) {
    if let skein_forge_client::Event::Fence { resource } = &event {
        writes::kept::fence(d, env, resource);
    }
    outage::track(d, env, &event);
    client_step(d, env, event, out);
    outage::refresh(d, env, out);
}

// An outage retry retains its exact read and its backoff in the outage table.
fn client_step(d: &mut Domain, env: &Env<Limits>, event: skein_forge_client::Event, out: &mut Queue<Request>) {
    use skein_forge_client::{Event as ClientEvent, Request as ClientRequest};
    if let ClientEvent::Read { owner, .. } = &event {
        let waiter = d.read_line.take_ask(*owner);
        if waiter.is_none() && d.read_line.waiting() {
            d.client_out.push(ClientRequest::Read { owner: *owner, result: Err(skein_forge::api::Error::Busy) });
        } else {
            let owner = *owner;
            let before = d.client_out.len();
            skein_forge_client::step(&mut d.client, &child_env(env), event, &mut d.client_out);
            if let Some(waiter) = waiter {
                // An entrance refusal is synchronous; an admitted read has no terminal here.
                let mut refusal = None;
                for request in d.client_out.iter().skip(usize::try_from(before).expect("queue length fits usize")) {
                    if let ClientRequest::Read { owner: refused, result: Err(error) } = request
                        && *refused == owner
                    {
                        refusal = Some(*error);
                        break;
                    }
                }
                read_line::attempted(d, waiter, refusal, env.now, out);
            }
        }
    } else {
        skein_forge_client::step(&mut d.client, &child_env(env), event, &mut d.client_out);
    }
    drain(d, env, out);
}

#[cfg(test)]
pub(crate) fn scripted_outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    task: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    d.client_out.push(skein_forge_client::Request::Outcome { entry, task, outcome });
    drain(d, env, out);
}

#[expect(
    clippy::too_many_lines,
    reason = "one client-output cohort updates connector state before routing its outcomes"
)]
fn drain(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    full::update(d, env, out);
    loop {
        writes::kept::sync_heads(d, env, out);
        let Some(request) = d.client_out.pop() else { break };
        match request {
            skein_forge_client::Request::Settled { left } => {
                let mut unfinished = List::with_capacity(env.limits.entries);
                for entry in left {
                    let row = d.entries.get(&entry).expect("client’s unfinished entry remains durable");
                    unfinished.push(crate::Unfinished { entry, task: row.entry.task }).expect("bounded entries");
                }
                emit(out, Request::Settled { unfinished: unfinished.into_boxed() });
            }
            skein_forge_client::Request::Save { record } => {
                let repository = match &record {
                    skein_forge_client::Stored::Live(row) => Some(row.watch.resource.repository),
                    skein_forge_client::Stored::Repository(row) => Some(row.repository),
                    skein_forge_client::Stored::Writer(_) => None,
                };
                if let Some(repository) = repository
                    && !d.repositories.contains_key(&repository)
                {
                    continue;
                }
                let record = match record {
                    skein_forge_client::Stored::Writer(row) => Stored::Writer(row),
                    row @ (skein_forge_client::Stored::Live(_) | skein_forge_client::Stored::Repository(_)) => {
                        Stored::Client(row)
                    }
                };
                emit(out, Request::Save { read_afresh: None, release: None, record });
            }
            skein_forge_client::Request::Erase { key } => {
                match &key {
                    skein_forge_client::Key::Repository(repository) => {
                        if release::contains(d, *repository) {
                            release::first_write(d);
                        }
                    }
                    skein_forge_client::Key::Live(_) | skein_forge_client::Key::Writer(_) => {}
                }
                emit(out, Request::Erase { key: Key::Client(key) });
            }
            skein_forge_client::Request::Progress { entry } => {
                if d.entries.contains_key(&entry.number) && d.repositories.contains_key(&entry.repository) {
                    save_entry_progress(d, entry, None, out);
                }
            }
            skein_forge_client::Request::Outcome { entry, task, outcome } => {
                let Some(kept) = d.entries.get(&entry) else {
                    // Keep the outbox's idempotent erasure, but no settled facts or terminal for an unheld entry.
                    if d.releasing.withdrawing != Some(entry) {
                        release::first_write(d);
                        emit(out, Request::Erase { key: Key::Entry(entry) });
                    }
                    continue;
                };
                kept.check_owner(entry, task);
                let looked = outcome != skein_forge_client::Outcome::Queued && d.entry_looks.remove(&entry);
                // The landing owns merge readback and round cleanup; neither creates a root effect hold.
                let mut landing_merge = false;
                let mut landing_cleanup = false;
                if let Some(row) = d.landing.rows.get(&task) {
                    for item in &row.entries {
                        if item.entry == entry {
                            landing_merge = item.operation == crate::Operation::Merge;
                            landing_cleanup =
                                item.operation == crate::Operation::Close || item.operation == crate::Operation::Delete;
                        }
                    }
                }
                let landing_settles = landing_merge || landing_cleanup;
                if !landing_settles && outage::outcome(d, env, entry, outcome, out) {
                    if looked {
                        wake_once(out, task);
                    }
                    continue;
                }
                writes::outbox::outcome(d, task, entry, outcome, out);
                if outcome == skein_forge_client::Outcome::Held && goal_branch::remake_held(d, env, entry, out) {
                    continue;
                }
                let account = if landing_merge {
                    None
                } else {
                    match settled_account(d, entry, outcome) {
                        Some(account) => Some(account.words),
                        None => None,
                    }
                };
                let (account, cleanup_account) = if landing_cleanup { (None, account) } else { (account, None) };
                let mut projection = false;
                for (_, row) in &d.issues {
                    projection |= row.pending == Some(entry);
                }
                let lost_comment = projection && outcome == skein_forge_client::Outcome::Held && posting(d, entry);
                d.named_outcome(entry, outcome);
                outcome_facts(d, env, entry, task, outcome, out);
                writes::kept::rejudge_effect(d, env, out);
                crate::follow_check::outcome(d, entry, outcome, out);
                crate::tiers::outcome(d, entry, outcome, out);
                crate::landing::outcome(d, env, entry, outcome, cleanup_account, out);
                match outcome {
                    skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {}
                    skein_forge_client::Outcome::Held if !landing_settles => {
                        if lost_comment {
                            // domain/forge/issues.md: an unfindable milestone counts
                            // as posted. Retire its entry without sending it again.
                            not_made(d, env, task, entry, out);
                        } else {
                            // It stays in the outbox, keeping its room, until a
                            // person or its task's close settles it.
                            if let Some(row) = d.entries.get_mut(&entry) {
                                row.held = true;
                                let record = Stored::Entry { entry: row.entry.clone(), held: true };
                                emit(out, Request::Save { read_afresh: None, release: None, record });
                            }
                            erase_flight(d, task, entry, out);
                        }
                    }
                    skein_forge_client::Outcome::Made { .. }
                    | skein_forge_client::Outcome::Failed(_)
                    | skein_forge_client::Outcome::Raced { .. }
                    | skein_forge_client::Outcome::Held
                    | skein_forge_client::Outcome::Withdrawn => {
                        if !d.waiting_release_outcomes.contains_key(&entry) {
                            take_entry(d, entry, task);
                            full::update(d, env, out);
                            emit(out, Request::Erase { key: Key::Entry(entry) });
                        }
                        erase_flight(d, task, entry, out);
                    }
                }
                if looked {
                    wake_once(out, task);
                }
                if projection {
                    emit(out, Request::ProjectionOutcome { entry, outcome });
                } else {
                    emit(out, Request::Outcome { entry, task, outcome, account });
                }
                if outcome != skein_forge_client::Outcome::Uncertain
                    && outcome != skein_forge_client::Outcome::Queued
                    && match d.releases.get(&task) {
                        Some(row) => !row.failed,
                        None => false,
                    }
                {
                    continue_release_once(out, task);
                }
            }
            skein_forge_client::Request::Changed { resource, result } => {
                if let Err(skein_forge::api::Error::Unavailable | skein_forge::api::Error::Timeout) = &result {
                    outage::observed(d, env, resource.repository.forge, true);
                }
                changed(d, env, resource, result, out);
            }
            skein_forge_client::Request::Moved { resource, expected, observed, actor } => {
                writes::kept::moved(d, env, &resource, expected, observed, actor, out);
            }
            skein_forge_client::Request::Call { call, repository, op } => {
                outage::called(d, env, call, repository, &op);
                emit(out, Request::Call { call, repository, op });
            }
            skein_forge_client::Request::Kept { owner, result } => {
                admission::kept(d, env, owner, result, out);
            }
            skein_forge_client::Request::Read { owner, result } => {
                if let Some(read) = d.lost.get_mut(&owner) {
                    read.last_answer = result.as_ref().err().copied();
                }
                if outage::read_ended(d, env, owner, &result, out) {
                    crate::landing::read_unavailable(d, owner);
                    if let Err(error) = result {
                        crate::closer::selection_unavailable(d, owner, error);
                    }
                    continue;
                }
                if writes::readback::reading(d, owner) {
                    writes::readback::answered(d, env, result, out);
                    continue;
                }
                if d.closing.selection_reads.contains_key(&owner) {
                    crate::closer::selection_answered(d, env, owner, result, out);
                } else if d.closing.reads.contains_key(&owner) {
                    crate::closer::answered(d, env, owner, result, out);
                } else if d.follow_reads.contains_key(&owner) {
                    crate::follows::answered(d, env, owner, result, out);
                } else if d.tier_ops.reads.contains_key(&owner) {
                    crate::tiers::answered(d, env, owner, result, out);
                } else if d.landing.pending.contains_key(&owner) {
                    crate::landing::answered(d, env, owner, result, out);
                } else if d.capabilities.contains_key(&owner) {
                    crate::capabilities::answered(d, env, owner, result, out);
                } else if d.brief_fetches.contains_key(&owner) || d.document_fetches.contains_key(&owner) {
                    brief::brief_answer(d, owner, result, env.limits.client.rows, env.now, out);
                } else if crate::drafts::reading(d, owner) {
                    crate::drafts::answer(d, env, owner, result, out);
                } else if d.adoptions.contains_key(&owner) {
                    adoption::adoption_read(d, env, owner, result, out);
                } else if d.landings.contains_key(&owner) {
                    landing_read(d, env, owner, result, out);
                } else if d.goal_file_reads.contains_key(&owner) {
                    topics::answer(d, env, owner, result, out);
                } else if d.makings.contains_key(&owner) {
                    goal_branch::answered(d, env, owner, result, out);
                } else if let Some(task) = crate::unmade::reader(d, owner) {
                    crate::unmade::answered(d, env, task, result, out);
                } else if d.steps.contains_key(&owner) {
                    step_read(d, env, owner, result, out);
                } else if d.pending_ci.contains_key(&owner) {
                    ci_read(d, env, owner, result, out);
                } else if d.workflows.reading(owner) {
                    crate::workflows::answered(d, env, owner, result, out);
                } else if crate::is_root_read(owner) {
                    emit(out, Request::Read { owner, result });
                } else {
                    lost_read(d, env, owner, result, out);
                }
            }
            skein_forge_client::Request::ReadAfreshDone => {
                if !d.stopping {
                    d.restart_fresh = Some(true);
                }
            }
            skein_forge_client::Request::OutboxDone => restart_outbox_done(d, env, out),
        }
    }
    read_afresh_complete(d, env, out);
}

pub(crate) fn restart_outbox_done(d: &mut Domain, _env: &Env<Limits>, out: &mut Queue<Request>) {
    if !d.stopping {
        d.restart_settling = false;
        emit(out, Request::RestartDone { stage: RestartStage::Settled });
    }
}

#[expect(clippy::too_many_lines, reason = "merge outcomes save the landing and its verification together")]
fn outcome_facts(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    task: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    let forbidden = match outcome {
        skein_forge_client::Outcome::Failed(
            skein_forge::api::Error::Forbidden | skein_forge::api::Error::Protected,
        ) => true,
        skein_forge_client::Outcome::Made { .. }
        | skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Uncertain
        | skein_forge_client::Outcome::Queued
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => false,
    };
    if forbidden && let Some(row) = d.entries.get(&entry) {
        let repository = row.entry.repository;
        crate::capabilities::refresh(d, env, repository, out);
    }
    projection_outcome(d, env, entry, outcome, out);
    change_outcome(d, env, entry, outcome, out);
    release_outcome(d, env, entry, outcome, out);
    goal_branch::outcome(d, entry, outcome, out);
    match outcome {
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Branch(commit), .. } => {
            if let Some(Outboxed { entry: kept, .. }) = d.entries.get(&entry) {
                let branch = match &kept.effect.write {
                    skein_forge::api::Write::CreateBranch { branch, .. } => Some(branch.clone()),
                    skein_forge::api::Write::CreateIssue { .. }
                    | skein_forge::api::Write::OpenPull { .. }
                    | skein_forge::api::Write::Post { .. }
                    | skein_forge::api::Write::Review { .. }
                    | skein_forge::api::Write::Edit { .. }
                    | skein_forge::api::Write::SetReviewers { .. }
                    | skein_forge::api::Write::Close { .. }
                    | skein_forge::api::Write::Reopen { .. }
                    | skein_forge::api::Write::Merge { .. }
                    | skein_forge::api::Write::Update { .. }
                    | skein_forge::api::Write::Status { .. }
                    | skein_forge::api::Write::Dispatch { .. }
                    | skein_forge::api::Write::Rerun { .. }
                    | skein_forge::api::Write::DeleteBranch { .. } => None,
                };
                if let Some(branch) = branch {
                    let resource = skein_forge_client::Resource {
                        repository: kept.repository,
                        what: skein_forge_client::What::Branch(branch),
                    };
                    if let Some(name) = from_client(&resource, &env.limits) {
                        let head = BranchHead { name: name.clone(), commit, owned: true };
                        let concerning = heads::concerning(d, &name, Some(task));
                        if heads::record(d, env, head.clone(), concerning, out) {
                            emit(
                                out,
                                Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) },
                            );
                        }
                    }
                }
            }
        }
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(commit), .. } => {
            let turn = d.landing.rows.contains_key(&task);
            let task = crate::landing::merged_owner(d, task, entry);
            if let Some(row) = d.changes.get(&task).cloned()
                && landed_commit(&row.change.state).is_none()
            {
                reds::landed(d, &env.limits, &row, out);
            }
            let signed = match d.changes.get(&task) {
                Some(row) => row.signed.clone(),
                None => Box::new([]),
            };
            if !turn {
                crate::landing_log::landed(d, env.wall, commit, task, signed, out);
                if !d.releases.contains_key(&task) {
                    crate::verification::landed(d, commit, task, out);
                    if let Some(row) = d.changes.get_mut(&task)
                        && landed_commit(&row.change.state).is_none()
                    {
                        let owned = row.change.owns_turn;
                        row.change.owns_turn = false;
                        row.change.state = change::State::Landed { merge: commit, since: env.wall };
                        let mut saved = row.clone();
                        let mut event = crate::change_history::step(d, &saved, crate::history::What::Landed);
                        event.head = Some(commit);
                        crate::change_history::write(d, env.wall, &mut saved, event, out);
                        wake_after_turn(d, &saved, owned);
                        save_change(d, out, saved);
                    }
                }
            }
            landed::landed_room(d, env, commit, task, out);
            if let Some(change) = d.changes.get(&task) {
                let repository = change.repository;
                let branch = change.base.clone();
                let before = match writes::branch(&env.limits, repository, &branch) {
                    Some(name) => d.branch_head(&name),
                    None => None,
                };
                crate::tiers::observed(d, env, repository, &branch, before, commit, out);
            }
        }
        skein_forge_client::Outcome::Made {
            made:
                skein_forge_client::Made::Created(_)
                | skein_forge_client::Made::Commented(_)
                | skein_forge_client::Made::Reviewed(_)
                | skein_forge_client::Made::Updated(_)
                | skein_forge_client::Made::Set,
            ..
        }
        | skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Uncertain
        | skein_forge_client::Outcome::Queued
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => {}
    }
}

pub(crate) fn emit(out: &mut Queue<Request>, request: Request) {
    out.push(request);
}

fn save_hold(out: &mut Queue<Request>, hold: Hold) {
    let read_afresh = if hold.writer.is_none() { Some(hold.name.clone()) } else { None };
    emit(out, Request::Save { record: Stored::Hold(hold), release: None, read_afresh });
}

// Saved-work branches join their task's release without a hub holding or a head read
// (temper/docs/design/domain/forge/branches.md).
fn saved_work(d: &mut Domain, env: &Env<Limits>, task: u64, branches: &[Name], out: &mut Queue<Request>) {
    for name in branches {
        if !valid_name(d, &env.limits, name) || d.holds.contains_key(name) || d.holds.len() == env.limits.holds {
            continue;
        }
        let row =
            Hold { beside: None, design_writers: Box::new([]), name: name.clone(), task, writer: None, drift: None };
        d.holds.insert(name.clone(), row.clone()).expect("saved-work hold capacity checked");
        emit(out, Request::Save { record: Stored::Hold(row), release: None, read_afresh: None });
    }
}

fn valid_name(d: &Domain, l: &Limits, name: &Name) -> bool {
    if !d.repositories.contains_key(&skein_forge::api::Repository { forge: name.forge, repository: name.repository }) {
        return false;
    }
    match &name.what {
        What::Repository | What::Pull(_) | What::Issue(_) => true,
        What::Branch(parts) => {
            if parts.is_empty() {
                return false;
            }
            let mut bytes = 0usize;
            for part in parts {
                if part.is_empty() || part.contains(&b'/') {
                    return false;
                }
                let Some(size) = bytes.checked_add(part.len()) else { return false };
                let Some(next) = size.checked_add(1) else {
                    return false;
                };
                bytes = next;
            }
            bytes <= skein_lib::usize_of(l.name_bytes)
        }
    }
}

pub(crate) fn to_client(name: &Name, l: &Limits) -> Option<skein_forge_client::Resource> {
    let repository = skein_forge::api::Repository { forge: name.forge, repository: name.repository };
    let what = match &name.what {
        What::Repository => skein_forge_client::What::Repository,
        What::Pull(number) => skein_forge_client::What::Pull(*number),
        What::Issue(number) => skein_forge_client::What::Issue(*number),
        What::Branch(parts) => {
            let mut bytes = List::with_capacity(l.name_bytes);
            for (index, part) in parts.iter().enumerate() {
                if index > 0 && bytes.push(b'/').is_err() {
                    return None;
                }
                for byte in part {
                    if bytes.push(*byte).is_err() {
                        return None;
                    }
                }
            }
            skein_forge_client::What::Branch(bytes.into_boxed())
        }
    };
    Some(skein_forge_client::Resource { repository, what })
}

pub(crate) fn participating(d: &Domain, resource: &skein_forge_client::Resource) -> bool {
    let (skein_forge_client::What::Issue(number) | skein_forge_client::What::Pull(number)) = &resource.what else {
        return false;
    };
    for (_, subscriber) in &d.subscriptions {
        if subscriber.topic == (Topic::Participation { repository: resource.repository, number: *number }) {
            return true;
        }
    }
    false
}

fn names(d: &mut Domain, env: &Env<Limits>, task: u64, resources: Box<[Name]>, out: &mut Queue<Request>) {
    admission::names(d, env, task, resources, out);
}

fn unnamed(d: &mut Domain, env: &Env<Limits>, task: u64, keep: bool, out: &mut Queue<Request>) {
    let mut list: List<skein_forge_client::Watch> = List::with_capacity(env.limits.client.resources);
    for (owner, row) in &d.names {
        if *owner == task {
            continue;
        }
        for name in row {
            if d.forge_ended(name.forge) || goal_run(d, name) {
                continue;
            }
            let Some(resource) = to_client(name, &env.limits) else { continue };
            let mut present = false;
            for watch in &list {
                if watch.resource == resource {
                    present = true;
                }
            }
            let participating = participating(d, &resource);
            if !present && list.push(skein_forge_client::Watch { resource, participating }).is_err() {
                crate::refusal::resource_limit(
                    out,
                    task,
                    b"unwatch resources",
                    Some(name.clone()),
                    "forge.client.resources",
                    env.limits.client.resources,
                    u64::from(list.len()).saturating_add(1),
                );
                return;
            }
        }
    }
    for name in &crate::tiers::watched(d) {
        if let Some(resource) = to_client(name, &env.limits) {
            let mut present = false;
            for watch in &list {
                if watch.resource == resource {
                    present = true;
                }
            }
            if !present && list.push(skein_forge_client::Watch { resource, participating: false }).is_err() {
                crate::refusal::resource_limit(
                    out,
                    task,
                    b"unwatch resources",
                    Some(name.clone()),
                    "forge.client.resources",
                    env.limits.client.resources,
                    u64::from(list.len()).saturating_add(1),
                );
                return;
            }
        }
    }
    skein_forge_client::step(
        &mut d.client,
        &child_env(env),
        skein_forge_client::Event::Keep { owner: Token::new(task), watches: list.into_boxed() },
        &mut d.client_out,
    );
    if admission::accepted(&d.client_out)
        && let Some(resources) = d.names.remove(&task)
    {
        if keep {
            keep_names(d, env, task, resources, out);
        } else {
            emit(out, Request::Erase { key: Key::Names(task) });
            release::first_write(d);
        }
        crate::removal::unnamed(d, env, out);
    }
    drain(d, env, out);
}

// Keep repository names alone, in name order (temper/docs/design/domain/forge/adoption-connection.md, 1).
fn keep_names(d: &mut Domain, env: &Env<Limits>, task: u64, resources: Box<[Name]>, out: &mut Queue<Request>) {
    let mut repositories = Map::with_capacity(env.limits.resources_per_task);
    for name in resources {
        repositories.insert(Name { what: What::Repository, ..name }, ()).expect("bounded task resources");
    }
    let mut resources = List::with_capacity(env.limits.resources_per_task);
    for (name, ()) in &repositories {
        resources.push(name.clone()).expect("bounded task repositories");
    }
    let resources = resources.into_boxed();
    if d.kept.len() == d.kept.capacity() {
        let mut oldest = None;
        for (owner, row) in &d.kept {
            if names_covered(d, *owner, row, task, &resources) {
                oldest = Some(*owner);
                break;
            }
        }
        if names_covered(d, task, &resources, task, &resources) {
            oldest = Some(match oldest {
                Some(owner) => owner.min(task),
                None => task,
            });
        }
        let oldest = oldest.expect("at most one last naming task per repository");
        emit(out, Request::Erase { key: Key::Names(oldest) });
        if oldest == task {
            return;
        }
        d.kept.remove(&oldest);
    }
    d.kept.insert(task, resources.clone()).expect("bounded kept names");
    emit(out, Request::Save { record: Stored::Names { task, resources }, release: None, read_afresh: None });
}

fn names_covered(d: &Domain, task: u64, resources: &[Name], released: u64, releasing: &[Name]) -> bool {
    for name in resources {
        let mut covered = false;
        for (owner, row) in d.names.iter().chain(d.kept.iter()) {
            if *owner > task {
                for later in row {
                    covered |= name.forge == later.forge && name.repository == later.repository;
                }
            }
        }
        if released > task {
            for later in releasing {
                covered |= name.forge == later.forge && name.repository == later.repository;
            }
        }
        if !covered {
            return false;
        }
    }
    true
}

fn hold(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    task: u64,
    resource: Name,
    from: Option<u64>,
    out: &mut Queue<Request>,
) {
    let holdable = match d.resource_facts(&resource) {
        Some((_, crate::resources::HoldKind::Exclusive { .. })) => true,
        Some((_, crate::resources::HoldKind::Shared)) | None => false,
    };
    if let Err((resource, cause)) = admission::validate(d, &env.limits, &resource) {
        emit(out, Request::ResourceRefused { task, operation: b"hold resource", resource, cause });
        return;
    }
    if !holdable {
        emit(out, Request::Refused { task });
        return;
    }
    if !d.holds.contains_key(&resource) && d.holds.len() == env.limits.holds {
        crate::refusal::resource_limit(
            out,
            task,
            b"hold resource",
            Some(resource),
            "forge.holds",
            env.limits.holds,
            u64::from(d.holds.len()).saturating_add(1),
        );
        return;
    }
    if let Some(existing) = d.holds.get(&resource) {
        if existing.task != task && Some(existing.task) != from {
            emit(out, Request::Taken { task, resource, by: existing.task });
            return;
        }
    } else if from.is_some() {
        emit(out, Request::Refused { task });
        return;
    }
    let writer = match d.holds.get(&resource) {
        Some(row) => row.writer,
        None => None,
    };
    let drift = match d.holds.get(&resource) {
        Some(row) => row.drift.clone(),
        None => None,
    };
    let design_writers = match d.holds.get(&resource) {
        Some(row) => row.design_writers.clone(),
        None => Box::new([]),
    };
    let row = match d.holds.get_mut(&resource) {
        Some(row) => {
            row.task = task;
            row.writer = writer;
            row.drift = drift;
            row.clone()
        }
        None => Hold { name: resource.clone(), task, writer, design_writers, beside: None, drift },
    };
    d.holds.insert(resource.clone(), row.clone()).expect("preflighted hold capacity");
    save_hold(out, row);
    goal_branch::start(d, env, resource, Some(entry), out);
}

#[expect(clippy::too_many_arguments, reason = "a claim carries slot writes and design writes beside their holders")]
fn claim(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    writes: &[Name],
    designs: &[Name],
    holders: &[u64],
    out: &mut Queue<Request>,
) {
    if writes.len() > skein_lib::usize_of(env.limits.resources_per_task) {
        crate::refusal::resource_limit(
            out,
            task,
            b"claim resources",
            writes.first().cloned(),
            "forge.resources_per_task",
            env.limits.resources_per_task,
            skein_lib::len_u64(writes.len()),
        );
        return;
    }
    if !design_claim_permitted(d, env, task, attempt, designs, holders, out) {
        return;
    }
    for name in writes {
        let Some(row) = d.holds.get(name) else {
            emit(out, Request::Refused { task });
            return;
        };
        if row.task != task && !holders.contains(&row.task) {
            emit(out, Request::Taken { task, resource: name.clone(), by: row.task });
            return;
        }
        if goal_branch::is_goal(d, name) && goal_branch::waiting(d, name) {
            emit(out, Request::Taken { task, resource: name.clone(), by: row.task });
            return;
        }
        if retired_reading(d, name) {
            emit(out, Request::Taken { task, resource: name.clone(), by: row.task });
            return;
        }
        match row.writer {
            Some(Writer::Run { task: owner, attempt: old } | Writer::Lost { task: owner, attempt: old })
                if owner == task && old > attempt =>
            {
                return;
            }
            Some(Writer::Run { task: owner, .. } | Writer::Lost { task: owner, .. }) if owner == task => {}
            Some(Writer::Run { task: owner, .. } | Writer::Lost { task: owner, .. }) => {
                emit(out, Request::Taken { task, resource: name.clone(), by: owner });
                return;
            }
            Some(Writer::Entry(entry)) => {
                emit(out, Request::Taken { task, resource: name.clone(), by: entry });
                return;
            }
            None => {}
        }
    }
    for name in designs {
        let row = d.holds.get_mut(name).expect("preflighted design hold");
        let mut designers = List::with_capacity(env.limits.holds);
        for writer in &row.design_writers {
            if writer.task != task {
                designers.push(*writer).expect("preflighted design writer capacity");
            }
        }
        designers
            .push(crate::DesignWriter { task, attempt, rereading: false })
            .expect("preflighted design writer capacity");
        row.design_writers = designers.into_boxed();
        save_hold(out, row.clone());
    }
    for name in writes {
        let Some(row) = d.holds.get_mut(name) else { unreachable!("preflighted hold") };
        row.writer = Some(Writer::Run { task, attempt });
        let saved = row.clone();
        save_hold(out, saved);
    }
    // A read begun while the delegate was idle must see a run that claims
    // before it finishes (temper/docs/design/domain/forge/change/repairs.md, A resolution's moved head).
    for (_, row) in &d.changes {
        if let Some((delegate, _)) = row.delegate
            && delegate == task
            && let Some(owner) = step_owner(d, row.task)
            && let Some(pending) = d.steps.get_mut(&owner)
        {
            pending.heard.delegate_running = true;
            break;
        }
    }
}

fn matches_later_claim(d: &Domain, task: u64, attempt: u64) -> bool {
    match d.waiting_claims.get(&task) {
        Some((newer, ..)) => *newer > attempt,
        None => false,
    }
}

fn forget_waiting_claim(d: &mut Domain, task: u64, attempt: u64) {
    if !matches_later_claim(d, task, attempt) {
        d.waiting_claims.remove(&task);
    }
}

fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    pushed: &[(Name, skein_forge::api::Commit, bool)],
    out: &mut Queue<Request>,
) {
    forget_waiting_claim(d, task, attempt);
    design::ended(d, env, task, attempt, out);
    let mut freed = List::with_capacity(env.limits.holds);
    for (name, row) in &d.holds {
        let mut reported = false;
        for (pushed_name, _, _) in pushed {
            if pushed_name == name {
                reported = true;
            }
        }
        if row.writer == Some(Writer::Run { task, attempt }) && reported {
            freed.push(name.clone()).expect("list sized to holds");
        }
    }
    for name in &freed {
        let Some(row) = d.holds.get_mut(name) else { unreachable!("collected hold") };
        row.writer = None;
        let saved = row.clone();
        save_hold(out, saved);
        let undrifted = row.drift.is_none();
        for (pushed_name, commit, checked) in pushed {
            if pushed_name == name && undrifted {
                record_delegate_head(d, env, task, name, *commit, *checked, out);
                crate::change_history::delivered(d, env.wall, task, name, *commit, *checked, out);
            }
        }
        lost_release(d, env, name, task, out);
    }
}

/// A delegate's last push, including an unchecked save, pins its next brief
/// without completing its delegation (temper/docs/design/domain/agent/README.md, 5).
fn record_delegate_head(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    name: &Name,
    commit: skein_forge::api::Commit,
    checked: bool,
    out: &mut Queue<Request>,
) {
    let mut pushed = None;
    for (_, row) in &d.changes {
        if let Some((child, _)) = row.delegate
            && child == task
            && row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
            && (row.change.last_head != Some(commit)
                || (checked && (row.change.last_checked != Some(commit) || row.change.last_checked_by != Some(task))))
        {
            let mut row = row.clone();
            if row.change.last_head != Some(commit) {
                row.regate = None;
            }
            row.change.last_head = Some(commit);
            if checked {
                row.change.last_checked = Some(commit);
                row.change.last_checked_by = Some(task);
            }
            pushed = Some(row);
            break;
        }
    }
    if let Some(mut row) = pushed {
        if d.branch_head(name) != Some(commit) {
            let mut event = crate::change_history::step(d, &row, crate::history::What::Pushed);
            event.head = Some(commit);
            crate::change_history::write(d, env.wall, &mut row, event, out);
        }
        // A checked report can add an early-start fact without moving the branch tip.
        let previous = d.changes.get(&row.task).expect("retained change").change.last_checked;
        if checked {
            wake_head_dependents(d, name, previous, commit);
        }
        d.changes.insert(row.task, row.clone()).expect("reported head belongs to a retained change");
        save_change(d, out, row);
    }
}

fn lost(d: &mut Domain, env: &Env<Limits>, task: u64, attempt: u64, out: &mut Queue<Request>) {
    forget_waiting_claim(d, task, attempt);
    design::ended(d, env, task, attempt, out);
    let mut names = List::with_capacity(env.limits.holds);
    for (name, row) in &d.holds {
        if d.forge_ended(name.forge) {
            continue;
        }
        if row.writer == Some(Writer::Run { task, attempt }) || row.writer == Some(Writer::Lost { task, attempt }) {
            names.push(name.clone()).expect("list sized to holds");
        }
    }
    for name in &names {
        let Some(row) = d.holds.get_mut(name) else { unreachable!("collected hold") };
        row.writer = Some(Writer::Lost { task, attempt });
        save_hold(out, row.clone());
        let Some(resource) = to_client(name, &env.limits) else { continue };
        let read = match resource.what {
            skein_forge_client::What::Branch(branch) => skein_forge::api::Read::Branch { branch },
            skein_forge_client::What::Repository
            | skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Issue(_) => {
                lost_failed(d, task, attempt, name, b"lost writer does not name a branch", out);
                continue;
            }
        };
        let Some(sequence) = d.sequence.checked_add(1) else {
            lost_failed(d, task, attempt, name, b"lost-writer read sequence exhausted", out);
            return;
        };
        d.sequence = sequence;
        let owner = Token::new(sequence | (1_u64 << 62));
        if d.lost
            .insert(
                owner,
                PendingLost {
                    task,
                    attempt,
                    name: name.clone(),
                    state: LostReadState::Asked,
                    last_answer: None,
                    kind: LostWriterKind::Slot,
                },
            )
            .is_err()
        {
            lost_failed(d, task, attempt, name, b"lost-writer read capacity exhausted", out);
            return;
        }
        child(d, env, skein_forge_client::Event::Read { owner, repository: resource.repository, read }, out);
    }
}

fn retired_reading(d: &Domain, name: &Name) -> bool {
    for (_, pending) in &d.lost {
        if pending.kind == LostWriterKind::Retired && &pending.name == name {
            return true;
        }
    }
    false
}

/// Read a lost slot before releasing it; only the record can explain its tip.
fn ask_lost_again(d: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let Some(pending) = d.lost.get_mut(&owner) else { return };
    pending.state = LostReadState::Asked;
    let Some(resource) = to_client(&pending.name, &env.limits) else { return };
    let skein_forge_client::What::Branch(branch) = resource.what else { unreachable!("a lost writer holds a branch") };
    child(
        d,
        env,
        skein_forge_client::Event::Read {
            owner,
            repository: resource.repository,
            read: skein_forge::api::Read::Branch { branch },
        },
        out,
    );
}

fn lost_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    use skein_forge::api::{Answer, Error};
    let Some(mut pending) = d.lost.remove(&owner) else { return };
    let Some(row) = d.holds.get(&pending.name) else { return };
    let owns_slot = match pending.kind {
        LostWriterKind::Slot => row.writer == Some(Writer::Lost { task: pending.task, attempt: pending.attempt }),
        LostWriterKind::Retired => row.writer.is_none(),
    };
    if !owns_slot {
        return;
    }
    let waiting = match &result {
        Err(Error::Busy) => Some(LostReadState::WaitingForRoom),
        Err(Error::NotConnected) => Some(LostReadState::WaitingForConnection),
        Ok(_) | Err(_) => None,
    };
    if let Some(state) = waiting {
        pending.state = state;
        d.lost.insert(owner, pending).expect("the read keeps its own lost slot");
        if state == LostReadState::WaitingForRoom {
            read_line::join(d, read_line::Waiter::Lost(owner), env.now, out);
        }
        return;
    }
    match result {
        Ok(Answer::Commit(commit)) => {
            if pending.kind == LostWriterKind::Slot && row.drift.is_none() {
                pin_ending_push(d, env, &pending, commit, out);
            }
        }
        Err(Error::Missing) => {}
        Err(error) => {
            failed_lost_read(d, owner, pending, error, out);
            return;
        }
        Ok(_) => {
            failed_lost_read(d, owner, pending, Error::InvalidAnswer, out);
            return;
        }
    }
    let row = d.holds.get_mut(&pending.name).expect("the slot still belongs to this read");
    row.writer = None;
    save_hold(out, row.clone());
    branch_due_for(d, &pending.name);
    lost_release(d, env, &pending.name, pending.task, out);
}

/// Pin the delegate's admitted save before making the slot claimable again.
fn pin_ending_push(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &PendingLost,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let sender = crate::Sender::Attempt { task: pending.task, attempt: pending.attempt };
    if writes::observed_push(d, &pending.name, sender, commit, out) {
        record_delegate_head(d, env, pending.task, &pending.name, commit, false, out);
    }
}

fn branch_due_for(d: &mut Domain, name: &Name) {
    for (task, row) in &d.changes {
        if row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
            && let Some(read) = d.settling.get_mut(task)
        {
            read.branch.due = true;
        }
    }
}

fn failed_lost_read(
    d: &mut Domain,
    owner: Token,
    mut pending: PendingLost,
    error: skein_forge::api::Error,
    out: &mut Queue<Request>,
) {
    lost_failed(d, pending.task, pending.attempt, &pending.name, &crate::effect_words::error_words(error), out);
    if pending.kind == LostWriterKind::Retired {
        // The old row remains durable. Reconnecting retries its read; neither a
        // failed observation nor a crash can admit a writer over unknown writes.
        pending.state = LostReadState::WaitingForConnection;
        d.lost.insert(owner, pending).expect("failed retired read retains its slot");
    }
}

fn lost_failed(d: &mut Domain, task: u64, attempt: u64, name: &Name, reason: &[u8], out: &mut Queue<Request>) {
    let words = release_failure::reread(d, name, task, attempt, reason);
    release_reads::failed(d, task, words.clone(), out);
    if let Some(hold) = d.holds.get(name)
        && hold.task != task
    {
        release_reads::failed(d, hold.task, words, out);
    }
    emit(out, Request::LostReadFailed { task, attempt, resource: name.clone(), reason: Box::from(reason) });
}

/// Both the holder and its delegate stay live until the writer's fresh read ends.
fn lost_release(d: &mut Domain, env: &Env<Limits>, name: &Name, writer: u64, out: &mut Queue<Request>) {
    continue_writer_release(d, env, writer, repository_reads::repository(name), out);
    if let Some(hold) = d.holds.get(name)
        && hold.task != writer
    {
        continue_writer_release(d, env, hold.task, repository_reads::repository(name), out);
    }
}

/// A writer's own release waits too, even when its branch belongs to its parent.
fn continue_writer_release(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    repository: skein_forge::api::Repository,
    out: &mut Queue<Request>,
) {
    release_reads::advanced(d, task, env.wall);
    if !repository_fresh(d, repository) || !repository_reads::release_fresh(d, task) {
        if d.releases.contains_key(&task) {
            repository_reads::park_release_in(d, task, repository);
            release_reads::waiting(d, env, task, release_reads::Read::Fresh);
        }
        return;
    }
    let Some(row) = d.releases.get(&task) else { return };
    if row.releasing && !row.failed && row.pending.is_none() {
        continue_release_once(out, task);
    }
}

fn enqueue(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Option<Token>,
    entry: skein_forge_client::Entry,
    out: &mut Queue<Request>,
) {
    let task = entry.task;
    if let Some(owner) = owner {
        d.take_call_room(owner, task);
    }
    let Some(repository) = d.repositories.get(&entry.repository) else {
        emit(out, Request::Refused { task });
        return;
    };
    if repository.role == Role::Context || !effect_permitted(repository.kinds, &entry.effect.write) {
        emit(out, Request::Refused { task });
        return;
    }
    if d.entries.contains_key(&entry.number) {
        emit(out, Request::Refused { task });
        return;
    }
    let path = writes::outbox::change_branch(d, entry.task, entry.repository);
    let Some(plan) = writes::outbox::plan(d, &env.limits, &entry, path, writes::outbox::Pin::NONE) else {
        emit(out, Request::Refused { task });
        return;
    };
    if (owner.is_none() && !crate::room::check(d, &env.limits, None)) || plan.blocked(d).is_some() {
        let outcome = skein_forge_client::Outcome::Failed(skein_forge::api::Error::Busy);
        d.named_outcome(entry.number, outcome);
        emit(out, Request::Outcome { entry: entry.number, task, outcome, account: None });
        return;
    }
    let release = Some(entry.number);
    plan.admit(d, &entry, out);
    keep_entry(d, entry, release, out);
}

/// Take one place for a new entry. A writer with a reservation gives it
/// back before this step (domain/forge/room.md, 2.2).
pub(crate) fn keep_entry(
    d: &mut Domain,
    entry: skein_forge_client::Entry,
    release: Option<u64>,
    out: &mut Queue<Request>,
) {
    let number = entry.number;
    let task = entry.task;
    assert!(
        crate::room::taken(d) < u64::from(d.entries.capacity()),
        "new outbox entry {number} of task {task} has room within forge.entries"
    );
    let owner_ended =
        d.room.closing.get(&task).copied().or(d.full.reserved_owner_ended(full::Reservation::Projection(number)));
    let row = Outboxed { entry: entry.clone(), owner_ended, held: false, handoff: Handoff::Uncommitted };
    let Ok(prior) = d.entries.insert(number, row) else {
        unreachable!("new outbox entry {number} of task {task} fits its checked place");
    };
    assert!(prior.is_none(), "new outbox entry {number} of task {task} has a new number");
    save_entry(d, entry, release, out);
}

/// The client's progress or an outage retry writes over its own place.
pub(crate) fn save_entry_progress(
    d: &mut Domain,
    entry: skein_forge_client::Entry,
    release: Option<u64>,
    out: &mut Queue<Request>,
) {
    let number = entry.number;
    let task = entry.task;
    let Some(kept) = d.entries.get(&number) else {
        unreachable!("outbox entry {number} of task {task} is kept when saving its progress");
    };
    kept.check_owner(number, task);
    writes::outbox::progress(d, &entry, out);
    let kept = d.entries.get_mut(&number).expect("progress keeps the outbox entry");
    kept.entry = entry.clone();
    kept.held = false;
    save_entry(d, entry, release, out);
}

fn save_entry(d: &mut Domain, entry: skein_forge_client::Entry, release: Option<u64>, out: &mut Queue<Request>) {
    if entry.attempt.is_some() {
        let number = entry.number;
        let task = entry.task;
        assert!(d.flights.insert((task, number), ()).is_ok(), "flight for outbox entry {number} of task {task} fits");
        emit(
            out,
            Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::Flight { task: entry.task, entry: entry.number },
            },
        );
    }
    emit(out, Request::Save { read_afresh: None, release, record: Stored::Entry { entry, held: false } });
}

/// Give back a place the retiring step has identified, exactly once.
pub(crate) fn take_entry(d: &mut Domain, number: u64, task: u64) {
    let Some(row) = d.entries.remove(&number) else {
        unreachable!("outbox entry {number} of task {task} is kept until it leaves");
    };
    row.check_owner(number, task);
}

fn effect_permitted(kinds: Kinds, write: &skein_forge::api::Write) -> bool {
    match write {
        skein_forge::api::Write::CreateIssue { .. } => kinds.issue,
        skein_forge::api::Write::OpenPull { .. } => kinds.open,
        skein_forge::api::Write::Post { .. } => kinds.comment,
        skein_forge::api::Write::Review { .. } | skein_forge::api::Write::SetReviewers { .. } => kinds.review,
        skein_forge::api::Write::Edit { .. }
        | skein_forge::api::Write::Close { .. }
        | skein_forge::api::Write::Reopen { .. } => kinds.issue || kinds.open,
        skein_forge::api::Write::Merge { .. } => kinds.land,
        skein_forge::api::Write::Update { .. }
        | skein_forge::api::Write::CreateBranch { .. }
        | skein_forge::api::Write::Dispatch { .. }
        | skein_forge::api::Write::Rerun { .. }
        | skein_forge::api::Write::DeleteBranch { .. } => kinds.branch,
        skein_forge::api::Write::Status { .. } => kinds.status,
    }
}

fn relay_hold(state: &mut change::State, cause: change::Cause, now: skein_lib::Wall) -> bool {
    if let change::State::Ended | change::State::Held { .. } = state {
        return false;
    }
    let was = Box::new(core::mem::replace(state, change::State::Ended));
    *state = change::State::Held { was, hold: change::Hold::new(cause, now) };
    true
}

fn register_change(d: &mut Domain, env: &Env<Limits>, mut row: ChangeRow, out: &mut Queue<Request>) {
    let task = row.task;
    if row.change.task != task || row.branch.is_empty() || row.base.is_empty() || d.changes.contains_key(&task) {
        emit(out, Request::Refused { task });
        return;
    }
    if !d.repositories.contains_key(&row.repository) {
        room::refused(out, b"register change", &row, crate::RefusalCause::Unadopted);
        return;
    }
    for name in [&row.branch, &row.base] {
        if name.len() > skein_lib::usize_of(env.limits.name_bytes) {
            room::refused(
                out,
                b"register change",
                &row,
                crate::RefusalCause::Limit {
                    member: "forge.name_bytes",
                    configured: u64::from(env.limits.name_bytes),
                    needed: u64::try_from(name.len()).expect("name length fits u64"),
                },
            );
            return;
        }
    }
    if d.changes.len() == env.limits.changes {
        room::refused(
            out,
            b"register change",
            &row,
            crate::RefusalCause::Limit {
                member: "forge.changes",
                configured: u64::from(env.limits.changes),
                needed: u64::from(d.changes.len()).saturating_add(1),
            },
        );
        return;
    }
    if let Err(needed) = crate::follow_check::select(d, &mut row, &env.limits) {
        room::refused(
            out,
            b"register change",
            &row,
            crate::RefusalCause::Limit {
                member: "forge.change_policy.gates",
                configured: u64::from(env.limits.change_policy.gates),
                needed,
            },
        );
        return;
    }
    row.change.way = d.landing.way(d.repositories.get(&row.repository).expect("adopted").project);
    d.changes.insert(task, row.clone()).expect("change capacity checked");
    if let Some(head) = row.change.last_head {
        let mut event = crate::change_history::step(d, &row, crate::history::What::Pushed);
        event.head = Some(head);
        crate::change_history::write(d, env.wall, &mut row, event, out);
    }
    save_queue_change(d, env, None, row, out);
}

fn produced_title(d: &mut Domain, task: u64, child: u64, title: &[u8], report: Box<[u8]>, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get_mut(&task) else { return };
    if row.delegate != Some((child, change::Delegate::Produce)) {
        return;
    }
    if row.pull.is_none() && row.title.is_empty() {
        row.title = if title.is_empty() {
            Box::from(DEFAULT_CHANGE_TITLE)
        } else {
            crate::text::fit(title, 256, "…".as_bytes())
        };
    }
    row.report = report;
    let saved = row.clone();
    d.procedure_wakes.insert(task, true).expect("one pending wake per change");
    save_change(d, out, saved);
}

fn delegated(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    child: u64,
    kind: change::Delegate,
    close_at: Option<change::CloseAt>,
    out: &mut Queue<Request>,
) {
    let base = delegate_base(d, task, kind);
    let Some(row) = d.changes.get_mut(&task) else {
        emit(out, Request::Refused { task });
        return;
    };
    if row.frozen() || row.delegate == Some((child, kind)) {
        return;
    }
    match kind {
        change::Delegate::Repair(why) => {
            row.regate = None;
            row.ci_recheck = None;
            row.result.repaired(why, row.change.last_head, env.limits.client.answer_bytes);
            if let Some(base) = base {
                row.result.judged(&base, env.limits.client.answer_bytes);
            }
            row.repair_tasks = task_account(&row.repair_tasks, child);
        }
        change::Delegate::Resolve { .. } => row.resolution_tasks = task_account(&row.resolution_tasks, child),
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Merge { .. }
        | change::Delegate::Gate { .. } => {}
    }
    row.delegate = Some((child, kind));
    if let Some(at) = close_at {
        row.close_at = Some(at);
        row.report = Box::new([]);
    }
    row.delegate_head = row.change.last_head;
    if let Some(read) = d.settling.get_mut(&task) {
        read.branch.producer = change::Status::Unknown;
    }
    match kind {
        change::Delegate::Resolve { .. } | change::Delegate::Merge { .. } => row.merge_result = None,
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Repair(_)
        | change::Delegate::Gate { .. } => {}
    }
    row.producer_failure = None;
    row.effect_account = None;
    row.delegate_status = change::Status::Unknown;
    row.delegate_flake = false;
    row.delegate_no_test = false;
    row.delegate_words = None;
    let mut saved = row.clone();
    crate::change_history::delegated(d, env.wall, &mut saved, child, kind, out);
    d.changes.insert(task, saved.clone()).expect("retained change");
    save_change(d, out, saved);
}

fn delegate_base(d: &Domain, task: u64, kind: change::Delegate) -> Option<skein_forge::api::Commit> {
    match kind {
        change::Delegate::Repair(why) => match d.landing_failure(task, why) {
            Some(failure) => failure.base,
            None => None,
        },
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Resolve { .. }
        | change::Delegate::Merge { .. }
        | change::Delegate::Gate { .. } => None,
    }
}

fn merge_delegate(kind: change::Delegate) -> bool {
    match kind {
        change::Delegate::Merge { .. } => true,
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Repair(_)
        | change::Delegate::Resolve { .. }
        | change::Delegate::Gate { .. } => false,
    }
}

fn task_account(prior: &[u64], child: u64) -> Box<[u64]> {
    let mut tasks = List::uncapped();
    for task in prior {
        tasks.push(*task).expect("uncapped task account");
    }
    if !prior.contains(&child) {
        tasks.push(child).expect("admitted delegate");
    }
    tasks.into_boxed()
}

fn delegate_refused(d: &mut Domain, env: &Env<Limits>, task: u64, child: u64, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get_mut(&task) else { return };
    let before = crate::panel::ready(row);
    let Some((expected, kind)) = row.delegate else { return };
    if expected != child {
        return;
    }
    row.delegate = None;
    row.delegate_status = change::Status::Unknown;
    row.delegate_flake = false;
    row.delegate_no_test = false;
    row.delegate_words = None;
    let owned = row.change.owns_turn;
    row.change.owns_turn = false;
    if let change::State::Held { .. } = row.change.state {
        // Retire the refused delegate without replacing the current hold.
    } else {
        let state = unasked(row, kind);
        row.change.state =
            change::State::Held { was: Box::new(state), hold: change::Hold::new(change::Cause::Failed, env.wall) };
    }
    let saved = row.clone();
    wake_after_turn(d, &saved, owned);
    save_queue_change(d, env, before, saved, out);
}

fn delegate_admission_refused(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    kind: change::Delegate,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get_mut(&task) else { return };
    if row.delegate.is_some() {
        return;
    }
    let asked = match (kind, &row.change.state) {
        (change::Delegate::Produce | change::Delegate::Close, change::State::Producing { requested }) => *requested,
        (change::Delegate::Repair(why), change::State::Repairing { requested, why: current }) => {
            *requested && why == *current
        }
        (
            change::Delegate::Resolve { base } | change::Delegate::Merge { base, .. },
            change::State::Resolving { requested, base: current },
        ) => *requested && base == *current,
        (change::Delegate::Gate { number, head }, change::State::Judging { target, asked, .. }) => {
            head == target.prefix && asked.contains(&number)
        }
        (change::Delegate::Gate { number, head }, change::State::Gating { head: current, asked }) => {
            head == *current && asked.contains(&number)
        }
        _ => false,
    };
    if !asked {
        return;
    }
    let before = crate::panel::ready(row);
    row.change.state = unasked(row, kind);
    match kind {
        change::Delegate::Resolve { .. } | change::Delegate::Merge { .. } => {
            row.change.state = change::State::Held {
                was: Box::new(row.change.state.clone()),
                hold: change::Hold::new(change::Cause::Failed, env.wall),
            };
        }
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Repair(_)
        | change::Delegate::Gate { .. } => {}
    }
    row.delegate_status = change::Status::Unknown;
    let saved = row.clone();
    save_queue_change(d, env, before, saved, out);
}

fn unasked(row: &mut ChangeRow, kind: change::Delegate) -> change::State {
    match (kind, &row.change.state) {
        (change::Delegate::Produce | change::Delegate::Close, change::State::Producing { .. }) => {
            change::State::Producing { requested: false }
        }
        (change::Delegate::Repair(why), change::State::Repairing { .. }) => {
            row.change.repairs = row.change.repairs.saturating_sub(1);
            change::State::Repairing { requested: false, why }
        }
        (
            change::Delegate::Resolve { base } | change::Delegate::Merge { base, .. },
            change::State::Resolving { .. },
        ) => {
            row.change.resolutions = row.change.resolutions.saturating_sub(1);
            change::State::Resolving { requested: false, base }
        }
        (change::Delegate::Gate { number, .. }, change::State::Judging { head, ready_since, target, asked }) => {
            let mut remaining = List::with_capacity(u32::try_from(asked.len()).expect("bounded gates"));
            for prior in asked {
                if *prior != number {
                    remaining.push(*prior).expect("same room");
                }
            }
            change::State::Judging {
                head: *head,
                ready_since: *ready_since,
                target: target.clone(),
                asked: remaining.into_boxed(),
            }
        }
        (change::Delegate::Gate { number, .. }, change::State::Gating { head, asked }) => {
            let mut remaining = List::with_capacity(u32::try_from(asked.len()).expect("bounded gate list"));
            for prior in asked {
                if *prior != number {
                    remaining.push(*prior).expect("same gate room");
                }
            }
            change::State::Gating { head: *head, asked: remaining.into_boxed() }
        }
        _ => row.change.state.clone(),
    }
}

// A result ends a stall at its delegate's wait; a close result also ends its
// failed producing hold (domain/forge/change/repairs.md).
#[expect(clippy::too_many_arguments, reason = "one child's terminal carries its status, failure standing and words")]
#[expect(clippy::too_many_lines, reason = "one delegate result records its answer, ending and readiness together")]
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "the typed delegate event distinguishes a flake from no-change, failure and cancellation"
)]
fn delegate_result(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    child: u64,
    ended_at: Option<skein_lib::Wall>,
    status: change::Status,
    no_change: bool,
    flake: bool,
    no_test: bool,
    failed: bool,
    lost: bool,
    cancelled: bool,
    words: &[u8],
    out: &mut Queue<Request>,
) {
    let base = match d.change(task) {
        Some(row) => match row.delegate {
            Some((_, kind)) => delegate_base(d, task, kind),
            None => None,
        },
        None => None,
    };
    if let Some(row) = d.changes.get(&task) {
        // A gate may end before the pushing child whose slot it took.
        if let Some((expected, change::Delegate::Gate { .. })) = row.delegate
            && expected == child
        {
            crate::waits::changed(d, task);
        } else {
            crate::waits::change_agent_ended(d, task);
        }
    }
    let exempt = read_session::until(d, task).is_none() && d.read_sessions.contains_key(&task);
    if exempt {
        read_session::ended(d, env, task, ended_at.unwrap_or(env.wall), out);
    }
    let Some(row) = d.changes.get_mut(&task) else { return };
    // A pushing child can end after a gate has replaced the delegate slot.
    // Its terminal still releases the procedure's conflict wait.
    d.procedure_wakes.insert(task, true).expect("one pending wake per change");
    let Some((expected, kind)) = row.delegate else { return };
    if expected != child
        || ((crate::change_history::asked(kind).is_some() || merge_delegate(kind) || kind == change::Delegate::Close)
            && (row.delegate_status == change::Status::Passed || row.delegate_status == change::Status::Failed))
    {
        return;
    }
    let fresh = row.ending.is_none() && !row.terminal();
    let owned = row.change.owns_turn;
    let empty_repair =
        if let change::Delegate::Repair(why) = kind { why.ci() || crate::recheck::landing(why) } else { false };
    let agent = match kind {
        change::Delegate::Gate { .. } => false,
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Repair(_)
        | change::Delegate::Resolve { .. }
        | change::Delegate::Merge { .. } => true,
    };
    if agent {
        row.change.since = ended_at.unwrap_or(env.wall).min(env.wall);
        if let Some(read) = d.settling.get_mut(&task) {
            read.branch.due = true;
        }
    }
    // A pushing delegate's terminal needs a branch read sent after its end,
    // even when a step already gathering facts observes the result first.
    if agent {
        let mut gathering = None;
        for (token, pending) in &d.steps {
            if pending.task == task {
                gathering = Some(*token);
            }
        }
        if let Some(gathering) = gathering {
            d.steps.get_mut(&gathering).expect("found pending step").woken = true;
        }
    }
    // Keep the merge's identity through the fresh branch read, including after restart.
    match kind {
        change::Delegate::Merge { .. } | change::Delegate::Close | change::Delegate::Resolve { .. } => {}
        change::Delegate::Produce | change::Delegate::Repair(_) | change::Delegate::Gate { .. } => row.delegate = None,
    }
    d.start_conflicts.remove(&task);
    row.delegate_status =
        if no_change && !empty_repair && kind != change::Delegate::Produce { change::Status::Failed } else { status };
    row.delegate_flake = flake
        && match kind {
            change::Delegate::Repair(_) => true,
            change::Delegate::Produce
            | change::Delegate::Close
            | change::Delegate::Gate { .. }
            | change::Delegate::Resolve { .. }
            | change::Delegate::Merge { .. } => false,
        };
    row.delegate_no_test = no_test && crate::routing::repair_verdict(kind, crate::REPAIR_NO_TEST);
    match kind {
        change::Delegate::Gate { number, head } => {
            if let Err(cause) = record_review(row, env, number, head, (status, failed, cancelled), words) {
                emit(out, Request::ResourceRefused { task, operation: b"record gate review", resource: None, cause });
                return;
            }
        }
        change::Delegate::Close => {
            crate::close::clear_findings(row);
            if row.delegate_status == change::Status::Passed {
                // Keep the raw result until a branch read sent after this end can judge it.
                let bound = match d.close_kind {
                    Some((_, result)) => skein_lib::usize_of(result),
                    None => 0,
                };
                if words.len() <= bound {
                    row.report = Box::from(words);
                } else {
                    row.delegate_status = change::Status::Failed;
                    row.producer_failure = Some(crate::close::failed_words(words, false, false));
                }
            } else if !cancelled {
                row.producer_failure = Some(crate::close::failed_words(words, failed, lost));
            }
        }
        change::Delegate::Produce => producer_result(row, no_change, failed, words),
        change::Delegate::Merge { .. } => completed_merge(row, status, words, d.merge_kind),
        change::Delegate::Repair(_) if empty_repair && status == change::Status::Passed && !failed && !cancelled => {
            let mut account = crate::ResultText::empty();
            account.add(words, env.limits.client.answer_bytes);
            row.delegate_words = Some(account);
        }
        change::Delegate::Repair(_) => completed_repair(
            row,
            child,
            status,
            no_change || failed || cancelled,
            words,
            env.limits.client.answer_bytes,
        ),
        change::Delegate::Resolve { .. } => producer_result(row, false, failed, words),
    }
    if cancelled {
        row.producer_failure = cancelled_delegate_words(kind, child, words);
    } else if (failed || (no_change && !empty_repair))
        && let Some(account) = failed_delegate_words(row, kind, child, no_change, words, base)
    {
        row.producer_failure = Some(account);
    }
    let since = row.change.since;
    if agent {
        read_session::ended(d, env, task, since, out);
    }
    let mut saved = d.changes.get(&task).expect("retained change").clone();
    let outcome = if cancelled {
        crate::history::Answer::Cancelled
    } else if failed {
        crate::history::Answer::Failed
    } else if no_change {
        crate::history::Answer::Nothing
    } else if let change::Delegate::Gate { .. } = kind {
        if status == change::Status::Passed { crate::history::Answer::Passed } else { crate::history::Answer::Refused }
    } else if status == change::Status::Passed {
        crate::history::Answer::Pushed
    } else {
        crate::history::Answer::Failed
    };
    crate::change_history::answered(d, env.wall, &mut saved, child, kind, outcome, out);
    let no_change = produced_nothing(&saved);
    let owes_readback = release_owes_readback(d, env, &saved);
    let ended = fresh
        && engine_release(d, env, &mut saved, change::ReleaseTrigger::DelegateEnded { owes_readback, no_change }, out);
    d.changes.insert(task, saved.clone()).expect("retained change");
    if fresh && let change::State::NoChange { .. } = saved.change.state {
        crate::readiness::ending(d, env, &mut saved, crate::history::Ending::NoChange, out);
    }
    crate::readiness::reconcile(d, env, &mut saved, None, out);
    wake_after_turn(d, &saved, owned);
    save_change(d, out, saved);
    if ended {
        emit(
            out,
            Request::ChangeDecision {
                task,
                decision: change::Decision::EndHold,
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
    }
}

fn producing_state(state: &change::State) -> bool {
    *state == (change::State::Producing { requested: false })
        || *state == (change::State::Producing { requested: true })
}

/// The last done dependency refreshes readiness before forming (domain/landing-turn.md, 2).
fn dependencies_met(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    read_session::ended(d, env, task, env.wall, out);
    let Some(row) = d.changes.get(&task) else { return };
    if row.terminal() {
        return;
    }
    let mut reading = None;
    for (token, pending) in &d.steps {
        if pending.task == task {
            reading = Some(*token);
        }
    }
    if let Some(owner) = reading {
        let pending = d.steps.get_mut(&owner).expect("dependent gather");
        pending.kept = false;
        pending.pending = Box::new([]);
    }
    d.change_due.remove(&task);
    emit(out, Request::WakeProcedure { task });
    let mut saved = d.changes.get(&task).expect("live dependent change").clone();
    let owned = saved.change.owns_turn;
    let owes_readback = release_owes_readback(d, env, &saved);
    let ended = engine_release(d, env, &mut saved, change::ReleaseTrigger::DependenciesDone { owes_readback }, out);
    if ended || producing_state(&saved.change.state) {
        saved.change.since = env.wall;
        d.changes.insert(task, saved.clone()).expect("retained dependent change");
        wake_after_turn(d, &saved, owned);
        save_change(d, out, saved);
        if ended {
            emit(
                out,
                Request::ChangeDecision {
                    task,
                    decision: change::Decision::EndHold,
                    entry: None,
                    evidence: None,
                    close_at: None,
                },
            );
        }
    }
}

/// Keep a failed delegate's identity and ending after its live slot is cleared
/// (temper/docs/design/domain/forge/change/repairs.md).
fn failed_delegate_words(
    row: &ChangeRow,
    kind: change::Delegate,
    child: u64,
    no_change: bool,
    words: &[u8],
    base: Option<skein_forge::api::Commit>,
) -> Option<Box<[u8]>> {
    let (name, recovery): (&[u8], &[u8]) = match kind {
        // A producer's whole result already occupies the hold account's bound.
        change::Delegate::Produce | change::Delegate::Close => return None,
        change::Delegate::Repair(_) => (b"repair", b"ask for a fresh repair"),
        change::Delegate::Resolve { .. } => (b"resolution", b"read the branch and base afresh"),
        change::Delegate::Merge { .. } => (b"merge job", b"read the branch and base afresh"),
        change::Delegate::Gate { .. } => {
            let state =
                if let change::State::Held { was, .. } = &row.change.state { was.as_ref() } else { &row.change.state };
            let name: &[u8] = if let change::State::Judging { .. } = state { b"judgement" } else { b"review" };
            (name, b"ask for a fresh review")
        }
    };
    let number = skein_lib::Decimal::of(child);
    let ending: &[u8] = if no_change { b"ended without making the required change" } else { b"failed for good" };
    let reason: &[u8] = if words.is_empty() { b"gave no account" } else { words };
    let judged = match base {
        Some(base) if !no_change => skein_lib::bytes::concat(&[b", judged against ", &crate::commit_text(&base)]),
        Some(_) | None => Box::new([]),
    };
    let parts = [b"its ".as_slice(), name, b", task ", number.as_bytes(), &judged, b", ", ending, b": ", reason];
    let tail = [b"; resolve this failure, then release this change to ".as_slice(), recovery, b"; or cancel it"];
    let mut tail_bytes = 0_usize;
    for part in tail {
        tail_bytes = tail_bytes.checked_add(part.len()).expect("bounded recovery words");
    }
    let bound = skein_lib::usize_of(jig_domain_tasks::HOLD_ACCOUNT_BYTES);
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded delegate account");
    }
    let mut reason = skein_lib::Writer::new(length);
    for part in parts {
        reason.put(part).expect("measured delegate account");
    }
    let reason = crate::text::fit(&reason.finish(), bound.checked_sub(tail_bytes).expect("recovery fits hold"), b"...");
    let mut account = skein_lib::Writer::new(reason.len().checked_add(tail_bytes).expect("bounded hold account"));
    account.put(&reason).expect("bounded reason");
    for part in tail {
        account.put(part).expect("measured recovery words");
    }
    Some(account.finish())
}

/// A delegate's cancellation account (temper/docs/design/domain/forge/change/repairs.md).
fn cancelled_delegate_words(kind: change::Delegate, child: u64, words: &[u8]) -> Option<Box<[u8]>> {
    let name: &[u8] = match kind {
        change::Delegate::Produce | change::Delegate::Close => b"producer",
        change::Delegate::Repair(_) => b"repair",
        change::Delegate::Resolve { .. } => b"resolution",
        change::Delegate::Merge { .. } => b"merge job",
        change::Delegate::Gate { .. } => return None,
    };
    let number = skein_lib::Decimal::of(child);
    let separator: &[u8] = if words.is_empty() { b"" } else { b": " };
    let parts = [b"its ".as_slice(), name, b", task ", number.as_bytes(), b", was cancelled", separator, words];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded cancellation account");
    }
    let mut account = skein_lib::Writer::new(length);
    for part in parts {
        account.put(part).expect("measured cancellation account");
    }
    Some(crate::text::fit(
        &account.finish(),
        usize::try_from(jig_domain_tasks::HOLD_ACCOUNT_BYTES).expect("bounded hold account"),
        b"...",
    ))
}

fn completed_merge(row: &mut ChangeRow, status: change::Status, words: &[u8], kind: Option<(u32, u32)>) {
    if status == change::Status::Failed {
        producer_result(row, false, true, words);
    }
    if status == change::Status::Passed {
        let bound = match kind {
            Some((_, result)) => skein_lib::usize_of(result),
            None => 0,
        };
        row.merge_result = Some(Box::from(words.get(..words.len().min(bound)).expect("merge result within its bytes")));
    }
}

fn completed_repair(row: &mut ChangeRow, child: u64, status: change::Status, rejected: bool, words: &[u8], bound: u32) {
    if status == change::Status::Passed && !rejected && !words.is_empty() {
        row.result.completed(child, words, bound);
    }
}

fn producer_result(row: &mut ChangeRow, no_change: bool, failed: bool, words: &[u8]) {
    if no_change && let change::State::Producing { .. } = row.change.state {
        row.change.state = change::State::NoChange { words: words.into() };
        row.change.owns_turn = false;
    }
    let held_producing = match &row.change.state {
        change::State::Held { was, .. } => {
            **was == (change::State::Producing { requested: true })
                || **was == (change::State::Producing { requested: false })
        }
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Repairing { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
    };
    row.producer_failure = if (failed && !words.is_empty()) || (no_change && held_producing) {
        Some(crate::text::fit(
            words,
            usize::try_from(jig_domain_tasks::HOLD_ACCOUNT_BYTES).expect("bounded hold account"),
            b"...",
        ))
    } else {
        None
    };
}

// A producer's no-change is a successful terminal with a bounded account but
// no retained delegate. Do not confuse it with a failed or cancelled producer.
fn produced_nothing(row: &ChangeRow) -> Option<Box<[u8]>> {
    if let change::State::Held { was, .. } = &row.change.state
        && producing_state(was)
        && row.close.is_none()
        && row.delegate.is_none()
        && row.delegate_status == change::Status::Passed
    {
        return row.producer_failure.clone();
    }
    None
}

fn review_terminal_words(status: change::Status, failed: bool, cancelled: bool) -> &'static [u8] {
    if cancelled {
        b"cancelled"
    } else if failed {
        b"failed"
    } else if status == change::Status::Passed {
        b"approved"
    } else {
        b"changes requested"
    }
}

fn record_review(
    row: &mut ChangeRow,
    env: &Env<Limits>,
    number: u64,
    head: [u8; 32],
    standing: (change::Status, bool, bool),
    words: &[u8],
) -> Result<(), crate::RefusalCause> {
    let (status, failed, cancelled) = standing;
    let verdict = review_terminal_words(status, failed, cancelled);
    for gate in &row.change.gates {
        if gate.number == number && gate.kind == change::GateKind::Agent {
            row.result.reviewed(&head, verdict, env.limits.client.answer_bytes);
        }
    }
    // A result racing the CI cancellation is no verdict or review failure.
    let cancelling = row.change.state == (change::State::CancellingGates { head });
    if cancelled || cancelling {
        return Ok(());
    }
    let report = change::GateReport { number, head, status, revision: row.gate_revision };
    let mut stale = false;
    for gate in &row.change.gates {
        if gate.number == number {
            stale = !report.at_revision(gate.kind, row.spec_revision);
        }
    }
    let mut verdicts = List::with_capacity(env.limits.change_policy.gates);
    for old in &row.verdicts {
        if old.number != number {
            verdicts.push(*old).expect("bounded prior gate reports");
        }
    }
    if (!failed || stale) && verdicts.push(report).is_err() {
        return Err(crate::RefusalCause::Limit {
            member: "forge.change_policy.gates",
            configured: u64::from(env.limits.change_policy.gates),
            needed: u64::from(verdicts.len()).saturating_add(1),
        });
    }
    row.verdicts = verdicts.into_boxed();
    if failed && !stale {
        row.change = change::review_failed(&row.change, number, head, env.wall, &env.limits.change_policy);
    }
    let mut remarks = List::with_capacity(env.limits.change_policy.gates);
    for old in &row.gate_remarks {
        if old.number != number {
            remarks.push(old.clone()).expect("bounded prior gate remarks");
        }
    }
    let text = crate::ResultText { bytes: Box::from(words), cut: 0 };
    let Some(words) = text.fitted(env.limits.client.answer_bytes) else {
        return Err(crate::RefusalCause::Limit {
            member: "forge.client.answer_bytes",
            configured: u64::from(env.limits.client.answer_bytes),
            needed: text.minimum_room(),
        });
    };
    remarks.push(crate::GateRemark { number, head, words }).expect("gate room checked");
    row.gate_remarks = remarks.into_boxed();
    Ok(())
}

fn change_terminal(row: &ChangeRow) -> Option<change::Decision> {
    if row.source.is_some()
        && let Some(crate::Ending::Failed { .. }) = &row.ending
    {
        return Some(change::Decision::Failed);
    }
    if row.change.state == change::State::Ended {
        return Some(change::Decision::Cancel);
    }
    if let change::State::NoChange { .. } = row.change.state {
        return Some(change::Decision::NoChange);
    }
    None
}

fn earlier_time(left: Option<skein_lib::Time>, right: Option<skein_lib::Time>) -> Option<skein_lib::Time> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, None) => left,
        (None, right) => right,
    }
}

fn retry_deadline(d: &Domain) -> Option<skein_lib::Time> {
    let mut next: Option<skein_lib::Time> = None;
    for (_, pending) in &d.steps {
        if let Some(at) = pending.retry_at {
            next = Some(match next {
                Some(before) => before.min(at),
                None => at,
            });
        }
    }
    next
}

fn retry_gathers(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut due = List::with_capacity(env.limits.changes);
    for (owner, pending) in &d.steps {
        if let Some(at) = pending.retry_at
            && at <= env.now
        {
            due.push(*owner).expect("bounded capacity retries");
        }
    }
    for owner in due.into_boxed() {
        if let Some(mut pending) = d.steps.remove(&owner) {
            // Only invalidation starts the gather again; admission pressure keeps
            // every answered fact (domain/forge/keeping-up.md, 1 and 3).
            if pending.woken {
                branch_due(d, pending.task);
                restart_step(d, env, pending, out);
            } else {
                let (repository, read) = pending.read.take().expect("refused gather read");
                pending.retry_at = None;
                d.steps.insert(owner, pending).expect("capacity retry keeps its room");
                gather_read(d, env, owner, repository, read, out);
            }
        }
    }
}

fn gather_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    repository: skein_forge::api::Repository,
    read: skein_forge::api::Read,
    out: &mut Queue<Request>,
) {
    d.steps.get_mut(&owner).expect("pending gather").read = Some((repository, read.clone()));
    child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

fn restart_step(d: &mut Domain, env: &Env<Limits>, pending: PendingStep, out: &mut Queue<Request>) {
    step_change(
        d,
        env,
        StepInput {
            task: pending.task,
            entry: pending.entry,
            heard: pending.heard,
            gates: pending.gates,
            queue_repair_active: pending.queue_repair_active,
            kept: pending.kept,
            merge_job: pending.merge_job,
            pending: pending.pending,
        },
        out,
    );
}

fn step_owner(d: &Domain, task: u64) -> Option<Token> {
    for (owner, pending) in &d.steps {
        if pending.task == task {
            return Some(*owner);
        }
    }
    None
}

// A read session survives restarts and capacity retries independently of the
// completed policy step's wait (change/repairs.md, Stalls).
fn arm_gather(d: &mut Domain, env: &Env<Limits>, pending: &PendingStep, out: &mut Queue<Request>) {
    read_session::refresh(d, env, pending, out);
    crate::waits::changed(d, pending.task);
}

#[expect(clippy::too_many_lines, reason = "one fresh-read chain keeps task standing, containment and connection waits")]
fn step_change(d: &mut Domain, env: &Env<Limits>, input: StepInput, out: &mut Queue<Request>) {
    crate::room::stepping(d, input.task);
    if d.stopping {
        return;
    }
    let StepInput { task, entry, mut heard, gates, queue_repair_active, kept, merge_job, pending: dependencies } =
        input;
    d.fresh_steps.remove(&task);
    if kept
        && let Some(row) = d.changes.get_mut(&task)
        && row.change.owns_turn
    {
        row.change.owns_turn = false;
        let mut saved = row.clone();
        if !heard.cancelled {
            crate::readiness::unavailable(d, env, &mut saved, crate::history::Unready::Dependency, out);
        }
        wake_after_turn(d, &saved, true);
        save_change(d, out, saved);
    }
    heard.start_conflict = d.start_conflict(task);
    heard.review_request = d.review_requests.get(&task).copied();
    if heard.released {
        if let Some(row) = d.changes.get(&task)
            && row.task_release.is_none()
            && row.change.released.is_none()
            && matches_held(&row.change.state)
        {
            release_change(d, env, task, out);
        }
        heard.released = false;
    }
    let Some(row) = d.changes.get(&task) else {
        emit(out, Request::Refused { task });
        return;
    };
    if row.frozen() {
        d.read_sessions.remove(&task);
        if let Some(owner) = step_owner(d, task) {
            d.steps.remove(&owner);
        }
    }
    if row.change.way == change::Way::Turn
        && let Some(change::Handed { ask: change::Ask::Landed { prefix }, .. }) = crate::landing::members::head(row)
    {
        step_verified(d, env, task, *prefix, heard, out);
        return;
    }
    if let Some(decision) = change_terminal(row) {
        emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None, close_at: None });
        return;
    }
    if let Some(merge) = landed_commit(&row.change.state) {
        step_verified(d, env, task, merge, heard, out);
        return;
    }
    if row.frozen() {
        return;
    }
    // A taskless turn owns the train read until it can hand its members
    // their prefixes or return them to the queue. Do not start a separate
    // gather (and charge its stall) against that same unsettled train tip.
    if !heard.cancelled && crate::unmade::waiting_member(d, task) {
        emit(
            out,
            Request::ChangeDecision {
                task,
                decision: change::Decision::Wait { until: None },
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    }
    let owner = step_owner(d, task);
    let close_unrequested = close_unrequested(row);
    if close_unrequested && heard.closer_waiting && !heard.cancelled {
        if let Some(owner) = owner {
            d.steps.remove(&owner);
        }
        read_session::forget(d, task, out);
        emit(
            out,
            Request::ChangeDecision {
                task,
                decision: change::Decision::Wait { until: None },
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    }
    if gates.len() > skein_lib::usize_of(env.limits.change_policy.gates) {
        crate::refusal::resource_limit(
            out,
            task,
            b"step change",
            None,
            "forge.change_policy.gates",
            env.limits.change_policy.gates,
            skein_lib::len_u64(gates.len()),
        );
        return;
    }
    if let Some(owner) = owner {
        let mut pending = d.steps.remove(&owner).expect("found pending step");
        // Entries hand off in number order (skein's forge/client-effects.md, 1.2);
        // the newest wake reserves the restart's entry and reads jig's current kept answer.
        pending.entry = entry;
        pending.heard.delegate = heard.delegate;
        pending.heard.delegate_running = heard.delegate_running;
        pending.heard.delegate_live = heard.delegate_live;
        pending.heard.cancelled = heard.cancelled;
        pending.heard.closer_waiting = heard.closer_waiting;
        pending.heard.lacking = heard.lacking;
        pending.heard.spend = heard.spend;
        pending.kept = kept;
        pending.merge_job = merge_job;
        pending.pending = dependencies;
        pending.heard.start_conflict = heard.start_conflict;
        pending.heard.review_request = heard.review_request;
        pending.queue_repair_active = queue_repair_active;
        pending.gates = gates;
        pending.woken = true;
        arm_gather(d, env, &pending, out);
        d.steps.insert(owner, pending).expect("gather keeps its room");
        return;
    }
    if connection_wait::waiting(d, task, row.repository.forge) {
        connection_wait::wait(d, env, task, out);
        return;
    }
    if d.steps.len() == env.limits.changes {
        crate::refusal::resource_limit(
            out,
            task,
            b"step change",
            None,
            "forge.changes",
            env.limits.changes,
            u64::from(d.steps.len()).saturating_add(1),
        );
        return;
    }
    let repository = row.repository;
    let selecting = row.change.state == (change::State::Taking { requested: false }) && row.kept.is_none();
    let branch =
        if selecting { row.source.as_ref().expect("taking source").branch.clone() } else { row.branch.clone() };
    let producer = if row.delegate_status == change::Status::Unknown { heard.delegate } else { row.delegate_status };
    let read_branch =
        close_unrequested || selecting || hints::has_head(d, row) || branch_is_due(d, env, task, producer);
    let Some(sequence) = d.sequence.checked_add(1) else {
        emit(out, Request::Refused { task });
        return;
    };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 62));
    let pending = PendingStep {
        task,
        entry,
        follow_tree: List::with_capacity(env.limits.key_entries),
        stage: if selecting { StepStage::Taking } else { StepStage::Branch },
        heard,
        gates: if gates.is_empty() { row.verdicts.clone() } else { gates },
        queue_repair_active,
        kept,
        merge_job,
        pending: dependencies,
        branch: None,
        branch_fresh: read_branch,
        drift: None,
        behind: None,
        base_tip: None,
        base_read: None,
        close_landing: None,
        pull: None,
        pull_ci: false,
        ci: change::Status::Unknown,
        execution: None,
        failed: crate::change_ci::Failed::new(Box::new([])),
        stall: None,
        base_ci: change::Status::Unknown,
        contains_base: change::Status::Unknown,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Pending,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
        retry_at: None,
        read: None,
        progress: List::uncapped(),
    };
    arm_gather(d, env, &pending, out);
    d.steps.insert(owner, pending).expect("step capacity checked");
    if read_branch {
        branch_sent(d, task, producer);
        gather_read(d, env, owner, repository, skein_forge::api::Read::Branch { branch }, out);
    } else {
        step_read(d, env, owner, Err(skein_forge::api::Error::Missing), out);
    }
}

fn close_unrequested(row: &ChangeRow) -> bool {
    row.close.is_some()
        && (row.change.state == (change::State::Producing { requested: false })
            || row.change.released == Some(change::Cause::CloseRefused))
}

fn branch_due(d: &mut Domain, task: u64) {
    if let Some(read) = d.settling.get_mut(&task) {
        read.branch.due = true;
    }
}

fn branch_is_due(d: &Domain, env: &Env<Limits>, task: u64, producer: change::Status) -> bool {
    let Some(settle) = d.settling.get(&task) else { return true };
    let read = settle.branch;
    let Some(at) = read.at else { return true };
    let result = match producer {
        change::Status::Passed | change::Status::Failed => producer != read.producer,
        change::Status::Unknown | change::Status::Pending => false,
    };
    read.due || result || env.now >= at.saturating_add(env.limits.client.fallback)
}

fn branch_sent(d: &mut Domain, task: u64, producer: change::Status) {
    if let Some(read) = d.settling.get_mut(&task) {
        read.branch.due = false;
        read.branch.at = None;
        read.branch.producer = producer;
    } else {
        let mut settle = crate::settle::Settle::NEW;
        settle.branch.producer = producer;
        d.settling.insert(task, settle).expect("one branch read per change");
    }
}

fn early_base(pending: &PendingStep) -> bool {
    for dependency in &pending.pending {
        match dependency.kind {
            change::PendingKind::Change { checked: Some(_), kept: false } | change::PendingKind::Elsewhere => {}
            change::PendingKind::Change { .. } | change::PendingKind::Other => return false,
        }
    }
    !pending.pending.is_empty()
}

pub(crate) fn landed_commit(state: &change::State) -> Option<skein_forge::api::Commit> {
    match state {
        change::State::Landed { merge, .. }
        | change::State::Red { merge, .. }
        | change::State::Cleared { merge }
        | change::State::Uncleared { merge } => Some(*merge),
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Held { .. } => None,
    }
}

fn step_verified(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    merge: skein_forge::api::Commit,
    heard: change::Heard,
    out: &mut Queue<Request>,
) {
    if !repository_reads::change_fresh(d, task) {
        d.fresh_steps
            .insert(task, FreshStep::Verified { merge, heard: Box::new(heard) })
            .expect("one parked step per change");
        return;
    }
    let mut row = d.changes.get(&task).expect("known landed change").clone();
    let pending = PendingStep {
        task,
        entry: 0,
        follow_tree: List::with_capacity(env.limits.key_entries),
        stage: StepStage::Branch,
        heard,
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
        branch: d.branch_head(&crate::tiers::name(row.repository, &row.branch)).or(row.change.last_head),
        branch_fresh: false,
        drift: None,
        behind: None,
        base_tip: Some(merge),
        base_read: Some(merge),
        close_landing: None,
        pull: None,
        pull_ci: false,
        ci: change::Status::Unknown,
        execution: None,
        failed: crate::change_ci::Failed::new(Box::new([])),
        stall: None,
        base_ci: change::Status::Unknown,
        contains_base: change::Status::Unknown,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Pending,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
        retry_at: None,
        read: None,
        progress: List::uncapped(),
    };
    let facts = change_facts(d, env, &row, &pending, merge);
    let stepped = change::step(&row.change, &facts, &heard, env.wall, &env.limits.change_policy);
    let prior = row.change.clone();
    row.change = stepped.change;
    if let Some(took) = stepped.took {
        crate::landing::members::take(&mut row, took, out);
    }
    crate::landing::change_advanced(d, &row, &prior, out);
    if landed_commit(&prior.state).is_none() && landed_commit(&row.change.state).is_some() {
        reds::landed(d, &env.limits, &row, out);
    }
    crate::change_history::advanced(d, env.wall, &mut row, &prior, out);
    d.changes.insert(task, row.clone()).expect("replaces change");
    save_change(d, out, row);
    emit(
        out,
        Request::ChangeDecision { task, decision: stepped.decision, entry: None, evidence: None, close_at: None },
    );
}

fn status(ci: skein_forge::api::Ci) -> change::Status {
    match ci {
        skein_forge::api::Ci::None => change::Status::Unknown,
        skein_forge::api::Ci::Pending => change::Status::Pending,
        skein_forge::api::Ci::Passed => change::Status::Passed,
        skein_forge::api::Ci::Failed => change::Status::Failed,
    }
}

// Any listed commit, or a range too large to list, disproves containment
// (temper/docs/design/domain/forge/keeping-up.md, 3).
pub(crate) fn ancestry_status(result: Result<skein_forge::api::Answer, skein_forge::api::Error>) -> change::Status {
    match result {
        Ok(skein_forge::api::Answer::Commits { commits, more }) => {
            if !commits.is_empty() {
                change::Status::Failed
            } else if more {
                change::Status::Unknown
            } else {
                change::Status::Passed
            }
        }
        Err(skein_forge::api::Error::TooLarge(_)) => change::Status::Failed,
        Ok(_) | Err(_) => change::Status::Unknown,
    }
}

#[expect(clippy::too_many_lines, reason = "one bounded fresh-read chain preserves its stage and facts")]
fn step_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.steps.remove(&owner) else { return };
    if d.stopping {
        return;
    }
    let Some(row) = d.changes.get(&pending.task) else { return };
    if !d.repositories.contains_key(&row.repository) {
        return;
    }
    arm_gather(d, env, &pending, out);
    let row = d.changes.get(&pending.task).expect("held change");
    if row.frozen() {
        d.read_sessions.remove(&pending.task);
        if let Some(decision) = change_terminal(row) {
            emit(
                out,
                Request::ChangeDecision { task: pending.task, decision, entry: None, evidence: None, close_at: None },
            );
        } else if let Some(merge) = landed_commit(&row.change.state) {
            step_verified(d, env, pending.task, merge, pending.heard, out);
        }
        return;
    }
    if pending.stage == StepStage::Taking {
        taking::answered(d, env, pending, result, out);
        return;
    }
    if pending.stage == StepStage::StallChecks || pending.stage == StepStage::StallJobs {
        if pending.stage == StepStage::StallChecks {
            pending.stall = Some(match result {
                Ok(skein_forge::api::Answer::Checks(checks)) => crate::stall::Report::checks(&checks),
                Ok(_) | Err(_) => crate::stall::Report::checks(&[]),
            });
        } else if let Some(report) = &mut pending.stall {
            report.answered(result);
        }
        if let Some(read) = pending.stall.as_mut().expect("diagnostic report").next() {
            pending.stage = StepStage::StallJobs;
            let repository = row.repository;
            d.steps.insert(owner, pending).expect("existing step");
            gather_read(d, env, owner, repository, read, out);
        } else {
            finish_step(d, env, pending, out);
        }
        return;
    }
    if let StepStage::FollowCheck(kind) = pending.stage {
        let mut row = row.clone();
        let action = crate::follow_check::answered(d, env, &mut row, kind, result, &mut pending.follow_tree, out);
        if pending.woken {
            // A hint during the workflow read asks for fresh branch and CI facts before another dispatch.
            d.changes.insert(row.task, row.clone()).expect("known change");
            save_change(d, out, row);
            finish_step(d, env, pending, out);
            return;
        }
        drive_check(d, env, row, pending, action, out);
        return;
    }
    if pending.stage != StepStage::Paths && result == Err(skein_forge::api::Error::Busy) {
        pending.retry_at = Some(env.now.saturating_add(Duration::from_secs(1)));
        d.steps.insert(owner, pending).expect("capacity retry keeps the gather");
        return;
    }
    if result == Err(skein_forge::api::Error::NotConnected) {
        connection_wait::wait(d, env, pending.task, out);
        return;
    }
    if let StepStage::MergeHooks(head) = pending.stage {
        if let Ok(hooks) = crate::hooks::answered(head, result) {
            d.merge_hooks.insert(pending.task, (head, hooks)).expect("one hook answer per change");
            finish_step(d, env, pending, out);
        } else {
            pending.retry_at = Some(env.now.saturating_add(env.limits.client.backoff));
            d.steps.insert(owner, pending).expect("hook retry keeps its gather");
        }
        return;
    }
    let closing = close_unrequested(row);
    let close_needs_tips = closing && (pending.branch.is_none() || row.delegate_status == change::Status::Failed);
    if pending.stage == StepStage::CloseLanding || (close_needs_tips && pending.stage == StepStage::Base) {
        let known = if let Ok(skein_forge::api::Answer::Commit(commit)) = &result { *commit != [0; 32] } else { false };
        if !known {
            wait_for_forge(d, env, &pending, None, out);
            return;
        }
    }
    if admission::read_failed(row, &pending, &result, out) {
        read_session::forget(d, pending.task, out);
        branch_due(d, pending.task);
        return;
    }
    let repository = row.repository;
    let branch = row.branch.clone();
    let base = row.base.clone();
    let pull_number = row.pull;
    let catch_up = row.catch_up;
    let preparing = row.change.state.clone();
    let ci_recheck =
        if let change::State::Checking { .. } = preparing { ci_recheck_at(row, pending.branch) } else { None };
    let made_update = row.effect == change::EffectResult::Made && matches_updating(&preparing);
    let had_head = hints::has_head(d, row);
    let required = match d.repositories.get(&repository) {
        Some(repository) => crate::ci::required(&repository.protection),
        None => &[],
    };
    let result_rows = result_contexts(&preparing) || row.runner_ci == Some(crate::RunnerCi::Released);
    let mut pull_ci = required.is_empty() && !result_rows;
    if let change::State::Repairing { .. } = preparing {
        pull_ci = required.is_empty();
    }
    let next = match pending.stage {
        StepStage::Taking
        | StepStage::StallChecks
        | StepStage::StallJobs
        | StepStage::FollowCheck(_)
        | StepStage::MergeHooks(_) => {
            unreachable!("special reads handled above")
        }
        StepStage::Branch => {
            if pending.branch_fresh
                && let Some(read) = d.settling.get_mut(&pending.task)
            {
                read.branch.at = Some(env.now);
            }
            pending.branch = match result {
                Ok(skein_forge::api::Answer::Commit(commit)) => Some(commit),
                Err(skein_forge::api::Error::Missing) => None,
                Ok(_) | Err(_) => unreachable!("fresh read answer checked"),
            };
            let judgement = change_reads::branch(d, env, &pending, out);
            if !change_reads::keep(d, env, &mut pending, judgement, out) {
                return;
            }
            if pending.drift.is_none() && pending.behind.is_none() {
                crate::settling::head_seen(d, pending.task, pending.branch, out);
                if let Some(commit) = pending.branch {
                    if let Some(read) = d.settling.get_mut(&pending.task) {
                        read.branch.present = true;
                    }
                    if !made_update && rebase::pending(d, pending.task).is_none() {
                        branch_changed(
                            d,
                            env,
                            skein_forge_client::Resource {
                                repository,
                                what: skein_forge_client::What::Branch(branch.clone()),
                            },
                            commit,
                            out,
                        );
                    }
                }
            } else if pending.behind.is_some() {
                finish_step(d, env, pending, out);
                return;
            }
            if goal_branch::base_wait(d, repository, &base) {
                wait_fresh(d, pending.task, &[repository]);
                wait_for_forge(d, env, &pending, None, out);
                return;
            }
            if pending.branch.is_none()
                && !hints::has_head(d, d.changes.get(&pending.task).expect("live change"))
                && catch_up.is_none()
                && !early_base(&pending)
                && !closing
            {
                finish_step(d, env, pending, out);
                return;
            }
            pending.stage = StepStage::Base;
            skein_forge::api::Read::Branch { branch: base }
        }
        StepStage::Base => {
            if goal_branch::base_wait(d, repository, &base) {
                wait_fresh(d, pending.task, &[repository]);
                wait_for_forge(d, env, &pending, None, out);
                return;
            }
            match result {
                Ok(skein_forge::api::Answer::Commit(commit)) => {
                    let tip = if let change::State::Preparing { main, .. } = preparing { main } else { commit };
                    pending.base_tip = Some(tip);
                    pending.base_read = Some(commit);
                    let judgement = change_reads::base(d, env, &pending, commit, out);
                    if !change_reads::keep(d, env, &mut pending, judgement, out) {
                        return;
                    }
                    if pending.behind.is_some() {
                        finish_step(d, env, pending, out);
                        return;
                    }

                    branch_changed(
                        d,
                        env,
                        skein_forge_client::Resource {
                            repository,
                            what: skein_forge_client::What::Branch(base.clone()),
                        },
                        commit,
                        out,
                    );
                }
                Ok(_) | Err(_) => {
                    unreachable!("fresh read answer checked");
                }
            }
            if close_needs_tips {
                pending.stage = StepStage::CloseLanding;
                let landing = &d.repositories.get(&repository).expect("adopted repository").landing;
                skein_forge::api::Read::Branch { branch: landing.clone() }
            } else if pending.branch.is_none() && !had_head && catch_up.is_none() {
                finish_step(d, env, pending, out);
                return;
            } else if let Some(commit) = catch_up {
                pending.stage = StepStage::CatchUp;
                skein_forge::api::Read::Commits { before: pending.base_tip.expect("base read"), after: commit, page: 1 }
            } else {
                pending.stage = StepStage::Pull;
                pending.pull_ci = pull_ci && pull_number.is_some();
                match pull_number {
                    Some(number) => skein_forge::api::Read::Pull { number, ci: pull_ci },
                    None => skein_forge::api::Read::PullFor { head: branch, base },
                }
            }
        }
        StepStage::CloseLanding => {
            let Ok(skein_forge::api::Answer::Commit(commit)) = result else { unreachable!("checked close tip read") };
            pending.close_landing = Some(commit);
            finish_step(d, env, pending, out);
            return;
        }
        StepStage::CatchUp => {
            pending.contains_catch_up = ancestry_status(result);
            if pending.contains_catch_up == change::Status::Unknown {
                wait_for_forge(d, env, &pending, Some(change_reads::retry_at(env)), out);
                return;
            }
            if pending.branch.is_none() && !had_head {
                finish_step(d, env, pending, out);
                return;
            }
            pending.stage = StepStage::Pull;
            pending.pull_ci = pull_ci && pull_number.is_some();
            match pull_number {
                Some(number) => skein_forge::api::Read::Pull { number, ci: pull_ci },
                None => skein_forge::api::Read::PullFor { head: branch, base },
            }
        }
        StepStage::Pull => {
            match result {
                Ok(skein_forge::api::Answer::Pull(pull)) => {
                    // Branch names can repeat across deployments and a repository's past.
                    let own = (pull.state == skein_forge::api::State::Open && pull.merged.is_none())
                        || row.pull == Some(pull.number);
                    pending.pull = if own { Some(pull) } else { None };
                }
                Err(skein_forge::api::Error::Missing) => pending.pull = None,
                Ok(_) | Err(_) => {
                    unreachable!("fresh read answer checked");
                }
            }
            let judgement = change_reads::pull(d, env, &pending, out);
            if !change_reads::keep(d, env, &mut pending, judgement, out) {
                return;
            }
            let numbered_missing =
                if let Some(change_reads::Drift { why: change::Cause::PullClosed, .. }) = pending.drift {
                    pending.pull.is_none()
                } else {
                    false
                };
            if pending.behind.is_some() || numbered_missing {
                finish_step(d, env, pending, out);
                return;
            }
            if let Some(pull) = &pending.pull {
                crate::change_history::observed_pull(d, env.wall, pending.task, pull, out);
            }
            pending.stage = StepStage::Ci;
            let Some(commit) = pending.branch else {
                pending.stage = StepStage::BaseCi;
                let Some(base_tip) = pending.base_tip else { unreachable!("base read") };
                d.steps.insert(owner, pending).expect("replaces step");
                gather_read(d, env, owner, repository, skein_forge::api::Read::Checks { commit: base_tip }, out);
                return;
            };
            let pull_ci = match &pending.pull {
                Some(pull) if pending.pull_ci && pull_number.is_some() && pull.commit == commit => Some(pull.ci),
                Some(_) | None => None,
            };
            if let Some(ci) = pull_ci {
                pending.ci = status(ci);
                ci_news::observe_verdict(d, env, pending.task, commit, ci);
                if let Some(row) = d.changes.get_mut(&pending.task)
                    && row.result.head != Some(commit)
                    && row.result.checked(commit, &[], env.limits.client.answer_bytes)
                {
                    let saved = row.clone();
                    save_change_parts(d, out, saved);
                }
                if ci == skein_forge::api::Ci::Failed
                    || result_rows
                    || execution_due(&preparing, &pending)
                    || ci_recheck.is_some()
                {
                    // Repairs retain the contexts they observed, beside the pull verdict.
                    skein_forge::api::Read::Checks { commit }
                } else if let Some(read) = step_after_ci(d, &mut pending, out) {
                    read
                } else {
                    step_paths(d, env, owner, pending, out);
                    return;
                }
            } else {
                skein_forge::api::Read::Checks { commit }
            }
        }
        StepStage::Ci => {
            if let Ok(skein_forge::api::Answer::Checks(rows)) = result {
                let ci = match &pending.pull {
                    Some(pull) if pending.pull_ci && Some(pull.commit) == pending.branch => pull.ci,
                    Some(_) | None => crate::ci::fold(&rows, required),
                };
                pending.ci = status(ci);
                if let Some(head) = pending.branch {
                    ci_news::observe(d, env, pending.task, head, &rows, ci);
                }
                if let Some(head) = pending.branch
                    && let Some(row) = d.changes.get_mut(&pending.task)
                {
                    let release = row.runner_ci == Some(crate::RunnerCi::Released);
                    if release {
                        row.runner_ci = if row.change.last_head == Some(head) {
                            Some(crate::RunnerCi::Counting {
                                attempts: crate::change_ci::release_baseline(&rows, head),
                            })
                        } else {
                            None
                        };
                    }
                    if row.result.checked(head, &rows, env.limits.client.answer_bytes) || release {
                        let saved = row.clone();
                        save_change_parts(d, out, saved);
                    }
                }
                if execution_due(&preparing, &pending) {
                    if ci_recheck.is_some() {
                        pending.failed = crate::change_ci::Failed::new(rows.clone());
                    }
                    pending.execution = Some(crate::execution::Gather::new(rows));
                } else {
                    pending.failed = crate::change_ci::Failed::new(rows);
                }
            } else if row.runner_ci == Some(crate::RunnerCi::Released) {
                pending.ci = change::Status::Unknown;
            }
            if pending.execution.is_none() && execution_due(&preparing, &pending) {
                pending.execution = Some(crate::execution::Gather::new(Box::new([])));
            }
            if let Some(read) = next_execution(&mut pending) {
                read
            } else if let Some(read) = next_ci_run(&mut pending, ci_recheck) {
                read
            } else if let Some(read) = step_after_ci(d, &mut pending, out) {
                read
            } else {
                step_paths(d, env, owner, pending, out);
                return;
            }
        }
        StepStage::ExecutionRun(run) => {
            if let Some(recheck) = ci_recheck
                && recheck.job.run == run
                && pending.failed.next_recheck(recheck.job).is_some()
            {
                pending.failed.read(run, result.clone());
            }
            pending.execution.as_mut().expect("execution gather").answered(run, result);
            if let Some(read) = next_execution(&mut pending) {
                read
            } else if let Some(read) = next_ci_run(&mut pending, ci_recheck) {
                read
            } else if let Some(read) = step_after_ci(d, &mut pending, out) {
                read
            } else {
                step_paths(d, env, owner, pending, out);
                return;
            }
        }
        StepStage::Run(run) => {
            pending.failed.read(run, result);
            if let Some(read) = next_ci_run(&mut pending, ci_recheck) {
                read
            } else if let Some(read) = step_after_ci(d, &mut pending, out) {
                read
            } else {
                step_paths(d, env, owner, pending, out);
                return;
            }
        }
        StepStage::BaseCi => {
            match result {
                Ok(skein_forge::api::Answer::Checks(rows)) => {
                    let ci = crate::ci::fold(&rows, required);
                    pending.base_ci = status(ci);
                    if let Some(head) = pending.base_tip {
                        ci_news::observe(d, env, pending.task, head, &rows, ci);
                    }
                }
                Ok(_) | Err(_) => pending.base_ci = change::Status::Unknown,
            }
            if let Some(tip) = pending.base_tip {
                crate::readiness::base_ci(d, env, pending.task, tip, pending.base_ci, out);
            }
            let Some(head) = pending.branch else {
                step_paths(d, env, owner, pending, out);
                return;
            };
            let Some(base_tip) = pending.base_tip else { unreachable!("base read") };
            pending.stage = StepStage::Compare;
            let asked_base = updating_base(&preparing).unwrap_or(base_tip);
            skein_forge::api::Read::Commits { before: head, after: asked_base, page: 1 }
        }
        StepStage::Compare => {
            pending.contains_base = ancestry_status(result);
            if let Some(read) = read_contains_head(d, &mut pending) {
                read
            } else if let Some(read) = step_reviews(&mut pending) {
                read
            } else {
                step_paths(d, env, owner, pending, out);
                return;
            }
        }
        StepStage::ContainsHead => {
            pending.contains_head = ancestry_status(result);
            let mut settle = d.settling.get(&pending.task).copied().unwrap_or(crate::settle::Settle::NEW);
            settle.contains_head = Some(crate::settle::HeadContainment {
                head: pending.branch.expect("containment has a head"),
                tip: pending.base_tip.expect("containment has a tip"),
                status: pending.contains_head,
            });
            d.settling.insert(pending.task, settle).expect("one settle row per change");
            if let Some(read) = step_reviews(&mut pending) {
                read
            } else {
                step_paths(d, env, owner, pending, out);
                return;
            }
        }
        StepStage::Paths => {
            if let Ok(skein_forge::api::Answer::Compare { before, after, files, changed, .. }) = result {
                pending.paths_whole = Some(before) == pending.base_tip
                    && Some(after) == pending.branch
                    && skein_lib::len_u64(files.len()) == changed;
                pending.paths = files;
            }
            finish_step(d, env, pending, out);
            return;
        }
        StepStage::Reviews(page) => match result {
            Ok(skein_forge::api::Answer::Reviews { reviews, more, .. }) => {
                let Some(total) = pending.reviews.len().checked_add(reviews.len()) else {
                    emit(out, Request::Refused { task: pending.task });
                    return;
                };
                if total > skein_lib::usize_of(env.limits.client.inbox) {
                    pending.reviews_complete = false;
                    step_paths(d, env, owner, pending, out);
                    return;
                }
                let mut gathered = List::with_capacity(env.limits.client.inbox);
                for review in &pending.reviews {
                    gathered.push(review.clone()).expect("review bound preflighted");
                }
                for review in reviews {
                    gathered.push(review).expect("review bound preflighted");
                }
                pending.reviews = gathered.into_boxed();
                if more {
                    let Some(next) = page.checked_add(1) else {
                        pending.reviews_complete = false;
                        step_paths(d, env, owner, pending, out);
                        return;
                    };
                    pending.stage = StepStage::Reviews(next);
                    let Some(pull) = &pending.pull else { unreachable!("review read has a pull") };
                    skein_forge::api::Read::Reviews { number: pull.number, page: next, skip: 0 }
                } else {
                    step_paths(d, env, owner, pending, out);
                    return;
                }
            }
            Ok(_) | Err(_) => {
                pending.reviews_complete = false;
                step_paths(d, env, owner, pending, out);
                return;
            }
        },
    };
    d.steps.insert(owner, pending).expect("replaces step");
    gather_read(d, env, owner, repository, next, out);
}

fn ci_recheck_at(row: &ChangeRow, branch: Option<skein_forge::api::Commit>) -> Option<crate::CiRecheck> {
    let recheck = row.ci_recheck?;
    if branch != Some(recheck.job.head) || branch != row.change.last_head {
        return None;
    }
    Some(recheck)
}

fn next_ci_run(pending: &mut PendingStep, recheck: Option<crate::CiRecheck>) -> Option<skein_forge::api::Read> {
    let pinned = if let Some(recheck) = recheck { pending.failed.next_recheck(recheck.job) } else { None };
    let read = if pinned.is_some() {
        pinned
    } else if pending.ci == change::Status::Failed {
        pending.failed.next(pending.branch?)
    } else {
        None
    }?;
    let skein_forge::api::Read::Run { run } = read else { unreachable!("CI run read") };
    pending.stage = StepStage::Run(run);
    Some(read)
}

fn execution_due(state: &change::State, pending: &PendingStep) -> bool {
    match state {
        change::State::Checking { head } => {
            pending.branch == Some(*head)
                && (pending.ci == change::Status::Pending || pending.ci == change::Status::Unknown)
        }
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Held { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
    }
}

fn next_execution(pending: &mut PendingStep) -> Option<skein_forge::api::Read> {
    let read = pending.execution.as_mut()?.next()?;
    let skein_forge::api::Read::Run { run } = read else { unreachable!("execution run") };
    pending.stage = StepStage::ExecutionRun(run);
    Some(read)
}

fn step_paths(d: &mut Domain, env: &Env<Limits>, owner: Token, mut pending: PendingStep, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    if let (Some(before), Some(after), Some(repository)) =
        (pending.base_tip, pending.branch, d.repositories.get(&row.repository))
    {
        for held in &d.landing_rules(repository.project) {
            if !held.rule.sets.is_empty() && crate::covers_branch(&held.rule.pattern, repository, &row.base) {
                let repository = row.repository;
                pending.stage = StepStage::Paths;
                d.steps.insert(owner, pending).expect("replaces step");
                gather_read(d, env, owner, repository, skein_forge::api::Read::Compare { before, after }, out);
                return;
            }
        }
    }
    finish_step(d, env, pending, out);
}

fn step_touched(d: &Domain, row: &ChangeRow, pending: &PendingStep) -> crate::Touched {
    match d.repositories.get(&row.repository) {
        Some(repository) => crate::touched(&d.project_classes(repository.project), &pending.paths, pending.paths_whole),
        None => crate::Touched::UNKNOWN,
    }
}

/// Reverse ancestry is asked only when the head lacks the tip, once per pair
/// (temper/docs/design/domain/forge/change/README.md, 2).
fn read_contains_head(d: &mut Domain, pending: &mut PendingStep) -> Option<skein_forge::api::Read> {
    let head = pending.branch?;
    let tip = pending.base_tip.expect("base read before ancestry");
    let mut settle = d.settling.get(&pending.task).copied().unwrap_or(crate::settle::Settle::NEW);
    if let Some(proof) = settle.contains_head {
        if proof.head == head && proof.tip == tip {
            pending.contains_head = proof.status;
            return None;
        }
        settle.contains_head = None;
        d.settling.insert(pending.task, settle).expect("one settle row per change");
    }
    if head == tip {
        pending.contains_head = change::Status::Passed;
    } else if pending.contains_base == change::Status::Failed {
        pending.stage = StepStage::ContainsHead;
        return Some(skein_forge::api::Read::Commits { before: tip, after: head, page: 1 });
    }
    None
}

fn step_reviews(pending: &mut PendingStep) -> Option<skein_forge::api::Read> {
    let pull = pending.pull.as_ref()?;
    pending.stage = StepStage::Reviews(1);
    Some(skein_forge::api::Read::Reviews { number: pull.number, page: 1, skip: 0 })
}

/// Landings, repairs and held observations retain the head's context rows.
fn result_contexts(state: &change::State) -> bool {
    match state {
        change::State::First { .. }
        | change::State::Queued { .. }
        | change::State::Held { .. }
        | change::State::Repairing { .. } => true,
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
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
    }
}

/// Once the head's verdict is known, the same commit also serves as the base's CI.
fn step_after_ci(
    d: &mut Domain,
    pending: &mut PendingStep,
    out: &mut Queue<Request>,
) -> Option<skein_forge::api::Read> {
    let head = pending.branch.expect("head read before CI");
    keep_panel_ci(d, pending.task, change::Ci { head, status: pending.ci }, out);
    let base_tip = pending.base_tip.expect("base read before CI");
    let asked_base = match d.changes.get(&pending.task) {
        Some(row) => updating_base(&row.change.state).unwrap_or(base_tip),
        None => base_tip,
    };
    if head == base_tip && head == asked_base {
        pending.base_ci = pending.ci;
        pending.contains_base = change::Status::Passed;
        let read = read_contains_head(d, pending);
        assert!(read.is_none(), "equal commits need no ancestry read");
        step_reviews(pending)
    } else {
        pending.stage = StepStage::BaseCi;
        Some(skein_forge::api::Read::Checks { commit: base_tip })
    }
}

pub(crate) fn queue_ready(d: &Domain, row: &ChangeRow, l: &Limits) -> (List<change::Ready>, Option<u64>) {
    let mut base_repair = None;
    for (_, repair) in &d.changes {
        if repair.repository == row.repository
            && repair.base == row.base
            && repair.base_repair
            && crate::panel::ready(repair).is_some()
            && !d.approval_short(repair)
        {
            base_repair = Some(repair.task);
            break;
        }
    }
    let mut ready = List::with_capacity(l.changes);
    for (_, other) in &d.changes {
        if other.repository != row.repository || other.base != row.base || d.approval_short(other) {
            continue;
        }
        let since = match other.change.state {
            change::State::Queued { ready_since, .. } | change::State::First { ready_since, .. } => Some(ready_since),
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
            | change::State::NoChange { .. }
            | change::State::Ended
            | change::State::Landed { .. }
            | change::State::Red { .. }
            | change::State::Cleared { .. }
            | change::State::Uncleared { .. }
            | change::State::Held { .. } => None,
        };
        if let Some(since) = since {
            ready
                .push(change::Ready { task: other.task, priority: other.priority, since })
                .expect("changes bound ready");
        }
    }
    (ready, base_repair)
}

fn queue_first(d: &Domain, row: &ChangeRow, now: skein_lib::Wall, l: &Limits) -> bool {
    if d.approval_short(row) {
        return false;
    }
    first_in_queue(d, row, now, l) == Some(row.task)
}

pub(crate) fn first_in_queue(d: &Domain, row: &ChangeRow, now: skein_lib::Wall, l: &Limits) -> Option<u64> {
    let (ready, repair) = queue_ready(d, row, l);
    if let Some(task) = repair {
        return Some(task);
    }
    for (_, owner) in &d.changes {
        if owner.repository == row.repository
            && owner.base == row.base
            && owns_turn(owner)
            && (!in_queue(&owner.change.state) || !d.approval_short(owner))
        {
            return Some(owner.task);
        }
    }
    change::first(ready.as_slice(), l.queue_window, now)
}

/// Only a live turn can exclude a ready change (temper/docs/design/domain/forge/queues.md).
pub(crate) fn owns_turn(row: &ChangeRow) -> bool {
    if !row.change.owns_turn || row.ending.is_some() {
        return false;
    }
    match row.change.state {
        change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::Landing { .. } => true,
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::CancellingGates { .. }
        | change::State::Queued { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::Held { .. } => false,
    }
}

fn wake_after_turn(d: &mut Domain, row: &ChangeRow, owned: bool) {
    if owned && !owns_turn(row) {
        wake_queue(d, row);
    }
}

fn finish_step(d: &mut Domain, env: &Env<Limits>, pending: PendingStep, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    let held = matches_held(&row.change.state) && row.change.released.is_none();
    if let Some(behind) = pending.behind
        && !held
    {
        arm_gather(d, env, &pending, out);
        read_session::behind(d, pending.task, behind, out);
    }
    let until = if held { None } else { read_session::until(d, pending.task) };
    let expired = match until {
        Some(until) => env.wall >= until,
        None => false,
    };
    if pending.behind.is_some() && (!pending.woken || expired) {
        change_reads::wait(d, env, pending, out);
        return;
    }
    if pending.woken && expired {
        wait_for_forge(d, env, &pending, None, out);
        return;
    }
    if pending.woken {
        restart_step(d, env, pending, out);
    } else {
        read_session::forget(d, pending.task, out);
        crate::waits::changed(d, pending.task);
        decide_step(d, env, pending, out);
    }
}

fn waits_for_dependencies(row: &ChangeRow, pending: &PendingStep) -> bool {
    if let change::State::Producing { .. } = row.change.state { !pending.pending.is_empty() } else { false }
}

/// Missing bases and uncertain reads count a stall except while producing waits for dependencies.
fn wait_for_forge(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &PendingStep,
    retry: Option<skein_lib::Wall>,
    out: &mut Queue<Request>,
) {
    d.merge_withheld.remove(&pending.task);
    let Some(row) = d.changes.get(&pending.task) else { return };
    if row.terminal() {
        read_session::forget(d, pending.task, out);
        emit(out, Request::WakeProcedure { task: pending.task });
        return;
    }
    if matches_held(&row.change.state) && row.change.released.is_none() {
        read_session::forget(d, pending.task, out);
        emit(
            out,
            Request::ChangeDecision {
                task: pending.task,
                decision: change::Decision::None,
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    }
    if connection_wait::waiting(d, row.task, row.repository.forge) {
        connection_wait::wait(d, env, row.task, out);
        return;
    }
    // An unrequested close needs both tips. Retry the whole pair even if a
    // base confirmation's wake was coalesced with this procedure step.
    let retry = if close_unrequested(row) && retry.is_none() { Some(change_reads::retry_at(env)) } else { retry };
    arm_gather(d, env, pending, out);
    let until = match read_session::until(d, pending.task) {
        Some(until) if env.wall >= until => {
            hold_read_session(d, env, pending.task, Some(pending), out);
            return;
        }
        Some(until) => Some(match retry {
            Some(retry) => retry.min(until),
            None => until,
        }),
        None => retry,
    };
    let mut row = d.changes.get(&pending.task).expect("waiting change remains").clone();
    crate::readiness::reconcile(d, env, &mut row, None, out);
    if let Some(prior) = d.changes.insert(row.task, row.clone()).expect("replaces change")
        && prior != row
    {
        save_change(d, out, row);
    }
    emit(
        out,
        Request::ChangeDecision {
            task: pending.task,
            decision: change::Decision::Wait { until },
            entry: None,
            evidence: None,
            close_at: None,
        },
    );
}

fn hold_read_session(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    pending: Option<&PendingStep>,
    out: &mut Queue<Request>,
) {
    if wait_fresh(d, task, &[]) {
        return;
    }
    let Some(mut row) = d.changes.get(&task).cloned() else { return };
    d.merge_withheld.remove(&task);
    let before = crate::panel::ready(&row);
    let prior = row.change.clone();
    let owned = row.change.owns_turn;
    row.change.owns_turn = false;
    let was = if let change::State::Held { was, .. } = row.change.state { was } else { Box::new(row.change.state) };
    row.change.state = change::State::Held { was, hold: change::Hold::new(change::Cause::Stalled, env.wall) };
    row.change.released = None;
    let evidence = match pending {
        Some(pending) => step_evidence(d, &row, pending, prior),
        None => crate::ChangeEvidence {
            behind: read_session::last_behind(d, task),
            readback_owed: read_session::readback_owed(d, task),
            nearest_write: None,
            touched: crate::Touched::UNKNOWN,
            prior,
            head: row.change.last_head,
            base_tip: [0; 32],
            contains_base: change::Status::Unknown,
            ci: change::Status::Unknown,
            gates: Box::new([]),
            reviews: Box::new([]),
            reviews_complete: false,
            size: None,
        },
    };
    crate::readiness::reconcile(d, env, &mut row, None, out);
    d.read_sessions.remove(&task);
    d.changes.insert(task, row.clone()).expect("held read's change");
    wake_after_turn(d, &row, owned);
    save_queue_change(d, env, before, row, out);
    emit(
        out,
        Request::ChangeDecision {
            task,
            decision: change::Decision::Hold(change::Cause::Stalled),
            entry: None,
            evidence: Some(evidence),
            close_at: None,
        },
    );
}

pub(super) fn hold_room_change(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    account: crate::EffectAccount,
    out: &mut Queue<Request>,
) {
    let mut row = d.changes.get(&task).expect("change waiting for room").clone();
    let before = crate::panel::ready(&row);
    let owned = row.change.owns_turn;
    row.change.owns_turn = false;
    row.change.state = change::State::Held {
        was: Box::new(row.change.state),
        hold: change::Hold::new(change::Cause::Stalled, env.wall),
    };
    row.effect_account = Some(account);
    if let Some(owner) = step_owner(d, task) {
        d.steps.remove(&owner);
    }
    d.change_due.remove(&task);
    d.read_sessions.remove(&task);
    d.fresh_steps.remove(&task);
    d.fresh_waits.remove(&task);
    d.procedure_wakes.remove(&task);
    d.merge_withheld.remove(&task);
    connection_wait::take(d, task);
    crate::room::pass(d, task);
    wake_after_turn(d, &row, owned);
    save_queue_change(d, env, before, row, out);
    emit(
        out,
        Request::ChangeDecision {
            task,
            decision: change::Decision::Hold(change::Cause::Stalled),
            entry: None,
            evidence: None,
            close_at: None,
        },
    );
}

fn titled_change(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &PendingStep,
    out: &mut Queue<Request>,
) -> Option<ChangeRow> {
    let task = pending.task;
    let mut row = d.changes.get(&task)?.clone();
    let producing = match row.delegate {
        Some((_, kind)) => kind == change::Delegate::Produce,
        None => false,
    };
    if row.title.is_empty()
        && d.start_conflict(task).is_none()
        && (producing || row.delegate_status == change::Status::Passed)
        && row.change.state == (change::State::Producing { requested: true })
    {
        if agent_asking(&row, pending.heard.delegate_live) || waits_for_dependencies(&row, pending) {
            row.change.since = env.wall;
            d.changes.insert(task, row.clone()).expect("replaces change");
            save_change(d, out, row);
            emit(
                out,
                Request::ChangeDecision {
                    task,
                    decision: change::Decision::Wait { until: None },
                    entry: None,
                    evidence: None,
                    close_at: None,
                },
            );
            return None;
        }
        if pending.heard.delegate_live {
            row.change.since = env.wall;
            d.changes.insert(task, row.clone()).expect("replaces change");
            save_change(d, out, row.clone());
        }
        let until = skein_lib::Wall::from_nanos(
            row.change.since.as_nanos().saturating_add(env.limits.change_policy.stall.as_nanos()),
        );
        let decision = if env.wall >= until {
            let before = crate::panel::ready(&row);
            let owned = row.change.owns_turn;
            row.change.owns_turn = false;
            row.change.state = change::State::Held {
                was: Box::new(row.change.state),
                hold: change::Hold::new(change::Cause::Stalled, env.wall),
            };
            wake_after_turn(d, &row, owned);
            d.changes.insert(task, row.clone()).expect("replaces change");
            save_queue_change(d, env, before, row, out);
            change::Decision::Hold(change::Cause::Stalled)
        } else {
            change::Decision::Wait { until: Some(until) }
        };
        emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None, close_at: None });
        return None;
    }
    Some(row)
}

fn change_standing(d: &Domain, env: &Env<Limits>, row: &ChangeRow, pending: &PendingStep) -> change::Heard {
    // An uncertain write can outlive the outbox entry whose result says Failed.
    let effect_pending = row.pending.is_some()
        || (row.change.released == Some(change::Cause::Failed)
            && failed_merge_hold(&row.change.state)
            && writes::release_readback(d, &env.limits, row));
    change::Heard {
        flake: row.delegate_flake && row.delegate_head == pending.branch,
        no_test: row.delegate_no_test && row.delegate_head == pending.branch,
        // Terminal evidence belongs to the current delegate, not to the read's start.
        delegate: if row.delegate_status == change::Status::Unknown {
            pending.heard.delegate
        } else {
            row.delegate_status
        },
        start_conflict: d.start_conflict(row.task),
        review_request: d.review_requests.get(&row.task).copied(),
        effect: if effect_pending { change::EffectResult::Pending } else { row.effect },
        ..pending.heard
    }
}

#[expect(clippy::too_many_lines, reason = "one coherent snapshot records a green repair ending before stepping onward")]
fn decide_step(d: &mut Domain, env: &Env<Limits>, mut pending: PendingStep, out: &mut Queue<Request>) {
    if !repository_reads::change_fresh(d, pending.task) {
        d.fresh_steps.insert(pending.task, FreshStep::Gather(Box::new(pending))).expect("one parked step per change");
        return;
    }
    pending.progress = List::uncapped();
    let Some(mut row) = titled_change(d, env, &pending, out) else { return };
    crate::job_history::merged(d, env.wall, &mut row, pending.branch, out);
    let before = crate::panel::ready(&row);
    let prior = row.change.clone();
    if row.room.is_some() {
        row.change.since = env.wall;
    }
    crate::readiness::way(d, &mut row);
    // Producing without a head uses no base facts except for closes, early starts and catch-ups.
    let base_tip = pending.base_tip.unwrap_or([0; 32]);
    row.pull = match &pending.pull {
        Some(pull) => Some(pull.number),
        None => row.pull,
    };
    if let Some(base_tip) = pending.base_tip {
        crate::settle::observe(d, env, &row, pending.pull.as_ref(), base_tip);
    }
    if pending.behind.is_none()
        && let Some(head) = pending.branch
        && takes_released_head(&row.change, head)
    {
        accept_released_branch(d, env, &mut row, head, out);
    }
    let touched = step_touched(d, &row, &pending);
    d.keep_touched(row.task, pending.branch, touched);
    if let Some(repository) = d.repositories.get(&row.repository) {
        let gates = match d.step_gates(&env.limits, &row, repository, touched) {
            Ok(gates) => gates,
            Err(crate::RefusalCause::InvalidName) => {
                emit(out, Request::Refused { task: row.task });
                return;
            }
            Err(
                cause @ (crate::RefusalCause::Limit { .. }
                | crate::RefusalCause::Unadopted
                | crate::RefusalCause::Backend(_)),
            ) => {
                room::refused(out, b"step change", &row, cause);
                return;
            }
        };
        row.change.gates = gates;
    }
    if let Err(needed) = crate::follow_check::select(d, &mut row, &env.limits) {
        room::refused(
            out,
            b"step change",
            &row,
            crate::RefusalCause::Limit {
                member: "forge.change_policy.gates",
                configured: u64::from(env.limits.change_policy.gates),
                needed,
            },
        );
        return;
    }
    // A workflow has its own runner-fault lifetime, not an agent's stall allowance.
    if crate::follow_check::asked(&row).is_some() {
        row.change.since = env.wall;
    }
    crate::close::judged(&mut row, pending.branch.is_some());
    if let Some(recheck) = &mut row.ci_recheck {
        if pending.branch.is_some() && pending.branch != Some(recheck.job.head) {
            row.ci_recheck = None;
        } else if let change::State::Checking { .. } = row.change.state
            && pending.branch == Some(recheck.job.head)
            && recheck.taken.is_none()
            && let Some(attempt) = pending.failed.measurement(*recheck)
        {
            recheck.taken = Some(attempt);
        }
    }
    let mut facts = change_facts(d, env, &row, &pending, base_tip);
    let mut heard = change_standing(d, env, &row, &pending);
    let mut regate = if let Some(why) = crate::recheck::released_repair(&row.change)
        && crate::recheck::landing(why)
    {
        d.release_regate(&row, why)
    } else if let Some(why) = crate::recheck::landing_repair(&row.change.state)
        && heard.delegate == change::Status::Passed
    {
        d.landing_regate(row.task, why)
    } else {
        None
    };
    facts.regate = regate.is_some();
    let live_close = match row.delegate {
        Some((_, change::Delegate::Close)) => pending.heard.delegate_live,
        Some(_) | None => false,
    };
    if row.delegate_status == change::Status::Passed && !live_close {
        row.delegate = None;
    }
    let mut reviewed = false;
    for _ in 0_u32..16 {
        if !reviewed && in_queue(&row.change.state) && !d.panel_approvals(&row).is_empty() {
            emit(
                out,
                Request::Reviewed { task: row.task, evidence: step_evidence(d, &row, &pending, row.change.clone()) },
            );
            reviewed = true;
        }
        let effect_prior = row.change.clone();
        let mut stepped = change::step(&row.change, &facts, &heard, env.wall, &env.limits.change_policy);
        if let change::Decision::ReadMerge { head } = stepped.decision {
            read_merge_hooks(d, env, pending, row.repository, head, out);
            return;
        }
        if stepped.decision == change::Decision::Hold(change::Cause::Failed)
            && row.catch_up.is_some()
            && effect_prior.state == (change::State::Producing { requested: false })
        {
            row.producer_failure = Some(catch_up_command_failure(d, &row, base_tip));
        }
        d.merge_withheld.remove(&row.task);
        if d.landing.held
            && let change::Decision::Effect(change::Effect::Merge { .. }) = stepped.decision
        {
            d.merge_withheld.insert(row.task).expect("one withheld merge per bounded change");
            stepped.change = row.change.clone();
            stepped.decision = change::Decision::Wait { until: None };
        }
        d.review_requests.remove(&row.task);
        heard.review_request = None;
        let advanced = stepped.change != row.change;
        if stepped.decision == change::Decision::Hold(change::Cause::Stalled)
            && pending.stall.is_none()
            && let change::State::Checking { head } = row.change.state
        {
            pending.stage = StepStage::StallChecks;
            d.sequence = d.sequence.checked_add(1).expect("owner ids fit");
            let owner = Token::new(d.sequence | (1_u64 << 62));
            let repository = row.repository;
            d.stall_words.remove(&row.task);
            d.steps.insert(owner, pending).expect("one step per change");
            gather_read(d, env, owner, repository, skein_forge::api::Read::Checks { commit: head }, out);
            return;
        }
        let released = held_cause(&effect_prior.state);
        if stepped.decision == change::Decision::EndHold {
            take_engine_revision(d, &mut row, env.wall);
            fence_release(d, env, &mut row, out);
        }
        let measured = if stepped.decision == change::Decision::Hold(change::Cause::RedStands)
            && !change::held_on_ci(&stepped.change.state)
            && facts.recheck == change::Recheck::Taken
            && effect_prior.released != Some(change::Cause::RedStands)
            && let Some(ask) = &facts.ask
        {
            d.regate_failure(&row, ask)
        } else {
            None
        };
        row.change = stepped.change;
        if let Some(cause) = released
            && (stepped.decision == change::Decision::EndHold || effect_prior.released.is_some())
            && !matches_held(&row.change.state)
            && !row.terminal()
        {
            forget_released_step(&mut row, &effect_prior.state);
            if cause == change::Cause::CloseRefused && facts.measure {
                crate::close::clear_findings(&mut row);
                facts.close_at = row.close_at;
            }
            if cause == change::Cause::Runner
                && let Some(head) = pending.branch
            {
                row.runner_ci = Some(crate::RunnerCi::Counting { attempts: pending.failed.baseline(head) });
            }
            heard = change_standing(d, env, &row, &pending);
        }
        if let Some(took) = stepped.took {
            crate::landing::members::take(&mut row, took, out);
            facts.ask = crate::landing::members::head(&row).cloned();
        }
        if let change::Decision::Delegate(change::Delegate::Gate { .. }) = stepped.decision {
            row.gate_revision = facts.revision;
        }
        if row.change.last_head != effect_prior.last_head {
            if let Some(crate::RunnerCi::Counting { .. }) = &row.runner_ci {
                row.runner_ci = None;
            }
            row.regate = None;
            row.ci_recheck = None;
        }
        take_landing_repair(
            &mut row,
            &mut regate,
            &effect_prior,
            &facts,
            heard.delegate,
            stepped.decision,
            env.limits.client.answer_bytes,
        );
        take_ci_repair(&mut row, &effect_prior, &facts, heard.delegate, env.limits.client.answer_bytes);
        if stepped.decision == change::Decision::Hold(change::Cause::RedStands) {
            row.producer_failure = if change::held_on_ci(&row.change.state) {
                match (row.change.last_head, row.repair_tasks.last()) {
                    (Some(commit), Some(repair)) => {
                        let checked = facts.recheck == change::Recheck::Taken
                            && effect_prior.released != Some(change::Cause::RedStands);
                        Some(pending.failed.red_stands(commit, *repair, checked))
                    }
                    (None, _) | (_, None) => None,
                }
            } else {
                crate::recheck::hold_words(&row, measured)
            };
        }
        if let change::Decision::Delegate(change::Delegate::Repair(_)) = stepped.decision {
            row.regate = None;
            row.ci_recheck = None;
        }
        facts.recheck = step_recheck(d, &row, &pending, facts.ask.as_ref());
        pending.progress.push((effect_prior.clone(), row.change.clone())).expect("bounded policy transitions");
        let decision = stepped.decision;
        // If the landing tip changed, the update's containment cannot serve Checking.
        let reread_base = matches_updating(&effect_prior.state)
            && !matches_updating(&row.change.state)
            && updating_base(&effect_prior.state) != Some(facts.base_tip)
            && decision == change::Decision::None;
        let decision = if reread_base { change::Decision::Wait { until: Some(env.wall) } } else { decision };
        match decision {
            change::Decision::Wait { .. } if crate::follow_check::asked(&row).is_some() => {
                drive_check(d, env, row, pending, None, out);
                return;
            }
            change::Decision::Delegate(change::Delegate::Gate { number: change::FOLLOW_CHECK_GATE, .. }) => {
                drive_check(d, env, row, pending, None, out);
                return;
            }
            change::Decision::None if advanced => {}
            change::Decision::ReadMerge { .. } => unreachable!("hook reads are handled before saving the decision"),
            change::Decision::RepairEnded { head } => {
                if row.ci_recheck.is_some() {
                    // An independently green head ends a CI repair, including one lifted from RedStands.
                    facts.recheck = change::Recheck::None;
                    if heard.effect == change::EffectResult::Failed || heard.effect == change::EffectResult::Conflict {
                        heard.effect = change::EffectResult::None;
                        row.effect = change::EffectResult::None;
                    }
                }
                row.result.ci_passed(&head, env.limits.client.answer_bytes);
                row.producer_failure = None;
                row.delegate_status = change::Status::Unknown;
                row.delegate_flake = false;
                row.delegate_no_test = false;
                row.delegate_words = None;
                heard.delegate = change::Status::Unknown;
            }
            change::Decision::Effect(effect) => {
                // A veto restores the ready state before applying the kept review facts.
                let prior = match effect {
                    change::Effect::Merge { .. } => effect_prior,
                    change::Effect::FollowCheck { .. }
                    | change::Effect::Open
                    | change::Effect::CreateBranch { .. }
                    | change::Effect::Reopen
                    | change::Effect::Prepare { .. }
                    | change::Effect::Retarget
                    | change::Effect::Update { .. }
                    | change::Effect::Rerun { .. } => prior,
                };
                emit_change_effect(d, env, &mut row, &pending, prior, effect, out);
                return;
            }
            decision @ (change::Decision::Failed
            | change::Decision::None
            | change::Decision::CancelProducer
            | change::Decision::Wait { .. }
            | change::Decision::Delegate(_)
            | change::Decision::CancelGates { .. }
            | change::Decision::CancelRepair { .. }
            | change::Decision::CancelMovedDelegate { .. }
            | change::Decision::EndHold
            | change::Decision::Ready
            | change::Decision::QueueRepair
            | change::Decision::NoChange
            | change::Decision::Finish { .. }
            | change::Decision::Cancel
            | change::Decision::ReplayJudgement(_)
            | change::Decision::Hold(_)) => {
                if decision == change::Decision::Hold(change::Cause::Runner) {
                    let commit = pending.branch.expect("runner fault at the checked head");
                    let words = pending.failed.hold_words(commit, crate::change_ci::baseline(&row));
                    row.runner_ci = Some(crate::RunnerCi::Held { words });
                }
                row.room = None;
                crate::room::leave(d, crate::room::Waiter::Change(row.task));
                keep_step_progress(d, env, &mut row, &pending, &facts, out);
                if let Some(report) = pending.stall.take() {
                    d.stall_words.insert(row.task, report).expect("bounded tasks");
                }
                crate::change_history::advanced(d, env.wall, &mut row, &prior, out);
                crate::landing::change_advanced(d, &row, &prior, out);
                let withdrawn = take_landed_merge(d, &mut row);
                d.changes.insert(row.task, row.clone()).expect("replaces change");
                if let Some(entry) = withdrawn {
                    withdraw(d, env, entry, out);
                }
                if landed_commit(&prior.state).is_none()
                    && let Some(commit) = landed_commit(&row.change.state)
                {
                    reds::landed(d, &env.limits, &row, out);
                    crate::landing_log::landed(d, env.wall, commit, row.task, Box::new([]), out);
                }
                if !prior.owns_turn && in_queue(&prior.state) && !in_queue(&row.change.state) {
                    wake_queue(d, &row);
                }
                wake_after_turn(d, &row, prior.owns_turn);
                save_queue_change(d, env, before, row.clone(), out);
                topics::refresh(d, env, row.task, out);
                crate::waits::change_policy(d, &row, decision, facts.execution.is_some());
                if let change::Decision::ReplayJudgement(replay) = decision {
                    let prior_rejection = if let change::Replay::Rejected { .. } = replay
                        && let change::State::Held { hold, .. } = effect_prior.state
                    {
                        hold.words
                    } else {
                        Box::new([])
                    };
                    emit(out, Request::ReplayJudgement { task: row.task, replay, prior_rejection });
                    return;
                }
                let close_at = if decision == change::Decision::Delegate(change::Delegate::Close) {
                    facts.close_tips
                } else {
                    None
                };
                let choice = if close_at.is_some() {
                    decision
                } else {
                    repair_poll(
                        env,
                        &row,
                        &facts,
                        &heard,
                        execution_poll(
                            env,
                            &facts,
                            execution_due(&row.change.state, &pending),
                            row.ci_recheck.is_some() && facts.recheck == change::Recheck::Owed,
                            crate::settle::cap(d, env, &row, &facts, decision),
                        ),
                    )
                };
                emit(
                    out,
                    Request::ChangeDecision {
                        task: row.task,
                        decision: choice,
                        entry: None,
                        evidence: if decision == change::Decision::Hold(change::Cause::BranchMoved) {
                            Some(step_evidence(d, &row, &pending, effect_prior))
                        } else {
                            None
                        },
                        close_at,
                    },
                );
                return;
            }
        }
    }
    emit(out, Request::Refused { task: row.task });
}

fn drive_check(
    d: &mut Domain,
    env: &Env<Limits>,
    mut row: ChangeRow,
    mut pending: PendingStep,
    action: Option<crate::follow_check::Action>,
    out: &mut Queue<Request>,
) {
    use crate::follow_check::Action;
    let head = if let change::State::Judging { target, .. } = &row.change.state {
        target.prefix
    } else {
        pending.branch.expect("follow check was asked at a read head")
    };
    let action = match action {
        Some(action) => action,
        None => crate::follow_check::advance(d, env, &mut row, head, out),
    };
    match action {
        Action::Write => {
            d.read_sessions.remove(&row.task);
            let prior = row.change.clone();
            emit_change_effect(d, env, &mut row, &pending, prior, change::Effect::FollowCheck { head }, out);
        }
        Action::Read { kind, read } => {
            let facts = change_facts(d, env, &row, &pending, pending.base_tip.unwrap_or([0; 32]));
            keep_step_progress(d, env, &mut row, &pending, &facts, out);
            pending.progress = List::uncapped();
            pending.stage = StepStage::FollowCheck(kind);
            d.sequence = d.sequence.checked_add(1).expect("follow check read sequence");
            let owner = Token::new(d.sequence | (1_u64 << 62));
            let repository = row.repository;
            d.changes.insert(row.task, row.clone()).expect("known change");
            save_change(d, out, row);
            arm_gather(d, env, &pending, out);
            d.steps.insert(owner, pending).expect("one read per change");
            gather_read(d, env, owner, repository, read, out);
        }
        Action::Done | Action::Wait | Action::Held => {
            let facts = change_facts(d, env, &row, &pending, pending.base_tip.unwrap_or([0; 32]));
            keep_step_progress(d, env, &mut row, &pending, &facts, out);
            row.room = None;
            crate::room::leave(d, crate::room::Waiter::Change(row.task));
            let task = row.task;
            d.read_sessions.remove(&task);
            let decision = match action {
                Action::Held => {
                    row.change.state = change::State::Held {
                        was: Box::new(row.change.state),
                        hold: change::Hold::new(change::Cause::Failed, env.wall),
                    };
                    change::Decision::Hold(change::Cause::Failed)
                }
                Action::Wait => {
                    let at = d.client.poll_at(&env.limits.client, env.now, row.repository, env.limits.client.poll);
                    change::Decision::Wait {
                        until: Some(skein_lib::Wall::from_nanos(
                            env.wall.as_nanos().saturating_add(at.as_nanos().saturating_sub(env.now.as_nanos())),
                        )),
                    }
                }
                Action::Done => change::Decision::Wait { until: None },
                Action::Read { .. } | Action::Write => unreachable!("settled check action"),
            };
            let quiet = if let Action::Wait = action { change::Decision::Wait { until: None } } else { decision };
            crate::waits::change_policy(d, &row, quiet, false);
            crate::readiness::reconcile(d, env, &mut row, None, out);
            d.changes.insert(task, row.clone()).expect("known change");
            save_change(d, out, row);
            emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None, close_at: None });
            if let Action::Done = action {
                emit(out, Request::WakeProcedure { task });
            }
        }
    }
}

/// Poll both the first reading after entering checking and later readings (stalls.md, 2.1).
fn execution_poll(
    env: &Env<Limits>,
    facts: &change::Facts,
    due: bool,
    measuring: bool,
    decision: change::Decision,
) -> change::Decision {
    if (facts.execution.is_some() || due || measuring)
        && let change::Decision::Wait { until } = decision
    {
        let delay = if facts.execution.is_some() || measuring { env.limits.client.poll_max } else { Duration::ZERO };
        let poll = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(delay.as_nanos()));
        return change::Decision::Wait { until: Some(until.unwrap_or(poll).min(poll)) };
    }
    decision
}

fn repair_poll(
    env: &Env<Limits>,
    row: &ChangeRow,
    facts: &change::Facts,
    heard: &change::Heard,
    decision: change::Decision,
) -> change::Decision {
    if let change::State::Repairing { why, .. } = row.change.state
        && why.ci()
        && heard.delegate_running
        && facts.branch == Some(facts.ci.head)
        && facts.ci.status == change::Status::Passed
        && let change::Decision::Wait { until: Some(until) } = decision
    {
        let poll = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.client.poll.as_nanos()));
        return change::Decision::Wait { until: Some(until.min(poll)) };
    }
    decision
}

pub(crate) fn in_queue(state: &change::State) -> bool {
    match state {
        change::State::Queued { .. } | change::State::First { .. } => true,
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
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::Held { .. } => false,
    }
}

/// New review facts change membership without changing the change's first ready time.
fn reviews_changed(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(mut row) = d.changes.get(&task).cloned() else { return };
    if row.frozen() {
        return;
    }
    crate::waits::changed(d, task);
    let yielded = yield_approval_turn(d, &mut row);
    crate::readiness::reviews(d, env, &mut row, out);
    if yielded {
        d.changes.insert(task, row.clone()).expect("replaces change");
        save_change(d, out, row.clone());
    }
    if row.change.way != change::Way::Queue {
        return;
    }
    if let Some(tasks) = crate::views::changes_mark(d, row.repository) {
        emit(out, Request::Marked { tasks: Box::from(tasks) });
    }
    wake_queue(d, &row);
    if in_queue(&row.change.state) {
        if d.approval_short(&row) {
            d.wait_change(env, task, None);
        } else {
            d.procedure_wakes.insert(task, true).expect("one pending wake per change");
        }
    }
}

fn yield_approval_turn(d: &mut Domain, row: &mut ChangeRow) -> bool {
    let change::State::First { head, ready_since } = row.change.state else { return false };
    if !d.approval_short(row) {
        return false;
    }
    row.change.state = change::State::Queued { head, ready_since };
    row.change.owns_turn = false;
    wake_queue(d, row);
    true
}

fn wake_queue(d: &mut Domain, row: &ChangeRow) {
    for (_, other) in &d.changes {
        if other.task != row.task
            && other.repository == row.repository
            && other.base == row.base
            && in_queue(&other.change.state)
        {
            d.procedure_wakes.insert(other.task, true).expect("one pending wake per change");
            d.wait_reports.changed(other.task);
        }
    }
}

/// The change's branch hold, including its shared writer slot (temper/docs/design/domain/forge/change/README.md, 1).
pub(crate) fn change_branch_hold(d: &Domain, row: &ChangeRow) -> Option<(u64, bool)> {
    for (name, hold) in &d.holds {
        if name.forge == row.repository.forge
            && name.repository == row.repository.repository
            && crate::items::branch_matches(name, &row.branch)
        {
            return Some((hold.task, hold.writer.is_some()));
        }
    }
    None
}

// The change's ended pushing child leaves no wait after its branch read.
// Other holders and effect entries keep their own settlement.
fn change_writers(d: &Domain, row: &ChangeRow, pending: &PendingStep) -> (bool, bool) {
    match change_branch_hold(d, row) {
        Some((holder, taken)) => (
            taken && !change_requester_writing(d, row) && !ended_branch_writer(d, row, pending)
                || held_branch_merge(d, row),
            holder != row.task,
        ),
        None => (false, false),
    }
}

fn ended_branch_writer(d: &Domain, row: &ChangeRow, pending: &PendingStep) -> bool {
    if pending.heard.delegate_live || !pending.branch_fresh {
        return false;
    }
    for (name, hold) in &d.holds {
        if name.forge == row.repository.forge
            && name.repository == row.repository.repository
            && crate::items::branch_matches(name, &row.branch)
        {
            return match hold.writer {
                Some(Writer::Run { task, .. } | Writer::Lost { task, .. }) => hold.task == row.task && task != row.task,
                Some(Writer::Entry(_)) | None => false,
            };
        }
    }
    false
}

/// The finished landing repair is read once, with the head it was asked to repair.
fn take_landing_repair(
    row: &mut ChangeRow,
    regate: &mut Option<crate::Regate>,
    prior: &change::Change,
    facts: &change::Facts,
    status: change::Status,
    decision: change::Decision,
    bound: u32,
) {
    let withheld = decision == change::Decision::Hold(change::Cause::RedStands);
    if let Some(why) = crate::recheck::released_repair(prior)
        && crate::recheck::landing(why)
    {
        if decision == change::Decision::None
            && prior.last_head == facts.branch
            && let change::State::Gating { .. } | change::State::Checking { .. } = row.change.state
        {
            row.regate = regate.take();
        }
        row.delegate_flake = false;
        row.delegate_no_test = false;
        return;
    }
    if crate::recheck::landing_repair(&prior.state).is_none() {
        return;
    }
    let rereading =
        decision == change::Decision::None && prior.state != row.change.state && status == change::Status::Passed;
    let ended = row.terminal() && status == change::Status::Passed && facts.branch.is_some();
    if rereading || withheld || ended {
        let unchanged = facts.branch.is_some() && facts.branch == facts.repair_head.or(prior.last_head);
        if rereading && unchanged {
            row.regate = regate.take();
        }
        crate::recheck::completed(row, unchanged, bound);
    }
}

fn ci_repair(state: &change::State) -> bool {
    let change::State::Repairing { requested: true, why } = state else { return false };
    why.ci()
}

fn take_ci_repair(
    row: &mut ChangeRow,
    prior: &change::Change,
    facts: &change::Facts,
    status: change::Status,
    bound: u32,
) {
    if ci_repair(&prior.state)
        && status == change::Status::Passed
        && prior.state != row.change.state
        && let Some(head) = facts.branch
    {
        crate::recheck::completed(row, Some(head) == facts.repair_head.or(prior.last_head), bound);
    }
}

fn step_recheck(d: &Domain, row: &ChangeRow, pending: &PendingStep, ask: Option<&change::Handed>) -> change::Recheck {
    if let change::State::Checking { .. } = row.change.state {
        let Some(recheck) = ci_recheck_at(row, pending.branch) else { return change::Recheck::None };
        if recheck.taken.is_some()
            && (row.effect == change::EffectResult::Failed || row.effect == change::EffectResult::Conflict)
        {
            // The failed write is a runner rerun after the recheck, not a refusal to recheck.
            return change::Recheck::None;
        }
        return if pending.failed.measurement(recheck).is_some() {
            change::Recheck::Taken
        } else {
            change::Recheck::Owed
        };
    }
    if let Some(why) = crate::recheck::released_repair(&row.change)
        && pending.branch.is_some()
        && pending.branch == row.change.last_head
    {
        let taken = if why.ci() {
            if let Some(recheck) = ci_recheck_at(row, pending.branch) { recheck.taken.is_some() } else { false }
        } else if let Some(regate) = &row.regate {
            regate.taken.is_some()
        } else {
            false
        };
        return if taken { change::Recheck::Taken } else { change::Recheck::None };
    }
    landing_recheck(d, row, pending.branch, ask)
}

fn landing_recheck(
    d: &Domain,
    row: &ChangeRow,
    branch: Option<skein_forge::api::Commit>,
    ask: Option<&change::Handed>,
) -> change::Recheck {
    if branch.is_none() || branch != row.change.last_head {
        return change::Recheck::None;
    }
    let Some(handed) = ask else { return change::Recheck::None };
    let change::Ask::Repair { .. } = handed.ask else { return change::Recheck::None };
    let Some(regate) = &row.regate else { return change::Recheck::None };
    if regate.taken.is_none() {
        change::Recheck::Owed
    } else if d.regate_failure(row, handed).is_some() {
        change::Recheck::Taken
    } else {
        change::Recheck::None
    }
}

fn measured_release(row: &ChangeRow) -> bool {
    if let Some(release) = &row.task_release { release.measure } else { false }
}

fn can_rerun(row: &ChangeRow, pending: &PendingStep) -> bool {
    let Some(job) = pending.failed.job(pending.branch.unwrap_or([0; 32])) else { return false };
    if row.change.released == Some(change::Cause::RedStands)
        && let Some(prior) = ci_recheck_at(row, pending.branch)
        && prior.job.run == job.run
        && prior.job.job == job.job
    {
        // A stale run must not let a previously taken attempt take the fresh check.
        return job.attempt >= prior.taken.unwrap_or(prior.job.attempt);
    }
    true
}

#[expect(clippy::too_many_lines, reason = "the coherent change snapshot includes question and early-start facts")]
fn change_facts(
    d: &Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    pending: &PendingStep,
    base_tip: skein_forge::api::Commit,
) -> change::Facts {
    let (verification_bound, wait) = d.change_verification_settings(row);
    let pull = match &pending.pull {
        Some(pull) if pull.merged.is_some() => change::Pull::Merged { commit: pull.merged.expect("checked merged") },
        Some(pull) if pull.state == skein_forge::api::State::Closed => change::Pull::Closed,
        Some(pull) => change::Pull::Open { head: pull.commit, base: pull.base_commit.unwrap_or(base_tip) },
        None => change::Pull::Missing,
    };
    let (writer_taken, requester_branch) = change_writers(d, row, pending);
    let may_merge = match d.repositories.get(&row.repository) {
        Some(repository) => repository.kinds.land,
        None => false,
    };
    let has_ci = match d.repositories.get(&row.repository) {
        Some(repository) => repository.ci,
        None => true,
    };
    let requires = match d.repositories.get(&row.repository) {
        Some(repository) => d.requires(repository, &row.base, step_touched(d, row, pending)),
        None => crate::Requires { ci: true, up_to_date: true },
    };
    let window = crate::settle::window(d, row.task);
    change::Facts {
        release_blocked: d.task_holds.get(&row.task) == Some(&TaskHold::Blocked),
        produced_nothing: if row.change.released.is_some() { produced_nothing(row) } else { None },
        revision: row.spec_revision,
        taken: row.kept,
        fault: if pending.ci == change::Status::Failed {
            pending.failed.fault(pending.branch.unwrap_or([0; 32]), crate::change_ci::baseline(row))
        } else {
            change::Fault::Change
        },
        rerun: match d.configured.get(&row.repository.forge) {
            Some(config) => config.kind == crate::Kind::Forgejo && can_rerun(row, pending),
            None => false,
        },
        regate: false,
        measure: row.change.released.is_some() && measured_release(row),
        semantic_recheck: if let Some(recheck) = row.ci_recheck { recheck.semantic } else { false },
        recheck: step_recheck(d, row, pending, crate::landing::members::head(row)),
        repair_head: row.delegate_head,
        pending: pending.pending.clone(),
        verification: d.change_verification(row.task),
        verification_bound,
        wait,
        ask: crate::landing::members::head(row).cloned(),
        landing: d.landing_of(row.task),
        branch: pending.branch,
        titled: !row.title.is_empty() || row.pull.is_some(),
        expected_base: base_tip,
        pull,
        ci: change::Ci {
            head: pending.branch.unwrap_or([0; 32]),
            status: if has_ci && requires.ci { pending.ci } else { change::Status::Passed },
        },
        execution: match &pending.execution {
            Some(gather) => Some(gather.reading()),
            None => None,
        },
        base_ci: change::Ci {
            head: base_tip,
            status: if !has_ci || !requires.ci || row.base_repair { change::Status::Passed } else { pending.base_ci },
        },
        base_tip,
        base_read: pending.base_read.unwrap_or(base_tip),
        contains_base: if requires.up_to_date { pending.contains_base } else { change::Status::Passed },
        contains_head: pending.contains_head,
        updates: requires.up_to_date,
        merge_job: merge_available(d, row, pending, base_tip),
        delegate: match row.delegate {
            Some((_, kind)) => Some(kind),
            None => None,
        },
        catch_up: row.catch_up,
        follows: row.follows.is_some(),
        close: row.close.is_some(),
        close_tips: if let Some(branch) = pending.base_read
            && let Some(landing) = pending.close_landing
        {
            Some(change::CloseAt { branch, landing, asked: true, refused: None, outdated: None })
        } else {
            None
        },
        close_at: row.close_at,
        contains_catch_up: pending.contains_catch_up,
        mergeable: match &pending.pull {
            Some(pull) if pull.mergeable => change::Status::Passed,
            Some(_) if window.is_some() => change::Status::Pending,
            Some(_) => change::Status::Failed,
            None => change::Status::Unknown,
        },
        gates: crate::follow_check::reports(row, &pending.gates, env.limits.change_policy.gates),
        live_gate: match row.delegate {
            Some((_, change::Delegate::Gate { head, .. })) => Some(head),
            Some(_) | None => None,
        },
        writer_taken,
        requester_branch,
        drift: match pending.drift {
            Some(drift) => Some(drift.why),
            None => None,
        },
        behind: match pending.behind {
            Some(_) => read_session::since(d, pending.task),
            None => None,
        },
        update_head: change_reads::update_head(d, env, row, pending.branch),
        first: queue_first(d, row, env.wall, &env.limits),
        may_merge,
        queue_repair_active: pending.queue_repair_active,
        kept: pending.kept,
        asking: agent_asking(row, pending.heard.delegate_live),
        settling: match window {
            Some((_, end)) => Some(end),
            None => None,
        },
    }
}

fn agent_asking(row: &ChangeRow, live: bool) -> bool {
    live && has_agent_question(row)
}

fn has_agent_question(row: &ChangeRow) -> bool {
    let Some((task, _)) = row.delegate else { return false };
    for question in &row.questions.open {
        if question.asker == task {
            return true;
        }
    }
    false
}

fn step_evidence(d: &Domain, row: &ChangeRow, pending: &PendingStep, prior: change::Change) -> crate::ChangeEvidence {
    crate::ChangeEvidence {
        behind: pending.behind.or(read_session::last_behind(d, pending.task)),
        readback_owed: read_session::readback_owed(d, pending.task),
        nearest_write: match pending.drift {
            Some(drift) => drift.nearest,
            None => None,
        },
        touched: step_touched(d, row, pending),
        prior,
        head: pending.branch,
        // An unfinished gather supplies evidence only to name its read stall.
        base_tip: pending.base_tip.unwrap_or([0; 32]),
        contains_base: pending.contains_base,
        ci: if match d.repositories.get(&row.repository) {
            Some(repository) => repository.ci,
            None => true,
        } {
            pending.ci
        } else {
            change::Status::Passed
        },
        gates: pending.gates.clone(),
        reviews: pending.reviews.clone(),
        reviews_complete: pending.reviews_complete,
        size: match &pending.pull {
            Some(pull) if Some(pull.commit) == pending.branch => pull.size,
            Some(_) | None => None,
        },
    }
}

fn keep_step_progress(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    pending: &PendingStep,
    facts: &change::Facts,
    out: &mut Queue<Request>,
) {
    let final_change = row.change.clone();
    for (prior, next) in &pending.progress {
        row.change = next.clone();
        crate::readiness::advanced(d, env, row, prior, facts, out);
        if matches_updating(&prior.state)
            && row.change.last_head != prior.last_head
            && let Some(head) = row.change.last_head
        {
            record_change_head(d, env, row, head, out);
            row.history = d.changes.get(&row.task).expect("retained change").history;
        }
    }
    row.change = final_change;
}

fn leave_room_change(d: &mut Domain, task: u64, out: &mut Queue<Request>) {
    crate::room::leave(d, crate::room::Waiter::Change(task));
    if let Some(row) = d.changes.get_mut(&task)
        && row.room.take().is_some()
    {
        let row = row.clone();
        save_change(d, out, row);
    }
}

#[expect(clippy::too_many_lines, reason = "the outbox admission and its history start share one decision")]
fn emit_change_effect(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    pending: &PendingStep,
    prior: change::Change,
    effect: change::Effect,
    out: &mut Queue<Request>,
) {
    if d.entries.contains_key(&pending.entry) {
        leave_room_change(d, row.task, out);
        emit(out, Request::Refused { task: row.task });
        return;
    }
    let write = match effect {
        change::Effect::FollowCheck { .. } => crate::follow_check::write(d, row, pending.entry),
        change::Effect::Rerun { .. }
        | change::Effect::Open
        | change::Effect::CreateBranch { .. }
        | change::Effect::Reopen
        | change::Effect::Retarget
        | change::Effect::Update { .. }
        | change::Effect::Prepare { .. }
        | change::Effect::Merge { .. } => change_write(row, effect, &pending.failed, &env.limits),
    };
    let Some(write) = write else {
        leave_room_change(d, row.task, out);
        emit(
            out,
            Request::ChangeDecision {
                task: row.task,
                decision: change::Decision::Hold(change::Cause::Failed),
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    };
    let permitted = match d.repositories.get(&row.repository) {
        Some(repository) => repository.role != Role::Context && effect_permitted(repository.kinds, &write),
        None => false,
    };
    if !permitted {
        leave_room_change(d, row.task, out);
        emit(out, Request::Refused { task: row.task });
        return;
    }
    let condition = match effect {
        change::Effect::Update { head, base } | change::Effect::Prepare { head, main: base } => {
            skein_forge_client::Condition::Update { head, base }
        }
        change::Effect::Merge { .. } => skein_forge_client::Condition::Merge { base: row.base.clone() },
        // landing.md 3: the top's task
        change::Effect::FollowCheck { .. }
        | change::Effect::Open
        | change::Effect::CreateBranch { .. }
        | change::Effect::Reopen
        | change::Effect::Retarget
        | change::Effect::Rerun { .. } => skein_forge_client::Condition::None,
    };
    let entry = skein_forge_client::Entry {
        number: pending.entry,
        task: row.task,
        repository: row.repository,
        effect: skein_forge_client::Effect { write, condition },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    let pin = writes::outbox::Pin { head: pending.branch, base: pending.base_tip };
    let plan = writes::outbox::plan(d, &env.limits, &entry, Some(&row.branch), pin);
    if !skein_forge_client::valid_entry(&entry, &env.limits.client) || plan.is_none() {
        leave_room_change(d, row.task, out);
        emit(
            out,
            Request::ChangeDecision {
                task: row.task,
                decision: change::Decision::Hold(change::Cause::Failed),
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    }
    let plan = plan.expect("valid write names");
    let waiter = crate::room::Waiter::Change(row.task);
    let blocked = plan.blocked(d);
    if !crate::room::check(d, &env.limits, Some(waiter)) || blocked.is_some() {
        let mut waiting = d.changes.get(&row.task).expect("registered change").clone();
        // No decided policy state or history is kept by a room wait.
        if waiting.room.is_none() {
            waiting.room = Some(crate::room::next_place(d));
        }
        crate::room::block_write(d, waiter, blocked);
        d.change_due.remove(&row.task);
        read_session::pause(d, row.task);
        d.changes.insert(row.task, waiting.clone()).expect("registered change");
        save_change(d, out, waiting);
        emit(
            out,
            Request::ChangeDecision {
                task: row.task,
                decision: change::Decision::Wait { until: None },
                entry: None,
                evidence: None,
                close_at: None,
            },
        );
        return;
    }
    row.room = None;
    crate::room::leave(d, crate::room::Waiter::Change(row.task));
    let facts = change_facts(d, env, row, pending, pending.base_tip.unwrap_or([0; 32]));
    keep_step_progress(d, env, row, pending, &facts, out);
    crate::landing::change_advanced(d, row, &prior, out);
    row.pending = Some(pending.entry);
    row.effect = change::EffectResult::Pending;
    row.effect_account = match effect {
        change::Effect::Rerun { head } => {
            let mut reason = None;
            let mut repair = None;
            if let Some((before, _)) = pending.progress.as_slice().last() {
                if let Some(why) = crate::recheck::released_repair(before) {
                    repair = Some(why);
                    reason = if measured_release(row) {
                        Some(b"a fresh measurement was asked".as_slice())
                    } else {
                        Some(b"a repair changed nothing".as_slice())
                    };
                } else if ci_repair(&before.state)
                    && let change::State::Repairing { why, .. } = before.state
                {
                    repair = Some(why);
                    reason = Some(b"a repair changed nothing".as_slice());
                }
            }
            if let Some(why) = repair {
                let skein_forge::api::Write::Rerun { attempt } = entry.effect.write else {
                    unreachable!("a CI rerun writes its chosen attempt")
                };
                row.ci_recheck =
                    Some(crate::CiRecheck { job: attempt, taken: None, semantic: why == change::Repair::Semantic });
            }
            match pending.failed.account(head, reason) {
                Some(words) => Some(crate::EffectAccount { entry: pending.entry, words }),
                None => None,
            }
        }
        change::Effect::FollowCheck { .. }
        | change::Effect::Open
        | change::Effect::CreateBranch { .. }
        | change::Effect::Reopen
        | change::Effect::Retarget
        | change::Effect::Update { .. }
        | change::Effect::Prepare { .. }
        | change::Effect::Merge { .. } => None,
    };
    let before = match d.changes.get(&row.task) {
        Some(row) => crate::panel::ready(row),
        None => None,
    };
    d.changes.insert(row.task, row.clone()).expect("replaces change");
    wake_after_turn(d, row, prior.owns_turn);
    save_queue_change(d, env, before, row.clone(), out);
    if let skein_forge::api::Write::Update { style, .. } = entry.effect.write {
        let forge_base = match &pending.pull {
            Some(pull) => pull.base_commit,
            None => None,
        };
        let mut event = crate::change_history::step(d, row, crate::history::What::Update { style, forge_base });
        event.head = pending.branch;
        event.base = match &entry.effect.condition {
            skein_forge_client::Condition::Update { base, .. } => Some(*base),
            skein_forge_client::Condition::Merge { .. }
            | skein_forge_client::Condition::Review { .. }
            | skein_forge_client::Condition::None => {
                unreachable!("update write carries its effect's pin")
            }
        };
        event.operation = crate::history::Operation::Of(pending.entry);
        crate::change_history::write(d, env.wall, row, event, out);
    }
    plan.admit(d, &entry, out);
    keep_entry(d, entry, None, out);
    if let change::State::Taking { .. } = row.change.state {
        let until = skein_lib::Wall::from_nanos(
            row.change.since.as_nanos().saturating_add(env.limits.change_policy.stall.as_nanos()),
        );
        d.wait_change(env, row.task, Some(until));
    }
    emit(
        out,
        Request::ChangeDecision {
            task: row.task,
            decision: change::Decision::Effect(effect),
            entry: Some(pending.entry),
            evidence: Some(step_evidence(d, row, pending, prior)),
            close_at: None,
        },
    );
}

/// What a forge takes in a pull request's body, in bytes at most (a forge counts characters).
/// temper/docs/design/domain/forge/change/README.md, 1.
const PULL_BODY_BYTES: usize = 65_536;

fn change_write(
    row: &ChangeRow,
    effect: change::Effect,
    failed: &crate::change_ci::Failed,
    limits: &Limits,
) -> Option<skein_forge::api::Write> {
    match effect {
        change::Effect::Rerun { head } => Some(skein_forge::api::Write::Rerun { attempt: failed.job(head)? }),
        change::Effect::Open if row.pull.is_some() => None,
        change::Effect::Open => {
            // domain/forge/change/README.md, 1: fit the explicit title independently of the brief.
            let room = skein_lib::usize_of(limits.client.op_bytes)
                .saturating_sub(row.branch.len())
                .saturating_sub(row.base.len());
            let marker = "…\nThe change's full words are on its task in temper.".as_bytes();
            let title_room = room.saturating_sub(marker.len().min(row.body.len()));
            let title = crate::text::fit(&row.title, title_room.min(256), "…".as_bytes());
            let title = if title.is_empty() { Box::from(DEFAULT_CHANGE_TITLE) } else { title };
            let body = crate::text::fit(&row.body, room.saturating_sub(title.len()).min(PULL_BODY_BYTES), marker);
            Some(skein_forge::api::Write::OpenPull { title, body, head: row.branch.clone(), base: row.base.clone() })
        }
        change::Effect::CreateBranch { head } => {
            Some(skein_forge::api::Write::CreateBranch { branch: row.branch.clone(), commit: head })
        }
        change::Effect::Reopen => Some(skein_forge::api::Write::Reopen { number: row.pull? }),
        change::Effect::Prepare { .. } => {
            Some(skein_forge::api::Write::Update { number: row.pull?, style: skein_forge::api::UpdateStyle::Rebase })
        }
        change::Effect::FollowCheck { .. } | change::Effect::Retarget => None,
        change::Effect::Update { .. } => {
            Some(skein_forge::api::Write::Update { number: row.pull?, style: skein_forge::api::UpdateStyle::Merge })
        }
        change::Effect::Merge { head, .. } => {
            Some(skein_forge::api::Write::Merge { number: row.pull?, head, how: skein_forge::api::MergeHow::Style })
        }
    }
}

fn settled_account(d: &Domain, entry: u64, outcome: skein_forge_client::Outcome) -> Option<crate::EffectAccount> {
    if let skein_forge_client::Outcome::Made { .. } = outcome {
        for (_, row) in &d.changes {
            if row.pending == Some(entry) {
                return row.effect_account.clone();
            }
        }
    }
    let row = d.entries.get(&entry)?;
    let words = crate::effect_words::account(&row.entry, d.repository(row.entry.repository), outcome)?;
    Some(crate::EffectAccount { entry, words })
}

/// Detach the landed change before withdrawal can synchronously answer it.
fn take_landed_merge(d: &Domain, row: &mut ChangeRow) -> Option<u64> {
    landed_commit(&row.change.state)?;
    let number = row.pending?;
    let entry = d.entries.get(&number)?;
    if let skein_forge::api::Write::Merge { .. } = entry.entry.effect.write {
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.effect_account = None;
        return Some(number);
    }
    None
}

/// A lost update keeps its account and decides again (temper/docs/design/domain/forge/updates.md, 1).
#[expect(
    clippy::too_many_lines,
    reason = "one settlement keeps its result, account, drift baseline and procedure signal together"
)]
pub(crate) fn change_outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    let mut task = None;
    for (number, row) in &d.changes {
        if row.pending == Some(entry) {
            task = Some(*number);
        }
    }
    let Some(task) = task else { return };
    if landed_commit(&d.changes.get(&task).expect("found change row").change.state).is_some() {
        return;
    }
    let (opening, update) = match d.entries.get(&entry) {
        Some(row) => match &row.entry.effect.write {
            skein_forge::api::Write::OpenPull { .. } => (true, false),
            skein_forge::api::Write::Update { .. } => (false, true),
            skein_forge::api::Write::CreateIssue { .. }
            | skein_forge::api::Write::Post { .. }
            | skein_forge::api::Write::Review { .. }
            | skein_forge::api::Write::Edit { .. }
            | skein_forge::api::Write::SetReviewers { .. }
            | skein_forge::api::Write::Close { .. }
            | skein_forge::api::Write::Reopen { .. }
            | skein_forge::api::Write::Merge { .. }
            | skein_forge::api::Write::Status { .. }
            | skein_forge::api::Write::Dispatch { .. }
            | skein_forge::api::Write::Rerun { .. }
            | skein_forge::api::Write::CreateBranch { .. }
            | skein_forge::api::Write::DeleteBranch { .. } => (false, false),
        },
        None => (false, false),
    };
    let unsent = match d.entries.get(&entry) {
        Some(row) => row.entry.attempt.is_none(),
        None => false,
    };
    let account = settled_account(d, entry, outcome);
    let account = taking::make_account(d.changes.get(&task).expect("found change row"), account, outcome);
    let Some(row) = d.changes.get_mut(&task) else { unreachable!("found change row") };
    let closed_opening = opening
        && match outcome {
            skein_forge_client::Outcome::Raced {
                made: skein_forge_client::Made::Created(_),
                why: skein_forge::api::Error::Closed,
            } => true,
            skein_forge_client::Outcome::Made { .. }
            | skein_forge_client::Outcome::Failed(_)
            | skein_forge_client::Outcome::Raced { .. }
            | skein_forge_client::Outcome::Uncertain
            | skein_forge_client::Outcome::Queued
            | skein_forge_client::Outcome::Held
            | skein_forge_client::Outcome::Withdrawn => false,
        };
    if opening
        && let skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Created(number), .. } = outcome
    {
        row.pull = Some(number);
    }
    row.effect = match outcome {
        skein_forge_client::Outcome::Raced { .. } if closed_opening => {
            if row.change.state == (change::State::Opening { requested: true }) {
                row.change.state = change::State::Opening { requested: false };
            }
            change::EffectResult::None
        }
        skein_forge_client::Outcome::Made { .. } => change::EffectResult::Made,
        skein_forge_client::Outcome::Failed(skein_forge::api::Error::Conflict) => change::EffectResult::Conflict,
        skein_forge_client::Outcome::Failed(
            skein_forge::api::Error::Stale | skein_forge::api::Error::BaseMoved { .. },
        ) if update && unsent => change::EffectResult::Stale,
        skein_forge_client::Outcome::Failed(
            skein_forge::api::Error::Timeout
            | skein_forge::api::Error::Unavailable
            | skein_forge::api::Error::Stale
            | skein_forge::api::Error::BaseMoved { .. },
        ) if update => change::EffectResult::None,
        skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => change::EffectResult::Failed,
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => return,
    };
    row.effect_account = account;
    row.pending = None;
    let mut saved = row.clone();
    if update {
        let result = match outcome {
            skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(head), .. } => {
                (crate::history::UpdateOutcome::Updated, Some(head), updating_base(&saved.change.state))
            }
            skein_forge_client::Outcome::Failed(skein_forge::api::Error::Conflict) => {
                (crate::history::UpdateOutcome::Conflict, None, None)
            }
            skein_forge_client::Outcome::Failed(skein_forge::api::Error::BaseMoved { base }) => {
                (crate::history::UpdateOutcome::Failed, None, base)
            }
            skein_forge_client::Outcome::Made { .. }
            | skein_forge_client::Outcome::Failed(_)
            | skein_forge_client::Outcome::Raced { .. }
            | skein_forge_client::Outcome::Uncertain
            | skein_forge_client::Outcome::Queued
            | skein_forge_client::Outcome::Held
            | skein_forge_client::Outcome::Withdrawn => (crate::history::UpdateOutcome::Failed, None, None),
        };
        let mut event = crate::change_history::step(
            d,
            &saved,
            crate::history::What::Updated { outcome: result.0, forge_base: result.2 },
        );
        event.operation = crate::history::Operation::Of(entry);
        event.head = result.1;
        event.base = updating_base(&saved.change.state);
        crate::change_history::write(d, env.wall, &mut saved, event, out);
        if result.0 == crate::history::UpdateOutcome::Conflict && saved.change.way == change::Way::Queue {
            crate::change_history::conflict(d, env.wall, &mut saved, out);
        }
    }
    if let skein_forge_client::Outcome::Failed(skein_forge::api::Error::Conflict) = outcome
        && saved.change.way == change::Way::Queue
        && let Some(kept) = d.entries.get(&entry)
        && let skein_forge::api::Write::Merge { .. } = kept.entry.effect.write
    {
        crate::change_history::conflict(d, env.wall, &mut saved, out);
    }
    if let skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(commit), .. } = outcome {
        let mut event = crate::change_history::step(d, &saved, crate::history::What::Turn { members: 1 });
        event.operation = crate::history::Operation::Of(entry);
        event.head = Some(commit);
        event.base = crate::change_history::base(d, &saved);
        crate::change_history::write(d, env.wall, &mut saved, event, out);
    }
    d.changes.insert(task, saved.clone()).expect("retained change");
    save_change(d, out, saved.clone());
    match outcome {
        skein_forge_client::Outcome::Made {
            made: skein_forge_client::Made::Updated(commit) | skein_forge_client::Made::Branch(commit),
            ..
        } => {
            record_change_head(d, env, &saved, commit, out);
        }
        skein_forge_client::Outcome::Made {
            made:
                skein_forge_client::Made::Created(_)
                | skein_forge_client::Made::Commented(_)
                | skein_forge_client::Made::Reviewed(_)
                | skein_forge_client::Made::Merged(_)
                | skein_forge_client::Made::Set,
            ..
        }
        | skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Uncertain
        | skein_forge_client::Outcome::Queued
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => {}
    }
    emit(
        out,
        Request::ChangeDecision { task, decision: change::Decision::None, entry: None, evidence: None, close_at: None },
    );
}

pub(crate) fn matches_updating(state: &change::State) -> bool {
    updating_base(state).is_some()
}

pub(crate) fn updating_base(state: &change::State) -> Option<skein_forge::api::Commit> {
    match state {
        change::State::Updating { base, .. } | change::State::Preparing { main: base, .. } => Some(*base),
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::Held { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => None,
    }
}

/// Mirror an owned head for existing workspace and procedure readers.
pub(crate) fn record_owned_head(
    d: &mut Domain,
    env: &Env<Limits>,
    name: &Name,
    commit: skein_forge::api::Commit,
    task: Option<u64>,
    out: &mut Queue<Request>,
) {
    let previous = d.branch_head(name);
    let head = BranchHead { name: name.clone(), commit, owned: true };
    let concerning = heads::concerning(d, name, task);
    if heads::record(d, env, head.clone(), concerning, out) {
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
    }
    wake_head_dependents(d, name, previous, commit);
}

fn record_change_head(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let resource = skein_forge_client::Resource {
        repository: row.repository,
        what: skein_forge_client::What::Branch(row.branch.clone()),
    };
    let Some(name) = from_client(&resource, &env.limits) else { return };
    if let Some(head) = d.heads.get(&name)
        && head.commit == commit
        && head.owned
    {
        return;
    }
    record_owned_head(d, env, &name, commit, Some(row.task), out);
}

pub(crate) fn matches_held(state: &change::State) -> bool {
    match state {
        change::State::Held { .. } => true,
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Preparing { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => false,
    }
}

fn held_cause(state: &change::State) -> Option<change::Cause> {
    if let change::State::Held { hold, .. } = state { Some(hold.cause) } else { None }
}

fn takes_released_head(change: &change::Change, head: skein_forge::api::Commit) -> bool {
    change.released_resolution
        || change.released == Some(change::Cause::BranchMoved)
        || ((change.released == Some(change::Cause::RedStands) || change.released == Some(change::Cause::Rejected))
            && change.last_head != Some(head))
}

/// A release adopts an observed head, including the client's drift baseline.
fn accept_released_branch(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    head: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    row.drift = None;
    let resource = skein_forge_client::Resource {
        repository: row.repository,
        what: skein_forge_client::What::Branch(row.branch.clone()),
    };
    let Some(name) = from_client(&resource, &env.limits) else { return };
    if let Some(hold) = d.holds.get_mut(&name) {
        hold.drift = None;
        save_hold(out, hold.clone());
    }
    writes::accept_head(d, &name, head, out);
    writes::kept::sync_heads(d, env, out);
    row.history = d.changes.get(&row.task).expect("retained change").history;
}

/// Removing the durable delegate reference retires its terminal from recovery
/// (temper/docs/design/domain/forge/change/repairs.md, A release).
fn retire_failed_delegate(row: &mut ChangeRow) {
    // The close change keeps the last job's account and observation for its own release.
    if row.close.is_some()
        || (row.delegate_status != change::Status::Failed && !row.delegate_flake && !row.delegate_no_test)
    {
        return;
    }
    match row.delegate {
        Some((_, change::Delegate::Resolve { .. } | change::Delegate::Merge { .. })) => {
            row.change.released_resolution = true;
        }
        Some((
            _,
            change::Delegate::Produce
            | change::Delegate::Close
            | change::Delegate::Repair(_)
            | change::Delegate::Gate { .. },
        ))
        | None => {}
    }
    row.delegate = None;
    row.delegate_status = change::Status::Unknown;
    row.delegate_flake = false;
    row.delegate_no_test = false;
    row.delegate_words = None;
    row.producer_failure = None;
    row.merge_result = None;
}

fn failed_merge_hold(state: &change::State) -> bool {
    let change::State::Held { was, hold } = state else { return false };
    let change::State::Landing { .. } = was.as_ref() else { return false };
    hold.cause == change::Cause::Failed
}

/// Drop only connector-side verdicts superseded by the policy's release.
fn forget_released_step(row: &mut ChangeRow, held: &change::State) {
    let Some(cause) = held_cause(held) else { return };
    if cause == change::Cause::Runner {
        row.runner_ci = Some(crate::RunnerCi::Released);
    }
    if cause == change::Cause::Failed || cause == change::Cause::RedStands || cause == change::Cause::CloseRefused {
        retire_failed_delegate(row);
    }
    if cause == change::Cause::RedStands {
        row.delegate_flake = false;
        row.delegate_no_test = false;
        row.producer_failure = None;
        row.delegate_status = change::Status::Unknown;
        row.ci_recheck = None;
    }
    if row.effect == change::EffectResult::Failed && failed_merge_hold(held) {
        row.effect = change::EffectResult::None;
        row.effect_account = None;
    }
    if row.effect == change::EffectResult::Failed || row.effect == change::EffectResult::Stale {
        match row.change.state {
            change::State::Taking { .. }
            | change::State::Updating { .. }
            | change::State::Preparing { .. }
            | change::State::Checking { .. }
            | change::State::Judging { .. }
            | change::State::Gating { .. }
            | change::State::CancellingGates { .. } => row.effect = change::EffectResult::None,
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Ready { .. }
            | change::State::Queued { .. }
            | change::State::First { .. }
            | change::State::WaitingProducer { .. }
            | change::State::Resolving { .. }
            | change::State::Repairing { .. }
            | change::State::Landing { .. }
            | change::State::Landed { .. }
            | change::State::Held { .. }
            | change::State::Red { .. }
            | change::State::Cleared { .. }
            | change::State::Uncleared { .. }
            | change::State::NoChange { .. }
            | change::State::Ended => {}
        }
    }
    if let Some(check) = &mut row.follow_check
        && check.held.is_some()
    {
        check.held = None;
        check.faults = 0;
        check.flight = None;
        check.page = 1;
    }
}

fn release_owes_readback(d: &Domain, env: &Env<Limits>, row: &ChangeRow) -> bool {
    if let Some(change::Handed { ask: change::Ask::Landed { .. }, .. }) = crate::landing::members::head(row) {
        return true;
    }
    if let change::State::Held { was, .. } = &row.change.state
        && let change::State::First { .. } | change::State::Landing { .. } = was.as_ref()
    {
        return true;
    }
    writes::release_readback(d, &env.limits, row)
}

fn fence_release(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, out: &mut Queue<Request>) {
    let task = row.task;
    if let Some(owner) = step_owner(d, task) {
        d.steps.remove(&owner);
    }
    d.fresh_steps.remove(&task);
    d.change_due.remove(&task);
    if let Some(read) = d.settling.get_mut(&task) {
        read.branch.due = true;
        read.contains_head = None;
    }
    read_session::restart(d, env, task, env.wall, out);
    row.behind_wait = None;
    fence_release_reads(d, env, row, out);
}

fn take_engine_revision(d: &Domain, row: &mut ChangeRow, now: skein_lib::Wall) {
    if let Some(TaskHold::Change { revision }) = d.task_holds.get(&row.task) {
        row.task_release = Some(crate::TaskRelease { revision: *revision, at: now, measure: false });
    }
}

fn engine_release(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    trigger: change::ReleaseTrigger,
    out: &mut Queue<Request>,
) -> bool {
    if d.task_holds.get(&row.task) == Some(&TaskHold::Blocked) {
        return false;
    }
    let Some(stepped) = change::release(&row.change, trigger, None, None, env.wall) else { return false };
    assert_eq!(stepped.decision, change::Decision::EndHold, "only engine triggers reach this path");
    take_engine_revision(d, row, env.wall);
    fence_release(d, env, row, out);
    let prior = core::mem::replace(&mut row.change, stepped.change);
    if !matches_held(&row.change.state) && !row.terminal() {
        forget_released_step(row, &prior.state);
    }
    fresh_release_reads(d, env, row, out);
    crate::readiness::released(d, env, row, change::Cause::Stalled, out);
    true
}

fn release_change(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(mut row) = d.changes.get(&task).cloned() else {
        emit(out, Request::Refused { task });
        return;
    };
    let before = crate::panel::ready(&row);
    let owes_readback = release_owes_readback(d, env, &row) || produced_nothing(&row).is_some();
    fence_release(d, env, &mut row, out);
    let trigger = change::ReleaseTrigger::Task { owes_readback, close: row.close.is_some() };
    let stepped = change::release(&row.change, trigger, None, None, env.wall).expect("task release is admitted");
    let prior = core::mem::replace(&mut row.change, stepped.change);
    if !matches_held(&row.change.state) && matches_held(&prior.state) {
        forget_released_step(&mut row, &prior.state);
        if row.room.is_some() {
            row.effect_account = None;
            crate::room::released(d, task, env.wall);
        }
    }
    row.drift = None;
    let saved = row;
    save_queue_change(d, env, before, saved.clone(), out);
    let mut affected = List::with_capacity(d.holds.len());
    for (name, hold) in &d.holds {
        if hold.task == task && hold.drift.is_some() {
            affected.push(name.clone()).expect("at most all held resources");
        }
    }
    for name in &affected {
        let hold = d.holds.get_mut(name).expect("collected held resource");
        hold.drift = None;
        save_hold(out, hold.clone());
    }
    fresh_release_reads(d, env, &saved, out);
}

// The change fences its branch and pull, not its shared base (keeping-up.md, 3).
fn fence_release_reads(d: &mut Domain, env: &Env<Limits>, row: &ChangeRow, out: &mut Queue<Request>) {
    child(
        d,
        env,
        skein_forge_client::Event::Fence {
            resource: skein_forge_client::Resource {
                repository: row.repository,
                what: skein_forge_client::What::Branch(row.branch.clone()),
            },
        },
        out,
    );
    if let Some(number) = row.pull {
        child(
            d,
            env,
            skein_forge_client::Event::Fence {
                resource: skein_forge_client::Resource {
                    repository: row.repository,
                    what: skein_forge_client::What::Pull(number),
                },
            },
            out,
        );
    }
}

// A release cannot join an in-flight observation from before it. Hints fence
// shared fresh reads in the client as well as advancing its watched reads.
fn fresh_release_reads(d: &mut Domain, env: &Env<Limits>, row: &ChangeRow, out: &mut Queue<Request>) {
    let landing = if row.close.is_some() {
        match d.repositories.get(&row.repository) {
            Some(repository) if repository.landing != row.base => Some(repository.landing.clone()),
            Some(_) | None => None,
        }
    } else {
        None
    };
    for branch in [&row.branch, &row.base].into_iter().chain(landing.as_ref()) {
        child(
            d,
            env,
            skein_forge_client::Event::Hint {
                hint: skein_forge::api::Hint {
                    repository: row.repository,
                    change: skein_forge::api::Change::Branch(branch.clone()),
                    key: None,
                },
            },
            out,
        );
    }
    if let Some(number) = row.pull {
        child(
            d,
            env,
            skein_forge_client::Event::Hint {
                hint: skein_forge::api::Hint {
                    repository: row.repository,
                    change: skein_forge::api::Change::Item(number),
                    key: None,
                },
            },
            out,
        );
    }
}

// Cancellation never writes to a pull whose merge is already recorded (domain/forge/change/README.md, 4).
fn cancellation_pull(d: &Domain, task: u64) -> Option<u64> {
    let change = d.changes.get(&task)?;
    // Keep the possible close until the fresh pull and merge facts can judge it.
    if !repository_fresh(d, change.repository) {
        return change.pull;
    }
    if landed_commit(&change.change.state).is_some() {
        return None;
    }
    if recorded_merge(d, task).is_some() {
        return None;
    }
    change.pull
}

/// No-change closes its open pull with its retained words (domain/forge/change/README.md, 2 and 4).
fn no_change_words(d: &Domain, task: u64) -> Option<&[u8]> {
    let row = d.changes.get(&task)?;
    match &row.change.state {
        change::State::NoChange { words } => Some(words),
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::Ended
        | change::State::Held { .. } => None,
    }
}

fn closing_pull(d: &Domain, task: u64, ending: ReleaseEnding) -> Option<u64> {
    match ending {
        ReleaseEnding::Cancelled => cancellation_pull(d, task),
        ReleaseEnding::Done if no_change_words(d, task).is_some() => cancellation_pull(d, task),
        ReleaseEnding::Done | ReleaseEnding::Failed => None,
    }
}

fn pull_seen_closed(d: &Domain, repository: skein_forge::api::Repository, number: u64, limits: &Limits) -> bool {
    let resource = skein_forge_client::Resource { repository, what: skein_forge_client::What::Pull(number) };
    let Some(name) = from_client(&resource, limits) else { return false };
    match d.pulls.get(&name) {
        Some(pull) => pull.state == skein_forge::api::State::Closed,
        None => false,
    }
}

fn recorded_merge(d: &Domain, task: u64) -> Option<skein_forge::api::Commit> {
    for (commit, owner) in &d.landed {
        if *owner == task {
            return Some(*commit);
        }
    }
    None
}

fn release_row(
    task: u64,
    root: u64,
    ending: ReleaseEnding,
    close_pull: Option<u64>,
    effects_settled: bool,
    releasing: bool,
) -> ReleaseRow {
    ReleaseRow {
        task,
        root,
        ending,
        close_pull,
        comment: None,
        pending: None,
        failed: false,
        effects_settled,
        releasing,
        room: None,
    }
}

// A failed release can take another ending from the same root (forge/change/README.md, 4).
fn retarget_release(row: &mut ReleaseRow, root: u64, ending: ReleaseEnding, close_pull: Option<u64>) -> bool {
    if row.root != root || (row.ending != ending && ending != ReleaseEnding::Cancelled && !row.failed) {
        return false;
    }
    if row.ending != ending {
        row.ending = ending;
        row.releasing = false;
        row.close_pull = close_pull;
        if close_pull.is_none() {
            row.comment = None;
        }
    }
    true
}

fn settle_effects(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    root: u64,
    ending: ReleaseEnding,
    out: &mut Queue<Request>,
) {
    let close_pull = closing_pull(d, task, ending);
    let settled = if let Some(row) = d.releases.get_mut(&task) {
        let changed = row.ending != ending;
        if !retarget_release(row, root, ending, close_pull) {
            release_failure::report(out, task, None, release_failure::conflict(row, root, ending));
            return;
        }
        if changed {
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
        }
        row.effects_settled
    } else {
        if d.releases.len() == env.limits.tasks {
            release_failure::full(task, env.limits.tasks, out);
            return;
        }
        let row = release_row(task, root, ending, close_pull, false, false);
        d.releases.insert(task, row.clone()).expect("release capacity checked");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
        false
    };
    crate::closer::ended(d, task);
    crate::landing::closing(d, env, task, ending, out);
    crate::tiers::closing(d, env, task, ending, out);
    if settled {
        emit(out, Request::EffectsSettled { task });
        return;
    }
    leave_room_change(d, task, out);
    crate::room::release_started(d, task, env.wall);
    if ending == ReleaseEnding::Cancelled || d.landing.rows.contains_key(&task) {
        withdraw_unsent(d, env, task, out);
    }
    settle_prior_effects(d, env, task, out);
}

fn withdraw_unsent(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let mut unsent = List::with_capacity(env.limits.entries);
    for (number, row) in &d.entries {
        if row.entry.task == task && !row.held && row.entry.attempt.is_none() {
            unsent.push(*number).expect("entry table bounded");
        }
    }
    for number in unsent.into_boxed() {
        withdraw(d, env, number, out);
        if let Some(row) = d.entries.get(&number)
            && row.entry.start.is_none()
        {
            // An unsent entry can still have a read in the client. Withdraw
            // bars its write there; its eventual answer holds no room here.
            row.check_owner(number, task);
            let account = crate::effect_words::account(
                &row.entry,
                d.repository(row.entry.repository),
                skein_forge_client::Outcome::Withdrawn,
            );
            writes::outbox::outcome(d, task, number, skein_forge_client::Outcome::Withdrawn, out);
            crate::landing::outcome(d, env, number, skein_forge_client::Outcome::Withdrawn, None, out);
            take_entry(d, number, task);
            emit(out, Request::Erase { key: Key::Entry(number) });
            d.named_outcome(number, skein_forge_client::Outcome::Withdrawn);
            emit(
                out,
                Request::Outcome { entry: number, task, outcome: skein_forge_client::Outcome::Withdrawn, account },
            );
        }
    }
}

fn settle_prior_effects(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    if !repository_reads::release_fresh(d, task) {
        repository_reads::park_release(d, task);
        return;
    }
    if let Some(issue) = d.issues.get(&task)
        && let Some(desired) = &issue.desired
        && desired.finished.is_some()
        && !issue.state.closed
    {
        return;
    }
    if !settle_held(d, env, task, out) {
        return;
    }
    let row = d.releases.get_mut(&task).expect("effect settlement row");
    if !row.effects_settled {
        row.effects_settled = true;
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
    }
    emit(out, Request::EffectsSettled { task });
}

#[expect(clippy::too_many_arguments, reason = "cleanup receives the closing task and its bounded cancel reason")]
fn release(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    root: u64,
    ending: ReleaseEnding,
    reason: Option<Box<[u8]>>,
    entry: u64,
    out: &mut Queue<Request>,
) {
    let reason = match ending {
        ReleaseEnding::Done => match no_change_words(d, task) {
            Some(words) => Some(Box::from(words)),
            None => reason,
        },
        ReleaseEnding::Cancelled | ReleaseEnding::Failed => reason,
    };
    let close_pull = closing_pull(d, task, ending);
    let mut row = match d.releases.get(&task) {
        Some(row) => row.clone(),
        None => {
            if d.releases.len() == env.limits.tasks {
                release_failure::full(task, env.limits.tasks, out);
                return;
            }
            release_row(task, root, ending, close_pull, true, false)
        }
    };
    if !retarget_release(&mut row, root, ending, close_pull) {
        release_failure::report(out, task, None, release_failure::conflict(&row, root, ending));
        return;
    }
    let advanced = !row.releasing || row.failed;
    if !row.releasing {
        row.comment = None;
        if !keep_release_comment(d, env, &mut row, reason, out) {
            return;
        }
    }
    row.releasing = true;
    row.failed = false;
    d.releases.insert(task, row.clone()).expect("release capacity checked");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
    leave_room_change(d, task, out);
    crate::room::release_started(d, task, env.wall);
    if advanced {
        release_reads::advanced(d, task, env.wall);
    }
    continue_release(d, env, task, entry, out);
}

fn continue_release(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    // The durable release and its place stay put while its facts are read.
    if !repository_reads::release_fresh(d, task) {
        if d.releases.contains_key(&task) {
            repository_reads::park_release(d, task);
            release_reads::waiting(d, env, task, release_reads::Read::Fresh);
            crate::room::pass(d, task);
        }
        return;
    }
    release_reads::ready(d, env, task);
    crate::room::stepping(d, task);
    continue_release_inner(d, env, task, entry, out);
    // A turn waiting on facts rather than room gives its promise back (room.md, 3).
    if crate::room::asked(d, crate::room::Waiter::Release(task)) {
        leave_room_release(d, task, out);
    }
}

pub(super) fn room_release_failed(d: &mut Domain, task: u64, words: Box<[u8]>, out: &mut Queue<Request>) {
    let mut row = d.releases.get(&task).expect("release waiting for room").clone();
    refuse_release_write(d, &mut row, words, out);
}

fn refuse_release_write(d: &mut Domain, row: &mut ReleaseRow, words: Box<[u8]>, out: &mut Queue<Request>) {
    crate::room::leave(d, crate::room::Waiter::Release(row.task));
    row.room = None;
    release_failure::held(d, row, words, out);
}

fn leave_room_release(d: &mut Domain, task: u64, out: &mut Queue<Request>) {
    crate::room::leave(d, crate::room::Waiter::Release(task));
    if let Some(row) = d.releases.get_mut(&task)
        && row.room.take().is_some()
    {
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
    }
}

/// A completed run may report no head. Read its remaining slots before either
/// its task or the holder releases resources (jig's domain/tasks/holds.md, 3).
fn release_writers(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) -> bool {
    let mut attempts = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if d.forge_ended(name.forge) {
            continue;
        }
        if let Some(Writer::Run { task: writer, attempt }) = hold.writer
            && writer == task
            && !attempts.as_slice().contains(&attempt)
        {
            attempts.push(attempt).expect("attempts bounded by holds");
        }
    }
    for attempt in attempts.into_boxed() {
        lost(d, env, task, attempt, out);
    }
    for (name, hold) in &d.holds {
        if d.forge_ended(name.forge) {
            continue;
        }
        let writing = match hold.writer {
            Some(Writer::Run { task: writer, .. } | Writer::Lost { task: writer, .. }) => {
                writer == task || hold.task == task
            }
            Some(Writer::Entry(_)) | None => false,
        };
        if writing || (hold.task == task && retired_reading(d, name)) {
            return false;
        }
        for writer in &hold.design_writers {
            if writer.task == task || hold.task == task {
                return false;
            }
        }
    }
    true
}

#[expect(clippy::too_many_lines, reason = "release drops each goal tier entry with its hold in the same transition")]
fn continue_release_inner(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    for _ in 0..=env.limits.holds {
        let Some(mut row) = d.releases.get(&task).cloned() else { return };
        if row.failed || row.pending.is_some() {
            return;
        }
        if !row.effects_settled {
            settle_prior_effects(d, env, task, out);
            return;
        }
        if !row.releasing {
            return;
        }
        if !release_writers(d, env, task, out) {
            release_reads::rereading(d, env, task);
            return;
        }
        // A reread refused synchronously may have held this release.
        if d.releases.get(&task).expect("active release").failed {
            return;
        }
        if !settle_held(d, env, task, out) {
            return;
        }
        if !release_pull(d, env, &mut row, entry, out) {
            return;
        }
        let mut selected = None;
        for (name, hold) in &d.holds {
            if hold.task == task {
                selected = Some((name.clone(), hold.writer, !hold.design_writers.is_empty()));
                break;
            }
        }
        if let Some((name, writer, designing)) = selected {
            if d.forge_ended(name.forge) {
                d.holds.remove(&name);
                release_reads::advanced(d, task, env.wall);
                crate::tiers::released(d, &name, out);
                reds::released(d, task, &name, out);
                emit(out, Request::Erase { key: Key::Hold(name) });
                continue;
            }
            if designing {
                release_reads::rereading(d, env, task);
                return;
            }
            if unpublished_branch(d, &name) && writer.is_none() {
                d.holds.remove(&name);
                release_reads::advanced(d, task, env.wall);
                crate::tiers::released(d, &name, out);
                reds::released(d, task, &name, out);
                emit(out, Request::Erase { key: Key::Hold(name) });
                continue;
            }
            if goal_branch::reading(d, &name) {
                release_reads::waiting(d, env, task, release_reads::Read::Making(name));
                return;
            }
            if let Some(writer) = writer {
                let words = release_failure::writing(d, &name, writer);
                release_failure::held(d, &mut row, words, out);
                return;
            }
            if goal_branch::is_goal(d, &name) {
                if !goal_branch::landed(d, &name) {
                    d.holds.remove(&name);
                    release_reads::advanced(d, task, env.wall);
                    crate::tiers::released(d, &name, out);
                    reds::released(d, task, &name, out);
                    emit(out, Request::Erase { key: Key::Hold(name) });
                    continue;
                }
            } else if row.ending == ReleaseEnding::Failed && row.root != task {
                let hold = d.holds.get_mut(&name).expect("selected hold");
                hold.task = row.root;
                save_hold(out, hold.clone());
                release_reads::advanced(d, task, env.wall);
                emit(out, Request::Retained { task, root: row.root, resource: name });
                continue;
            }
            if let Some(resource) = to_client(&name, &env.limits) {
                match resource.what {
                    skein_forge_client::What::Branch(branch) => {
                        enqueue_release(
                            d,
                            env,
                            &mut row,
                            entry,
                            resource.repository,
                            skein_forge::api::Write::DeleteBranch { branch },
                            Some(name),
                            out,
                        );
                        return;
                    }
                    skein_forge_client::What::Repository
                    | skein_forge_client::What::Pull(_)
                    | skein_forge_client::What::Issue(_) => {}
                }
            }
            d.holds.remove(&name);
            release_reads::advanced(d, task, env.wall);
            crate::tiers::released(d, &name, out);
            reds::released(d, task, &name, out);
            emit(out, Request::Erase { key: Key::Hold(name) });
            continue;
        }
        // The task remains live until every foreign hold has let go of its
        // writer link (jig/docs/design/domain/engine/restart.md, Load, then link).
        // Its lost/design read wakes this release when the link is cleared.
        // Removed forges retain historical links without a live task.
        for (_, hold) in &d.holds {
            if d.forge_ended(hold.name.forge) {
                continue;
            }
            let slot = match hold.writer {
                Some(Writer::Run { task: writer, .. } | Writer::Lost { task: writer, .. }) => writer == task,
                Some(Writer::Entry(_)) | None => false,
            };
            if slot {
                release_reads::rereading(d, env, task);
                return;
            }
            for writer in &hold.design_writers {
                if writer.task == task {
                    release_reads::rereading(d, env, task);
                    return;
                }
            }
        }
        let mut topics = List::with_capacity(env.limits.subscriptions);
        for ((owner, topic), _) in &d.subscriptions {
            if *owner == task {
                topics.push(topic.clone()).expect("subscription capacity");
            }
        }
        for topic in &topics {
            unsubscribe(d, env, task, topic.clone(), out);
        }
        unnamed(d, env, task, true, out);
        end_change(d, env, task, out);
        heads::free(d, env, out);
        crate::room::leave(d, crate::room::Waiter::Release(task));
        d.releases.remove(&task);
        d.room.closing.remove(&task);
        d.release_reads.remove(&task);
        crate::waits::report(d, env, task, out);
        crate::waits::forget(d, task);
        emit(out, Request::Erase { key: Key::Release(task) });
        emit(out, Request::Released { task });
        return;
    }
    unreachable!("every release pass removes a hold or completes");
}

// Removed forges and pulls seen closed need no remote cleanup (domain/forge/adoption-connection.md, 1; forge/change/README.md, 4).
fn release_pull(d: &mut Domain, env: &Env<Limits>, row: &mut ReleaseRow, entry: u64, out: &mut Queue<Request>) -> bool {
    let Some(number) = row.close_pull else { return true };
    let Some(change) = d.changes.get(&row.task) else {
        let why = skein_lib::bytes::concat(&[
            b"no change record for pull request #",
            skein_lib::Decimal::of(number).as_bytes(),
        ]);
        let words = release_failure::task_words(row.task, &why);
        release_failure::held(d, row, words, out);
        return false;
    };
    let repository = change.repository;
    if d.forge_ended(repository.forge)
        || cancellation_pull(d, row.task).is_none()
        || pull_seen_closed(d, repository, number, &env.limits)
    {
        row.close_pull = None;
        row.comment = None;
        release_reads::advanced(d, row.task, env.wall);
        d.releases.insert(row.task, row.clone()).expect("replaces release");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
        true
    } else {
        let write = release_pull_write(row, &d.namespace, number);
        enqueue_release(d, env, row, entry, repository, write, None, out);
        false
    }
}

/// The panel outlives its task, but never its task's live state (domain/forge/change/gates.md, 1).
fn end_change(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    d.fresh_steps.remove(&task);
    d.fresh_waits.remove(&task);
    crate::room::leave(d, crate::room::Waiter::Change(task));
    connection_wait::take(d, task);
    let merge = recorded_merge(d, task);
    let cancelled = match d.releases.get(&task) {
        Some(row) => row.ending == ReleaseEnding::Cancelled,
        None => false,
    };
    crate::verification::released(d, task, out);
    let Some(row) = d.changes.get_mut(&task) else { return };
    let before = crate::panel::ready(row);
    let prior = row.change.clone();
    let fresh = row.ending.is_none() && !row.terminal();
    let owned = row.change.owns_turn;
    let ended = ended_change_state(&row.change.state, merge, cancelled, env.wall);
    if ended == change::State::Ended
        && row.change.ended_held.is_none()
        && let change::State::Held { hold: change::Hold { cause: why, .. }, .. } = &row.change.state
    {
        row.change.ended_held = Some(*why);
    }
    row.room = None;
    row.change.state = ended;
    row.ending = match &row.change.state {
        change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => Some(crate::Ending::Landed),
        change::State::NoChange { .. } => Some(crate::Ending::NoChange),
        change::State::Ended => match row.ending.take() {
            Some(ending) => Some(ending),
            None if cancelled => Some(crate::Ending::Cancelled),
            None => Some(crate::Ending::Failed {
                line: temper_engine_domain_forge_closer::first_line(
                    row.producer_failure.as_deref().unwrap_or(b"change failed"),
                )
                .into(),
            }),
        },
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Held { .. } => unreachable!("ending reduces the change to a terminal state"),
    };
    row.change.released = None;
    row.change.owns_turn = false;
    row.delegate = None;
    row.drift = None;
    let mut saved = row.clone();
    if landed_commit(&prior.state).is_none() && landed_commit(&saved.change.state).is_some() {
        reds::landed(d, &env.limits, &saved, out);
    } else {
        reds::erase_passes(d, task, out);
    }
    crate::change_history::advanced(d, env.wall, &mut saved, &prior, out);
    d.changes.insert(task, saved.clone()).expect("retained change");
    if fresh && let Some(ending) = crate::readiness::ending_kind(saved.ending.as_ref().expect("ending set")) {
        crate::readiness::ending(d, env, &mut saved, ending, out);
    }
    wake_after_turn(d, &saved, owned);
    save_queue_change(d, env, before, saved, out);
}

fn ended_change_state(
    state: &change::State,
    merge: Option<skein_forge::api::Commit>,
    cancelled: bool,
    now: skein_lib::Wall,
) -> change::State {
    match state {
        change::State::Landed { merge, .. } | change::State::Red { merge, .. } if cancelled => {
            change::State::Uncleared { merge: *merge }
        }
        change::State::NoChange { words } => change::State::NoChange { words: words.clone() },
        change::State::Landed { merge, since } => change::State::Landed { merge: *merge, since: *since },
        change::State::Red { merge, since } => change::State::Red { merge: *merge, since: *since },
        change::State::Cleared { merge } => change::State::Cleared { merge: *merge },
        change::State::Uncleared { merge } => change::State::Uncleared { merge: *merge },
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Ended
        | change::State::Held { .. } => match merge {
            Some(merge) if cancelled => change::State::Uncleared { merge },
            Some(merge) => change::State::Landed { merge, since: now },
            None => change::State::Ended,
        },
    }
}

fn release_comment_key(task: u64, namespace: &[u8]) -> Box<[u8]> {
    crate::purpose::EffectPurpose::Release { task, resource: Box::from(b"reason".as_slice()) }
        .key(namespace, u32::MAX)
        .expect("a bounded release identity and a valid namespace")
}

fn keep_release_comment(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ReleaseRow,
    reason: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) -> bool {
    let Some(body) = reason else { return true };
    let Some(number) = row.close_pull else { return true };
    // Do not keep an oversized comment in the release record. Comments that
    // fit here still have to fit as a keyed write when the pull is found.
    if body.len() <= skein_lib::usize_of(env.limits.client.op_bytes) {
        row.comment = Some(body);
        return true;
    }
    let Some(change) = d.changes.get(&row.task) else { return true };
    let repository = change.repository;
    if d.forge_ended(repository.forge)
        || (repository_fresh(d, repository) && pull_seen_closed(d, repository, number, &env.limits))
    {
        return true;
    }
    let write = skein_forge::api::Write::Post { number, key: release_comment_key(row.task, &d.namespace), body };
    let why = release_failure::op_bound(env.limits.client.op_bytes, skein_forge::measure::write(&write));
    let words = release_failure::not_sent(d, repository, &write, &why);
    refuse_release_write(d, row, words, out);
    false
}

fn release_pull_write(row: &ReleaseRow, namespace: &[u8], number: u64) -> skein_forge::api::Write {
    match &row.comment {
        Some(body) => {
            skein_forge::api::Write::Post { number, key: release_comment_key(row.task, namespace), body: body.clone() }
        }
        None => skein_forge::api::Write::Close { number },
    }
}

#[expect(clippy::too_many_arguments, reason = "one cleanup effect carries its durable entry and held resource")]
fn enqueue_release(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ReleaseRow,
    number: u64,
    repository: skein_forge::api::Repository,
    write: skein_forge::api::Write,
    resource: Option<Name>,
    out: &mut Queue<Request>,
) {
    let entry = skein_forge_client::Entry {
        number,
        task: row.task,
        repository,
        effect: skein_forge_client::Effect { write, condition: skein_forge_client::Condition::None },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    if let Some(why) = release_write_refusal(d, &env.limits, &entry) {
        let words = release_failure::not_sent(d, repository, &entry.effect.write, &why);
        refuse_release_write(d, row, words, out);
        return;
    }
    let path = writes::outbox::change_branch(d, entry.task, entry.repository);
    let Some(plan) = writes::outbox::plan(d, &env.limits, &entry, path, writes::outbox::Pin::NONE) else {
        let words = release_failure::not_sent(d, repository, &entry.effect.write, b"its branch name is invalid");
        refuse_release_write(d, row, words, out);
        return;
    };
    let waiter = crate::room::Waiter::Release(row.task);
    let blocked = plan.blocked(d);
    if !crate::room::check(d, &env.limits, Some(waiter)) || blocked.is_some() {
        if row.room.is_none() {
            row.room = Some(crate::room::next_place(d));
        }
        crate::room::block_write(d, waiter, blocked);
        d.releases.insert(row.task, row.clone()).expect("replaces release");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
        return;
    }
    row.room = None;
    crate::room::leave(d, waiter);
    row.pending = Some((number, resource));
    d.releases.insert(row.task, row.clone()).expect("replaces release");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
    plan.admit(d, &entry, out);
    keep_entry(d, entry, Some(number), out);
}

fn release_write_refusal(d: &Domain, limits: &Limits, entry: &skein_forge_client::Entry) -> Option<Box<[u8]>> {
    use skein_forge::api::Write;
    use skein_lib::{Decimal, bytes};
    if d.entries.contains_key(&entry.number) {
        return Some(bytes::concat(&[b"outbox entry ", Decimal::of(entry.number).as_bytes(), b" is already kept"]));
    }
    let Some(repository) = d.repositories.get(&entry.repository) else {
        return Some(Box::from(b"the repository is not adopted".as_slice()));
    };
    if repository.role == Role::Context {
        return Some(Box::from(b"the repository is context".as_slice()));
    }
    let write = &entry.effect.write;
    if !effect_permitted(repository.kinds, write) {
        let kind: &[u8] = match write {
            Write::Post { .. } => b"comment",
            Write::Close { .. } => b"close",
            Write::DeleteBranch { .. } => b"branch",
            Write::CreateIssue { .. }
            | Write::OpenPull { .. }
            | Write::Review { .. }
            | Write::Edit { .. }
            | Write::SetReviewers { .. }
            | Write::Reopen { .. }
            | Write::Merge { .. }
            | Write::Update { .. }
            | Write::Status { .. }
            | Write::Dispatch { .. }
            | Write::Rerun { .. }
            | Write::CreateBranch { .. } => {
                unreachable!("release writes only comments, closes and branch deletions")
            }
        };
        return Some(bytes::concat(&[b"its adoption does not take kind ", kind]));
    }
    if !skein_forge_client::valid_entry(entry, &limits.client) {
        return Some(release_failure::op_bound(
            limits.client.op_bytes,
            skein_forge_client::measure::effect(&entry.effect),
        ));
    }
    None
}

fn release_outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    if outcome == skein_forge_client::Outcome::Uncertain || outcome == skein_forge_client::Outcome::Queued {
        return;
    }
    let mut owner = None;
    for (task, row) in &d.releases {
        if match &row.pending {
            Some((number, _)) => *number == entry,
            None => false,
        } {
            owner = Some(*task);
            break;
        }
    }
    let Some(task) = owner else { return };
    let account = settled_account(d, entry, outcome);
    if !repository_reads::release_fresh(d, task) {
        d.waiting_release_outcomes.insert(entry, (outcome, account)).expect("one outcome per pending release");
        repository_reads::park_release(d, task);
        return;
    }
    finish_release_outcome(d, env, task, entry, outcome, account, out);
}

fn finish_release_outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    account: Option<crate::EffectAccount>,
    out: &mut Queue<Request>,
) {
    let mut row = d.releases.get(&task).expect("selected release").clone();
    let (_, resource) = row.pending.take().expect("selected pending release");
    let removed = match d.entries.get(&entry) {
        Some(entry) => d.forge_ended(entry.entry.repository.forge),
        None => false,
    };
    if removed {
        // Removal leaves the remote forge untouched (temper/docs/design/domain/forge/adoption-connection.md, 1).
        if let Some(name) = resource {
            d.holds.remove(&name);
            crate::tiers::released(d, &name, out);
            reds::released(d, task, &name, out);
            emit(out, Request::Erase { key: Key::Hold(name) });
        } else {
            row.close_pull = None;
            row.comment = None;
        }
    } else if resource.is_none() && row.comment.is_some() {
        // A permanent post failure must not prevent the pull from closing.
        // Uncertain/queued outcomes returned above keep the reason and keyed entry.
        row.comment = None;
    } else {
        match outcome {
            skein_forge_client::Outcome::Made { .. } => match resource {
                Some(name) => {
                    d.holds.remove(&name);
                    crate::tiers::released(d, &name, out);
                    reds::released(d, task, &name, out);
                    emit(out, Request::Erase { key: Key::Hold(name) });
                }
                None => row.close_pull = None,
            },
            skein_forge_client::Outcome::Failed(_)
            | skein_forge_client::Outcome::Raced { .. }
            | skein_forge_client::Outcome::Held
            | skein_forge_client::Outcome::Withdrawn => {
                if resource.is_none()
                    && let Some(number) = row.close_pull
                    && let Some(change) = d.changes.get(&task)
                    && pull_seen_closed(d, change.repository, number, &env.limits)
                {
                    row.close_pull = None;
                } else {
                    row.failed = true;
                    let words = match account {
                        Some(account) => account.words,
                        None => release_outcome_words(d, &row, resource.as_ref(), outcome),
                    };
                    release_failure::report(out, task, Some(entry), words);
                }
            }
            skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {
                unreachable!("uncertain returned above")
            }
        }
    }
    release_reads::advanced(d, task, env.wall);
    d.releases.insert(task, row.clone()).expect("replaces release");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
}

fn release_outcome_words(
    d: &Domain,
    row: &ReleaseRow,
    resource: Option<&Name>,
    outcome: skein_forge_client::Outcome,
) -> Box<[u8]> {
    let why = crate::effect_words::outcome_words(outcome).expect("a terminal cleanup failure has answer words");
    if let Some(name) = resource {
        return release_failure::branch(d, b"delete branch", name, &why);
    }
    if let Some(number) = row.close_pull {
        if let Some(change) = d.changes.get(&row.task) {
            return crate::effect_words::write_account(
                &skein_forge::api::Write::Close { number },
                change.repository,
                d.repository(change.repository),
                &why,
            );
        }
        let why = skein_lib::bytes::concat(&[b"close item #", skein_lib::Decimal::of(number).as_bytes(), b": ", &why]);
        return release_failure::task_words(row.task, &why);
    }
    release_failure::task_words(row.task, &why)
}

#[expect(
    clippy::too_many_lines,
    reason = "one projection admission saves facts, waits, or describes its next keyed effect"
)]
fn project(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    repository: skein_forge::api::Repository,
    view: issues::GoalView,
    out: &mut Queue<Request>,
) {
    let goal = view.goal;
    let ended = if view.finished.is_some() {
        crate::room::owner_ended(d, goal, env.wall);
        Some(d.room.closing.get(&goal).copied().unwrap_or(env.wall))
    } else {
        None
    };
    let Some(adopted) = d.repositories.get(&repository) else {
        fail_projection(d, env, goal, Some(repository), view.finished.is_some(), out);
        return;
    };
    if view.repository != u64::from(repository.repository)
        || adopted.role == Role::Context
        || !adopted.kinds.issue
        || (!d.issues.contains_key(&goal) && d.issues.len() == env.limits.issues)
    {
        fail_projection(d, env, goal, Some(repository), view.finished.is_some(), out);
        return;
    }
    let mut prior = match d.issues.get(&goal) {
        Some(row) if row.repository != repository => {
            fail_projection(d, env, goal, Some(repository), view.finished.is_some(), out);
            return;
        }
        Some(row) => row.clone(),
        None => IssueRow {
            goal,
            repository,
            state: issues::Projected::default(),
            number: None,
            pending: None,
            before: None,
            failed: false,
            desired: None,
        },
    };
    let Some(policy) = projection_limits(d, &env.limits, &prior, &view) else {
        fail_projection(d, env, goal, Some(repository), view.finished.is_some(), out);
        return;
    };
    let closing = view.finished.is_some();
    let Some(view) = issues::gather(prior.desired.as_ref(), view, &policy) else {
        fail_projection(d, env, goal, Some(repository), closing, out);
        return;
    };
    let changed = prior.desired.as_ref() != Some(&view);
    if changed {
        prior.desired = Some(view.clone());
        d.issues.insert(goal, prior.clone()).expect("projection room checked");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(prior.clone()) });
    }
    if !repository_fresh(d, repository) {
        d.projection_waits.insert(goal).expect("one wait per issue");
        return;
    }
    if prior.failed {
        // An Open lost to a crash consumes its committed open bit without
        // yielding an issue number. It can never be sent again, even after
        // its held outbox entry has settled (domain/forge/effects.md, 1 and domain/forge/issues.md).
        if view.finished.is_some() || (prior.state.opened && prior.number.is_none()) {
            settled_projection(d, goal, out);
            return;
        }
        if !changed
            && let Some(when) = d.projection_due.get(&goal)
            && *when > env.wall
        {
            emit(out, Request::ProjectAfter { goal, when: Some(*when) });
            return;
        }
    }
    if prior.pending.is_some() {
        emit(out, Request::ProjectAfter { goal, when: None });
        return;
    }
    let projected = issues::project(&view, &prior.state, env.wall, &policy);
    match projected.decision {
        issues::Decision::None | issues::Decision::Wait(_) => match projected.decision {
            issues::Decision::Wait(when) => emit(out, Request::ProjectAfter { goal, when: Some(when) }),
            issues::Decision::None => settled_projection(d, goal, out),
            issues::Decision::Hold | issues::Decision::Effect(_) => unreachable!("matched above"),
        },
        issues::Decision::Hold => fail_projection(d, env, goal, Some(repository), closing, out),
        issues::Decision::Effect(effect) => {
            assert!(
                !d.projection_room.contains(&number),
                "one outbox reservation for projection entry {number} of goal {goal}"
            );
            if !crate::room::check(d, &env.limits, None) {
                wait_for_room(d, env, goal, out);
                return;
            }
            if d.entries.contains_key(&number) {
                emit(out, Request::ProjectAfter { goal, when: None });
                return;
            }
            let Some(write) = project_write(d, &prior, effect, &env.limits) else {
                fail_projection(d, env, goal, Some(repository), closing, out);
                return;
            };
            let entry = skein_forge_client::Entry {
                number,
                task: goal,
                repository,
                effect: skein_forge_client::Effect { write, condition: skein_forge_client::Condition::None },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            };
            let row = IssueRow {
                goal,
                repository,
                state: projected.projected,
                number: prior.number,
                pending: Some(number),
                before: Some(prior.state),
                failed: false,
                desired: Some(view),
            };
            let Some(description) = crate::describe_projection_effect(adopted, &entry.effect.write) else {
                fail_projection(d, env, goal, Some(repository), closing, out);
                return;
            };
            assert!(
                crate::room::taken(d) < u64::from(d.entries.capacity()),
                "outbox reservation for projection entry {number} of goal {goal} has room within forge.entries"
            );
            let inserted = d.projection_room.insert(number);
            assert!(inserted.is_ok(), "outbox reservation for projection entry {number} of goal {goal} fits its room");
            d.full.reserve(full::Reservation::Projection(number), goal, ended);
            emit(out, Request::ProjectionEffect { row, entry, description });
        }
    }
}

/// A projection meeting the full outbox waits the policy's interval
/// (temper/docs/design/domain/forge/issues.md).
fn wait_for_room(d: &mut Domain, env: &Env<Limits>, goal: u64, out: &mut Queue<Request>) {
    let when =
        skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.issue_policy.interval.as_nanos()));
    d.projection_due.insert(goal, when).expect("one deadline per issue");
    d.full_outbox_met = true;
    emit(out, Request::ProjectAfter { goal, when: Some(when) });
}

fn projection_limits(d: &Domain, limits: &Limits, prior: &IssueRow, view: &issues::GoalView) -> Option<issues::Limits> {
    let title_bound = skein_lib::usize_of(limits.issue_policy.title_bytes);
    let title_bytes = if view.title.len() <= title_bound {
        view.title.len()
    } else {
        let mut end = title_bound.saturating_sub("…".len());
        while !view.title.is_char_boundary(end) {
            end = end.checked_sub(1).expect("UTF-8 title boundary");
        }
        end.checked_add("…".len().min(title_bound)).expect("bounded title")
    };
    // A create retains its purpose key too; an edit carries only title and body.
    let key_bytes = if prior.state.opened {
        0
    } else {
        let key = projection_key(&d.namespace, view.goal, issues::Key::Open, limits)?;
        u32::try_from(key.len()).expect("bounded key")
    };
    Some(issues::Limits {
        body_bytes: limits.issue_policy.body_bytes.min(
            limits
                .client
                .op_bytes
                .saturating_sub(key_bytes)
                .saturating_sub(u32::try_from(title_bytes).expect("bounded title")),
        ),
        ..limits.issue_policy
    })
}

/// A projection's failed row and timer are independent of the task's lifecycle.
fn fail_projection(
    d: &mut Domain,
    env: &Env<Limits>,
    goal: u64,
    repository: Option<skein_forge::api::Repository>,
    closing: bool,
    out: &mut Queue<Request>,
) {
    if !d.issues.contains_key(&goal)
        && d.issues.len() < env.limits.issues
        && let Some(repository) = repository
        && d.repositories.contains_key(&repository)
    {
        d.issues
            .insert(
                goal,
                IssueRow {
                    goal,
                    repository,
                    state: issues::Projected::default(),
                    number: None,
                    pending: None,
                    before: None,
                    failed: true,
                    desired: None,
                },
            )
            .expect("projection room checked");
    }
    if let Some(row) = d.issues.get_mut(&goal) {
        row.failed = true;
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(row.clone()) });
    }
    emit(out, Request::ProjectionFailed { goal });
    if closing {
        emit(out, Request::ProjectionSettled { goal });
    } else if d.issues.contains_key(&goal) {
        let when = skein_lib::Wall::from_nanos(
            env.wall.as_nanos().saturating_add(env.limits.issue_policy.interval.as_nanos()),
        );
        d.projection_due.insert(goal, when).expect("one deadline per issue");
        emit(out, Request::ProjectAfter { goal, when: Some(when) });
    }
}

fn project_write(d: &Domain, row: &IssueRow, effect: issues::Effect, l: &Limits) -> Option<skein_forge::api::Write> {
    match effect {
        issues::Effect::Open { goal, key, title, body, .. } => {
            let key = projection_key(&d.namespace, goal, key, l)?;
            Some(skein_forge::api::Write::CreateIssue { key, title: Box::from(title.as_bytes()), body })
        }
        issues::Effect::Body { title, body, .. } => Some(skein_forge::api::Write::Edit {
            number: row.number?,
            title: Some(Box::from(title.as_bytes())),
            body: Some(body),
        }),
        issues::Effect::Comment { goal, key, body, .. } => {
            let key = projection_key(&d.namespace, goal, key, l)?;
            Some(skein_forge::api::Write::Post { number: row.number?, key, body: Box::from(body.as_bytes()) })
        }
        issues::Effect::Close { .. } => Some(skein_forge::api::Write::Close { number: row.number? }),
    }
}

fn projection_key(namespace: &[u8], goal: u64, key: issues::Key, l: &Limits) -> Option<Box<[u8]>> {
    let (part, number) = match key {
        issues::Key::Open => (1, 0),
        issues::Key::Body(revision) => (2, revision),
        issues::Key::Milestone(milestone) => match milestone {
            issues::MilestoneKey::Lifecycle { task, position } => {
                return crate::purpose::EffectPurpose::ProjectionHistory { goal, task, position, family: 1 }
                    .key(namespace, l.client.op_bytes);
            }
            issues::MilestoneKey::TaskRevision { task, revision } => {
                return crate::purpose::EffectPurpose::ProjectionHistory { goal, task, position: revision, family: 2 }
                    .key(namespace, l.client.op_bytes);
            }
            issues::MilestoneKey::PlanAccepted => (3, 0),
            issues::MilestoneKey::Revision(number) => (4, number),
            issues::MilestoneKey::ChangeLanded(number) => (5, number),
            issues::MilestoneKey::ChangeHeld(number) => (6, number),
            issues::MilestoneKey::Report(number) => (7, number),
            issues::MilestoneKey::Finished => (8, 0),
        },
        issues::Key::Close => (9, 0),
    };
    crate::purpose::EffectPurpose::Projection { goal, part, number }.key(namespace, l.client.op_bytes)
}

fn projection_outcome(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    let lost_comment = outcome == skein_forge_client::Outcome::Held && posting(d, entry);
    let mut goal = None;
    for (number, row) in &d.issues {
        if row.pending == Some(entry) {
            goal = Some(*number);
        }
    }
    let Some(goal) = goal else { return };
    let Some(row) = d.issues.get_mut(&goal) else { unreachable!("found issue row") };
    match outcome {
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => return,
        // The projected milestone prefix already includes this comment. A
        // crash-lost creation must never be resent (domain/forge/effects.md, 1 and domain/forge/issues.md).
        skein_forge_client::Outcome::Held if lost_comment => {}
        skein_forge_client::Outcome::Held if row.number.is_none() => {
            // Keep the consumed Open bit so future feeds cannot create again.
            row.failed = true;
        }
        skein_forge_client::Outcome::Made { made, .. } => {
            if row.number.is_none() {
                match made {
                    skein_forge_client::Made::Created(number) => row.number = Some(number),
                    skein_forge_client::Made::Commented(_)
                    | skein_forge_client::Made::Reviewed(_)
                    | skein_forge_client::Made::Merged(_)
                    | skein_forge_client::Made::Updated(_)
                    | skein_forge_client::Made::Branch(_)
                    | skein_forge_client::Made::Set => row.failed = true,
                }
            }
        }
        skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => {
            if let Some(before) = row.before.take() {
                row.state = before;
            }
            row.failed = true;
        }
    }
    row.pending = None;
    row.before = None;
    let saved = row.clone();
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(saved) });
    if row.failed {
        emit(out, Request::ProjectionFailed { goal });
        if match &row.desired {
            Some(view) => view.finished.is_some(),
            None => false,
        } {
            settled_projection(d, goal, out);
        } else {
            let when = skein_lib::Wall::from_nanos(
                env.wall.as_nanos().saturating_add(env.limits.issue_policy.interval.as_nanos()),
            );
            d.projection_due.insert(goal, when).expect("one deadline per issue");
            emit(out, Request::ProjectAfter { goal, when: Some(when) });
        }
    } else {
        emit(out, Request::ProjectAfter { goal, when: None });
    }
}

fn posting(d: &Domain, entry: u64) -> bool {
    match d.entries.get(&entry) {
        Some(row) => match &row.entry.effect.write {
            skein_forge::api::Write::Post { .. } => true,
            skein_forge::api::Write::CreateIssue { .. }
            | skein_forge::api::Write::OpenPull { .. }
            | skein_forge::api::Write::Review { .. }
            | skein_forge::api::Write::SetReviewers { .. }
            | skein_forge::api::Write::Edit { .. }
            | skein_forge::api::Write::Close { .. }
            | skein_forge::api::Write::Reopen { .. }
            | skein_forge::api::Write::Merge { .. }
            | skein_forge::api::Write::Update { .. }
            | skein_forge::api::Write::CreateBranch { .. }
            | skein_forge::api::Write::DeleteBranch { .. }
            | skein_forge::api::Write::Status { .. }
            | skein_forge::api::Write::Dispatch { .. }
            | skein_forge::api::Write::Rerun { .. } => false,
        },
        None => false,
    }
}

fn release_projection(d: &mut Domain, goal: u64, out: &mut Queue<Request>) {
    let Some(row) = d.issues.get_mut(&goal) else { return };
    if row.pending.is_some() {
        return;
    }
    row.failed = false;
    let saved = row.clone();
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(saved) });
    emit(out, Request::ProjectAfter { goal, when: None });
}

fn committed(d: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    if d.stopping {
        return;
    }
    if let Some(row) = d.entries.get(&number) {
        let task = row.entry.task;
        let merge = match row.entry.effect.write {
            skein_forge::api::Write::Merge { .. } => true,
            skein_forge::api::Write::CreateIssue { .. }
            | skein_forge::api::Write::Post { .. }
            | skein_forge::api::Write::Edit { .. }
            | skein_forge::api::Write::Close { .. }
            | skein_forge::api::Write::OpenPull { .. }
            | skein_forge::api::Write::Status { .. }
            | skein_forge::api::Write::Review { .. }
            | skein_forge::api::Write::SetReviewers { .. }
            | skein_forge::api::Write::Reopen { .. }
            | skein_forge::api::Write::Update { .. }
            | skein_forge::api::Write::CreateBranch { .. }
            | skein_forge::api::Write::Dispatch { .. }
            | skein_forge::api::Write::Rerun { .. }
            | skein_forge::api::Write::DeleteBranch { .. } => false,
        };
        if merge {
            let direct = d.changes.contains_key(&task);
            if direct {
                clear_merge_reviews(&mut d.review_facts, &mut d.review_heads, task, out);
            } else if let Some(row) = d.landing.rows.get(&task) {
                for member in &row.landing.members {
                    if d.landing_of(member.change) == Some(task) {
                        clear_merge_reviews(&mut d.review_facts, &mut d.review_heads, member.change, out);
                    }
                }
            }
        }
    }
    if let Some(row) = d.entries.get_mut(&number)
        && row.handoff == Handoff::Uncommitted
    {
        row.handoff = Handoff::Ready;
    }
    handoff_one(d, env, out);
}

// True only when a handoff could have changed the outbox line's inputs.
fn handoff_one(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    if d.stopping || d.restore_phase == RestorePhase::Restoring {
        return false;
    }
    // skein's forge/client-effects.md, 1.2: each new Make is above every held entry.
    // Decision reads and goal-branch makings retain their reserved numbers; startup
    // reads have none. Built changes, projections and releases wait for lower chains
    // and entries to finish or commit. Hand over at most one per step.
    let Some(number) = d.next_handoff() else { return false };
    let row = d.entries.get_mut(&number).expect("selected entry remains");
    row.handoff = Handoff::Handed;
    let entry = row.entry.clone();
    let event = if outage::recommitted(d, number) {
        skein_forge_client::Event::Remake { entry }
    } else {
        skein_forge_client::Event::Make { entry }
    };
    child(d, env, event, out);
    true
}

/// A person's choice for a held entry: again hands it back to the client,
/// which looks for it first and then makes it under a new attempt; the
/// entry kept its room, so the client has room for it.
fn resolve(d: &mut Domain, env: &Env<Limits>, number: u64, again: bool, out: &mut Queue<Request>) {
    let Some(row) = d.entries.get_mut(&number) else { return };
    if !row.held {
        return;
    }
    let task = row.entry.task;
    if !again {
        not_made(d, env, task, number, out);
        return;
    }
    row.check_owner(number, task);
    row.held = false;
    let entry = row.entry.clone();
    emit(
        out,
        Request::Save { read_afresh: None, release: None, record: Stored::Entry { entry: entry.clone(), held: false } },
    );
    child(d, env, skein_forge_client::Event::Remake { entry }, out);
}

fn erase_flight(d: &mut Domain, task: u64, entry: u64, out: &mut Queue<Request>) {
    if d.flights.remove(&(task, entry)).is_some() {
        emit(out, Request::Erase { key: Key::Flight { task, entry } });
    }
}

/// Settle one entry as not made, freeing its room.
fn not_made(d: &mut Domain, env: &Env<Limits>, task: u64, number: u64, out: &mut Queue<Request>) {
    take_entry(d, number, task);
    outage::forget_entry(d, number);
    crate::landing::not_made(d, env, task, number, out);
    emit(out, Request::Erase { key: Key::Entry(number) });
    erase_flight(d, task, number, out);
    emit(out, Request::NotMade { entry: number, task });
}

/// A closing task's held entries are settled not made, as many as the
/// output has room for, so its effects settle and their room is freed. True
/// when none of the task's entries remains; one settled here continues the
/// close in a later step.
fn settle_held(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) -> bool {
    let mut held = List::with_capacity(d.entries.len());
    let mut pending = false;
    for (number, row) in &d.entries {
        if row.entry.task != task {
            continue;
        }
        if row.held {
            held.push(*number).expect("held entries are bounded by the outbox");
        } else {
            pending = true;
        }
    }
    if held.is_empty() {
        return !pending;
    }
    for number in &held {
        // Both possible erases, a landing save and mark, settlement, and the continuation below.
        let needed = if d.landing.rows.contains_key(&task) { 6 } else { 4 };
        if out.room() < needed {
            break;
        }
        not_made(d, env, task, *number, out);
    }
    continue_release_once(out, task);
    false
}

fn withdraw(d: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    if let Some(row) = d.entries.get(&number)
        && !row.held
    {
        if row.handoff != Handoff::Handed && row.entry.attempt.is_none() {
            d.client_out.push(skein_forge_client::Request::Outcome {
                entry: number,
                task: row.entry.task,
                outcome: skein_forge_client::Outcome::Withdrawn,
            });
            drain(d, env, out);
        } else {
            child(d, env, skein_forge_client::Event::Withdraw { entry: number }, out);
        }
    }
}

fn veto_change(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    prior: change::Change,
    leaves_queue: bool,
    out: &mut Queue<Request>,
) {
    let unsent = match d.entries.get(&entry) {
        Some(kept) => kept.entry.effect.write.clone(),
        None => return,
    };
    let Some(row) = d.changes.get_mut(&task) else { return };
    if row.pending != Some(entry) {
        return;
    }
    let before = crate::panel::ready(row);
    let owned = row.change.owns_turn;
    row.change = prior;
    if let skein_forge::api::Write::Rerun { attempt } = unsent
        && let Some(recheck) = row.ci_recheck
        && recheck.job == attempt
    {
        row.ci_recheck = None;
    }
    crate::follow_check::veto(row, entry, &unsent);
    if leaves_queue {
        row.change.owns_turn = false;
        match row.change.state {
            change::State::First { head, .. } | change::State::Queued { head, .. } => {
                row.change.state = change::State::Gating { head, asked: Box::new([]) };
            }
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
            | change::State::NoChange { .. }
            | change::State::Ended
            | change::State::Landed { .. }
            | change::State::Red { .. }
            | change::State::Cleared { .. }
            | change::State::Uncleared { .. }
            | change::State::Held { .. } => {}
        }
    }
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.effect_account = None;
    let mut saved = row.clone();
    let yielded = yield_approval_turn(d, &mut saved);
    let repository = saved.repository;
    wake_after_turn(d, &saved, owned);
    d.changes.insert(task, saved.clone()).expect("replaces change");
    save_queue_change(d, env, before, saved, out);
    if yielded && let Some(tasks) = crate::views::changes_mark(d, repository) {
        emit(out, Request::Marked { tasks: Box::from(tasks) });
    }
    writes::outbox::outcome(d, task, entry, skein_forge_client::Outcome::Withdrawn, out);
    take_entry(d, entry, task);
    emit(out, Request::Erase { key: Key::Entry(entry) });
}

fn subscribe(d: &mut Domain, env: &Env<Limits>, subscription: Subscriber, out: &mut Queue<Request>) {
    let task = subscription.task;
    let ci = match &subscription.topic {
        Topic::Ci { repository, head } => Some((*repository, *head)),
        Topic::Landings { .. } | Topic::Pull { .. } | Topic::Participation { .. } | Topic::Findings { .. } => None,
    };
    if let Some(problem) = admission::subscription_problem(d, &env.limits, &subscription) {
        match problem.failure {
            Some((resource, cause)) => {
                emit(out, Request::ResourceRefused { task, operation: b"subscribe", resource, cause });
            }
            None => emit(out, Request::Refused { task }),
        }
        return;
    }
    let key = (task, subscription.topic.clone());
    d.subscriptions.insert(key, subscription.clone()).expect("preflighted subscription capacity");
    let landing = match &subscription.topic {
        Topic::Landings { .. } => true,
        Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } | Topic::Findings { .. } => false,
    };
    let tasks = subscription.goal_tasks.clone();
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Subscription(subscription) });
    if let Some(tasks) = tasks {
        for task in tasks {
            topics::refresh(d, env, task, out);
        }
    }
    if let Some((repository, head)) = ci {
        ci_refresh(d, env, repository, head, out);
    }
    if landing {
        crate::tiers::watch(d, env, out);
    }
}

fn unsubscribe(d: &mut Domain, env: &Env<Limits>, task: u64, topic: Topic, out: &mut Queue<Request>) {
    if d.subscriptions.remove(&(task, topic.clone())).is_some() {
        emit(out, Request::Erase { key: Key::Subscription { task, topic: topic.clone() } });
        match topic {
            Topic::Ci { repository, head } => {
                let mut named = false;
                for (_, subscriber) in &d.subscriptions {
                    if subscriber.topic == (Topic::Ci { repository, head }) {
                        named = true;
                    }
                }
                if !named && d.ci.remove(&(repository, head)).is_some() {
                    emit(out, Request::Erase { key: Key::Ci { repository, head } });
                }
            }
            Topic::Landings { .. } => crate::tiers::watch(d, env, out),
            Topic::Pull { .. } | Topic::Participation { .. } | Topic::Findings { .. } => {}
        }
    }
}

fn ci_refresh(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    head: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    if d.forge_ended(repository.forge) {
        return;
    }
    for (_, pending) in &d.pending_ci {
        if pending.repository == repository && pending.head == head {
            return;
        }
    }
    if d.pending_ci.len() == d.pending_ci.capacity() {
        return;
    }
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 60));
    d.pending_ci.insert(owner, PendingCi { repository, head, refresh: false }).expect("CI read capacity checked");
    child(
        d,
        env,
        skein_forge_client::Event::Read { owner, repository, read: skein_forge::api::Read::Checks { commit: head } },
        out,
    );
}

fn ci_hint(d: &mut Domain, env: &Env<Limits>, hint: &skein_forge::api::Hint, out: &mut Queue<Request>) {
    let head = match &hint.change {
        skein_forge::api::Change::Commit(head) => *head,
        skein_forge::api::Change::Item(_)
        | skein_forge::api::Change::Branch(_)
        | skein_forge::api::Change::BranchMoved { .. } => {
            return;
        }
    };
    let mut subscribed = false;
    for (_, subscriber) in &d.subscriptions {
        match &subscriber.topic {
            Topic::Ci { repository, head: watched } if *repository == hint.repository && *watched == head => {
                subscribed = true;
                break;
            }
            Topic::Ci { .. }
            | Topic::Landings { .. }
            | Topic::Pull { .. }
            | Topic::Participation { .. }
            | Topic::Findings { .. } => {}
        }
    }
    if subscribed {
        ci_refresh(d, env, hint.repository, head, out);
    }
}

fn ci_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(pending) = d.pending_ci.remove(&owner) else { return };
    if !d.repositories.contains_key(&pending.repository) {
        return;
    }
    if let Ok(skein_forge::api::Answer::Checks(rows)) = result {
        let required = match d.repositories.get(&pending.repository) {
            Some(repository) => crate::ci::required(&repository.protection),
            None => &[],
        };
        let ci = crate::ci::fold(&rows, required);
        ci_news::read(d, env, pending.repository, pending.head, Some(&rows), ci);
        ci_observed(d, pending.repository, pending.head, ci, out);
    } else {
        ci_news::read(d, env, pending.repository, pending.head, None, skein_forge::api::Ci::None);
    }
    if pending.refresh && !d.stopping {
        ci_refresh(d, env, pending.repository, pending.head, out);
    }
}

fn ci_observed(
    d: &mut Domain,
    repository: skein_forge::api::Repository,
    head: skein_forge::api::Commit,
    status: skein_forge::api::Ci,
    out: &mut Queue<Request>,
) {
    let topic = Topic::Ci { repository, head };
    if !d.repositories.contains_key(&repository) {
        return;
    }
    let mut named = false;
    for (_, subscriber) in &d.subscriptions {
        if subscriber.topic == topic {
            named = true;
        }
    }
    let same = match d.ci.get(&(repository, head)) {
        Some(prior) => prior.status == status,
        None => false,
    };
    if !named || same {
        return;
    }
    if d.ci.len() == d.ci.capacity() && !d.ci.contains_key(&(repository, head)) {
        return;
    }
    let row = CiState { repository, head, status };
    d.ci.insert((repository, head), row.clone()).expect("CI subscription capacity");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Ci(row) });
    publish(d, topic, News::Ci { head, status }, out);
}

fn changed(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    if !d.repositories.contains_key(&resource.repository) {
        return;
    }
    match result {
        Ok(skein_forge::api::Answer::Commit(commit)) => branch_changed(d, env, resource, commit, out),
        Ok(skein_forge::api::Answer::Pull(pull)) => pull_changed(d, env, resource, pull, out),
        Ok(skein_forge::api::Answer::Item { item, comments, more: _ }) => {
            let topic = Topic::Participation { repository: resource.repository, number: item.number };
            if !comments.is_empty() {
                publish(d, topic, News::Changed { number: item.number }, out);
            }
        }
        Ok(skein_forge::api::Answer::Reviews { reviews, .. }) => match resource.what {
            skein_forge_client::What::Pull(number) if !reviews.is_empty() => {
                let topic = Topic::Participation { repository: resource.repository, number };
                publish(d, topic, News::Changed { number }, out);
            }
            skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Repository
            | skein_forge_client::What::Branch(_)
            | skein_forge_client::What::Issue(_) => {}
        },
        Ok(
            skein_forge::api::Answer::Items { .. }
            | skein_forge::api::Answer::Located { .. }
            | skein_forge::api::Answer::Branches(_)
            | skein_forge::api::Answer::Statuses { .. }
            | skein_forge::api::Answer::Remarks { .. }
            | skein_forge::api::Answer::PullFiles { .. }
            | skein_forge::api::Answer::Compare { .. }
            | skein_forge::api::Answer::Checks(_)
            | skein_forge::api::Answer::Directory { .. }
            | skein_forge::api::Answer::File { .. }
            | skein_forge::api::Answer::JobState { .. }
            | skein_forge::api::Answer::Job { .. }
            | skein_forge::api::Answer::Protection(_)
            | skein_forge::api::Answer::Settings(_)
            | skein_forge::api::Answer::Collaborators { .. }
            | skein_forge::api::Answer::Permission(_)
            | skein_forge::api::Answer::Created(_)
            | skein_forge::api::Answer::Commented(_)
            | skein_forge::api::Answer::Reviewed(_)
            | skein_forge::api::Answer::Merged(_)
            | skein_forge::api::Answer::Runs { .. }
            | skein_forge::api::Answer::Run(_)
            | skein_forge::api::Answer::Commits { .. }
            | skein_forge::api::Answer::Tree { .. }
            | skein_forge::api::Answer::Queued
            | skein_forge::api::Answer::Branch(_)
            | skein_forge::api::Answer::Done,
        )
        | Err(_) => {}
    }
}

pub(crate) fn from_client(resource: &skein_forge_client::Resource, l: &Limits) -> Option<Name> {
    let what = match &resource.what {
        skein_forge_client::What::Repository => What::Repository,
        skein_forge_client::What::Pull(number) => What::Pull(*number),
        skein_forge_client::What::Issue(number) => What::Issue(*number),
        skein_forge_client::What::Branch(branch) => {
            if branch.is_empty() || branch.len() > usize::try_from(l.name_bytes).ok()? {
                return None;
            }
            let mut parts = List::with_capacity(l.name_bytes);
            let mut start = 0usize;
            for (index, byte) in branch.iter().enumerate() {
                if *byte == b'/' {
                    if index == start {
                        return None;
                    }
                    parts.push(Box::from(branch.get(start..index)?)).ok()?;
                    start = index.checked_add(1)?;
                }
            }
            if start == branch.len() {
                return None;
            }
            parts.push(Box::from(branch.get(start..)?)).ok()?;
            What::Branch(parts.into_boxed())
        }
    };
    Some(Name { forge: resource.repository.forge, repository: resource.repository.repository, what })
}

// The root reads jig's pending dependency graph when it receives this wake.
fn wake_head_dependents(
    d: &mut Domain,
    name: &Name,
    previous: Option<skein_forge::api::Commit>,
    commit: skein_forge::api::Commit,
) {
    if previous == Some(commit) {
        return;
    }
    for (task, row) in &d.changes {
        if row.frozen() || row.repository.forge != name.forge || row.repository.repository != name.repository {
            continue;
        }
        if crate::items::branch_matches(name, &row.branch) && !d.procedure_wakes.contains_key(task) {
            d.procedure_wakes.insert(*task, false).expect("one pending wake per change");
        }
        if crate::items::branch_matches(name, &row.base)
            && let change::State::First { .. } | change::State::Landing { .. } = row.change.state
        {
            d.procedure_wakes.insert(*task, true).expect("one pending wake per change");
        }
    }
}

fn completed_branch(d: &Domain, repository: skein_forge::api::Repository, branch: &[u8]) -> bool {
    let mut completed = false;
    for (_, row) in &d.changes {
        if row.repository == repository && row.branch.as_ref() == branch {
            if !row.frozen() {
                return false;
            }
            completed = true;
        }
    }
    // A goal's own watch still judges its branch, beside retained ended changes.
    for (task, names) in &d.names {
        if let Some(row) = d.changes.get(task)
            && row.frozen()
        {
            continue;
        }
        for name in names {
            if name.forge == repository.forge
                && name.repository == repository.repository
                && crate::items::branch_matches(name, branch)
            {
                return false;
            }
        }
    }
    completed
}

fn owned_head(d: &Domain, name: &Name) -> bool {
    match d.heads.get(name) {
        Some(row) => row.owned,
        None => false,
    }
}

fn branch_changed(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    if !d.repositories.contains_key(&resource.repository) {
        return;
    }
    let Some(name) = from_client(&resource, &env.limits) else { return };
    if let Some(record) = d.writes.get(&name) {
        if record.head == Some(commit) {
            d.kept_moves.remove(&name);
            if d.client.expected(&resource) != Some(commit)
                && let Some(cached) = d.client.cached(&resource)
                && cached.tip == Some(commit)
            {
                writes::kept::refresh_head(d, &name);
            }
        } else if d.holds.contains_key(&name) || owned_head(d, &name) {
            // Raw Changed news is not ownership. A held or owned branch
            // only advances its workspace tip through the record.
            return;
        }
    }
    if goal_branch::is_goal(d, &name) && d.holds.contains_key(&name) {
        return;
    }
    if let Some(hold) = d.holds.get(&name)
        && hold.drift.is_some()
    {
        return;
    }
    record_branch_tip(d, env, name, &resource, commit, out);
}

fn record_branch_tip(
    d: &mut Domain,
    env: &Env<Limits>,
    name: Name,
    resource: &skein_forge_client::Resource,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let Some(branch) = (match &resource.what {
        skein_forge_client::What::Branch(branch) => Some(branch.clone()),
        skein_forge_client::What::Repository
        | skein_forge_client::What::Pull(_)
        | skein_forge_client::What::Issue(_) => None,
    }) else {
        return;
    };
    let previous = match d.heads.get(&name) {
        Some(row) => Some(row.commit),
        None => None,
    };
    let recorded = match d.heads.get(&name) {
        Some(row) => row.owned,
        None => false,
    };
    let row = BranchHead { name: name.clone(), commit, owned: recorded };
    let concerning = heads::concerning(d, &name, None);
    if !heads::record(d, env, row.clone(), concerning, out) {
        return;
    }
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(row.clone()) });
    wake_head_dependents(d, &row.name, previous, commit);
    crate::tiers::observed(d, env, resource.repository, &branch, previous, commit, out);
    let Some(before) = previous else {
        crate::workflows::first_tip(d, env, resource.repository, &branch, commit, out);
        return;
    };
    if before == commit || completed_branch(d, resource.repository, &branch) {
        return;
    }
    let topic = Topic::Landings { repository: resource.repository, branch: branch.clone() };
    if d.landings.len() >= env.limits.landings.max(env.limits.client.resources) {
        crate::workflows::landed(d, env, resource.repository, &branch, commit, None, out);
        publish(d, topic, News::Landing { before, after: commit, files: None }, out);
        return;
    }
    let Some(sequence) = d.sequence.checked_add(1) else {
        crate::workflows::landed(d, env, resource.repository, &branch, commit, None, out);
        publish(d, topic, News::Landing { before, after: commit, files: None }, out);
        return;
    };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 63));
    d.landings.insert(owner, PendingLanding { topic, before, after: commit }).expect("landing capacity preflighted");
    child(
        d,
        env,
        skein_forge_client::Event::Read {
            owner,
            repository: resource.repository,
            read: skein_forge::api::Read::Compare { before, after: commit },
        },
        out,
    );
}

fn landing_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(pending) = d.landings.remove(&owner) else { return };
    if let Topic::Landings { repository, branch } = &pending.topic
        && completed_branch(d, *repository, branch)
    {
        return;
    }
    let files = match result {
        Ok(skein_forge::api::Answer::Compare { before, after, files, changed, .. })
            if before == pending.before && after == pending.after && skein_lib::len_u64(files.len()) == changed =>
        {
            Some(files)
        }
        Ok(_) | Err(_) => None,
    };
    match &pending.topic {
        Topic::Landings { repository, branch } => {
            crate::workflows::landed(d, env, *repository, branch, pending.after, files.as_deref(), out);
        }
        Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } | Topic::Findings { .. } => {
            unreachable!("landing comparison belongs to a landing")
        }
    }
    publish(d, pending.topic, News::Landing { before: pending.before, after: pending.after, files }, out);
}

/// A changed pull invalidates a gather even while its procedure is already due.
fn wake_pull_changes(
    d: &mut Domain,
    repository: skein_forge::api::Repository,
    pull: &skein_forge::api::Pull,
    facts: bool,
) {
    for (_, row) in &d.changes {
        if row.frozen() || row.repository != repository || row.pull != Some(pull.number) {
            continue;
        }
        if !facts {
            let required = match d.repositories.get(&repository) {
                Some(repository) => !crate::ci::required(&repository.protection).is_empty(),
                None => false,
            };
            if required || ci_news::seen(d, row.task, pull.commit, pull.ci) {
                continue;
            }
        }
        let mut reading = None;
        for (owner, pending) in &d.steps {
            let different = match &pending.pull {
                Some(gathered) => !same_pull_facts(gathered, pull) || (pending.pull_ci && gathered.ci != pull.ci),
                None => true,
            };
            if pending.task == row.task && different {
                reading = Some(*owner);
            }
        }
        if let Some(owner) = reading {
            d.steps.get_mut(&owner).expect("change's pending gather").woken = true;
        }
        d.procedure_wakes.insert(row.task, true).expect("one pending wake per change");
    }
}

fn same_pull_facts(left: &skein_forge::api::Pull, right: &skein_forge::api::Pull) -> bool {
    let mut left = left.clone();
    left.ci = right.ci;
    left == *right
}

pub(crate) fn pull_changed(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    pull: skein_forge::api::Pull,
    out: &mut Queue<Request>,
) {
    if !d.repositories.contains_key(&resource.repository) {
        return;
    }
    crate::landing::pull_changed(d, resource.repository, &pull, out);
    let name = from_client(&resource, &env.limits);
    let prior = match &name {
        Some(name) => d.pulls.get(name).cloned(),
        None => None,
    };
    // Only what the gather reads wakes the change: an unchanged poll does not (8.3).
    let facts = match (&prior, &name) {
        (Some(row), Some(name)) => {
            row.head != pull.commit || row.state != pull.state || d.pull_mergeable.get(name) != Some(&pull.mergeable)
        }
        (Some(_), None) | (None, _) => true,
    };
    let ci = match &prior {
        Some(row) => row.ci != pull.ci,
        None => true,
    };
    if facts || ci {
        wake_pull_changes(d, resource.repository, &pull, facts);
    }
    let mut changed = List::with_capacity(env.limits.changes);
    for (_, row) in &d.changes {
        if !row.frozen() && row.repository == resource.repository && row.pull == Some(pull.number) {
            // A cancelled change's release closes its own pull (8.6): no drift.
            let cancelling = match d.releases.get(&row.task) {
                Some(release) => release.ending == ReleaseEnding::Cancelled,
                None => false,
            };
            let change = if row.base != pull.base {
                Some(crate::DriftChange::Retargeted {
                    number: pull.number,
                    before: row.base.clone(),
                    after: pull.base.clone(),
                })
            } else if pull.state == skein_forge::api::State::Closed && pull.merged.is_none() && !cancelling {
                Some(crate::DriftChange::PullClosed { number: pull.number })
            } else {
                None
            };
            if let Some(change) = change {
                changed.push((row.branch.clone(), change)).expect("bounded owned changes");
            }
        }
    }
    for (branch, change) in changed.into_boxed() {
        drift(
            d,
            env,
            skein_forge_client::Resource {
                repository: resource.repository,
                what: skein_forge_client::What::Branch(branch),
            },
            change,
            out,
        );
    }
    let Some(name) = name else { return };
    let state = PullState { name: name.clone(), head: pull.commit, ci: pull.ci, state: pull.state };
    if d.pulls.insert(name.clone(), state.clone()).is_err() {
        return;
    }
    d.pull_mergeable.insert(name, pull.mergeable).expect("mergeable beside each pull row");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::PullState(state) });
    let topic = Topic::Pull { repository: resource.repository, number: pull.number };
    let changed_pull = match &prior {
        Some(row) => row.head != pull.commit || row.state != pull.state,
        None => true,
    };
    if changed_pull {
        let mut rows = List::with_capacity(env.limits.changes);
        for (_, row) in &d.changes {
            if row.repository == resource.repository && row.pull == Some(pull.number) {
                rows.push(row.clone()).expect("retained changes");
            }
        }
        for mut row in rows.into_boxed() {
            crate::change_history::recorded(d, env.wall, &mut row, &pull, out);
        }
        publish(d, topic, News::Changed { number: pull.number }, out);
    }
    let changed_ci = match &prior {
        Some(row) => row.head != pull.commit || row.ci != pull.ci,
        None => true,
    };
    if changed_ci {
        let required = match d.repositories.get(&resource.repository) {
            Some(repository) => crate::ci::required(&repository.protection),
            None => &[],
        };
        if required.is_empty() {
            ci_observed(d, resource.repository, pull.commit, pull.ci, out);
        } else {
            ci_news::refreshing(d, resource.repository, pull.commit);
            ci_refresh(d, env, resource.repository, pull.commit, out);
        }
    }
}

pub(crate) fn overlap(files: &[Box<[u8]>], paths: &[Box<[u8]>]) -> bool {
    for file in files {
        for path in paths {
            if file.as_ref() == path.as_ref() {
                return true;
            }
            if file.starts_with(path) && file.get(path.len()) == Some(&b'/') {
                return true;
            }
        }
    }
    false
}

pub(crate) fn classify(d: &Domain, subscription: &Subscriber, news: &News) -> Class {
    match news {
        News::Landing { after, files, .. } => {
            if let Some(owner) = d.landed.get(after)
                && subscription.own_change == Some(*owner)
            {
                return Class::Kept;
            }
            match files {
                None => Class::Wakes,
                Some(files) if overlap(files, &subscription.paths) => Class::Wakes,
                Some(_) => Class::Kept,
            }
        }
        News::Ci { .. } | News::Changed { .. } | News::Red(_) => Class::Wakes,
    }
}

fn publish(d: &Domain, topic: Topic, news: News, out: &mut Queue<Request>) {
    for (_, subscriber) in &d.subscriptions {
        if subscriber.topic == topic {
            let class = topics::classify(d, subscriber, &news);
            emit(
                out,
                Request::News {
                    task: subscriber.task,
                    subscription: subscriber.number,
                    topic: topic.clone(),
                    news: news.clone(),
                    class,
                },
            );
        }
    }
}

// temper/docs/design/domain/forge/change/README.md, 1, and temper/docs/design/domain/forge/change/drift.md, 1: the holder's delegates share its branch.
fn change_on_held_branch(row: &ChangeRow, name: &Name, holder: u64) -> bool {
    row.task != holder
        && row.repository.forge == name.forge
        && row.repository.repository == name.repository
        && crate::items::branch_matches(name, &row.branch)
        && !row.frozen()
}

// The requester goes on beside its change; every other writer still stops it (domain/forge/change/drift.md, 1).
fn change_requester_writing(d: &Domain, row: &ChangeRow) -> bool {
    for (name, hold) in &d.holds {
        if change_on_held_branch(row, name, hold.task) {
            return match hold.writer {
                Some(Writer::Run { task, .. }) => task == hold.task,
                Some(Writer::Lost { .. } | Writer::Entry(_)) => false,
                None => !hold.design_writers.is_empty(),
            };
        }
    }
    false
}

/// A merge entry writes the held base from admission until settlement, beside any design writer.
fn held_branch_merge(d: &Domain, row: &ChangeRow) -> bool {
    for (_, entry) in &d.entries {
        if !entry.held
            && entry.entry.repository == row.repository
            && let skein_forge_client::Condition::Merge { base } = &entry.entry.effect.condition
            && *base == row.branch
        {
            return true;
        }
    }
    false
}

/// A queued keep observation can follow the outcome that already confirmed this update.
fn frozen_observation(d: &Domain, resource: &skein_forge_client::Resource, change: &crate::DriftChange) -> bool {
    let number = match change {
        crate::DriftChange::PullClosed { number } | crate::DriftChange::Retargeted { number, .. } => *number,
        crate::DriftChange::Branch { .. } => match &resource.what {
            skein_forge_client::What::Pull(number) => *number,
            skein_forge_client::What::Branch(branch) => return completed_branch(d, resource.repository, branch),
            skein_forge_client::What::Repository | skein_forge_client::What::Issue(_) => return false,
        },
    };
    for (_, row) in &d.changes {
        if row.repository == resource.repository && row.pull == Some(number) && row.frozen() {
            return true;
        }
    }
    false
}

pub(crate) fn drift(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    change: crate::DriftChange,
    out: &mut Queue<Request>,
) {
    if frozen_observation(d, &resource, &change) {
        return;
    }
    let mut affected = List::with_capacity(env.limits.holds);
    for (name, row) in &d.holds {
        if to_client(name, &env.limits) == Some(resource.clone()) {
            if let Some(change) = d.changes.get(&row.task)
                && change.frozen()
            {
                continue;
            }
            let repeated = match &row.drift {
                Some(known) => known.change == change,
                None => false,
            };
            if repeated {
                continue;
            }
            if deleting_entry(d, row.task, &resource, &change) {
                continue;
            }
            emit(out, Request::Drift { task: row.task, resource: name.clone() });
            affected.push((row.task, name.clone())).expect("one affected task per hold");
        }
    }
    for (task, name) in &affected {
        if let Some(hold) = d.holds.get_mut(name) {
            hold.drift = Some(crate::Drift { at: env.wall, change: change.clone() });
            save_hold(out, hold.clone());
        }
        if let Some(row) = d.changes.get_mut(task) {
            row.drift = Some(match change {
                crate::DriftChange::Branch { .. } => change::Cause::BranchMoved,
                crate::DriftChange::PullClosed { .. } => change::Cause::PullClosed,
                crate::DriftChange::Retargeted { .. } => change::Cause::Retargeted,
            });
            let saved = row.clone();
            save_change(d, out, saved);
        }
    }
    let mut changes = List::with_capacity(env.limits.changes);
    for (holder, name) in &affected {
        for (_, row) in &d.changes {
            if change_on_held_branch(row, name, *holder) && !deleting_entry(d, row.task, &resource, &change) {
                changes.push((row.task, name.clone())).expect("one affected row per change");
            }
        }
    }
    for (task, name) in changes.into_boxed() {
        let Some(row) = d.changes.get_mut(&task) else { unreachable!("collected change") };
        row.drift = Some(match change {
            crate::DriftChange::Branch { .. } => change::Cause::BranchMoved,
            crate::DriftChange::PullClosed { .. } => change::Cause::PullClosed,
            crate::DriftChange::Retargeted { .. } => change::Cause::Retargeted,
        });
        let saved = row.clone();
        save_change(d, out, saved);
        emit(out, Request::Drift { task, resource: name });
    }
}

fn deleting_entry(d: &Domain, task: u64, resource: &skein_forge_client::Resource, change: &crate::DriftChange) -> bool {
    match change {
        crate::DriftChange::Branch { observed: None, .. } => {}
        crate::DriftChange::Branch { observed: Some(_), .. }
        | crate::DriftChange::PullClosed { .. }
        | crate::DriftChange::Retargeted { .. } => return false,
    }
    let Some(release) = d.releases.get(&task) else { return false };
    let Some((number, _)) = &release.pending else { return false };
    let Some(Outboxed { entry, .. }) = d.entries.get(number) else { return false };
    match &entry.effect.write {
        skein_forge::api::Write::DeleteBranch { branch } => match &resource.what {
            skein_forge_client::What::Branch(found) => entry.repository == resource.repository && branch == found,
            skein_forge_client::What::Repository
            | skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Issue(_) => false,
        },
        skein_forge::api::Write::CreateIssue { .. }
        | skein_forge::api::Write::OpenPull { .. }
        | skein_forge::api::Write::Post { .. }
        | skein_forge::api::Write::Review { .. }
        | skein_forge::api::Write::Edit { .. }
        | skein_forge::api::Write::SetReviewers { .. }
        | skein_forge::api::Write::Close { .. }
        | skein_forge::api::Write::Reopen { .. }
        | skein_forge::api::Write::Merge { .. }
        | skein_forge::api::Write::Update { .. }
        | skein_forge::api::Write::Status { .. }
        | skein_forge::api::Write::Dispatch { .. }
        | skein_forge::api::Write::Rerun { .. }
        | skein_forge::api::Write::CreateBranch { .. } => false,
    }
}

fn clear_merge_reviews(
    facts: &mut Map<u64, crate::ReviewFacts>,
    heads: &mut Map<u64, ReviewHead>,
    task: u64,
    out: &mut Queue<Request>,
) {
    let changed = facts.remove(&task).is_some();
    heads.remove(&task);
    if changed {
        emit(out, Request::Marked { tasks: Box::new([crate::APPROVALS_SUBJECT, task]) });
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive durable row restoration dispatcher")]
fn restore(d: &mut Domain, env: &Env<Limits>, record: Stored) {
    let started = d.restore_started.expect("restore_begin precedes every restored row");
    if d.restore_fit.is_some() {
        return;
    }
    if let Stored::Application { until, .. } = &record
        && *until <= started
    {
        return;
    }
    if let Err(failure) = restore::row(d, &env.limits, &record) {
        d.restore_fit = Some(Box::new(failure));
        return;
    }
    if d.restore_phase != RestorePhase::Active {
        d.restore_phase = RestorePhase::Restoring;
    }
    if d.restore_phase == RestorePhase::Restoring
        && let Err(failure) = d.reconciliation.restore(&record)
    {
        d.restore_fit = Some(Box::new(failure));
        return;
    }
    match record {
        Stored::HistoryProject(row) => {
            d.history_projects.insert(row.project, row).expect("restored history project room checked");
        }
        // Demand-loaded history is never restored (landing-history.md, 4.4).
        Stored::HistoryEvent(_)
        | Stored::HistoryChange(_)
        | Stored::HistoryLanding(_)
        | Stored::HistoryLast(_)
        | Stored::LimitHold(_) => {}
        Stored::Making(mut row) => {
            row.state = match row.state {
                crate::MakingState::Making => crate::MakingState::Waiting,
                state @ (crate::MakingState::Waiting
                | crate::MakingState::Made { .. }
                | crate::MakingState::Adopting { .. }
                | crate::MakingState::Failed(_)) => state,
            };
            d.repository_makings.insert(row.key.clone(), row).expect("restored making within limits");
        }
        Stored::Forge(record) => {
            make::restore_first(d, &record);
            d.identities.insert(record.number, u32::from(record.number)).expect("restored forge credential fits");
            d.configured
                .insert(record.number, crate::forges::configured_of(&record))
                .expect("restored configured forge fits");
            d.records.insert(record.number, record).expect("restored forge fits");
        }
        Stored::Red(row) => {
            d.reds.insert((row.goal, row.key.clone()), row).expect("restored red room checked");
        }
        Stored::RedNumber { last } => d.red_number = Some(last),
        Stored::RedPass(row) => {
            d.red_passes.insert((row.change, row.key.clone()), row).expect("restored red pass room checked");
        }
        Stored::ForgeNumber { last } => {
            d.forge_number = d.forge_number.max(last);
        }
        Stored::Playground(row) => d.playground = Some(row),
        Stored::Follow(row) => {
            d.follow_rows.insert((row.project, row.follow), row).expect("restored follow within limits");
        }
        Stored::Tier(row) => {
            d.tier_ops.restored.insert((row.repository, row.tier.clone())).expect("restored tier within limits");
            d.tiers.insert((row.repository, row.tier.clone()), row).expect("restored tier within limits");
        }
        Stored::Milestone(row) => {
            d.milestones.insert(row.task, row).expect("restored milestone within project bound");
        }
        Stored::LandingLog(row) => {
            d.landing_logs.insert(row.repository, row).expect("restored log within limits");
        }
        Stored::Verification(row) => {
            d.verifications.insert((row.repository, row.commit), row).expect("restored verification within limits");
        }
        Stored::Landing(mut row) => {
            if row.landing.phase == temper_engine_domain_forge_landing::Phase::Landed
                || row.landing.phase == temper_engine_domain_forge_landing::Phase::Cancelled
            {
                row.room = None;
            }
            crate::landing::restoring(d, env, &row);
            d.landing.restore_row(row);
        }
        Stored::Receipt { key, receipt } => {
            d.landing.receipts.insert(key, receipt).expect("restored receipt within limits");
        }
        Stored::Application { token, until, outcome } => {
            if let Err(failure) = d.application.restore(token, until, outcome) {
                d.restore_fit = Some(Box::new(failure));
            }
        }
        Stored::Call { key, answer } => {
            d.named_calls.insert(key, answer).expect("restored named-call room checked");
        }
        Stored::Repository(row) => {
            d.repositories.insert(row.provider, Box::new(row)).expect("restored repository within limits");
        }
        Stored::Hold(row) => {
            d.holds.insert(row.name.clone(), row).expect("restored hold within limits");
        }
        Stored::Names { task, resources } => {
            d.names.insert(task, resources).expect("restored names within limits");
        }
        Stored::Subscription(row) => {
            d.subscriptions.insert((row.task, row.topic.clone()), row).expect("restored subscription within limits");
        }
        Stored::BranchHead(row) => {
            d.heads.insert(row.name.clone(), row).expect("restored branch head within limits");
        }
        Stored::Writes(row) => {
            d.writes.insert(row.name.clone(), row).expect("uncapped writes");
        }
        Stored::PullState(row) => {
            d.pulls.insert(row.name.clone(), row).expect("restored pull within limits");
        }
        Stored::Ci(row) => {
            d.ci.insert((row.repository, row.head), row).expect("restored CI within limits");
        }
        Stored::Landed { commit, task } => {
            d.landed.insert(commit, task).expect("restored landing within limits");
        }
        Stored::ProposedEffect(row) => {
            assert!(d.proposed_effects.insert(row.number, row).is_ok(), "restored proposed effect fits");
        }
        Stored::Entry { entry, held } => {
            if !held {
                d.entry_looks.insert(entry.number).expect("restored entries fit outbox");
            }
            d.entries
                .insert(
                    entry.number,
                    Outboxed {
                        entry,
                        owner_ended: None,
                        held,
                        handoff: if held { Handoff::Handed } else { Handoff::Restored },
                    },
                )
                .expect("restored entry within limits");
        }
        Stored::RefusedSetting(key) => {
            d.refused_settings.insert(key, ()).expect("refused setting identity fits");
        }
        Stored::Setting(row) => {
            d.setting_rows.insert(row.key, row.edit).expect("restored setting within limits");
        }
        Stored::PersonBrief(row) => {
            d.person_briefs.insert((row.party, row.project), row).expect("restored person brief room checked");
        }
        Stored::ProjectFacts(row) => {
            d.project_facts.insert(row.project, row).expect("restored project facts room checked");
        }
        Stored::Device(row) => {
            if d.devices.is_none() {
                d.devices = Some(Box::new(crate::devices::State::new(None, env.limits.devices())));
            }
            d.devices
                .as_mut()
                .expect("restoring a device retains its state")
                .rows
                .insert((row.party, row.number), row)
                .expect("restored device room checked");
        }
        Stored::LandedDraft(row) => {
            crate::drafts::restore(d, &env.limits, row);
        }
        Stored::Away(row) => {
            d.aways.insert((row.party, row.project), row).expect("restored away room checked");
        }
        Stored::Finding(row) => {
            d.finding_rows_mut(&env.limits)
                .insert((row.project, row.number), row)
                .expect("restored finding room checked");
        }
        Stored::Decided(row) => {
            d.decisions.insert((row.project, row.number), row).expect("restored decision room checked");
        }
        Stored::Looked(row) => {
            d.looked.insert((row.party, row.project), row).expect("restored looked room checked");
        }
        Stored::Dismissed { party, what } => {
            d.dismissed.insert((party, what), ()).expect("restored dismissal room checked");
        }
        Stored::Binding { address, subject } => {
            d.bindings.insert(address, subject).expect("restored binding within limits");
        }
        Stored::Flight { task, entry } => {
            // Cross-record reconciliation is the later restart stage (domain/root-restart.md, 1.1).
            d.flights.insert((task, entry), ()).expect("restored flights within entry limit");
        }
        Stored::Writer(row) => {
            let mut discarded = Queue::uncapped();
            child(
                d,
                env,
                skein_forge_client::Event::Restore { record: skein_forge_client::Stored::Writer(row) },
                &mut discarded,
            );
            assert!(discarded.is_empty(), "restoration emits no output");
            d.writers.insert(row.forge, row.author).expect("restored writer within limits");
        }
        Stored::Client(row) => {
            let mut discarded = Queue::uncapped();
            child(d, env, skein_forge_client::Event::Restore { record: row }, &mut discarded);
            assert!(discarded.is_empty(), "restoration emits no output");
        }
        Stored::Closer(row) => {
            if !row.done {
                d.change_due.insert(row.task, env.wall).expect("restored closer wake room checked");
            }
            crate::closer::restored(d, d.restore_started.unwrap_or(env.wall), &row);
            d.closing.rows.insert(row.task, row).expect("restored closer room checked");
        }
        Stored::ChangeCold(row) => {
            d.change_cold.insert(row.task, *row).expect("restored cold change room checked");
        }
        Stored::ChangeHot(row) => {
            d.merge_withheld.remove(&row.task);
            d.change_due.insert(row.task, env.wall).expect("restored change wake room checked");
            d.change_hot.insert(row.task, *row).expect("restored hot change room checked");
        }
        Stored::Change(mut row) => {
            d.merge_withheld.remove(&row.task);
            if row.frozen() {
                row.room = None;
            }
            let (cold, _) = row.clone().split();
            d.change_cold.insert(row.task, cold).expect("restored change room checked");
            // Volatile waits are rebuilt by stepping every restored change (domain/forge/effects.md, 2).
            d.change_due.insert(row.task, env.wall).expect("a wake per restored change");
            d.changes.insert(row.task, *row).expect("restored change within limits");
        }
        Stored::Issue(row) => {
            if row.pending.is_none() && !row.failed && !row.state.closed && row.desired.is_some() {
                // A committed desired view may still await its body interval,
                // milestone, or closing write when the transient timer is lost.
                d.projection_due.insert(row.goal, env.wall).expect("projection timers are bounded by issues");
            }
            d.issues.insert(row.goal, row).expect("restored issue within limits");
        }
        Stored::Release(row) => {
            let task = row.task;
            d.releases.insert(task, row).expect("restored release within limits");
            release_reads::advanced(d, task, d.restore_started.unwrap_or(env.wall));
        }
    }
}

fn restored(d: &mut Domain, env: &Env<Limits>, clock: skein_forge_client::RecoveryClock, out: &mut Queue<Request>) {
    if d.unjoined_change().is_some() {
        return;
    }
    admission::restore_names(d, out);
    if d.fit_restored(&env.limits).is_err() {
        return;
    }
    for (_, subscriber) in &d.subscriptions {
        match &subscriber.topic {
            Topic::Ci { .. } if subscriber.goal_tasks.is_some() => {
                emit(out, Request::GoalUntopic { goal: subscriber.task, topic: subscriber.topic.clone() });
            }
            Topic::Ci { .. }
            | Topic::Landings { .. }
            | Topic::Pull { .. }
            | Topic::Participation { .. }
            | Topic::Findings { .. } => {}
        }
    }
    d.rebuild_settings();
    d.restore_landing_settings();
    crate::landing_log::restored(d, out);
    d.restore_phase = RestorePhase::Restoring;
    d.restart_reads_started = false;
    d.fresh_repositories = Set::with_capacity(d.repositories.capacity());
    d.restart_settling = true;
    child(d, env, skein_forge_client::Event::Restored { clock }, out);
    retire_ended(d, env, out);
    writes::kept::watches(d, env);
    for (_, record) in &d.records {
        if !d.forge_ended(record.number) {
            emit(out, Request::ForgeKept(record.clone()));
        }
    }
    crate::removal::restart(d, out);
    crate::removal::restart_releases(d, env, out);
    emit(out, Request::RestartDone { stage: RestartStage::Restored });
}

fn held_landing(d: &Domain, entry: &skein_forge_client::Entry) -> bool {
    let mut landing = d.landing.rows.contains_key(&entry.task);
    if let skein_forge::api::Write::Merge { .. } = entry.effect.write {
        landing = true;
    }
    d.landing.held && landing
}

fn settle_outbox(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    // A held entry waits for a person, out of the client's outbox.
    let mut entries = List::with_capacity(env.limits.entries);
    for (_, row) in &d.entries {
        if held_landing(d, &row.entry) {
            continue;
        }
        if !row.held && row.handoff == Handoff::Restored {
            entries.push(row.entry.clone()).expect("entry list capacity");
        }
    }
    for entry in &entries {
        // The restored set precedes entries kept by startup reconciliation.
        d.entries.get_mut(&entry.number).expect("restored entry remains").handoff = Handoff::Handed;
        child(d, env, skein_forge_client::Event::Make { entry: entry.clone() }, out);
    }
    child(d, env, skein_forge_client::Event::SettleOutbox, out);
}

fn read_afresh(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    crate::landing::rebuild(d);
    d.restore_phase = RestorePhase::Active;
    crate::playground::restart(d, out);
    d.restart_fresh = Some(false);
    let mut lost_runs = List::with_capacity(d.holds.len());
    for (_, hold) in &d.holds {
        if let Some(Writer::Lost { task, attempt }) = hold.writer
            && !lost_runs.as_slice().contains(&(task, attempt))
        {
            lost_runs.push((task, attempt)).expect("runs bounded by holds");
        }
    }
    let mut retired = List::with_capacity(d.lost.len());
    for (owner, pending) in &d.lost {
        if pending.state == LostReadState::Restoring {
            retired.push(*owner).expect("retired reads bounded by holds");
        }
    }
    for owner in retired.into_boxed() {
        ask_lost_again(d, env, owner, out);
    }
    for (task, attempt) in &lost_runs {
        lost(d, env, *task, *attempt, out);
    }
    let mut ci = List::with_capacity(env.limits.subscriptions);
    for (_, subscriber) in &d.subscriptions {
        match &subscriber.topic {
            Topic::Ci { repository, head } if !ci.as_slice().contains(&(*repository, *head)) => {
                ci.push((*repository, *head)).expect("subscription capacity");
            }
            Topic::Ci { .. }
            | Topic::Landings { .. }
            | Topic::Pull { .. }
            | Topic::Participation { .. }
            | Topic::Findings { .. } => {}
        }
    }
    let mut changes = List::with_capacity(env.limits.changes);
    for (task, row) in &d.changes {
        changes.push(*task).expect("change capacity");
        if let change::State::Landing { .. } = row.change.state {
            d.fresh_waits.insert(*task, Box::new([row.repository])).expect("restored landing judgement waiter");
        }
        if !row.questions.waiting.is_empty() || !row.questions.replies.is_empty() {
            emit(out, Request::WakeProcedure { task: *task });
        }
    }
    for (task, row) in &d.landing.rows {
        for entry in &row.entries {
            if entry.operation == crate::Operation::Merge {
                let repositories = crate::landing::repositories(row, env.limits.repositories);
                d.fresh_waits.insert(*task, repositories).expect("restored landing judgement waiter");
            }
        }
    }
    for task in &changes {
        topics::refresh(d, env, *task, out);
    }
    for (repository, head) in &ci {
        ci_refresh(d, env, *repository, *head, out);
    }
    crate::landing::refresh(d, env, out);
    goal_branch::restart(d, env, d.restore_started.unwrap_or(env.wall), out);
    crate::tiers::restart(d, env, out);
    d.restart_reads_started = true;
    child(d, env, skein_forge_client::Event::ReadAfresh, out);
}

// A permanently retired forge has no future observation to await. Its
// kept task names retain repository rows; live watches and release gates retire.
/// Drop the forge's local reads now; the event's environment retires its watches before returning.
pub(crate) fn retire_forge(d: &mut Domain, forge: u16) {
    let mut repositories = List::with_capacity(d.repositories.len());
    for (repository, _) in &d.repositories {
        if repository.forge == forge {
            repositories.push(*repository).expect("bounded repositories");
        }
    }
    for repository in repositories.into_boxed() {
        release::retire(d, repository);
    }
    connection_wait::retire(d, forge);
    outage::retire(d, forge);
    goal_branch::retire(d, forge);
    d.retire_pending = true;
}

fn retire_ended(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut ended = List::with_capacity(d.configured.capacity());
    for (forge, _) in &d.configured {
        if d.forge_ended(*forge) {
            ended.push(*forge).expect("configured forge bound");
        }
    }
    if ended.is_empty() {
        return;
    }
    for forge in ended.into_boxed() {
        retire_forge(d, forge);
        let mut entries = List::with_capacity(d.entries.capacity());
        for (number, row) in &d.entries {
            if row.entry.repository.forge == forge {
                entries.push(*number).expect("bounded outbox entries");
            }
        }
        for number in entries.into_boxed() {
            let task = d.entries.get(&number).expect("collected entry").entry.task;
            d.named_outcome(number, skein_forge_client::Outcome::Withdrawn);
            writes::outbox::outcome(d, task, number, skein_forge_client::Outcome::Withdrawn, out);
            outcome_facts(d, env, number, task, skein_forge_client::Outcome::Withdrawn, out);
            not_made(d, env, task, number, out);
            child(d, env, skein_forge_client::Event::Withdraw { entry: number }, out);
            if let Some(row) = d.releases.get(&task)
                && !row.failed
            {
                continue_release_once(out, task);
            }
        }
        let mut releases = List::with_capacity(d.releases.capacity());
        for (task, row) in &d.releases {
            let mut removed = match d.changes.get(task) {
                Some(change) => change.repository.forge == forge,
                None => false,
            };
            for (name, hold) in &d.holds {
                removed |= hold.task == *task && name.forge == forge;
            }
            if row.room.is_some() && removed {
                releases.push(*task).expect("bounded release rows");
            }
        }
        for task in releases.into_boxed() {
            leave_room_release(d, task, out);
            continue_release_once(out, task);
        }
        let mut changes = List::with_capacity(d.changes.capacity());
        for (task, row) in &d.changes {
            if row.repository.forge == forge && !row.terminal() {
                changes.push(*task).expect("bounded changes");
            }
        }
        for task in changes.into_boxed() {
            end_change(d, env, task, out);
            d.change_due.remove(&task);
            d.read_sessions.remove(&task);
            emit(
                out,
                Request::ChangeDecision {
                    task,
                    decision: change::Decision::Cancel,
                    entry: None,
                    evidence: None,
                    close_at: None,
                },
            );
        }
    }
    d.retire_pending = false;
    let mut watches = Map::with_capacity(env.limits.client.resources);
    for (_, names) in &d.names {
        for name in names {
            if d.forge_ended(name.forge) || goal_run(d, name) {
                continue;
            }
            let Some(resource) = to_client(name, &env.limits) else { continue };
            if watches.contains_key(&resource) {
                continue;
            }
            let participating = participating(d, &resource);
            watches
                .insert(resource.clone(), skein_forge_client::Watch { resource, participating })
                .expect("existing bounded watch set");
        }
    }
    assert!(crate::tiers::add_watches(d, env, &mut watches), "tier watch capacity");
    let mut list = List::with_capacity(env.limits.client.resources);
    for (_, watch) in &watches {
        list.push(watch.clone()).expect("bounded watches");
    }
    child(d, env, skein_forge_client::Event::Keep { owner: Token::new(u64::MAX), watches: list.into_boxed() }, out);
}

/// One pass over the forge's repositories (temper/docs/design/domain/forge/adoption-connection.md, 1).
pub(crate) fn erase_removed_repositories(
    d: &mut Domain,
    env: &Env<Limits>,
    forge: u16,
    out: &mut Queue<Request>,
) -> bool {
    let mut erased = List::with_capacity(d.repositories.capacity());
    let mut kept = false;
    for (repository, _) in &d.repositories {
        if repository.forge != forge {
            continue;
        }
        let mut named = false;
        for (_, names) in d.names.iter().chain(d.kept.iter()) {
            for name in names {
                named |= name.forge == forge && name.repository == repository.repository;
            }
        }
        for (name, _) in &d.holds {
            named |= name.forge == forge && name.repository == repository.repository;
        }
        for ((_, topic), _) in &d.subscriptions {
            let provider = match topic {
                Topic::Findings { .. } => continue,
                Topic::Landings { repository, .. }
                | Topic::Ci { repository, .. }
                | Topic::Pull { repository, .. }
                | Topic::Participation { repository, .. } => repository,
            };
            named |= provider == repository;
        }
        if named {
            kept = true;
        } else {
            erased.push(*repository).expect("bounded repositories");
        }
    }
    for repository in erased.into_boxed() {
        release::repository(d, env, repository, out);
    }
    !kept
}

// A making's write belongs to its line and lane; reading afresh ends on reads
// alone (temper/docs/design/domain/forge/branches.md, 1).
fn read_afresh_complete(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.restart_fresh == Some(true)
        && d.landing.pending.is_empty()
        && d.pending_ci.is_empty()
        && d.goal_file_reads.is_empty()
        && d.lost.is_empty()
        && !goal_branch::reading_afresh(d)
    {
        d.restart_fresh = None;
        writes::readback::restart(d, out);
        emit(out, Request::RestartDone { stage: RestartStage::ReadAfresh });
        read_session::fresh(d, env);
    }
    if d.restart_reads_started {
        repository_reads::wake(d, env, out);
        if fresh_reads_in(d) {
            d.fresh_branches = Set::with_capacity(d.fresh_branches.capacity());
            d.restart_reads_started = false;
        }
    }
}

/// One repository's restart reads are in (temper/docs/design/domain/forge/catching-up.md, 2).
pub(crate) fn repository_fresh(d: &Domain, repository: skein_forge::api::Repository) -> bool {
    if fresh_reads_in(d) {
        return true;
    }
    if d.restore_phase == RestorePhase::Restoring || !d.restart_reads_started {
        return false;
    }
    if d.client.fresh_owed_in(repository) != 0 || d.landing.reading_in(repository) {
        return false;
    }
    for (_, pending) in &d.pending_ci {
        if pending.repository == repository {
            return false;
        }
    }
    for (_, pending) in &d.goal_file_reads {
        if pending.repository == repository {
            return false;
        }
    }
    for (_, pending) in &d.lost {
        if repository_reads::repository(&pending.name) == repository {
            return false;
        }
    }
    for (_, pending) in &d.makings {
        if goal_branch::read_out(pending) && repository_reads::repository(&pending.name) == repository {
            return false;
        }
    }
    true
}

/// The restart's fresh reads are in (temper/docs/design/domain/forge/catching-up.md, 2).
pub(crate) fn fresh_reads_in(d: &Domain) -> bool {
    d.restore_phase != RestorePhase::Restoring && d.restart_fresh.is_none()
}

impl Domain {
    /// Whether decisions on this repository's restored facts may proceed.
    #[must_use]
    pub fn repository_fresh(&self, repository: skein_forge::api::Repository) -> bool {
        repository_fresh(self, repository)
    }

    /// Whether decisions on restored forge facts may proceed.
    #[must_use]
    pub fn fresh_reads_in(&self) -> bool {
        fresh_reads_in(self)
    }
}

fn continue_release_once(out: &mut Queue<Request>, task: u64) {
    for request in out.iter() {
        if let Request::ContinueRelease { task: prior } = request
            && *prior == task
        {
            return;
        }
    }
    emit(out, Request::ContinueRelease { task });
}

fn wake_once(out: &mut Queue<Request>, task: u64) {
    for request in out.iter() {
        if let Request::WakeProcedure { task: prior } = request
            && *prior == task
        {
            return;
        }
    }
    emit(out, Request::WakeProcedure { task });
}

pub(crate) fn wait_fresh(d: &mut Domain, task: u64, repositories: &[skein_forge::api::Repository]) -> bool {
    let mut waiting = List::with_capacity(d.repositories.capacity());
    let mut forge_wide = repositories.is_empty();
    if let Some(prior) = d.fresh_waits.get(&task) {
        forge_wide |= prior.is_empty();
        for repository in prior {
            if !waiting.as_slice().contains(repository) {
                waiting.push(*repository).expect("wait repositories are adopted");
            }
        }
    }
    if forge_wide {
        waiting = List::with_capacity(d.repositories.capacity());
    } else {
        for repository in repositories {
            if !waiting.as_slice().contains(repository) {
                waiting.push(*repository).expect("wait repositories are adopted");
            }
        }
    }
    if repository_reads::all_fresh(d, waiting.as_slice()) {
        d.fresh_waits.remove(&task);
        return false;
    }
    d.fresh_waits.insert(task, waiting.into_boxed()).expect("one restart waiter per procedure");
    true
}

pub(crate) fn entries_looked(d: &Domain, task: u64) -> bool {
    for number in &d.entry_looks {
        if let Some(row) = d.entries.get(number)
            && row.entry.task == task
        {
            return false;
        }
    }
    true
}

pub(crate) fn branch_read(d: &Domain, name: &Name) -> bool {
    repository_fresh(d, repository_reads::repository(name)) || d.fresh_branches.contains(name)
}

/// Whether an earlier answer already woke this branch's followers. The client's
/// last read can complete the repository before its observation is routed here.
pub(crate) fn branch_observed(d: &Domain, name: &Name) -> bool {
    fresh_reads_in(d)
        || d.fresh_repositories.contains(&repository_reads::repository(name))
        || d.fresh_branches.contains(name)
}

fn settled_projection(d: &Domain, goal: u64, out: &mut Queue<Request>) {
    let settled = match d.issues.get(&goal) {
        Some(row) => {
            (row.state.closed || row.failed)
                && match &row.desired {
                    Some(view) => view.finished.is_some(),
                    None => false,
                }
        }
        None => false,
    };
    if settled {
        emit(out, Request::ProjectionSettled { goal });
    }
}

/// Capture terminal CI in the save already ending the change; a later save
/// without a fresh read keeps the observation restored with the row.
fn retain_terminal_ci(d: &mut Domain, row: &mut ChangeRow) {
    if !row.terminal() {
        return;
    }
    d.review_requests.remove(&row.task);
    d.review_facts.remove(&row.task);
    d.review_heads.remove(&row.task);
    let read = match d.panel_ci.get(&row.task) {
        Some(read) if Some(read.head) == row.change.last_head => Some(*read),
        Some(_) | None => row.ci,
    };
    row.ci = match read {
        Some(read) if Some(read.head) == row.change.last_head => Some(read),
        Some(_) | None => None,
    };
    d.changes.get_mut(&row.task).expect("saved change is retained").ci = row.ci;
}

/// Remember the cold part last put: callers may already have replaced the in-memory row.
pub(crate) fn save_change_parts(d: &mut Domain, out: &mut Queue<Request>, mut row: ChangeRow) {
    if row.settling && crate::settling::settling_head(d.landing_of(row.task), &row).is_none() {
        row.settling = false;
        d.changes.get_mut(&row.task).expect("saved change").settling = false;
        crate::settling::mark_train_waits(d, &row);
    }
    crate::waits::changed(d, row.task);
    if row.frozen() {
        d.merge_withheld.remove(&row.task);
        row.regate = None;
        row.delegate_words = None;
        let current = d.changes.get_mut(&row.task).expect("saved change is retained");
        current.regate = None;
        current.delegate_words = None;
    }
    if let Some(recheck) = row.ci_recheck
        && (row.frozen() || row.change.last_head != Some(recheck.job.head))
    {
        row.ci_recheck = None;
        d.changes.get_mut(&row.task).expect("saved change is retained").ci_recheck = None;
    }
    if row.behind_wait.is_some() && (row.frozen() || matches_held(&row.change.state) && row.change.released.is_none()) {
        row.behind_wait = None;
        d.changes.get_mut(&row.task).expect("saved change").behind_wait = None;
    }
    let (cold, hot) = row.split();
    if d.change_cold.get(&cold.task) != Some(&cold) {
        d.change_cold.insert(cold.task, cold.clone()).expect("one cold part per change");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::ChangeCold(Box::new(cold)) });
    }
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::ChangeHot(Box::new(hot)) });
}

/// Every saved record invalidates its own panel (domain/forge/change/panel.md, 1).
pub(crate) fn save_change(d: &mut Domain, out: &mut Queue<Request>, mut row: ChangeRow) {
    if row.frozen() {
        crate::room::leave(d, crate::room::Waiter::Change(row.task));
        row.room = None;
        d.changes.get_mut(&row.task).expect("saved change").room = None;
        connection_wait::take(d, row.task);
    }
    retain_terminal_ci(d, &mut row);
    let task = row.task;
    let mut tasks = List::with_capacity(crate::MARKED);
    tasks.push(task).expect("own subject");
    if let Some(landing) = d.landing_of(row.task) {
        tasks.push(landing).expect("member landing subject");
    }
    // A retained ready time keeps the queue and status panels fresh after the turn ends.
    if (row.change.owns_turn || row.change.ready_since.is_some() || row.base_repair)
        && let Some(subject) = crate::views::changes_mark(d, row.repository)
    {
        for subject in subject {
            tasks.push(subject).expect("project changes and status subjects");
        }
    }
    let tasks = tasks.into_boxed();
    save_change_parts(d, out, row);
    emit(out, Request::Marked { tasks });
}

/// Include the nearest affected followers when a saved row changes its queue order.
fn save_queue_change(
    d: &mut Domain,
    env: &Env<Limits>,
    before: Option<change::Ready>,
    mut row: ChangeRow,
    out: &mut Queue<Request>,
) {
    crate::readiness::reconcile(d, env, &mut row, None, out);
    d.changes.insert(row.task, row.clone()).expect("existing change");
    save_queue_panel(d, env, before, row, out);
}

fn save_queue_panel(
    d: &mut Domain,
    env: &Env<Limits>,
    before: Option<change::Ready>,
    mut row: ChangeRow,
    out: &mut Queue<Request>,
) {
    if row.terminal() {
        connection_wait::take(d, row.task);
    }
    let after = crate::panel::ready(&row);
    if before == after {
        save_change(d, out, row);
        return;
    }
    let (ready, repair) = queue_ready(d, &row, &env.limits);
    let old_repair = if row.base_repair && before.is_some() { Some(row.task) } else { repair };
    let mut followers = List::with_capacity(env.limits.changes);
    for candidate in &ready {
        if candidate.task == row.task {
            continue;
        }
        let was_behind = match before {
            Some(old) => crate::panel::queue_before(&old, candidate, old_repair, &env.limits, env.wall),
            None => false,
        };
        let now_behind = match after {
            Some(new) => crate::panel::queue_before(&new, candidate, repair, &env.limits, env.wall),
            None => false,
        };
        if was_behind || now_behind {
            followers.push(*candidate).expect("queue bounds followers");
            d.wait_reports.changed(candidate.task);
        }
    }
    let mut tasks = List::with_capacity(crate::MARKED);
    tasks.push(row.task).expect("own subject");
    if let Some(landing) = d.landing_of(row.task) {
        tasks.push(landing).expect("member landing subject");
    }
    if let Some(subject) = crate::views::changes_mark(d, row.repository) {
        for subject in subject {
            tasks.push(subject).expect("project changes and status subjects");
        }
    }
    let mut previous = None;
    while tasks.room() > 0 {
        let mut next = None;
        for candidate in &followers {
            if let Some(previous) = previous
                && !crate::panel::queue_before(&previous, candidate, repair, &env.limits, env.wall)
            {
                continue;
            }
            next = match next {
                Some(current) if crate::panel::queue_before(&current, candidate, repair, &env.limits, env.wall) => {
                    Some(current)
                }
                Some(_) | None => Some(*candidate),
            };
        }
        let Some(next) = next else { break };
        tasks.push(next.task).expect("nearest follower room");
        previous = Some(next);
    }
    retain_terminal_ci(d, &mut row);
    save_change_parts(d, out, row);
    emit(out, Request::Marked { tasks: tasks.into_boxed() });
}

/// A read at a different head or with a different verdict changes the whole panel.
pub(crate) fn keep_panel_ci(d: &mut Domain, task: u64, read: change::Ci, out: &mut Queue<Request>) {
    assert!(d.changes.contains_key(&task), "CI belongs to a held change");
    if d.panel_ci.get(&task) != Some(&read) {
        d.panel_ci.insert(task, read).expect("one CI verdict per held change");
        emit(out, Request::Marked { tasks: Box::new([task]) });
    }
}

// Admission mirrors skein's Create grammar without an io dependency in this domain.
// The tests pin both grammars to the same boundary cases (protocol/setup/owned-repositories.md, 1).
fn setup_login(name: &[u8]) -> bool {
    temper_client_protocol_wire::source::org(name)
}
fn setup_repository(name: &[u8]) -> bool {
    temper_client_protocol_wire::source::repository(name)
}

// One rule for a producer's failure and the box pipe closing (domain/forge/adoption-connection.md, 1).
fn failed_at(held: &crate::Connection, failed: Option<crate::FailedStep>) -> Option<crate::FailedStep> {
    match held {
        crate::Connection::SettingUp { step: Some(crate::SetUpStep::Fetch) } => Some(crate::FailedStep::Fetch),
        crate::Connection::SettingUp { step: Some(crate::SetUpStep::Prepare) } => Some(crate::FailedStep::Prepare),
        crate::Connection::SettingUp { step: Some(crate::SetUpStep::Run | crate::SetUpStep::Source) } => {
            Some(crate::FailedStep::Run)
        }
        crate::Connection::Connecting => Some(crate::FailedStep::Connect),
        crate::Connection::SetUpFailed { .. } => failed,
        crate::Connection::NotConnected
        | crate::Connection::NotSetUp
        | crate::Connection::SettingUp { step: None }
        | crate::Connection::Connected { .. }
        | crate::Connection::Stopped { .. } => None,
    }
}

impl Domain {
    /// Find a referencing row only on an unmade link (temper/docs/design/domain/root-restart.md, 1.1).
    #[expect(clippy::too_many_lines, reason = "one exhaustive failure-path scan of the referencing family")]
    pub(super) fn restored_link_key(
        &self,
        kind: RecordKind,
        need: crate::reconcile::Need,
        client_repository: bool,
    ) -> Key {
        use crate::reconcile::Need;
        match kind {
            RecordKind::Call => {
                for (key, _) in &self.named_calls {
                    if need.task(key.task, None) {
                        return Key::Call(*key);
                    }
                }
            }
            RecordKind::ProposedEffect => {
                for (key, row) in &self.proposed_effects {
                    if need.entry(&row.entry) {
                        return Key::ProposedEffect(*key);
                    }
                }
            }
            RecordKind::Repository => {
                for (key, _) in &self.repositories {
                    if need == Need::Forge(key.forge) {
                        return Key::Repository(*key);
                    }
                }
            }
            RecordKind::Hold => {
                for (key, row) in &self.holds {
                    for writer in &row.design_writers {
                        if need.task(writer.task, Some(key.forge)) {
                            return Key::Hold(key.clone());
                        }
                    }
                    if need.name(key)
                        || need.task(row.task, Some(key.forge))
                        || match row.writer {
                            Some(Writer::Run { task, .. } | Writer::Lost { task, .. }) => {
                                need.task(task, Some(key.forge))
                            }
                            Some(Writer::Entry(_)) | None => false,
                        }
                    {
                        return Key::Hold(key.clone());
                    }
                }
            }
            RecordKind::Names => {
                for (key, resources) in self.names.iter().chain(self.kept.iter()) {
                    for name in resources {
                        if need.name(name) {
                            return Key::Names(*key);
                        }
                    }
                }
            }
            RecordKind::Subscription => {
                for (key, _) in &self.subscriptions {
                    let repository = match &key.1 {
                        Topic::Findings { .. } => {
                            if need.task(key.0, None) {
                                return Key::Subscription { task: key.0, topic: key.1.clone() };
                            }
                            continue;
                        }
                        Topic::Landings { repository, .. }
                        | Topic::Ci { repository, .. }
                        | Topic::Pull { repository, .. }
                        | Topic::Participation { repository, .. } => *repository,
                    };
                    if need.repository(repository) || need.task(key.0, Some(repository.forge)) {
                        return Key::Subscription { task: key.0, topic: key.1.clone() };
                    }
                }
            }
            RecordKind::BranchHead => {
                for (key, _) in &self.heads {
                    if need.name(key) {
                        return Key::BranchHead(key.clone());
                    }
                }
            }
            RecordKind::PullState => {
                for (key, _) in &self.pulls {
                    if need.name(key) {
                        return Key::PullState(key.clone());
                    }
                }
            }
            RecordKind::Ci => {
                for (key, _) in &self.ci {
                    if need.repository(key.0) {
                        return Key::Ci { repository: key.0, head: key.1 };
                    }
                }
            }
            RecordKind::Entry => {
                for (key, row) in &self.entries {
                    if need.repository(row.entry.repository)
                        || need == (Need::EntryOwner { task: row.entry.task, entry: *key })
                    {
                        return Key::Entry(*key);
                    }
                }
            }
            RecordKind::Closer => {
                for (task, row) in &self.closing.rows {
                    if need.task(*task, None) || need.task(row.goal, None) {
                        return Key::Closer(*task);
                    }
                }
            }
            RecordKind::Change => {
                for (key, row) in &self.changes {
                    if need.repository(row.repository) {
                        return Key::Change(*key);
                    }
                }
            }
            RecordKind::Issue => {
                for (key, row) in &self.issues {
                    if need.repository(row.repository) || need.task(*key, Some(row.repository.forge)) {
                        return Key::Issue(*key);
                    }
                }
            }
            RecordKind::Release => {
                for (key, row) in &self.releases {
                    if need.task(row.task, None)
                        || need.task(row.root, None)
                        || match &row.pending {
                            Some((_, Some(name))) => need.name(name),
                            Some((_, None)) | None => false,
                        }
                    {
                        return Key::Release(*key);
                    }
                }
            }
            RecordKind::Application => {
                for (key, (_, outcome)) in &self.application.kept {
                    if match outcome {
                        crate::ApplicationOutcome::Adopted(row) => need.repository(row.repository),
                        crate::ApplicationOutcome::CoreRefused(_)
                        | crate::ApplicationOutcome::ForgeRefused(_)
                        | crate::ApplicationOutcome::MakeKept
                        | crate::ApplicationOutcome::Follow(_)
                        | crate::ApplicationOutcome::Opened { .. }
                        | crate::ApplicationOutcome::OpenRefused(_)
                        | crate::ApplicationOutcome::MakeRefused(_) => false,
                    } {
                        return Key::Application(key.raw());
                    }
                }
            }
            RecordKind::Landing => {
                for (key, row) in &self.landing.rows {
                    for member in &row.landing.members {
                        if need.repository(skein_forge::api::Repository {
                            forge: member.repository.forge,
                            repository: member.repository.repository,
                        }) {
                            return Key::Landing(*key);
                        }
                    }
                }
            }
            RecordKind::Follow => {
                for ((project, follow), row) in &self.follow_rows {
                    if need.repository(row.source)
                        || match row.task {
                            Some(task) => need.task(task, Some(row.source.forge)),
                            None => false,
                        }
                    {
                        return Key::Follow { project: *project, follow: *follow };
                    }
                }
            }
            RecordKind::Tier => {
                for ((repository, tier), row) in &self.tiers {
                    if need.repository(*repository)
                        || match row.task {
                            Some(task) => need.task(task, Some(repository.forge)),
                            None => false,
                        }
                    {
                        return Key::Tier { repository: *repository, tier: tier.clone() };
                    }
                }
            }
            RecordKind::LandingLog => {
                for (repository, _) in &self.landing_logs {
                    if need.repository(*repository) {
                        return Key::LandingLog { repository: *repository };
                    }
                }
            }
            RecordKind::Verification => {
                for ((repository, commit), _) in &self.verifications {
                    if need.repository(*repository) {
                        return Key::Verification { repository: *repository, commit: *commit };
                    }
                }
            }
            RecordKind::Receipt => {
                for (key, _) in &self.landing.receipts {
                    if need.repository(key.repository) {
                        return Key::Receipt(key.clone());
                    }
                }
            }
            RecordKind::Flight => {
                for ((task, entry), ()) in &self.flights {
                    if need == (Need::Entry { task: *task, entry: *entry }) {
                        return Key::Flight { task: *task, entry: *entry };
                    }
                }
            }
            RecordKind::Writer => {
                for (forge, _) in &self.writers {
                    if need == Need::Forge(*forge) {
                        return Key::Writer(*forge);
                    }
                }
            }
            RecordKind::Client => match need {
                Need::Forge(forge) => return Key::Writer(forge),
                Need::Repository(repository) if client_repository => {
                    return Key::Client(skein_forge_client::Key::Repository(repository));
                }
                Need::Repository(repository) => {
                    for key in &self.client.live_keys() {
                        if key.repository == repository && self.client.cached(key).is_some() {
                            return Key::Client(skein_forge_client::Key::Live(key.clone()));
                        }
                    }
                }
                Need::Task { .. } | Need::EntryOwner { .. } | Need::Entry { .. } => {
                    unreachable!("client forge or repository link")
                }
            },
            RecordKind::Red => {
                for (_, row) in &self.reds {
                    if need == (Need::Task { task: row.goal, retired: None }) {
                        return Key::Red { goal: row.goal, key: row.key.clone() };
                    }
                }
            }
            RecordKind::RedPass => {
                for (_, row) in &self.red_passes {
                    if need == (Need::Task { task: row.change, retired: None }) {
                        return Key::RedPass { change: row.change, key: row.key.clone() };
                    }
                }
            }
            RecordKind::RedNumber
            | RecordKind::Forge
            | RecordKind::Landed
            | RecordKind::Binding
            | RecordKind::Setting
            | RecordKind::Making
            | RecordKind::Milestone
            | RecordKind::HistoryProject
            | RecordKind::PersonBrief
            | RecordKind::ProjectFacts
            | RecordKind::Device
            | RecordKind::LandedDraft
            | RecordKind::Away
            | RecordKind::Decided
            | RecordKind::Finding
            | RecordKind::Looked
            | RecordKind::Dismissed => {
                unreachable!("row has no links")
            }
        }
        unreachable!("unmade link has a referencing retained row")
    }
}

#[cfg(test)]
#[path = "tests/queue_turn.rs"]
mod queue_turn_tests;

#[cfg(test)]
#[path = "tests/verification.rs"]
mod verification_tests;

fn bind(d: &mut Domain, request: Token, mut address: Box<[u8]>, subject: Box<[u8]>, out: &mut Queue<Request>) {
    address.make_ascii_lowercase();
    assert!(crate::binding_address(&address) && subject.len() <= 255, "admitted binding fields");
    let answer = if let Some(bound) = d.bindings.get(&address) {
        if *bound == subject { crate::BindAnswer::Bound } else { crate::BindAnswer::Another }
    } else if d.bindings.insert(address.clone(), subject.clone()).is_ok() {
        out.push(Request::Save { record: Stored::Binding { address, subject }, read_afresh: None, release: None });
        crate::BindAnswer::Bound
    } else {
        crate::BindAnswer::Full
    };
    out.push(Request::Bound { request, answer });
}

fn read_merge_hooks(
    d: &mut Domain,
    env: &Env<Limits>,
    mut pending: PendingStep,
    repository: skein_forge::api::Repository,
    head: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    pending.stage = StepStage::MergeHooks(head);
    d.sequence = d.sequence.checked_add(1).expect("read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 62));
    arm_gather(d, env, &pending, out);
    d.steps.insert(owner, pending).expect("one gather per change");
    gather_read(d, env, owner, repository, crate::hooks::read(head, env.limits.client.answer_bytes), out);
}

impl Domain {
    /// The selected command at the merge's pinned head, if discovery has finished.
    #[must_use]
    pub fn merge_command(
        &self,
        task: u64,
        from: Option<skein_forge::api::Commit>,
    ) -> Option<(skein_forge::api::Commit, &[u8])> {
        let &(head, hooks) = self.merge_hooks.get(&task)?;
        if let Some(from) = from
            && from != head
        {
            return None;
        }
        let row = self.changes.get(&task)?;
        let repository = self.repositories.get(&row.repository)?;
        let legacy = match hooks {
            crate::Hooks::Legacy => self.merge_job(repository.project),
            crate::Hooks::Directory { .. } => None,
        };
        Some((head, hooks.merge(legacy)?))
    }
}

fn catch_up_command_failure(d: &Domain, row: &ChangeRow, head: skein_forge::api::Commit) -> Box<[u8]> {
    if let Some(&(at, hooks)) = d.merge_hooks.get(&row.task)
        && at == head
        && d.merge_command(row.task, Some(head)).is_none()
    {
        return hooks.missing_merge(head);
    }
    if d.merge_kind.is_none() {
        Box::from(b"change's merge: job kind 3 is not declared.".as_slice())
    } else {
        Box::from(b"change's merge: the job cannot be described within its bounds or authority.".as_slice())
    }
}

// The worker wire uses two four-byte lengths; the request is compact JSON.
// temper/docs/design/domain/worker/jobs.md; temper/docs/design/domain/forge/change/repairs.md.
fn merge_available(d: &Domain, row: &ChangeRow, pending: &PendingStep, base_tip: [u8; 32]) -> change::MergeJob {
    if !pending.merge_job || (d.merge_kind.is_none() && row.catch_up.is_none()) {
        return change::MergeJob::Disabled;
    }
    let Some(&(head, _)) = d.merge_hooks.get(&row.task) else { return change::MergeJob::Discover };
    let available = merge_spec_fits(d, row, head, row.catch_up.unwrap_or(base_tip));
    change::MergeJob::Checked { head, available }
}

fn merge_spec_fits(d: &Domain, row: &ChangeRow, head: [u8; 32], base: [u8; 32]) -> bool {
    let Some((bound, _)) = d.merge_kind else { return false };
    let Some(repository) = d.repositories.get(&row.repository) else { return false };
    let Some((_, command)) = d.merge_command(row.task, Some(head)) else { return false };
    let mut length = 0_usize;
    for part in [
        repository.name.len(),
        1,
        command.len(),
        8,
        b"{\"version\":1,\"head\":\"\",\"base\":\"\"}".len(),
        crate::commit_text(&head).len(),
        crate::commit_text(&base).len(),
    ] {
        let Some(sum) = length.checked_add(part) else { return false };
        length = sum;
    }
    length <= skein_lib::usize_of(bound)
}

fn empty_rules(limits: &Limits) -> crate::settings::HeldRules {
    crate::settings::HeldRules {
        deployment: Box::new([]),
        every_project: Box::new([]),
        projects: Map::with_capacity(limits.judge_projects),
    }
}
