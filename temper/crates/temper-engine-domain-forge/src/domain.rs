//! Top-level forge routing and commit fence (jig's domain/connectors.md,
//! sections 2–5). Durable rows are held here and mirrored by root store
//! writes.
//! A requester's confirmed branch moves become its change's new head
//! (temper/docs/design/domain/forge/change/README.md, 3).
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Map, Queue, Set, Token};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;

use crate::{
    BranchHead, ChangeRow, CiState, Class, Event, Hold, IssueRow, Key, Kinds, Limits, Name, News, PullState,
    RecordKind, ReleaseEnding, ReleaseRow, Repository, Request, RestartStage, Role, Stored, Subscriber, Topic, What,
    Writer,
};
use crate::{adoption, brief, held, topics};

/// The title for an empty explicit pull title (domain/forge/change/README.md, 1).
pub const DEFAULT_CHANGE_TITLE: &[u8] = b"Update repository";

mod admission;
mod ci_news;
mod connection_wait;
#[path = "goal_branch.rs"]
mod goal_branch;
pub(crate) mod heads;
mod hints;
mod landed;
#[path = "make.rs"]
pub(crate) mod make;
#[path = "outage.rs"]
mod outage;
#[path = "restore.rs"]
mod restore;
mod room;
pub use restore::{RestoreFit, RestoreSet};
#[path = "design.rs"]
mod design;
use design::{branch_written, design_claim_permitted, goal_run, pending_writer, read_designs, unpublished_branch};

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
    comparison: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
    baseline: Option<skein_forge::api::Commit>,
    state: LostReadState,
    kind: LostWriterKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum LostWriterKind {
    Slot,
    Design,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum LostReadState {
    Asked,
    WaitingForRoom,
    WaitingForConnection,
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
    Branch,
    Base,
    Pull,
    Ci,
    BaseCi,
    Run(u64),
    Compare,
    ContainsHead,
    CatchUp,
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
    base_tip: Option<skein_forge::api::Commit>,
    pull: Option<skein_forge::api::Pull>,
    pull_ci: bool,
    ci: change::Status,
    failed: crate::change_ci::Failed,
    base_ci: change::Status,
    contains_base: change::Status,
    contains_head: change::Status,
    contains_catch_up: change::Status,
    reviews: Box<[skein_forge::api::Review]>,
    reviews_complete: bool,
    paths: Box<[Box<[u8]>]>,
    paths_whole: bool,
    woken: bool,
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
/// (domain/forge/README.md, 4; jig's `domain/connectors.md`, 4.3).
#[derive(Hash, Debug)]
pub(crate) struct Outboxed {
    pub(crate) entry: skein_forge_client::Entry,
    held: bool,
    handoff: Handoff,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Handoff {
    Uncommitted,
    /// Restored entries enter the client only at `SettleOutbox`.
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

/// The bounded forge connector subtree.
#[derive(Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the volatile full-outbox mark is independent of connector phases")]
pub struct Domain {
    retire_pending: bool,
    watch_due: Option<skein_lib::Time>,
    pub(crate) machine: Option<crate::Machine>,
    pub(crate) records: Map<u16, crate::ForgeRecord>,
    pub(crate) forge_number: u16,
    pub(crate) configured: Map<u16, crate::Configured>,
    /// The connector was built with `--box`; a closed pipe does not change it
    /// (temper/docs/design/domain/forge/adoption.md, 6).
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
    pub(crate) project_facts: Map<u32, crate::ProjectFacts>,
    pub(crate) devices: Option<Box<crate::devices::State>>,
    pub(crate) drafts: Option<Box<crate::drafts::State>>,
    pub(crate) aways: Map<(u64, u32), crate::Away>,
    pub(crate) decisions: Map<(u32, u32), crate::Decided>,
    pub(crate) findings: Option<Box<Map<(u32, u32), crate::Finding>>>,
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
    pub(crate) connect: crate::connect::State,
    pub(crate) installations: Map<(u16, u64), crate::Installation>,
    pub(crate) repositories: Map<skein_forge::api::Repository, Box<Repository>>,
    pub(crate) names: Map<u64, Box<[Name]>>,
    pub(crate) holds: Map<Name, Hold>,
    pub(crate) subscriptions: Map<(u64, Topic), Subscriber>,
    pub(crate) goal_files: Map<u64, topics::Files>,
    pub(crate) goal_file_reads: Map<Token, topics::Pending>,
    pub(crate) heads: Map<Name, BranchHead>,
    pulls: Map<Name, PullState>,
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
    pub(crate) sequence: u64,
    pub(crate) closing: crate::closer::State,
    pub(crate) changes: Map<u64, ChangeRow>,
    pub(crate) steps: Map<Token, PendingStep>,
    // Volatile, bounded by changes (temper/docs/design/domain/forge/adoption.md, 1).
    change_connections: Option<Box<Map<u64, skein_lib::Wall>>>,
    start_conflicts: Map<u64, skein_forge::api::Commit>,
    // Volatile input for the next change step, never restored (domain/forge/change/gates.md, 1).
    review_requests: Map<u64, skein_forge::api::Commit>,
    makings: Map<Token, goal_branch::Making>,
    pub(crate) panel_ci: Map<u64, change::Ci>,
    pub(crate) issues: Map<u64, IssueRow>,
    releases: Map<u64, ReleaseRow>,
    pub(crate) kept: Map<u64, Box<[Name]>>,
    namespace: Box<[u8]>,
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
    /// Restored timers stay passive until the scripted fresh-read stage (domain/root.md, 8.1).
    restore_phase: RestorePhase,
    pub(crate) reconciliation: crate::reconcile::Reconciliation,
    restore_started: Option<skein_lib::Wall>,
    restore_fit: Option<Box<RestoreFit>>,
    outage: Option<Box<outage::State>>,
    client: skein_forge_client::Domain,
    client_out: Queue<skein_forge_client::Request>,
    // A refused brief read waits for the next client reclamation.
    pub(crate) brief_reclaim: u64,
    pub(crate) brief_fetches: Map<Token, brief::BriefFetch>,
    pub(crate) brief_planned: Map<Token, held::Planned>,
    pub(crate) brief_documents: Map<Token, Box<crate::BriefDocument>>,
    pub(crate) document_fetches: Map<Token, crate::brief_document::Fetch>,
    pub(crate) brief_pending: Map<Token, held::Pending>,
    pub(crate) brief_held: Map<Token, held::Held>,
    merge_kind: Option<(u32, u32)>,
    close_kind: Option<(u32, u32)>,
    projection_due: Map<u64, skein_lib::Wall>,
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
    /// The machine's person login for an owned forge (domain/agent/agents.md, 1).
    #[must_use]
    pub fn owned_person(&self, forge: u16) -> Option<&[u8]> {
        let config = self.configured(forge)?;
        if config.mode != crate::Mode::Owned {
            return None;
        }
        let machine = self.machine.as_ref()?;
        if machine.owned_person.is_empty() { None } else { Some(&machine.owned_person) }
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
        crate::Forges { rows: rows.into_boxed() }
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
    /// One decision shared by the document and setup admission (domain/forge/adoption.md, 1).
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
                outcome @ (crate::ApplicationOutcome::Adopted(_)
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
            Some(kept) if kept.head == row.change.last_head && !row.terminal() => kept.touched,
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
        if row.terminal() || self.review_facts.get(&task) == Some(&facts) {
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
        if connection_wait::contains(self, task) {
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

    /// Available outbox room including the root's staged descriptions.
    #[must_use]
    pub fn effect_room(&self, limits: &Limits, staged: u32) -> bool {
        self.entries.len().saturating_add(staged) < limits.entries
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
            project_facts: Map::with_capacity(l.judge_projects),
            devices: match config.push {
                Some(push) => Some(Box::new(crate::devices::State::new(Some(push), l.devices()))),
                None => None,
            },
            drafts: None,
            aways: Map::with_capacity(l.bindings.saturating_mul(l.judge_projects)),
            decisions: Map::with_capacity(l.decided()),
            findings: None,
            looked: Map::with_capacity(l.bindings.saturating_mul(l.judge_projects)),
            dismissed: Map::with_capacity(l.dismissed()),
            connections: Map::with_capacity(l.forges),
            fetched: None,
            first: Map::with_capacity(l.forges),
            ended: Map::with_capacity(l.forges),
            apps: Map::with_capacity(l.forges),
            setup_answers: Map::with_capacity(l.applications),
            setup_pending: Map::with_capacity(l.applications),
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
            sequence: 0,
            closing: crate::closer::State::new(l.tasks),
            changes: Map::with_capacity(l.changes),
            steps: Map::with_capacity(l.changes),
            change_connections: None,
            start_conflicts: Map::with_capacity(l.changes),
            review_requests: Map::with_capacity(l.changes),
            makings: Map::with_capacity(l.holds),
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
            brief_reclaim: 0,
            brief_fetches: Map::with_capacity(l.brief_sections),
            brief_planned: Map::with_capacity(l.brief_sections),
            brief_documents: Map::with_capacity(l.brief_sections),
            document_fetches: Map::with_capacity(l.brief_sections),
            brief_pending: Map::with_capacity(l.brief_sections),
            brief_held: Map::with_capacity(l.brief_sections),
            merge_kind: None,
            close_kind: None,
            projection_due: Map::with_capacity(l.issues),
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
        match (self.client.next_deadline(), brief::next_deadline(self)) {
            (Some(client), Some(brief)) => Some(client.min(brief)),
            (client, brief) => client.or(brief),
        }
        .into_iter()
        .chain(if self.stopping { None } else { ci_news::deadline(self) })
        .chain(self.watch_due)
        .chain(self.playground_due)
        .chain(if self.stopping { None } else { crate::follows::deadline(self) })
        .chain(crate::tiers::deadline(self))
        .chain(goal_branch::deadline(self))
        .chain(if self.stopping { None } else { outage::deadline(self) })
        .min()
    }

    /// Earliest procedure or projection wait, reported to the root's clock.
    #[must_use]
    pub fn next_wall_deadline(&self) -> Option<skein_lib::Wall> {
        if self.stopping || self.restore_phase == RestorePhase::Restoring {
            return None;
        }
        let mut next: Option<skein_lib::Wall> = None;
        for deadlines in [&self.projection_due, &self.change_due, &self.landing.due, &self.tier_ops.due] {
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
            return self.client.is_ready();
        }
        self.tier_ops.dirty
            || crate::follows::waiting(self)
            || self.next_handoff().is_some()
            || !self.procedure_wakes.is_empty()
            || goal_branch::room_waiting(self)
            || self.client.is_ready()
            || ((self.lost_waiting() || brief::waiting(self) || goal_branch::read_waiting(self))
                && self.client.has_room())
    }

    fn next_handoff(&self) -> Option<u64> {
        for (number, row) in &self.entries {
            if row.handoff == Handoff::Handed {
                continue;
            }
            if row.handoff != Handoff::Ready || goal_branch::reserved_before(self, *number) {
                return None;
            }
            for (_, pending) in &self.steps {
                if pending.entry < *number {
                    return None;
                }
            }
            for (_, pending) in &self.landing.pending {
                if pending.entry < *number {
                    return None;
                }
            }
            return Some(*number);
        }
        None
    }

    /// Whether a lost writer's read waits to be asked again.
    fn lost_waiting(&self) -> bool {
        let mut waiting = false;
        for (_, pending) in &self.lost {
            waiting |= pending.state == LostReadState::WaitingForRoom;
        }
        waiting
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
        self.client.reclaim();
        self.brief_reclaim = self.brief_reclaim.wrapping_add(1);
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

    /// A released task's repositories (temper/docs/design/domain/forge/adoption.md, 1).
    #[must_use]
    pub fn kept_names(&self, task: u64) -> Option<&[Name]> {
        match self.kept.get(&task) {
            Some(names) => Some(names),
            None => None,
        }
    }

    /// Split restored live and kept names (temper/docs/design/domain/forge/README.md, 9).
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
    ) -> Box<[(Name, skein_forge::api::Commit)]> {
        let mut names = List::with_capacity(limits.resources_per_task);
        for (name, hold) in &self.holds {
            if hold.writer != Some(Writer::Run { task, attempt }) {
                continue;
            }
            for reported in pushed {
                if untag(reported.tag) == Some((name.forge, name.repository)) {
                    names.push((name.clone(), reported.commit)).expect("claimed writer destinations are bounded");
                    break;
                }
            }
        }
        names.into_boxed()
    }

    pub(crate) fn goal_branch_waiting(&self, name: &Name) -> bool {
        goal_branch::waiting(self, name)
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

    /// The project's role on a repository (temper/docs/design/domain/forge/adoption.md, 1).
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

    /// Current durable change state used by the root's procedure route.
    #[must_use]
    pub fn change(&self, task: u64) -> Option<&ChangeRow> {
        self.changes.get(&task)
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
        save_queue_change(self, env, before, saved, out);
    }

    /// Live change procedures in one repository, bounded by the connector's change table.
    #[must_use]
    pub fn changes_for(&self, repository: skein_forge::api::Repository, limit: u32) -> Box<[u64]> {
        let mut tasks = List::with_capacity(limit);
        for (task, row) in &self.changes {
            if row.repository == repository {
                tasks.push(*task).expect("change table bounds this list");
            }
        }
        tasks.into_boxed()
    }
}

/// Decide one parent event and collect its durable records and outputs.
#[expect(clippy::too_many_lines, reason = "one exhaustive connector input dispatcher")]
pub fn step(d: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if d.restore_fit.is_some() {
        return;
    }
    match event {
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
        Event::Closer { row } => crate::closer::save(d, row, out),
        Event::CloserRelayed { task, relay } => {
            if let Some(mut row) = d.closer(task).cloned() {
                row.relay = relay;
                crate::closer::save(d, row, out);
            }
        }
        Event::ReadCloserSelection { task, repository } => crate::closer::read_selection(d, env, task, repository, out),
        Event::CheckCloser { task, step, repository, branch, base } => {
            crate::closer::check(d, env, task, step, repository, branch, base, out);
        }
        Event::ChangeEnding { task, ending } => {
            if let Some(row) = d.changes.get_mut(&task) {
                let owned = row.change.owns_turn;
                row.change.owns_turn = false;
                row.ending = Some(ending);
                let row = row.clone();
                wake_after_turn(d, &row, owned);
                save_change(d, out, row);
            }
        }
        Event::TierWords { task, words } => crate::tiers::words::projected(d, task, &words, out),
        Event::StatusTask { project, task } => crate::status::task(d, project, Some(task), out),
        Event::StatusRefused { project } => crate::status::task(d, project, None, out),
        Event::StatusHeard { task, report } => crate::status::heard(d, env.wall, task, report, out),
        Event::Follow { row } => crate::follows::register(d, row, out),
        Event::TierTask { repository, tier, number, task } => {
            crate::tiers::task(d, repository, tier, number, Some(task), out);
        }
        Event::TierRefused { repository, tier, number } => crate::tiers::task(d, repository, tier, number, None, out),
        Event::StepTier { task, entry, heard } => crate::tiers::step(d, env, task, entry, heard, out),
        Event::VetoTier { task, entry } => crate::tiers::veto(d, task, entry, out),
        Event::OrderLanding { task, changes } => crate::landing::order(d, task, &changes, out),
        Event::JoinLanding { task, member, after } => crate::landing::join(d, task, member, &after, out),
        Event::Landing { row } => crate::landing::register(d, env, row, out),
        Event::VetoLanding { task, entry } => crate::landing::veto(d, task, entry, out),
        Event::StepLanding { task, entry, heard } => crate::landing::step(d, env, task, entry, heard, out),
        Event::Settle => {
            d.stopping = true;
            d.watch_due = None;
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
                    // A restart's NotConnected narrows nothing (domain/forge/README.md,
                    // 9). Resume these reads as client call room becomes free.
                    let mut waiting = List::with_capacity(env.limits.holds);
                    for (owner, pending) in &d.lost {
                        if pending.name.forge == forge && pending.state == LostReadState::WaitingForConnection {
                            waiting.push(*owner).expect("lost reads bounded by holds");
                        }
                    }
                    goal_branch::connected(d, forge);
                    for owner in &waiting {
                        d.lost.get_mut(owner).expect("collected lost read").state = LostReadState::WaitingForRoom;
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
            crate::removal::secrets_erased(d, forge, out);
            crate::playground::removed(d, out);
        }
        Event::BoxRemoved { forge } => {
            crate::removal::box_removed(d, forge, out);
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
        Event::FollowAct { request, token, retention, project, follow, act, authorized } => {
            crate::follows::asked(d, env, request, token, retention, project, follow, act, authorized, out);
        }
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
        Event::ReleaseProject { project, request } => adoption::release(d, project, request, out),
        Event::Adopt { request } => adoption::adopt(d, env, request, out),
        Event::ForgetAdoption { repository, restore } => match restore {
            Some(previous) => {
                d.repositories.insert(repository, Box::new(previous.clone())).expect("restores an admitted repository");
                emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Repository(previous) });
                emit(out, Request::MarkedForges);
            }
            None => {
                if d.repositories.remove(&repository).is_some() {
                    crate::landing_log::erased(d, repository, out);
                    crate::drafts::erase_repository(d, repository, out);
                    emit(out, Request::Erase { key: Key::Repository(repository) });
                    emit(out, Request::MarkedForges);
                }
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
            if d.kept.remove(&task).is_some() {
                emit(out, Request::Erase { key: Key::Names(task) });
                crate::removal::unnamed(d, out);
            } else {
                unnamed(d, env, task, false, out);
            }
        }
        Event::Hold { entry, task, resource, from } => hold(d, env, entry, task, resource, from, out),
        Event::Claim { task, attempt, writes, designs, holders } => {
            claim(d, env, task, attempt, &writes, &designs, &holders, out);
        }
        Event::Answered { task, attempt, pushed } => answered(d, env, task, attempt, &pushed, out),
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
        Event::Enqueue { entry } => enqueue(d, env, entry, out),
        Event::KeepProjection { row, entry } => {
            d.issues.insert(row.goal, row.clone()).expect("projection capacity admitted");
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Issue(row) });
            keep_entry(d, entry, None, out);
        }
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
            }
        }
        Event::ProducedTitle { task, child, title, report } => produced_title(d, task, child, &title, report, out),
        Event::Delegated { task, child, kind } => delegated(d, env, task, child, kind, out),
        Event::DelegateRefused { task, child } => delegate_refused(d, env, task, child, out),
        Event::DelegateAdmissionRefused { task, kind } => delegate_admission_refused(d, env, task, kind, out),
        Event::DelegateResult { task, child, status, no_change, failed, cancelled, words } => {
            delegate_result(d, env, task, child, status, no_change, failed, cancelled, &words, out);
        }
        Event::RelayRefused { task, questions, words } => {
            if let Some(row) = d.changes.get_mut(&task) {
                let before = crate::panel::ready(row);
                let owned = row.change.owns_turn;
                row.questions = questions;
                row.change.state = relay_hold(&row.change.state, change::Hold::Failed);
                row.change.owns_turn = false;
                row.producer_failure = Some(words);
                let saved = row.clone();
                wake_after_turn(d, &saved, owned);
                save_queue_change(d, env, before, saved, out);
            }
        }
        Event::Relayed { task, questions } => {
            if let Some(row) = d.changes.get_mut(&task) {
                let before = crate::panel::ready(row);
                let owned = row.change.owns_turn;
                let was_asking = agent_asking(row);
                let hold = match questions.open.first() {
                    Some(question) => {
                        question.stands == change::Stands::Taken
                            && questions.taken >= env.limits.change_policy.questions
                    }
                    None => false,
                };
                row.questions = questions;
                let answered = was_asking && !agent_asking(row);
                if hold {
                    row.change.state = relay_hold(&row.change.state, change::Hold::Questions);
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
                && !row.terminal()
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
            step_change(
                d,
                env,
                StepInput { task, entry, heard, gates, queue_repair_active, kept, merge_job, pending },
                out,
            );
        }
        Event::ReleaseChange { task } => release_change(d, env, task, out),
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
        Event::Client(event) => child(d, env, event, out),
        Event::Restore { record } => restore(d, env, record),
        Event::Restored { clock } => restored(d, env, clock, out),
        Event::ReadAfresh => read_afresh(d, env, out),
        Event::SettleOutbox => settle_outbox(d, env, out),
    }
    if d.retire_pending {
        d.retire_pending = false;
        retire_ended(d, env, out);
    }
}

/// Drive one child call after prior saves have committed.
pub fn resume(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
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
    if goal_branch::resume_room(d, out) {
        return;
    }
    let before = out.len();
    handoff_one(d, env, out);
    if out.len() != before {
        // An admission refusal already spent this step's child output burst.
        return;
    }
    let child_env = child_env(env);
    skein_forge_client::resume(&mut d.client, &child_env, &mut d.client_out);
    drain(d, env, out);
    if d.client.has_room() {
        brief::retry(d, out);
        ask_lost_again(d, env, out);
        goal_branch::retry(d, env, out);
    }
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
    if d.restore_fit.is_some() {
        return;
    }
    if d.restore_phase == RestorePhase::Restoring && !d.stopping {
        return;
    }
    if d.stopping {
        skein_forge_client::fire(&mut d.client, &child_env(env), &mut d.client_out);
        drain(d, env, out);
        return;
    }
    crate::playground::expire(d, env.now, out);
    brief::expire(d, env.now, out);
    crate::follows::fire(d, env.now);
    goal_branch::fire(d, env.now);
    ci_news::fire(d, env, out);
    admission::fire(d, env, out);
    outage::fire(d, env, out);
    crate::tiers::fire(d, env, out);
    let mut goals = List::with_capacity(env.limits.issues);
    for (goal, when) in &d.projection_due {
        if *when <= env.wall && goals.room() > 0 {
            goals.push(*goal).expect("bounded ready projection");
        }
    }
    for goal in &goals {
        d.projection_due.remove(goal);
        emit(out, Request::ProjectAfter { goal: *goal, when: None });
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
    let child_env = child_env(env);
    skein_forge_client::fire(&mut d.client, &child_env, &mut d.client_out);
    drain(d, env, out);
}

/// Give a forge its writer once access is known (domain/forge/adoption.md, 1).
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
    outage::track(d, env, &event);
    skein_forge_client::step(&mut d.client, &child_env(env), event, &mut d.client_out);
    drain(d, env, out);
    outage::refresh(d, env, out);
}

#[expect(
    clippy::too_many_lines,
    reason = "one client-output cohort updates connector state before routing its outcomes"
)]
fn drain(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    for _ in 0..d.client_out.len() {
        // A handler may drain the remaining output while requesting its next read.
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
                let record = match record {
                    skein_forge_client::Stored::Writer(row) => Stored::Writer(row),
                    row @ (skein_forge_client::Stored::Live(_) | skein_forge_client::Stored::Repository(_)) => {
                        Stored::Client(row)
                    }
                };
                emit(out, Request::Save { read_afresh: None, release: None, record });
            }
            skein_forge_client::Request::Erase { key } => emit(out, Request::Erase { key: Key::Client(key) }),
            skein_forge_client::Request::Progress { entry } => keep_entry(d, entry, None, out),
            skein_forge_client::Request::Outcome { entry, task, outcome } => {
                if outage::outcome(d, env, entry, outcome, out) {
                    continue;
                }
                if outcome == skein_forge_client::Outcome::Held && goal_branch::remake_held(d, env, entry, out) {
                    continue;
                }
                let account = match settled_account(d, entry, outcome) {
                    Some(account) => Some(account.words),
                    None => None,
                };
                let mut projection = false;
                for (_, row) in &d.issues {
                    projection |= row.pending == Some(entry);
                }
                let lost_comment = projection && outcome == skein_forge_client::Outcome::Held && posting(d, entry);
                d.named_outcome(entry, outcome);
                settled_facts(d, env, entry, task, outcome, out);
                crate::follow_check::outcome(d, entry, outcome, out);
                crate::tiers::outcome(d, entry, outcome, out);
                let retain = crate::landing::outcome(d, entry, outcome, out);
                match outcome {
                    skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {}
                    skein_forge_client::Outcome::Held => {
                        if lost_comment {
                            // domain/forge/issues.md: an unfindable milestone counts
                            // as posted. Retire its entry without sending it again.
                            not_made(d, entry, out);
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
                    | skein_forge_client::Outcome::Withdrawn => {
                        if !retain {
                            d.entries.remove(&entry);
                            emit(out, Request::Erase { key: Key::Entry(entry) });
                        }
                        erase_flight(d, task, entry, out);
                    }
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
                    emit(out, Request::ContinueRelease { task });
                }
            }
            skein_forge_client::Request::Changed { resource, result } => {
                if let Err(skein_forge::api::Error::Unavailable | skein_forge::api::Error::Timeout) = &result {
                    outage::observed(d, env, resource.repository.forge, true);
                }
                changed(d, env, resource, result, out);
            }
            skein_forge_client::Request::Drift { resource, expected, observed } => {
                drift(d, env, resource, crate::DriftChange::Branch { expected, observed }, out);
            }
            skein_forge_client::Request::Call { call, repository, op } => {
                outage::called(d, env, call, repository, &op);
                emit(out, Request::Call { call, repository, op });
            }
            skein_forge_client::Request::Kept { owner, result } => {
                admission::kept(d, env, owner, result, out);
            }
            skein_forge_client::Request::Read { owner, result } => {
                if outage::read_ended(d, env, owner, &result) {
                    continue;
                }
                if d.closing.selection_reads.contains_key(&owner) {
                    crate::closer::selection_answered(d, owner, result, out);
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
            skein_forge_client::Request::ReadAfreshDone => d.restart_fresh = Some(true),
            skein_forge_client::Request::OutboxDone => emit(out, Request::RestartDone { stage: RestartStage::Settled }),
        }
    }
    read_afresh_complete(d, out);
}

fn settled_facts(
    d: &mut Domain,
    env: &Env<Limits>,
    entry: u64,
    task: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) {
    outcome_facts(d, env, entry, task, outcome, out);
    settle_merge_writer(d, env, entry, outcome);
}

fn settle_merge_writer(d: &mut Domain, env: &Env<Limits>, entry: u64, outcome: skein_forge_client::Outcome) {
    match outcome {
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => return,
        skein_forge_client::Outcome::Made { .. }
        | skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => {}
    }
    if let Some(row) = d.entries.get(&entry) {
        let entry = row.entry.clone();
        admission::merge_writer(d, env, &entry, false);
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
    release_outcome(d, entry, outcome, out);
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
            let task = crate::landing::merged_owner(d, task, entry);
            let signed = match d.changes.get(&task) {
                Some(row) => row.signed.clone(),
                None => Box::new([]),
            };
            crate::landing_log::landed(d, env.wall, commit, task, signed, out);
            if !d.releases.contains_key(&task) {
                crate::verification::landed(d, commit, task, out);
                if let Some(row) = d.changes.get_mut(&task)
                    && landed_commit(&row.change.state).is_none()
                {
                    let owned = row.change.owns_turn;
                    row.change.owns_turn = false;
                    row.change.state = change::State::Landed { merge: commit, since: env.wall };
                    let saved = row.clone();
                    wake_after_turn(d, &saved, owned);
                    save_change(d, out, saved);
                }
            }
            landed::landed_room(d, env, commit, task, out);
            if let Some(change) = d.changes.get(&task) {
                let repository = change.repository;
                let branch = change.base.clone();
                let resource =
                    skein_forge_client::Resource { repository, what: skein_forge_client::What::Branch(branch.clone()) };
                let name = from_client(&resource, &env.limits);
                let before = match &name {
                    Some(name) => match d.heads.get(name) {
                        Some(head) => Some(head.commit),
                        None => None,
                    },
                    None => None,
                };
                let owned = match &name {
                    Some(name) => match d.heads.get(name) {
                        Some(head) => head.owned,
                        None => false,
                    },
                    None => false,
                };
                if let Some(name) = name
                    && (d.holds.contains_key(&name) || owned)
                {
                    record_written_head(d, env, resource, name, commit, task, out);
                }
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
        let row = Hold { design_writers: Box::new([]), name: name.clone(), task, writer: None, drift: None };
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
        }
        crate::removal::unnamed(d, out);
    }
    drain(d, env, out);
}

// Keep repository names alone, in name order (temper/docs/design/domain/forge/adoption.md, 1).
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
    let row = Hold { name: resource.clone(), task, writer, design_writers, drift };
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
        if let Some(resource) = to_client(name, &env.limits) {
            child(d, env, skein_forge_client::Event::Writer { resource, taken: true }, out);
        }
    }
    for name in writes {
        let Some(row) = d.holds.get_mut(name) else { unreachable!("preflighted hold") };
        row.writer = Some(Writer::Run { task, attempt });
        let saved = row.clone();
        save_hold(out, saved);
        if let Some(resource) = to_client(name, &env.limits) {
            child(d, env, skein_forge_client::Event::Writer { resource, taken: true }, out);
        }
    }
}

fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    pushed: &[(Name, skein_forge::api::Commit)],
    out: &mut Queue<Request>,
) {
    read_designs(d, env, task, attempt, out);
    let mut freed = List::with_capacity(env.limits.holds);
    for (name, row) in &d.holds {
        let mut reported = false;
        for (pushed_name, _) in pushed {
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
        let holder = row.task;
        if let Some(resource) = to_client(name, &env.limits) {
            for (pushed_name, commit) in pushed {
                if pushed_name == name && undrifted {
                    let head = BranchHead { name: name.clone(), commit: *commit, owned: true };
                    let previous = d.branch_head(name);
                    let concerning = heads::concerning(d, name, Some(holder));
                    let recorded = heads::record(d, env, head.clone(), concerning, out);
                    wake_head_dependents(d, name, previous, *commit);
                    if recorded {
                        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
                    }
                    record_delegate_head(d, task, name, *commit, out);
                    child(
                        d,
                        env,
                        skein_forge_client::Event::Pushed { resource: resource.clone(), commit: *commit },
                        out,
                    );
                }
            }
            let taken = branch_written(d, name, None);
            child(d, env, skein_forge_client::Event::Writer { resource, taken }, out);
        }
    }
}

/// A delegate's last push, including an unchecked save, pins its next brief
/// without completing its delegation (temper/docs/design/domain/agent/README.md, 5).
fn record_delegate_head(
    d: &mut Domain,
    task: u64,
    name: &Name,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let mut pushed = None;
    for (_, row) in &d.changes {
        if let Some((child, _)) = row.delegate
            && child == task
            && row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
            && row.change.last_head != Some(commit)
        {
            let mut row = row.clone();
            row.change.last_head = Some(commit);
            pushed = Some(row);
            break;
        }
    }
    if let Some(row) = pushed {
        d.changes.insert(row.task, row.clone()).expect("reported head belongs to a retained change");
        save_change(d, out, row);
    }
}

fn lost(d: &mut Domain, env: &Env<Limits>, task: u64, attempt: u64, out: &mut Queue<Request>) {
    read_designs(d, env, task, attempt, out);
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
        let baseline = d.branch_head(name);
        if d.lost
            .insert(
                owner,
                PendingLost {
                    task,
                    attempt,
                    name: name.clone(),
                    comparison: None,
                    baseline,
                    state: LostReadState::Asked,
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

/// Ask one waiting lost writer's read again: its branch, or the ancestry
/// of the head it found with the head the forge last knew.
fn ask_lost_again(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut waiting = None;
    for (owner, pending) in &d.lost {
        if pending.state == LostReadState::WaitingForRoom && waiting.is_none() {
            waiting = Some(*owner);
        }
    }
    let Some(owner) = waiting else { return };
    let baseline = d.branch_head(&d.lost.get(&owner).expect("found above").name);
    let pending = d.lost.get_mut(&owner).expect("found above");
    pending.state = LostReadState::Asked;
    if pending.comparison.is_none() {
        pending.baseline = baseline;
    }
    let comparison = pending.comparison;
    let Some(resource) = to_client(&pending.name, &env.limits) else { return };
    let read = match (comparison, resource.what) {
        (Some((baseline, observed)), _) => {
            skein_forge::api::Read::Commits { before: observed, after: baseline, page: 1 }
        }
        (None, skein_forge_client::What::Branch(branch)) => skein_forge::api::Read::Branch { branch },
        (
            None,
            skein_forge_client::What::Repository
            | skein_forge_client::What::Pull(_)
            | skein_forge_client::What::Issue(_),
        ) => {
            unreachable!("a lost writer holds a branch")
        }
    };
    child(d, env, skein_forge_client::Event::Read { owner, repository: resource.repository, read }, out);
}

#[expect(clippy::too_many_lines, reason = "slot and design writers share the fresh branch and ancestry read")]
fn lost_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.lost.remove(&owner) else { return };
    let Some(row) = d.holds.get(&pending.name) else { return };
    if !pending_writer(row, &pending) {
        // temper/docs/design/domain/forge/change/repairs.md, pinned pull
        // sections: a superseded lost reread still records its delegate's
        // head, undrifted, without freeing the later attempt's writer slot.
        let later = match row.writer {
            Some(Writer::Run { task, attempt }) => task == pending.task && attempt > pending.attempt,
            Some(Writer::Lost { .. } | Writer::Entry(_)) | None => false,
        };
        if pending.kind == LostWriterKind::Slot
            && later
            && row.drift.is_none()
            && let Some(commit) = superseded_head(&pending, result)
        {
            record_delegate_head(d, pending.task, &pending.name, commit, out);
        }
        return;
    }
    // The writer slot is freed only once the branch is read afresh (jig's
    // domain/tasks/holds.md, 3). Busy waits for call room; NotConnected waits
    // for this forge's connection (domain/forge/README.md, 9), without ready work.
    let waiting = match result {
        Err(skein_forge::api::Error::Busy) => Some(LostReadState::WaitingForRoom),
        Err(skein_forge::api::Error::NotConnected) => Some(LostReadState::WaitingForConnection),
        Ok(_) | Err(_) => None,
    };
    if let Some(state) = waiting {
        pending.state = state;
        d.lost.insert(owner, pending).expect("the read keeps its own lost slot");
        return;
    }
    let Some(resource) = to_client(&pending.name, &env.limits) else { return };
    for (task, change) in &d.changes {
        if change.repository == resource.repository
            && crate::items::branch_matches(&pending.name, &change.branch)
            && let Some(read) = d.settling.get_mut(task)
        {
            read.branch.due = true;
        }
    }
    let baseline = d.branch_head(&pending.name);
    let stale = match pending.comparison {
        Some((before, _)) => baseline != Some(before),
        None => baseline != pending.baseline,
    };
    if pending.kind == LostWriterKind::Design && stale {
        pending.comparison = None;
        pending.state = LostReadState::WaitingForRoom;
        d.lost.insert(owner, pending).expect("the design read retains its slot");
        ask_lost_again(d, env, out);
        return;
    }
    let observed = match (pending.comparison, result) {
        (None, Ok(skein_forge::api::Answer::Commit(commit))) => {
            if let Some(before) = baseline
                && before != commit
            {
                pending.comparison = Some((before, commit));
                d.lost.insert(owner, pending).expect("comparison occupies its reserved lost-read slot");
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read {
                        owner,
                        repository: resource.repository,
                        read: skein_forge::api::Read::Commits { before: commit, after: before, page: 1 },
                    },
                    out,
                );
                return;
            }
            Some(commit)
        }
        (Some((before, after)), result) => {
            let error = match &result {
                Err(error) => *error,
                Ok(_) => skein_forge::api::Error::InvalidAnswer,
            };
            match ancestry_status(result) {
                change::Status::Passed => {}
                change::Status::Failed => drift(
                    d,
                    env,
                    resource.clone(),
                    crate::DriftChange::Branch { expected: before, observed: Some(after) },
                    out,
                ),
                change::Status::Unknown | change::Status::Pending => {
                    lost_failed(
                        d,
                        pending.task,
                        pending.attempt,
                        &pending.name,
                        &crate::effect_words::error_words(error),
                        out,
                    );
                    return;
                }
            }
            Some(after)
        }
        (None, Err(skein_forge::api::Error::Missing)) => {
            if let Some(expected) = baseline {
                drift(d, env, resource.clone(), crate::DriftChange::Branch { expected, observed: None }, out);
            }
            // A run can end before its first push; fresh absence releases
            // its writer when no previous head was recorded.
            None
        }
        (_, result) => {
            let error = match result {
                Ok(_) => skein_forge::api::Error::InvalidAnswer,
                Err(error) => error,
            };
            lost_failed(d, pending.task, pending.attempt, &pending.name, &crate::effect_words::error_words(error), out);
            return;
        }
    };
    let Some(row) = d.holds.get_mut(&pending.name) else { return };
    if !pending_writer(row, &pending) {
        return;
    }
    match pending.kind {
        LostWriterKind::Slot => row.writer = None,
        LostWriterKind::Design => {
            let mut writers = List::with_capacity(env.limits.holds);
            for writer in &row.design_writers {
                if writer.task != pending.task || writer.attempt != pending.attempt {
                    writers.push(*writer).expect("retained design writer count");
                }
            }
            row.design_writers = writers.into_boxed();
        }
    }
    let undrifted = row.drift.is_none();
    save_hold(out, row.clone());
    if undrifted && let Some(commit) = observed {
        let previous = d.branch_head(&pending.name);
        let head = BranchHead { name: pending.name.clone(), commit, owned: true };
        let concerning = heads::concerning(d, &pending.name, Some(pending.task));
        let recorded = heads::record(d, env, head.clone(), concerning, out);
        wake_head_dependents(d, &pending.name, previous, commit);
        if recorded {
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
        }
        record_delegate_head(d, pending.task, &pending.name, commit, out);
        child(d, env, skein_forge_client::Event::Pushed { resource: resource.clone(), commit }, out);
    }
    let taken = branch_written(d, &pending.name, None);
    child(d, env, skein_forge_client::Event::Writer { resource, taken }, out);
    lost_release(d, &pending.name, false, out);
}

/// A later attempt keeps its slot while the lost read's branch or successful
/// ancestry answer still pins the delegate's next brief (domain/forge/change/repairs.md).
fn superseded_head(
    pending: &PendingLost,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Option<skein_forge::api::Commit> {
    use skein_forge::api::Answer;
    if let Some((_, head)) = pending.comparison {
        return if ancestry_status(result) == change::Status::Passed { Some(head) } else { None };
    }
    match result {
        Ok(Answer::Commit(head)) => Some(head),
        Ok(
            Answer::Located { .. }
            | Answer::Items { .. }
            | Answer::Item { .. }
            | Answer::Pull(_)
            | Answer::Reviews { .. }
            | Answer::Statuses { .. }
            | Answer::Remarks { .. }
            | Answer::Branches(_)
            | Answer::PullFiles { .. }
            | Answer::Compare { .. }
            | Answer::Checks(_)
            | Answer::Job { .. }
            | Answer::File { .. }
            | Answer::Directory { .. }
            | Answer::Protection(_)
            | Answer::Settings(_)
            | Answer::Collaborators { .. }
            | Answer::Permission(_)
            | Answer::Created(_)
            | Answer::Commented(_)
            | Answer::Reviewed(_)
            | Answer::Merged(_)
            | Answer::Queued
            | Answer::Branch(_)
            | Answer::Runs { .. }
            | Answer::Run(_)
            | Answer::Tree { .. }
            | Answer::Commits { .. }
            | Answer::Done,
        )
        | Err(_) => None,
    }
}

fn lost_failed(d: &Domain, task: u64, attempt: u64, name: &Name, reason: &[u8], out: &mut Queue<Request>) {
    lost_release(d, name, true, out);
    emit(out, Request::LostReadFailed { task, attempt, resource: name.clone(), reason: Box::from(reason) });
}

/// Wake the holder's existing release after its lost writer's reread ends.
fn lost_release(d: &Domain, name: &Name, failed: bool, out: &mut Queue<Request>) {
    let Some(hold) = d.holds.get(name) else { return };
    let task = hold.task;
    let Some(row) = d.releases.get(&task) else { return };
    if row.releasing && !row.failed && row.pending.is_none() {
        emit(
            out,
            if failed { Request::ReleaseFailed { task, account: None } } else { Request::ContinueRelease { task } },
        );
    }
}

fn enqueue(d: &mut Domain, env: &Env<Limits>, entry: skein_forge_client::Entry, out: &mut Queue<Request>) {
    let task = entry.task;
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
    if d.entries.len() == env.limits.entries {
        crate::refusal::resource_limit(
            out,
            task,
            b"enqueue effect",
            None,
            "forge.entries",
            env.limits.entries,
            u64::from(d.entries.len()).saturating_add(1),
        );
        return;
    }
    let release = Some(entry.number);
    admission::update_writer(d, env, &entry, true);
    keep_entry(d, entry, release, out);
}

/// Save one entry in the client's hands: newly committed, for the client to
/// make once the commit is durable, or a known entry's progress.
pub(crate) fn keep_entry(
    d: &mut Domain,
    entry: skein_forge_client::Entry,
    release: Option<u64>,
    out: &mut Queue<Request>,
) {
    let handoff = match d.entries.get(&entry.number) {
        Some(row) => row.handoff,
        None => Handoff::Uncommitted,
    };
    if entry.attempt.is_some() {
        d.flights.insert((entry.task, entry.number), ()).expect("flights are bounded by entries");
        emit(
            out,
            Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::Flight { task: entry.task, entry: entry.number },
            },
        );
    }
    let row = Outboxed { entry: entry.clone(), held: false, handoff };
    d.entries.insert(entry.number, row).expect("entry capacity checked or a known entry replaced");
    emit(out, Request::Save { read_afresh: None, release, record: Stored::Entry { entry, held: false } });
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

fn relay_hold(state: &change::State, why: change::Hold) -> change::State {
    if *state == change::State::Ended {
        return change::State::Ended;
    }
    let was = match state {
        change::State::Held { was, .. } => was.as_ref().clone(),
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
        | change::State::NoChange { .. }
        | change::State::Ended => state.clone(),
    };
    change::State::Held { was: Box::new(was), why }
}

fn register_change(d: &mut Domain, env: &Env<Limits>, mut row: ChangeRow, out: &mut Queue<Request>) {
    let task = row.task;
    if row.change.task != task || row.branch.is_empty() || row.base.is_empty() || d.changes.contains_key(&task) {
        emit(out, Request::Refused { task });
        return;
    }
    if !d.repositories.contains_key(&row.repository) {
        room::refused(out, &row, crate::RefusalCause::Unadopted);
        return;
    }
    for name in [&row.branch, &row.base] {
        if name.len() > skein_lib::usize_of(env.limits.name_bytes) {
            room::refused(
                out,
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
            &row,
            crate::RefusalCause::Limit {
                member: "forge.changes",
                configured: u64::from(env.limits.changes),
                needed: u64::from(d.changes.len()).saturating_add(1),
            },
        );
        return;
    }
    if !crate::follow_check::select(d, &mut row, &env.limits) {
        emit(out, Request::Refused { task });
        return;
    }
    row.change.way = d.landing.way(d.repositories.get(&row.repository).expect("adopted").project);
    d.changes.insert(task, row.clone()).expect("change capacity checked");
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
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get_mut(&task) else {
        emit(out, Request::Refused { task });
        return;
    };
    match kind {
        change::Delegate::Repair(why) => {
            row.result.repaired(why, row.change.last_head, env.limits.client.answer_bytes);
            row.repair_tasks = task_account(&row.repair_tasks, child);
        }
        change::Delegate::Resolve { .. } => row.resolution_tasks = task_account(&row.resolution_tasks, child),
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Merge { .. }
        | change::Delegate::Gate { .. } => {}
    }
    row.delegate = Some((child, kind));
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
    let saved = row.clone();
    save_change(d, out, saved);
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
    let owned = row.change.owns_turn;
    row.change.owns_turn = false;
    let state = unasked(row, kind);
    row.change.state = change::State::Held { was: Box::new(state), why: change::Hold::Failed };
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
fn delegate_wait(state: &change::State) -> bool {
    match state {
        change::State::Producing { requested }
        | change::State::Repairing { requested, .. }
        | change::State::Resolving { requested, .. } => *requested,
        change::State::WaitingProducer { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
        | change::State::Held { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
    }
}

#[expect(clippy::too_many_arguments, reason = "one child's terminal carries its status, failure standing and words")]
fn delegate_result(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    child: u64,
    status: change::Status,
    no_change: bool,
    failed: bool,
    cancelled: bool,
    words: &[u8],
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get_mut(&task) else { return };
    // A pushing child can end after a gate has replaced the delegate slot.
    // Its terminal still releases the procedure's conflict wait.
    d.procedure_wakes.insert(task, true).expect("one pending wake per change");
    let Some((expected, kind)) = row.delegate else { return };
    if expected != child {
        return;
    }
    let owned = row.change.owns_turn;
    let mut ended = false;
    if let change::State::Held { was, why } = &row.change.state
        && ((*why == change::Hold::Stalled && delegate_wait(was))
            || (*why == change::Hold::Failed
                && kind == change::Delegate::Close
                && **was == (change::State::Producing { requested: true })))
    {
        row.change.state = (**was).clone();
        row.change.owns_turn = false;
        ended = true;
    }
    let agent = match kind {
        change::Delegate::Gate { .. } => false,
        change::Delegate::Produce
        | change::Delegate::Close
        | change::Delegate::Repair(_)
        | change::Delegate::Resolve { .. }
        | change::Delegate::Merge { .. } => true,
    };
    if agent {
        row.change.since = env.wall;
        if let Some(read) = d.settling.get_mut(&task) {
            read.branch.due = true;
        }
    }
    // Keep the merge's identity through the fresh branch read, including after restart.
    match kind {
        change::Delegate::Merge { .. } | change::Delegate::Close | change::Delegate::Resolve { .. } => {}
        change::Delegate::Produce | change::Delegate::Repair(_) | change::Delegate::Gate { .. } => row.delegate = None,
    }
    d.start_conflicts.remove(&task);
    row.delegate_status = if no_change { change::Status::Failed } else { status };
    match kind {
        change::Delegate::Gate { number, head } => {
            if let Err(cause) = record_review(row, env, number, head, (status, failed, cancelled), words) {
                emit(out, Request::ResourceRefused { task, operation: b"record gate review", resource: None, cause });
                return;
            }
        }
        change::Delegate::Close => {
            producer_result(row, false, failed, words);
            if status == change::Status::Passed {
                let bound = match d.close_kind {
                    Some((_, result)) => result,
                    None => 0,
                };
                row.report = crate::text::fit(words, skein_lib::usize_of(bound), b"...");
            }
        }
        change::Delegate::Produce => producer_result(row, no_change, failed, words),
        change::Delegate::Merge { .. } => completed_merge(row, status, words, d.merge_kind),
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
    }
    let saved = row.clone();
    wake_after_turn(d, &saved, owned);
    save_change(d, out, saved);
    if ended {
        emit(out, Request::ChangeDecision { task, decision: change::Decision::EndHold, entry: None, evidence: None });
    }
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
        change::State::Producing { .. }
        | change::State::Repairing { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
    row.producer_failure = if (failed || (no_change && held_producing)) && !words.is_empty() {
        Some(crate::text::fit(
            words,
            usize::try_from(jig_domain_tasks::HOLD_ACCOUNT_BYTES).expect("bounded hold account"),
            b"...",
        ))
    } else {
        None
    };
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
    let report = change::GateReport { number, head, status };
    let mut verdicts = List::with_capacity(env.limits.change_policy.gates);
    for old in &row.verdicts {
        if old.number != number {
            verdicts.push(*old).expect("bounded prior gate reports");
        }
    }
    if !failed && verdicts.push(report).is_err() {
        return Err(crate::RefusalCause::Limit {
            member: "forge.change_policy.gates",
            configured: u64::from(env.limits.change_policy.gates),
            needed: u64::from(verdicts.len()).saturating_add(1),
        });
    }
    row.verdicts = verdicts.into_boxed();
    if failed {
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
    if row.change.state == change::State::Ended {
        return Some(change::Decision::Cancel);
    }
    if let change::State::NoChange { .. } = row.change.state {
        return Some(change::Decision::NoChange);
    }
    None
}

#[expect(clippy::too_many_lines, reason = "one fresh-read chain keeps task standing, containment and connection waits")]
fn step_change(d: &mut Domain, env: &Env<Limits>, input: StepInput, out: &mut Queue<Request>) {
    if d.stopping {
        return;
    }
    let StepInput { task, entry, mut heard, gates, queue_repair_active, kept, merge_job, pending: dependencies } =
        input;
    if kept
        && let Some(row) = d.changes.get_mut(&task)
        && row.change.owns_turn
    {
        row.change.owns_turn = false;
        let saved = row.clone();
        wake_after_turn(d, &saved, true);
        save_change(d, out, saved);
    }
    heard.start_conflict = d.start_conflict(task);
    heard.review_request = d.review_requests.get(&task).copied();
    if heard.released {
        release_change(d, env, task, out);
        heard.released = false;
    }
    let Some(row) = d.changes.get(&task) else {
        emit(out, Request::Refused { task });
        return;
    };
    if let Some(decision) = change_terminal(row) {
        emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None });
        return;
    }
    if let Some(merge) = landed_commit(&row.change.state) {
        step_verified(d, env, task, merge, heard, out);
        return;
    }
    let mut owner = None;
    for (token, pending) in &d.steps {
        if pending.task == task {
            owner = Some(*token);
        }
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
        let pending = d.steps.get_mut(&owner).expect("found pending step");
        // Entries hand off in number order (skein's forge/client.md, 7.2);
        // the newest wake reserves the restart's entry and reads jig's current kept answer.
        pending.entry = entry;
        pending.heard.delegate = heard.delegate;
        pending.heard.delegate_running = heard.delegate_running;
        pending.heard.delegate_live = heard.delegate_live;
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
    let branch = row.branch.clone();
    let producer = if row.delegate_status == change::Status::Unknown { heard.delegate } else { row.delegate_status };
    let read_branch = hints::has_head(d, row) || branch_is_due(d, env, task, producer);
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
        stage: StepStage::Branch,
        heard,
        gates: if gates.is_empty() { row.verdicts.clone() } else { gates },
        queue_repair_active,
        kept,
        merge_job,
        pending: dependencies,
        branch: None,
        branch_fresh: read_branch,
        base_tip: None,
        pull: None,
        pull_ci: false,
        ci: change::Status::Unknown,
        failed: crate::change_ci::Failed::new(Box::new([])),
        base_ci: change::Status::Unknown,
        contains_base: change::Status::Unknown,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Pending,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
    };
    d.steps.insert(owner, pending).expect("step capacity checked");
    if read_branch {
        branch_sent(d, task, producer);
        child(
            d,
            env,
            skein_forge_client::Event::Read { owner, repository, read: skein_forge::api::Read::Branch { branch } },
            out,
        );
    } else {
        step_read(d, env, owner, Err(skein_forge::api::Error::Missing), out);
    }
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
            change::PendingKind::Change { head: Some(_), kept: false } | change::PendingKind::Elsewhere => {}
            change::PendingKind::Change { .. } | change::PendingKind::Other => return false,
        }
    }
    !pending.pending.is_empty()
}

fn landed_commit(state: &change::State) -> Option<skein_forge::api::Commit> {
    match state {
        change::State::Landed { merge, .. }
        | change::State::Red { merge, .. }
        | change::State::Cleared { merge }
        | change::State::Uncleared { merge } => Some(*merge),
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
        branch: row.change.last_head,
        branch_fresh: false,
        base_tip: Some(merge),
        pull: None,
        pull_ci: false,
        ci: change::Status::Unknown,
        failed: crate::change_ci::Failed::new(Box::new([])),
        base_ci: change::Status::Unknown,
        contains_base: change::Status::Unknown,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Pending,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
    };
    let facts = change_facts(d, env, &row, &pending, merge);
    let stepped = change::step(&row.change, &facts, &heard, env.wall, &env.limits.change_policy);
    row.change = stepped.change;
    d.changes.insert(task, row.clone()).expect("replaces change");
    save_change(d, out, row);
    emit(out, Request::ChangeDecision { task, decision: stepped.decision, entry: None, evidence: None });
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

// Local read admission has made no forge call; retry when the client's room returns.
fn retry_step(env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let until = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(1_000_000_000));
    emit(
        out,
        Request::ChangeDecision {
            task,
            decision: change::Decision::Wait { until: Some(until) },
            entry: None,
            evidence: None,
        },
    );
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
    if row.terminal() {
        if let Some(decision) = change_terminal(row) {
            emit(out, Request::ChangeDecision { task: pending.task, decision, entry: None, evidence: None });
        } else if let Some(merge) = landed_commit(&row.change.state) {
            step_verified(d, env, pending.task, merge, pending.heard, out);
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
        branch_due(d, pending.task);
        retry_step(env, pending.task, out);
        return;
    }
    if result == Err(skein_forge::api::Error::NotConnected) {
        connection_wait::wait(d, env, pending.task, out);
        return;
    }
    if admission::read_failed(row, &pending, &result, out) {
        branch_due(d, pending.task);
        return;
    }
    let repository = row.repository;
    let branch = row.branch.clone();
    let base = row.base.clone();
    let pull_number = row.pull;
    let catch_up = row.catch_up;
    let preparing = row.change.state.clone();
    let made_update = row.effect == change::EffectResult::Made && matches_updating(&preparing);
    let had_head = hints::has_head(d, row);
    let required = match d.repositories.get(&repository) {
        Some(repository) => crate::ci::required(&repository.protection),
        None => &[],
    };
    let result_rows = result_contexts(&preparing);
    let mut pull_ci = required.is_empty() && !result_rows;
    if let change::State::Repairing { .. } = preparing {
        pull_ci = required.is_empty();
    }
    let next = match pending.stage {
        StepStage::FollowCheck(_) => unreachable!("workflow reads handled above"),
        StepStage::Branch => {
            if pending.branch_fresh
                && let Some(read) = d.settling.get_mut(&pending.task)
            {
                read.branch.at = Some(env.now);
            }
            match result {
                Ok(skein_forge::api::Answer::Commit(commit)) => {
                    pending.branch = Some(commit);
                    if let Some(read) = d.settling.get_mut(&pending.task) {
                        read.branch.present = true;
                    }
                    settled_update_drift(d, env, pending.task, commit, out);
                    if !made_update {
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
                Err(skein_forge::api::Error::Missing) => pending.branch = None,
                Ok(_) | Err(_) => {
                    unreachable!("fresh read answer checked");
                }
            }
            if goal_branch::base_wait(d, repository, &base) {
                wait_for_forge(d, env, &pending, None, out);
                return;
            }
            if pending.branch.is_none()
                && !hints::has_head(d, d.changes.get(&pending.task).expect("live change"))
                && catch_up.is_none()
                && !early_base(&pending)
            {
                finish_step(d, env, pending, out);
                return;
            }
            pending.stage = StepStage::Base;
            skein_forge::api::Read::Branch { branch: base }
        }
        StepStage::Base => {
            if goal_branch::base_wait(d, repository, &base) {
                wait_for_forge(d, env, &pending, None, out);
                return;
            }
            match result {
                Ok(skein_forge::api::Answer::Commit(commit)) => {
                    let tip = if let change::State::Preparing { main, .. } = preparing { main } else { commit };
                    pending.base_tip = Some(tip);
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
            if pending.branch.is_none() && !had_head && catch_up.is_none() {
                finish_step(d, env, pending, out);
                return;
            }
            if let Some(commit) = catch_up {
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
        StepStage::CatchUp => {
            pending.contains_catch_up = ancestry_status(result);
            if pending.contains_catch_up == change::Status::Unknown {
                let retry = skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(1_000_000_000));
                wait_for_forge(d, env, &pending, Some(retry), out);
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
                        || row.pull == Some(pull.number)
                        || row.change.last_head == Some(pull.commit);
                    pending.pull = if own { Some(pull) } else { None };
                }
                Err(skein_forge::api::Error::Missing) => pending.pull = None,
                Ok(_) | Err(_) => {
                    unreachable!("fresh read answer checked");
                }
            }
            pending.stage = StepStage::Ci;
            let Some(commit) = pending.branch else {
                pending.stage = StepStage::BaseCi;
                let Some(base_tip) = pending.base_tip else { unreachable!("base read") };
                d.steps.insert(owner, pending).expect("replaces step");
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read {
                        owner,
                        repository,
                        read: skein_forge::api::Read::Checks { commit: base_tip },
                    },
                    out,
                );
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
                    emit(
                        out,
                        Request::Save { read_afresh: None, release: None, record: Stored::Change(Box::new(saved)) },
                    );
                }
                if ci == skein_forge::api::Ci::Failed || result_rows {
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
                    && row.result.checked(head, &rows, env.limits.client.answer_bytes)
                {
                    let saved = row.clone();
                    emit(
                        out,
                        Request::Save { read_afresh: None, release: None, record: Stored::Change(Box::new(saved)) },
                    );
                }
                pending.failed = crate::change_ci::Failed::new(rows);
            }
            if pending.ci == change::Status::Failed
                && let Some(read) = pending.failed.next(pending.branch.unwrap_or([0; 32]))
            {
                let skein_forge::api::Read::Run { run } = read else { unreachable!("failed run") };
                pending.stage = StepStage::Run(run);
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
            if let Some(read) = pending.failed.next(pending.branch.unwrap_or([0; 32])) {
                let skein_forge::api::Read::Run { run } = read else { unreachable!("failed run") };
                pending.stage = StepStage::Run(run);
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
    child(d, env, skein_forge_client::Event::Read { owner, repository, read: next }, out);
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
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read {
                        owner,
                        repository,
                        read: skein_forge::api::Read::Compare { before, after },
                    },
                    out,
                );
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
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
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
    let (ready, repair) = queue_ready(d, row, l);
    match repair {
        Some(task) => row.task == task,
        None => {
            for (_, owner) in &d.changes {
                if owner.repository == row.repository
                    && owner.base == row.base
                    && owns_turn(owner)
                    && (!in_queue(&owner.change.state) || !d.approval_short(owner))
                {
                    return row.task == owner.task;
                }
            }
            change::first(ready.as_slice(), l.queue_window, now) == Some(row.task)
        }
    }
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
        | change::State::Gating { .. }
        | change::State::Landing { .. } => true,
        change::State::Producing { .. }
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
    if pending.woken {
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
    } else {
        decide_step(d, env, pending, out);
    }
}

/// Missing bases and uncertain reads are forge waits, bounded by the unchanged step's stall.
fn wait_for_forge(
    d: &mut Domain,
    env: &Env<Limits>,
    pending: &PendingStep,
    retry: Option<skein_lib::Wall>,
    out: &mut Queue<Request>,
) {
    let Some(mut row) = d.changes.get(&pending.task).cloned() else { return };
    if let change::State::Held { .. } = row.change.state {
        emit(
            out,
            Request::ChangeDecision { task: row.task, decision: change::Decision::None, entry: None, evidence: None },
        );
        return;
    }
    if pending.heard.delegate_live {
        row.change.since = env.wall;
    }
    let until = skein_lib::Wall::from_nanos(
        row.change.since.as_nanos().saturating_add(env.limits.change_policy.stall.as_nanos()),
    );
    let decision = if env.wall >= until {
        row.change.owns_turn = false;
        row.change.state = change::State::Held { was: Box::new(row.change.state), why: change::Hold::Stalled };
        change::Decision::Hold(change::Hold::Stalled)
    } else {
        let until = match retry {
            Some(retry) => retry.min(until),
            None => until,
        };
        change::Decision::Wait { until: Some(until) }
    };
    if let Some(prior) = d.changes.insert(row.task, row.clone()).expect("replaces change")
        && prior != row
    {
        save_change(d, out, row);
    }
    emit(out, Request::ChangeDecision { task: pending.task, decision, entry: None, evidence: None });
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
        if agent_asking(&row) {
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
            row.change.state = change::State::Held { was: Box::new(row.change.state), why: change::Hold::Stalled };
            wake_after_turn(d, &row, owned);
            d.changes.insert(task, row.clone()).expect("replaces change");
            save_queue_change(d, env, before, row, out);
            change::Decision::Hold(change::Hold::Stalled)
        } else {
            change::Decision::Wait { until: Some(until) }
        };
        emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None });
        return None;
    }
    Some(row)
}

fn change_standing(d: &Domain, row: &ChangeRow, pending: &PendingStep) -> change::Heard {
    change::Heard {
        // Terminal evidence belongs to the current delegate, not to the read's start.
        delegate: if row.delegate_status == change::Status::Unknown {
            pending.heard.delegate
        } else {
            row.delegate_status
        },
        start_conflict: d.start_conflict(row.task),
        review_request: d.review_requests.get(&row.task).copied(),
        effect: if row.pending.is_some() { change::EffectResult::Pending } else { row.effect },
        ..pending.heard
    }
}

#[expect(clippy::too_many_lines, reason = "one coherent snapshot records a green repair ending before stepping onward")]
fn decide_step(d: &mut Domain, env: &Env<Limits>, pending: PendingStep, out: &mut Queue<Request>) {
    let Some(mut row) = titled_change(d, env, &pending, out) else { return };
    let before = crate::panel::ready(&row);
    let prior = row.change.clone();
    // Producing without a head uses no base facts except early starts and catch-ups.
    let base_tip = pending.base_tip.unwrap_or([0; 32]);
    row.pull = match &pending.pull {
        Some(pull) => Some(pull.number),
        None => row.pull,
    };
    if let Some(pull) = &pending.pull {
        let changed = if pull.base != row.base {
            Some(crate::DriftChange::Retargeted {
                number: pull.number,
                before: row.base.clone(),
                after: pull.base.clone(),
            })
        } else if pull.state == skein_forge::api::State::Closed && pull.merged.is_none() {
            Some(crate::DriftChange::PullClosed { number: pull.number })
        } else {
            None
        };
        if let Some(change) = changed {
            let resource = skein_forge_client::Resource {
                repository: row.repository,
                what: skein_forge_client::What::Branch(row.branch.clone()),
            };
            if let Some(name) = from_client(&resource, &env.limits)
                && let Some(hold) = d.holds.get_mut(&name)
            {
                hold.drift = Some(crate::Drift { at: env.wall, change });
                save_hold(out, hold.clone());
            }
        }
    }
    if let Some(base_tip) = pending.base_tip {
        crate::settle::observe(d, env, &row, pending.pull.as_ref(), base_tip);
    }
    if row.change.released == Some(change::Hold::BranchMoved)
        && let Some(head) = pending.branch
    {
        accept_released_branch(d, env, &mut row, head, out);
    }
    let touched = step_touched(d, &row, &pending);
    d.keep_touched(row.task, pending.branch, touched);
    if let Some(repository) = d.repositories.get(&row.repository) {
        let Some(gates) = d.step_gates(&env.limits, &row, repository, touched) else {
            emit(out, Request::Refused { task: row.task });
            return;
        };
        row.change.gates = gates;
    }
    if !crate::follow_check::select(d, &mut row, &env.limits) {
        emit(out, Request::Refused { task: row.task });
        return;
    }
    // A workflow has its own runner-fault lifetime, not an agent's stall allowance.
    if crate::follow_check::asked(&row).is_some() {
        row.change.since = env.wall;
    }
    let facts = change_facts(d, env, &row, &pending, base_tip);
    let mut heard = change_standing(d, &row, &pending);
    // A close job's failure escalation is not its terminal: jig still names it
    // live, including under its Failures hold (domain/forge/change/repairs.md).
    let live_close = match row.delegate {
        Some((_, change::Delegate::Close)) => pending.heard.delegate_live,
        Some(_) | None => false,
    };
    if row.delegate_status != change::Status::Unknown && !live_close {
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
        let stepped = change::step(&row.change, &facts, &heard, env.wall, &env.limits.change_policy);
        d.review_requests.remove(&row.task);
        heard.review_request = None;
        let advanced = stepped.change != row.change;
        row.change = stepped.change;
        let decision = stepped.decision;
        let took_update = matches_updating(&effect_prior.state) && row.change.last_head != effect_prior.last_head;
        if took_update && let Some(head) = row.change.last_head {
            record_change_head(d, env, &row, head, out);
        }
        if matches_updating(&effect_prior.state) && !matches_updating(&row.change.state) && row.pending.is_none() {
            admission::change_writer(d, env, &row, false);
        }
        // If the landing tip changed, Updating's containment cannot serve Checking.
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
            change::Decision::RepairEnded { head } => {
                row.result.ci_passed(&head, env.limits.client.answer_bytes);
                row.producer_failure = None;
                row.delegate_status = change::Status::Unknown;
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
            decision @ (change::Decision::None
            | change::Decision::CancelProducer
            | change::Decision::Wait { .. }
            | change::Decision::Delegate(_)
            | change::Decision::CancelGates { .. }
            | change::Decision::CancelRepair { .. }
            | change::Decision::CancelResolution { .. }
            | change::Decision::EndHold
            | change::Decision::Ready
            | change::Decision::QueueRepair
            | change::Decision::NoChange
            | change::Decision::Finish { .. }
            | change::Decision::Cancel
            | change::Decision::Hold(_)) => {
                crate::landing::change_advanced(d, &row, &prior, decision, out);
                d.changes.insert(row.task, row.clone()).expect("replaces change");
                if landed_commit(&prior.state).is_none()
                    && let Some(commit) = landed_commit(&row.change.state)
                {
                    crate::landing_log::landed(d, env.wall, commit, row.task, Box::new([]), out);
                }
                if !prior.owns_turn && in_queue(&prior.state) && !in_queue(&row.change.state) {
                    wake_queue(d, &row);
                }
                wake_after_turn(d, &row, prior.owns_turn);
                save_queue_change(d, env, before, row.clone(), out);
                topics::refresh(d, env, row.task, out);
                emit(
                    out,
                    Request::ChangeDecision {
                        task: row.task,
                        decision: repair_poll(
                            env,
                            &row,
                            &facts,
                            &heard,
                            crate::settle::cap(d, env, &row, &facts, decision),
                        ),
                        entry: None,
                        evidence: None,
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
    let head = pending.branch.expect("follow check was asked at a read head");
    let action = match action {
        Some(action) => action,
        None => crate::follow_check::advance(d, env, &mut row, head, out),
    };
    match action {
        Action::Write => {
            let prior = row.change.clone();
            emit_change_effect(d, env, &mut row, &pending, prior, change::Effect::FollowCheck { head }, out);
        }
        Action::Read { kind, read } => {
            pending.stage = StepStage::FollowCheck(kind);
            d.sequence = d.sequence.checked_add(1).expect("follow check read sequence");
            let owner = Token::new(d.sequence | (1_u64 << 62));
            let repository = row.repository;
            d.changes.insert(row.task, row.clone()).expect("known change");
            save_change(d, out, row);
            d.steps.insert(owner, pending).expect("one read per change");
            child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
        }
        Action::Done | Action::Wait | Action::Held => {
            let task = row.task;
            let decision = match action {
                Action::Held => {
                    row.change.state =
                        change::State::Held { was: Box::new(row.change.state), why: change::Hold::Failed };
                    change::Decision::Hold(change::Hold::Failed)
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
            d.changes.insert(task, row.clone()).expect("known change");
            save_change(d, out, row);
            emit(out, Request::ChangeDecision { task, decision, entry: None, evidence: None });
            if let Action::Done = action {
                emit(out, Request::WakeProcedure { task });
            }
        }
    }
}

fn repair_poll(
    env: &Env<Limits>,
    row: &ChangeRow,
    facts: &change::Facts,
    heard: &change::Heard,
    decision: change::Decision,
) -> change::Decision {
    if let change::State::Repairing { why: change::Repair::Ci | change::Repair::Semantic, .. } = row.change.state
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
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
    if row.terminal() || row.change.way != change::Way::Queue {
        return;
    }
    if yield_approval_turn(d, &mut row) {
        d.changes.insert(task, row.clone()).expect("replaces change");
        save_change(d, out, row.clone());
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

#[expect(clippy::too_many_lines, reason = "the coherent change snapshot includes question and early-start facts")]
fn change_facts(
    d: &Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    pending: &PendingStep,
    base_tip: skein_forge::api::Commit,
) -> change::Facts {
    let verification_bound = match d.repositories.get(&row.repository) {
        Some(repository) => match d.after_landing(repository.project) {
            Some(after) => after.bound,
            None => Duration::from_secs(7200),
        },
        None => Duration::from_secs(7200),
    };
    let pull = match &pending.pull {
        Some(pull) if pull.merged.is_some() => change::Pull::Merged { commit: pull.merged.expect("checked merged") },
        Some(pull) if pull.state == skein_forge::api::State::Closed => change::Pull::Closed,
        Some(pull) => change::Pull::Open { head: pull.commit, base: pull.base_commit.unwrap_or(base_tip) },
        None => change::Pull::Missing,
    };
    let retargeted = match &pending.pull {
        Some(pull) => pull.base != row.base,
        None => false,
    };
    let requester_writing = change_requester_writing(d, row);
    let (writer_taken, requester_branch) = match change_branch_hold(d, row) {
        Some((holder, taken)) => (taken && !requester_writing || held_branch_merge(d, row), holder != row.task),
        None => (false, false),
    };
    // A fresh procedure read can precede the keep's drift read. Only a head
    // confirmed by the branch's writer is a requester's new head, except while
    // its requester's own run writes it. A read of the change's previous head
    // can still precede that confirmation.
    let mut branch_moved = false;
    if requester_branch
        && !requester_writing
        && !writer_taken
        && pending.branch.is_some()
        && pending.branch != row.change.last_head
        && !matches_updating(&row.change.state)
        && !updating_entry(d, row.task)
    {
        for (name, head) in &d.heads {
            if name.forge == row.repository.forge
                && name.repository == row.repository.repository
                && crate::items::branch_matches(name, &row.branch)
                && pending.branch != Some(head.commit)
            {
                branch_moved = true;
                break;
            }
        }
    }
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
        fault: pending.failed.fault(pending.branch.unwrap_or([0; 32])),
        pending: pending.pending.clone(),
        verification: d.change_verification(row.task),
        verification_bound,
        ask: crate::landing::ask_for(d, row),
        branch: pending.branch,
        titled: !row.title.is_empty() || row.pull.is_some(),
        expected_base: base_tip,
        pull,
        ci: change::Ci {
            head: pending.branch.unwrap_or([0; 32]),
            status: if has_ci && requires.ci { pending.ci } else { change::Status::Passed },
        },
        base_ci: change::Ci {
            head: base_tip,
            status: if !has_ci || !requires.ci || row.base_repair { change::Status::Passed } else { pending.base_ci },
        },
        base_tip,
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
        drift: if retargeted {
            Some(change::Hold::Retargeted)
        } else if branch_moved {
            Some(change::Hold::BranchMoved)
        } else {
            row.drift
        },
        first: queue_first(d, row, env.wall, &env.limits),
        may_merge,
        queue_repair_active: pending.queue_repair_active,
        kept: pending.kept,
        asking: agent_asking(row),
        settling: match window {
            Some((_, end)) => Some(end),
            None => None,
        },
    }
}

fn agent_asking(row: &ChangeRow) -> bool {
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
        touched: step_touched(d, row, pending),
        prior,
        head: pending.branch,
        base_tip: pending.base_tip.expect("step read base tip"),
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
        emit(out, Request::Refused { task: row.task });
        return;
    }
    if d.entries.len() == env.limits.entries {
        crate::refusal::resource_limit(
            out,
            row.task,
            b"emit change effect",
            None,
            "forge.entries",
            env.limits.entries,
            u64::from(d.entries.len()).saturating_add(1),
        );
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
        emit(
            out,
            Request::ChangeDecision {
                task: row.task,
                decision: change::Decision::Hold(change::Hold::Failed),
                entry: None,
                evidence: None,
            },
        );
        return;
    };
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
    crate::landing::change_advanced(d, row, &prior, change::Decision::Effect(effect), out);
    row.pending = Some(pending.entry);
    row.effect = change::EffectResult::Pending;
    row.effect_account = match effect {
        change::Effect::Rerun { head } => match pending.failed.account(head) {
            Some(words) => Some(crate::EffectAccount { entry: pending.entry, words }),
            None => None,
        },
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
    admission::update_writer(d, env, &entry, true);
    keep_entry(d, entry, None, out);
    emit(
        out,
        Request::ChangeDecision {
            task: row.task,
            decision: change::Decision::Effect(effect),
            entry: Some(pending.entry),
            evidence: Some(step_evidence(d, row, pending, prior)),
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

/// A lost update keeps its account and decides again (temper/docs/design/domain/forge/README.md, 6).
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
    let update = match d.entries.get(&entry) {
        Some(row) => match &row.entry.effect.write {
            skein_forge::api::Write::Update { .. } => true,
            skein_forge::api::Write::CreateIssue { .. }
            | skein_forge::api::Write::OpenPull { .. }
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
            | skein_forge::api::Write::DeleteBranch { .. } => false,
        },
        None => false,
    };
    let account = settled_account(d, entry, outcome);
    let Some(row) = d.changes.get_mut(&task) else { unreachable!("found change row") };
    let pull_closed = match outcome {
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
    row.effect = match outcome {
        skein_forge_client::Outcome::Made { .. } => change::EffectResult::Made,
        skein_forge_client::Outcome::Failed(skein_forge::api::Error::Conflict) => change::EffectResult::Conflict,
        skein_forge_client::Outcome::Failed(
            skein_forge::api::Error::Timeout | skein_forge::api::Error::Unavailable | skein_forge::api::Error::Stale,
        ) if update => change::EffectResult::None,
        skein_forge_client::Outcome::Failed(_)
        | skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => change::EffectResult::Failed,
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => return,
    };
    row.effect_account = account;
    row.pending = None;
    if pull_closed {
        row.drift = Some(change::Hold::PullClosed);
    }
    let saved = row.clone();
    save_change(d, out, saved.clone());
    if pull_closed {
        let branch = skein_forge_client::Resource {
            repository: saved.repository,
            what: skein_forge_client::What::Branch(saved.branch.clone()),
        };
        if from_client(&branch, &env.limits).is_some() {
            drift(
                d,
                env,
                branch,
                crate::DriftChange::PullClosed { number: saved.pull.expect("raced pull is known") },
                out,
            );
        }
    }
    match outcome {
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(commit), .. } => {
            record_change_head(d, env, &saved, commit, out);
        }
        skein_forge_client::Outcome::Made {
            made:
                skein_forge_client::Made::Created(_)
                | skein_forge_client::Made::Commented(_)
                | skein_forge_client::Made::Reviewed(_)
                | skein_forge_client::Made::Merged(_)
                | skein_forge_client::Made::Branch(_)
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
    if update && let Some(row) = d.entries.get(&entry) {
        let entry = row.entry.clone();
        admission::update_writer(d, env, &entry, false);
    }
    emit(out, Request::ChangeDecision { task, decision: change::Decision::None, entry: None, evidence: None });
}

/// The old head may still be read while an asynchronous update appears; a third head is not its answer.
fn settled_update_drift(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    observed: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get(&task) else { return };
    let change::State::Updating { from, .. } = row.change.state else { return };
    if row.pending.is_some() || row.effect != change::EffectResult::Made || observed == from {
        return;
    }
    let resource = skein_forge_client::Resource {
        repository: row.repository,
        what: skein_forge_client::What::Branch(row.branch.clone()),
    };
    let Some(name) = from_client(&resource, &env.limits) else { return };
    if let Some(head) = d.heads.get(&name)
        && head.commit != observed
    {
        let expected = head.commit;
        drift(d, env, resource, crate::DriftChange::Branch { expected, observed: Some(observed) }, out);
    }
}

fn matches_updating(state: &change::State) -> bool {
    updating_base(state).is_some()
}

fn updating_base(state: &change::State) -> Option<skein_forge::api::Commit> {
    match state {
        change::State::Updating { base, .. } => Some(*base),
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Preparing { .. }
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

/// A write's answer confirms the head before any later procedure read judges drift.
fn record_written_head(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    name: Name,
    commit: skein_forge::api::Commit,
    task: u64,
    out: &mut Queue<Request>,
) {
    let head = BranchHead { name: name.clone(), commit, owned: true };
    let concerning = heads::concerning(d, &name, Some(task));
    if heads::record(d, env, head.clone(), concerning, out) {
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head) });
    }
    child(d, env, skein_forge_client::Event::Pushed { resource, commit }, out);
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
    record_written_head(d, env, resource, name, commit, row.task, out);
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
    let head = BranchHead { name: name.clone(), commit: head, owned: true };
    if heads::record(d, env, head.clone(), Some(row.task), out) {
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head.clone()) });
    }
    child(d, env, skein_forge_client::Event::Pushed { resource, commit: head.commit }, out);
}

/// Released updates and re-runs drop failed results, retaining conflicts
/// (temper/docs/design/domain/forge/README.md, 6, and change/README.md, 2, step 3).
fn release_change(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get_mut(&task) else {
        emit(out, Request::Refused { task });
        return;
    };
    let before = crate::panel::ready(row);
    let released = change::released(&row.change, env.wall);
    let lifted = released.state != row.change.state;
    row.change = released;
    if lifted && row.effect == change::EffectResult::Failed {
        match row.change.state {
            change::State::Updating { .. }
            | change::State::Preparing { .. }
            | change::State::Checking { .. }
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
    if lifted
        && let Some(check) = &mut row.follow_check
        && check.held.is_some()
    {
        check.held = None;
        check.faults = 0;
        check.flight = None;
        check.page = 1;
    }
    row.drift = None;
    let mut saved = row.clone();
    save_queue_change(d, env, before, saved.clone(), out);
    let mut affected = List::with_capacity(d.holds.len());
    let mut observed = None;
    for (name, hold) in &d.holds {
        if hold.task == task && hold.drift.is_some() {
            affected.push(name.clone()).expect("at most all held resources");
            if let Some(drift) = &hold.drift {
                match &drift.change {
                    crate::DriftChange::Branch { observed: head, .. } => observed = *head,
                    crate::DriftChange::PullClosed { .. } | crate::DriftChange::Retargeted { .. } => {}
                }
            }
        }
    }
    for name in &affected {
        let hold = d.holds.get_mut(name).expect("collected held resource");
        hold.drift = None;
        save_hold(out, hold.clone());
    }
    if saved.change.released == Some(change::Hold::BranchMoved)
        && let Some(head) = observed
    {
        // A poll may finish before the procedure's fresh snapshot does.
        accept_released_branch(d, env, &mut saved, head, out);
    }
}

// Cancellation never writes to a pull whose merge is already recorded (domain/forge/change/README.md, 4).
fn cancellation_pull(d: &Domain, task: u64) -> Option<u64> {
    let change = d.changes.get(&task)?;
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
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
    }
}

fn settle_effects(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    root: u64,
    ending: ReleaseEnding,
    out: &mut Queue<Request>,
) {
    crate::landing::closing(d, task, ending, out);
    crate::tiers::closing(d, env, task, ending, out);
    let close_pull = closing_pull(d, task, ending);
    if let Some(row) = d.releases.get_mut(&task) {
        if row.root != root || (row.ending != ending && ending != ReleaseEnding::Cancelled) {
            emit(out, Request::ReleaseFailed { task, account: None });
            return;
        }
        if row.ending != ending {
            row.ending = ending;
            row.releasing = false;
            row.close_pull = close_pull;
            emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
        }
        if row.effects_settled {
            emit(out, Request::EffectsSettled { task });
            return;
        }
    } else {
        if d.releases.len() == env.limits.tasks {
            emit(out, Request::ReleaseFailed { task, account: None });
            return;
        }
        let row = release_row(task, root, ending, close_pull, false, false);
        d.releases.insert(task, row.clone()).expect("release capacity checked");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
    }
    if ending == ReleaseEnding::Cancelled {
        withdraw_unsent(d, env, task, out);
    }
    settle_prior_effects(d, task, out);
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
            let account = crate::effect_words::account(
                &row.entry,
                d.repository(row.entry.repository),
                skein_forge_client::Outcome::Withdrawn,
            );
            d.entries.remove(&number);
            emit(out, Request::Erase { key: Key::Entry(number) });
            d.named_outcome(number, skein_forge_client::Outcome::Withdrawn);
            emit(
                out,
                Request::Outcome { entry: number, task, outcome: skein_forge_client::Outcome::Withdrawn, account },
            );
        }
    }
}

fn settle_prior_effects(d: &mut Domain, task: u64, out: &mut Queue<Request>) {
    if let Some(issue) = d.issues.get(&task)
        && let Some(desired) = &issue.desired
        && desired.finished.is_some()
        && !issue.state.closed
    {
        return;
    }
    if !settle_held(d, task, out) {
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
    if let Some(reason) = &reason
        && reason.len() > skein_lib::usize_of(env.limits.client.op_bytes)
    {
        emit(out, Request::ReleaseFailed { task, account: None });
        return;
    }
    let close_pull = closing_pull(d, task, ending);
    if let Some(row) = d.releases.get_mut(&task) {
        if row.root != root || (row.ending != ending && ending != ReleaseEnding::Cancelled) {
            emit(out, Request::ReleaseFailed { task, account: None });
            return;
        }
        if row.ending != ending {
            row.ending = ending;
            row.releasing = false;
            row.close_pull = close_pull;
        }
        if row.failed {
            row.failed = false;
        }
        if !row.releasing && row.close_pull.is_some() {
            row.comment = reason;
        }
        row.releasing = true;
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
    } else {
        if d.releases.len() == env.limits.tasks {
            emit(out, Request::ReleaseFailed { task, account: None });
            return;
        }
        let mut row = release_row(task, root, ending, close_pull, true, true);
        if row.close_pull.is_some() {
            row.comment = reason;
        }
        d.releases.insert(task, row.clone()).expect("release capacity checked");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
    }
    continue_release(d, env, task, entry, out);
}

#[expect(clippy::too_many_lines, reason = "release drops each goal tier entry with its hold in the same transition")]
fn continue_release(d: &mut Domain, env: &Env<Limits>, task: u64, entry: u64, out: &mut Queue<Request>) {
    for _ in 0..=env.limits.holds {
        let Some(mut row) = d.releases.get(&task).cloned() else { return };
        if row.failed || row.pending.is_some() {
            return;
        }
        if !row.effects_settled {
            settle_prior_effects(d, task, out);
            return;
        }
        if !row.releasing {
            return;
        }
        if !settle_held(d, task, out) {
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
                crate::tiers::released(d, &name, out);
                emit(out, Request::Erase { key: Key::Hold(name) });
                continue;
            }
            if let Some(Writer::Lost { task: lost_task, attempt }) = writer {
                for (_, pending) in &d.lost {
                    if pending.name == name && pending.task == lost_task && pending.attempt == attempt {
                        return;
                    }
                }
            }
            if designing {
                return;
            }
            if unpublished_branch(d, &name) && writer.is_none() {
                d.holds.remove(&name);
                crate::tiers::released(d, &name, out);
                emit(out, Request::Erase { key: Key::Hold(name) });
                continue;
            }
            if goal_branch::reading(d, &name) {
                return;
            }
            if writer.is_some() {
                emit(out, Request::ReleaseFailed { task, account: None });
                return;
            }
            if goal_branch::is_goal(d, &name) {
                if !goal_branch::landed(d, &name) {
                    d.holds.remove(&name);
                    crate::tiers::released(d, &name, out);
                    emit(out, Request::Erase { key: Key::Hold(name) });
                    continue;
                }
            } else if row.ending == ReleaseEnding::Failed && row.root != task {
                let hold = d.holds.get_mut(&name).expect("selected hold");
                hold.task = row.root;
                save_hold(out, hold.clone());
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
            crate::tiers::released(d, &name, out);
            emit(out, Request::Erase { key: Key::Hold(name) });
            continue;
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
        d.releases.remove(&task);
        emit(out, Request::Erase { key: Key::Release(task) });
        emit(out, Request::Released { task });
        return;
    }
    unreachable!("every release pass removes a hold or completes");
}

// Removal leaves the remote forge untouched (temper/docs/design/domain/forge/adoption.md, 1).
fn release_pull(d: &mut Domain, env: &Env<Limits>, row: &mut ReleaseRow, entry: u64, out: &mut Queue<Request>) -> bool {
    let Some(number) = row.close_pull else { return true };
    let Some(change) = d.changes.get(&row.task) else {
        emit(out, Request::ReleaseFailed { task: row.task, account: None });
        return false;
    };
    let repository = change.repository;
    if d.forge_ended(repository.forge) || cancellation_pull(d, row.task).is_none() {
        row.close_pull = None;
        row.comment = None;
        d.releases.insert(row.task, row.clone()).expect("replaces release");
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
        true
    } else {
        let write = release_pull_write(row, &d.namespace, number, &env.limits);
        enqueue_release(d, env, row, entry, repository, write, None, out);
        false
    }
}

/// The panel outlives its task, but never its task's live state (domain/forge/change/gates.md, 1).
fn end_change(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    connection_wait::take(d, task);
    let merge = recorded_merge(d, task);
    let cancelled = match d.releases.get(&task) {
        Some(row) => row.ending == ReleaseEnding::Cancelled,
        None => false,
    };
    crate::verification::released(d, task, out);
    let Some(row) = d.changes.get_mut(&task) else { return };
    let before = crate::panel::ready(row);
    let owned = row.change.owns_turn;
    let ended = ended_change_state(&row.change.state, merge, cancelled, env.wall);
    if ended == change::State::Ended
        && row.change.ended_held.is_none()
        && let change::State::Held { why, .. } = &row.change.state
    {
        row.change.ended_held = Some(*why);
    }
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
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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
    let saved = row.clone();
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
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
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

fn release_pull_write(row: &ReleaseRow, namespace: &[u8], number: u64, limits: &Limits) -> skein_forge::api::Write {
    match &row.comment {
        Some(body) => {
            let key =
                crate::purpose::EffectPurpose::Release { task: row.task, resource: Box::from(b"reason".as_slice()) }
                    .key(namespace, limits.client.op_bytes)
                    .expect("release key fits the checked operation bound");
            skein_forge::api::Write::Post { number, key, body: body.clone() }
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
    if d.entries.len() == env.limits.entries || d.entries.contains_key(&number) {
        emit(out, Request::ReleaseFailed { task: row.task, account: None });
        return;
    }
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
    row.pending = Some((number, resource));
    d.releases.insert(row.task, row.clone()).expect("replaces release");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) });
    keep_entry(d, entry, Some(number), out);
}

fn release_outcome(d: &mut Domain, entry: u64, outcome: skein_forge_client::Outcome, out: &mut Queue<Request>) {
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
    let mut row = d.releases.get(&task).expect("selected release").clone();
    let (_, resource) = row.pending.take().expect("selected pending release");
    let removed = match d.entries.get(&entry) {
        Some(entry) => d.forge_ended(entry.entry.repository.forge),
        None => false,
    };
    if removed {
        // Removal leaves the remote forge untouched (temper/docs/design/domain/forge/adoption.md, 1).
        if let Some(name) = resource {
            d.holds.remove(&name);
            crate::tiers::released(d, &name, out);
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
                    emit(out, Request::Erase { key: Key::Hold(name) });
                }
                None => row.close_pull = None,
            },
            skein_forge_client::Outcome::Failed(_)
            | skein_forge_client::Outcome::Raced { .. }
            | skein_forge_client::Outcome::Held
            | skein_forge_client::Outcome::Withdrawn => {
                row.failed = true;
                emit(out, Request::ReleaseFailed { task, account });
            }
            skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {
                unreachable!("uncertain returned above")
            }
        }
    }
    d.releases.insert(task, row.clone()).expect("replaces release");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Release(row) });
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
    if prior.failed {
        // An Open lost to a crash consumes its committed open bit without
        // yielding an issue number. It can never be sent again, even after
        // its held outbox entry has settled (domain/forge/README.md, 4 and domain/forge/issues.md).
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
            if d.entries.len() == env.limits.entries {
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
        // crash-lost creation must never be resent (domain/forge/README.md, 4 and domain/forge/issues.md).
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
            for (change, row) in &d.changes {
                if (direct && *change == task) || (!direct && row.change.landing == Some(task)) {
                    let facts = d.review_facts.remove(change);
                    d.review_heads.remove(change);
                    if facts.is_some() {
                        emit(out, Request::Marked { tasks: Box::new([crate::APPROVALS_SUBJECT, *change]) });
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

fn handoff_one(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.stopping || d.restore_phase == RestorePhase::Restoring {
        return;
    }
    // skein's forge/client.md, 7.2: each new Make is above every held entry.
    // Fresh-read chains and goal-branch makings retain their reserved numbers; built changes,
    // projections and releases share this outbox and wait for lower chains
    // and entries to finish or commit. Hand over at most one per step.
    let Some(number) = d.next_handoff() else { return };
    let row = d.entries.get_mut(&number).expect("selected entry remains");
    row.handoff = Handoff::Handed;
    let entry = row.entry.clone();
    admission::merge_writer(d, env, &entry, true);
    let event = if outage::recommitted(d, number) {
        skein_forge_client::Event::Remake { entry }
    } else {
        skein_forge_client::Event::Make { entry }
    };
    child(d, env, event, out);
}

/// A person's choice for a held entry: again hands it back to the client,
/// which looks for it first and then makes it under a new attempt; the
/// entry kept its room, so the client has room for it.
fn resolve(d: &mut Domain, env: &Env<Limits>, number: u64, again: bool, out: &mut Queue<Request>) {
    let Some(row) = d.entries.get_mut(&number) else { return };
    if !row.held {
        return;
    }
    if !again {
        not_made(d, number, out);
        return;
    }
    row.held = false;
    let entry = row.entry.clone();
    emit(
        out,
        Request::Save { read_afresh: None, release: None, record: Stored::Entry { entry: entry.clone(), held: false } },
    );
    admission::merge_writer(d, env, &entry, true);
    child(d, env, skein_forge_client::Event::Remake { entry }, out);
}

fn erase_flight(d: &mut Domain, task: u64, entry: u64, out: &mut Queue<Request>) {
    if d.flights.remove(&(task, entry)).is_some() {
        emit(out, Request::Erase { key: Key::Flight { task, entry } });
    }
}

/// Settle one held entry as not made, freeing its room.
fn not_made(d: &mut Domain, number: u64, out: &mut Queue<Request>) {
    outage::forget_entry(d, number);
    let row = d.entries.remove(&number).expect("a held entry lives");
    emit(out, Request::Erase { key: Key::Entry(number) });
    erase_flight(d, row.entry.task, number, out);
    emit(out, Request::NotMade { entry: number, task: row.entry.task });
}

/// A closing task's held entries are settled not made, as many as the
/// output has room for, so its effects settle and their room is freed. True
/// when none of the task's entries remains; one settled here continues the
/// close in a later step.
fn settle_held(d: &mut Domain, task: u64, out: &mut Queue<Request>) -> bool {
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
        // Both possible erases, settlement, and the continuation below.
        if out.room() < 4 {
            break;
        }
        not_made(d, *number, out);
    }
    emit(out, Request::ContinueRelease { task });
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
    crate::follow_check::veto(row, entry, &unsent);
    if leaves_queue {
        row.change.owns_turn = false;
        match row.change.state {
            change::State::First { head, .. } | change::State::Queued { head, .. } => {
                row.change.state = change::State::Gating { head, asked: Box::new([]) };
            }
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
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
    d.entries.remove(&entry);
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
    match result {
        Ok(skein_forge::api::Answer::Commit(commit)) => branch_changed(d, env, resource, commit, out),
        Ok(skein_forge::api::Answer::Pull(pull)) => pull_changed(d, env, resource, pull, out),
        Ok(skein_forge::api::Answer::Item { item, comments, more: _ }) => {
            let topic = Topic::Participation { repository: resource.repository, number: item.number };
            if !comments.is_empty() {
                publish(d, topic, News::Changed { number: item.number }, out);
            }
        }
        Ok(skein_forge::api::Answer::Reviews { reviews, more: _, .. }) => match resource.what {
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

fn from_client(resource: &skein_forge_client::Resource, l: &Limits) -> Option<Name> {
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
        if row.repository.forge == name.forge
            && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
            && !d.procedure_wakes.contains_key(task)
        {
            d.procedure_wakes.insert(*task, false).expect("one pending wake per change");
        }
    }
}

fn completed_branch(d: &Domain, repository: skein_forge::api::Repository, branch: &[u8]) -> bool {
    for (_, row) in &d.changes {
        if row.repository == repository && row.branch.as_ref() == branch && row.terminal() {
            return true;
        }
    }
    false
}

fn branch_changed(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    commit: skein_forge::api::Commit,
    out: &mut Queue<Request>,
) {
    let Some(name) = from_client(&resource, &env.limits) else { return };
    // A held goal's head is confirmed by its writers and merge outcomes.
    // An observation must not replace the baseline used to detect drift.
    if goal_branch::is_goal(d, &name) && d.holds.contains_key(&name) {
        return;
    }
    if let Some(hold) = d.holds.get(&name)
        && hold.drift.is_some()
    {
        return;
    }
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
        if row.repository != repository || row.pull != Some(pull.number) {
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

fn pull_changed(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    pull: skein_forge::api::Pull,
    out: &mut Queue<Request>,
) {
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
        if row.repository == resource.repository && row.pull == Some(pull.number) {
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

#[expect(
    clippy::match_like_matches_macro,
    clippy::wildcard_enum_match_arm,
    reason = "the outbox test uses explicit matches in this strict subset"
)]
fn updating_entry(d: &Domain, task: u64) -> bool {
    let Some(change) = d.changes.get(&task) else { return false };
    // domain/landing.md, 5.2: the pending prepare's rebase moves its own branch.
    if !match change.change.state {
        change::State::Updating { .. } | change::State::Preparing { .. } => true,
        _ => false,
    } {
        return false;
    }
    let Some(number) = change.pending else { return false };
    let Some(Outboxed { entry, .. }) = d.entries.get(&number) else { return false };
    match entry.effect.write {
        skein_forge::api::Write::Update { .. } => true,
        _ => false,
    }
}

// temper/docs/design/domain/forge/change/README.md, 1 and 3: the holder's delegates share its branch.
fn change_on_held_branch(row: &ChangeRow, name: &Name, holder: u64) -> bool {
    row.task != holder
        && row.repository.forge == name.forge
        && row.repository.repository == name.repository
        && crate::items::branch_matches(name, &row.branch)
        && row.change.state != change::State::Ended
}

// The requester goes on beside its change; every other writer still stops it (domain/forge/change/README.md, 3).
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
fn settled_update_observation(
    d: &Domain,
    resource: &skein_forge_client::Resource,
    drift: &crate::DriftChange,
    limits: &Limits,
) -> bool {
    let crate::DriftChange::Branch { expected, observed: Some(observed) } = drift else { return false };
    let Some(name) = from_client(resource, limits) else { return false };
    let Some(head) = d.heads.get(&name) else { return false };
    if !head.owned || head.commit != *observed {
        return false;
    }
    for (_, row) in &d.changes {
        if row.repository == resource.repository
            && crate::items::branch_matches(&name, &row.branch)
            && row.pending.is_none()
            && row.effect == change::EffectResult::Made
            && let change::State::Updating { from, .. } = row.change.state
            && from == *expected
        {
            return true;
        }
    }
    false
}

fn drift(
    d: &mut Domain,
    env: &Env<Limits>,
    resource: skein_forge_client::Resource,
    change: crate::DriftChange,
    out: &mut Queue<Request>,
) {
    if settled_update_observation(d, &resource, &change, &env.limits) {
        return;
    }
    let mut affected = List::with_capacity(env.limits.holds);
    for (name, row) in &d.holds {
        if to_client(name, &env.limits) == Some(resource.clone()) {
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
                crate::DriftChange::Branch { .. } => change::Hold::BranchMoved,
                crate::DriftChange::PullClosed { .. } => change::Hold::PullClosed,
                crate::DriftChange::Retargeted { .. } => change::Hold::Retargeted,
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
            crate::DriftChange::Branch { .. } => change::Hold::BranchMoved,
            crate::DriftChange::PullClosed { .. } => change::Hold::PullClosed,
            crate::DriftChange::Retargeted { .. } => change::Hold::Retargeted,
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
        Stored::ForgeNumber { last } => {
            d.forge_number = d.forge_number.max(last);
        }
        Stored::Playground(row) => d.playground = Some(row),
        Stored::Follow(row) => {
            d.follow_rows.insert((row.project, row.follow), row).expect("restored follow within limits");
        }
        Stored::Tier(row) => {
            d.tiers.insert((row.repository, row.tier.clone()), row).expect("restored tier within limits");
        }
        Stored::LandingLog(row) => {
            d.landing_logs.insert(row.repository, row).expect("restored log within limits");
        }
        Stored::Verification(row) => {
            d.verifications.insert((row.repository, row.commit), row).expect("restored verification within limits");
        }
        Stored::Landing(row) => {
            d.landing.rows.insert(row.landing.task, row).expect("restored landing within limits");
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
            d.entries
                .insert(
                    entry.number,
                    Outboxed { entry, held, handoff: if held { Handoff::Handed } else { Handoff::Restored } },
                )
                .expect("restored entry within limits");
        }
        Stored::RefusedSetting(key) => {
            d.refused_settings.insert(key, ()).expect("refused setting identity fits");
        }
        Stored::Setting(row) => {
            d.setting_rows.insert(row.key, row.edit).expect("restored setting within limits");
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
            // Cross-record reconciliation is the later restart stage (domain/root.md, 8.1).
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
            d.closing.rows.insert(row.task, row).expect("restored closer room checked");
        }
        Stored::Change(row) => {
            // Volatile waits are rebuilt by stepping every restored change (domain/forge/README.md, 9).
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
            d.releases.insert(row.task, row).expect("restored release within limits");
        }
    }
}

fn restored(d: &mut Domain, env: &Env<Limits>, clock: skein_forge_client::RecoveryClock, out: &mut Queue<Request>) {
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
    d.restore_phase = RestorePhase::Restoring;
    child(d, env, skein_forge_client::Event::Restored { clock }, out);
    retire_ended(d, env, out);
    admission::writers(d, env);
    child(d, env, skein_forge_client::Event::PauseOutbox, out);
    for (_, record) in &d.records {
        if !d.forge_ended(record.number) {
            emit(out, Request::ForgeKept(record.clone()));
        }
    }
    crate::removal::restart(d, out);
    emit(out, Request::RestartDone { stage: RestartStage::Restored });
}

fn settle_outbox(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    // A held entry waits for a person, out of the client's outbox.
    let mut entries = List::with_capacity(env.limits.entries);
    for (_, row) in &d.entries {
        if !row.held && row.handoff != Handoff::Handed {
            entries.push(row.entry.clone()).expect("entry list capacity");
        }
    }
    for entry in &entries {
        // Startup reconciliation can build entries before Restored too.
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
    let mut count = d.holds.len();
    for (_, hold) in &d.holds {
        count = count.saturating_add(u32::try_from(hold.design_writers.len()).expect("bounded design writer count"));
    }
    let mut lost_runs = List::with_capacity(count);
    for (_, hold) in &d.holds {
        for writer in &hold.design_writers {
            if writer.rereading && !lost_runs.as_slice().contains(&(writer.task, writer.attempt)) {
                lost_runs.push((writer.task, writer.attempt)).expect("rereading runs bounded by holds");
            }
        }
        if let Some(Writer::Lost { task, attempt }) = hold.writer
            && !lost_runs.as_slice().contains(&(task, attempt))
        {
            lost_runs.push((task, attempt)).expect("runs bounded by holds");
        }
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
    for (task, _) in &d.changes {
        changes.push(*task).expect("change capacity");
    }
    for task in &changes {
        topics::refresh(d, env, *task, out);
    }
    for (repository, head) in &ci {
        ci_refresh(d, env, *repository, *head, out);
    }
    crate::landing::refresh(d, env, out);
    goal_branch::restart(d, env, out);
    crate::tiers::restart(d, env, out);
    child(d, env, skein_forge_client::Event::ReadAfresh, out);
}

// A permanently retired forge has no future observation to await. Its
// kept task names retain repository rows; live watches and release gates retire.
/// Drop the forge's local reads now; the event's environment retires its watches before returning.
pub(crate) fn retire_forge(d: &mut Domain, forge: u16) {
    connection_wait::retire(d, forge);
    outage::retire(d, forge);
    goal_branch::retire(d, forge);
    let mut removed = List::with_capacity(d.lost.capacity());
    for (owner, pending) in &d.lost {
        if pending.name.forge == forge {
            removed.push(*owner).expect("bounded lost reads");
        }
    }
    for owner in removed.into_boxed() {
        d.lost.remove(&owner);
    }
    let mut ci = List::with_capacity(d.pending_ci.len());
    for (owner, pending) in &d.pending_ci {
        if pending.repository.forge == forge {
            ci.push(*owner).expect("bounded CI reads");
        }
    }
    for owner in ci.into_boxed() {
        d.pending_ci.remove(&owner);
    }
    let mut files = List::with_capacity(d.goal_file_reads.capacity());
    for (owner, pending) in &d.goal_file_reads {
        if pending.repository.forge == forge {
            files.push(*owner).expect("bounded goal file reads");
        }
    }
    for owner in files.into_boxed() {
        d.goal_file_reads.remove(&owner);
    }
    let mut steps = List::with_capacity(d.steps.capacity());
    for (owner, pending) in &d.steps {
        if let Some(row) = d.changes.get(&pending.task)
            && row.repository.forge == forge
        {
            steps.push(*owner).expect("bounded change reads");
        }
    }
    for owner in steps.into_boxed() {
        d.steps.remove(&owner);
    }
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
            settled_facts(d, env, number, task, skein_forge_client::Outcome::Withdrawn, out);
            not_made(d, number, out);
            if let Some(row) = d.releases.get(&task)
                && !row.failed
            {
                emit(out, Request::ContinueRelease { task });
            }
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
            emit(
                out,
                Request::ChangeDecision { task, decision: change::Decision::Cancel, entry: None, evidence: None },
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

/// One pass over the forge's repositories (temper/docs/design/domain/forge/adoption.md, 1).
pub(crate) fn erase_removed_repositories(d: &mut Domain, forge: u16, out: &mut Queue<Request>) -> bool {
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
        d.repositories.remove(&repository);
        crate::landing_log::erased(d, repository, out);
        crate::drafts::erase_repository(d, repository, out);
        emit(out, Request::Erase { key: Key::Repository(repository) });
        emit(out, Request::Erase { key: Key::Client(skein_forge_client::Key::Repository(repository)) });
    }
    !kept
}

fn read_afresh_complete(d: &mut Domain, out: &mut Queue<Request>) {
    if d.restart_fresh == Some(true)
        && d.landing.pending.is_empty()
        && d.pending_ci.is_empty()
        && d.goal_file_reads.is_empty()
        && d.lost.is_empty()
        && d.makings.is_empty()
    {
        d.restart_fresh = None;
        emit(out, Request::RestartDone { stage: RestartStage::ReadAfresh });
    }
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

/// Every saved record invalidates its own panel (domain/forge/change/README.md, 5).
pub(crate) fn save_change(d: &mut Domain, out: &mut Queue<Request>, mut row: ChangeRow) {
    if row.terminal() {
        connection_wait::take(d, row.task);
    }
    retain_terminal_ci(d, &mut row);
    let task = row.task;
    let mut tasks = List::with_capacity(crate::MARKED);
    tasks.push(task).expect("own subject");
    if let Some(landing) = row.change.landing {
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
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Change(Box::new(row)) });
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
        }
    }
    let mut tasks = List::with_capacity(crate::MARKED);
    tasks.push(row.task).expect("own subject");
    if let Some(landing) = row.change.landing {
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
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Change(Box::new(row)) });
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
// The tests pin both grammars to the same boundary cases (protocol/setup/owned.md, 7).
fn setup_login(name: &[u8]) -> bool {
    temper_client_protocol_wire::source::org(name)
}
fn setup_repository(name: &[u8]) -> bool {
    temper_client_protocol_wire::source::repository(name)
}

/// Pack a repository's complete forge identity (domain/forge/adoption.md).
#[must_use]
pub const fn tag(forge: u16, repository: u32) -> u64 {
    let [forge_high, forge_low] = forge.to_be_bytes();
    let [one, two, three, four] = repository.to_be_bytes();
    u64::from_be_bytes([0, 0, forge_high, forge_low, one, two, three, four])
}

/// Decode a repository tag, rejecting bits outside the forge identity's width.
#[must_use]
pub const fn untag(tag: u64) -> Option<(u16, u32)> {
    let [high, low, forge_high, forge_low, one, two, three, four] = tag.to_be_bytes();
    if high != 0 || low != 0 {
        None
    } else {
        Some((u16::from_be_bytes([forge_high, forge_low]), u32::from_be_bytes([one, two, three, four])))
    }
}

// One rule for a producer's failure and the box pipe closing (domain/forge/adoption.md, 1).
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
    /// Find a referencing row only on an unmade link (temper/docs/design/domain/root.md, 8.1).
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
            RecordKind::Forge
            | RecordKind::Landed
            | RecordKind::Binding
            | RecordKind::Setting
            | RecordKind::Making
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

pub(crate) fn branch_resource_name(repository: &Repository, branch: &[u8]) -> Option<Box<[Box<[u8]>]>> {
    let mut segments = List::uncapped();
    for segment in [b"forge".as_slice(), &repository.host, &repository.owner, &repository.name, b"branch"] {
        if segment.is_empty() {
            return None;
        }
        segments.push(Box::from(segment)).ok()?;
    }
    let mut start = 0;
    for at in 0..=branch.len() {
        if at == branch.len() || branch.get(at) == Some(&b'/') {
            let part = branch.get(start..at)?;
            if part.is_empty() {
                return None;
            }
            segments.push(Box::from(part)).ok()?;
            start = at.checked_add(1)?;
        }
    }
    Some(segments.into_boxed())
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

// The worker wire uses two four-byte lengths; the request is C2's compact JSON.
// temper/docs/design/domain/worker/jobs.md; temper/docs/design/domain/forge/change/repairs.md.
fn merge_available(d: &Domain, row: &ChangeRow, pending: &PendingStep, base_tip: [u8; 32]) -> bool {
    if !pending.merge_job {
        return false;
    }
    let Some((bound, _)) = d.merge_kind else { return false };
    let Some(repository) = d.repositories.get(&row.repository) else { return false };
    let Some(command) = d.merge_job(repository.project) else { return false };
    let head = pending.branch.unwrap_or(base_tip);
    let base = row.catch_up.unwrap_or(base_tip);
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
