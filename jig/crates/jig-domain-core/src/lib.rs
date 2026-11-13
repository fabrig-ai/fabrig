//! The engine core composes the task hub, policy, parties, fleet, accounts,
//! briefs, notes and views. It keeps work in jig's vocabulary and asks its
//! application's root to route the store, hosts, parties and connectors.
//! See `domain/engine/README.md`, sections 3 to 5.
//!
//! The core never holds a connector's value or a store encoding.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]

pub mod acts;
mod admission;
pub mod documents;
pub mod effects;
mod funding;
mod inbox;
mod limits;
use jig_domain_core_vocabulary::numbers;
pub mod public;
pub mod reads;
mod routing;
pub mod runs;
pub mod settings;
pub mod start;
mod stored;
#[cfg(test)]
mod tests;
use jig_domain_core_vocabulary::translate;
pub(crate) mod vocabulary;

extern crate alloc;

use crate::{
    admission::{deletion, policy},
    effects::resources,
    reads::watches::{WatchBinding, WatchedSubject},
    routing::{flight, task::erase},
    runs::state::RetainedResult,
    settings::{charters, fleet_needs, templates},
};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_fleet as fleet;
use jig_domain_link::Problem;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{List, Map, Queue, Wall};

pub use crate::{
    admission::{
        amendments::TaskAmendDenied,
        policy::{grants_to_authority, snapshot as policy_snapshot},
    },
    documents::Offered,
    effects::{
        EffectKey, EffectOrigin, EffectPurpose,
        projections::{
            MilestoneId, Projection, ProjectionFeed, ProjectionGoal, ProjectionKey, ProjectionMilestone,
            ProjectionRecord, ProjectionTask, finish_decision, projection_bytes, projection_record_bytes,
        },
    },
    funding::pools::CreateStart,
    limits::{
        CORE_SECTIONS, DOCUMENT_HEADING_BYTES, DOCUMENT_KIND, DOCUMENT_PATH_BYTES, DOCUMENT_PRIORITY,
        brief_holds_documents, charter_owned_bytes, overlay_owned_bytes, parts_owned_bytes, policy_owned_bytes, valid,
    },
    numbers::{Counters, Deployment, Family, fresh},
    public::{
        PublicAnswer, PublicApplication, PublicBoard, PublicCard, PublicCounts, PublicDocument, PublicExecutor,
        PublicGoal, PublicHold, PublicLine, PublicPhase, PublicProject, PublicProjects, PublicRead,
    },
    reads::{ProposalRead, SettingsAnswer, SettingsRead},
    routing::{
        fire, resume_fleet,
        sibling_routes::{MadeRoute, PersonMessage, SentRoute},
        step, want_accounts,
    },
    runs::{
        brief_text::{BriefPart, BriefTextLimits, note_index_text, read_brief_part},
        conversation::{DeliveryOutcome, Settled, SettledAnswer, SettledCall},
        delegation::{
            Delegate, Delegating, Dependency, ProcedureStep, StepAmendment, StepCancel, StepEnd, StepMessage,
            resolved_delegate_authority, symbolic_grants,
        },
        host_rule::HostRule,
        live::LiveAsk,
        run_choice::{RunAllowance, RunChoice, RunWait},
        state::{
            EFFORT_BYTES, EFFORTS, GoalRoute, MODEL_DESCRIPTION_BYTES, MODEL_NAME_BYTES, Model, PendingRelay,
            PersonProposalRoute, PersonTaskRoute, RestoringProof, RoutedCall, RunAdmission, RunCharter, RunPolicy,
            Transcript, most_costly, reserve_floor,
        },
        transcript_text::{CUT_TURN_TEXT, TranscriptRow, TranscriptText},
    },
    settings::{
        charters::{Charter, CharterPart, Document},
        templates::Template,
    },
    start::{Census, HeldLimit, Linked, ReadResults, Restore, RestoreRange, restore_cursor},
    stored::{
        CallKey, CallPart, CallRecord, CallReplay, CoreKey, CoreRecord, CurrentWindow, EscalationDecisionRecord, Key,
        Made, ProposalDecisionRecord, Record, Restored, RunProof, TerminalRecord, TurnProof, TurnRecord,
    },
    translate::{authority_to_people, pattern_to_people, people_authority, requirement_to_authority, task_authority},
    vocabulary::{
        Ask, Assignment, Carried, CoreBriefBudgets, EscalationChoice, Event, Held, HoldingsRefusal, Limits, LinkNow,
        NamedAction, Now, PayloadRefusal, PlannedDocument, ProposalChoice, PublicIndexLimits, Request, StepRefusal,
        Timer, ToolKind, TranscriptLoad, WatchRefusal, Write,
    },
};

pub use jig_domain_core_vocabulary::{
    Config, ConfiguredPolicy, EndpointFacts, RECURRING, SeedProject, Settings, ToolFamilies,
};

/// The engine's children, routing and durable live state (domain/engine/README.md,
/// section 3). Configuration limits stay immutable (programming-model.md,
/// section 10.1); the hand-off bound also validates restored projection history.
#[derive(Hash, Debug)]
#[expect(clippy::partial_pub_fields, reason = "the keyed escalation route remains private to the core")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "pending generation, worker placement, stopping and role-table fullness are independent facts"
)]
pub struct Core {
    erasures: erase::Erasures,
    deletions: deletion::Deletions,
    pub(crate) start: start::Script,
    person_answers: bool,
    routes: u32,
    run_inbox: u32,
    pub(crate) run_now: skein_lib::Time,
    pub(crate) run_wall: Wall,
    /// Last reported store map bytes (protocol/store/README.md, 6.2).
    pub fill: u64,
    pub(crate) workers_in_process: bool,
    pub(crate) stopping: bool,
    pub(crate) settling: bool,
    pub(crate) draining: Option<bool>,
    pub(crate) question_activity: Map<(u64, u64), tasks::ActivityLine>,
    pub(crate) census_progress: Map<u64, inbox::Progress>,
    pub(crate) census_position: Option<(u64, u64, u64)>,
    pub(crate) ended_chats: Map<u64, inbox::EndedChat>,
    pub(crate) proposed_input: Option<Result<Box<settings::Proposal>, people::Malformed>>,
    pub(crate) settings_proposals: Map<(people::Proposer, u32), settings::Proposal>,
    pub(crate) kinds: Box<[settings::Kind]>,
    pub(crate) restoring: settings::Restoring,
    pub(crate) settings_judging: settings::application::Judging,
    pub(crate) settings_input: Option<(Result<Box<settings::Change>, people::Refusal>, u32, bool)>,
    pub(crate) key_input: Option<Carried>,
    pub(crate) application_input: Option<routing::application_routes::Input>,
    pub(crate) task_capacity_mark: u64,
    pub(crate) settings_holders: u32,
    pub(crate) opened: bool,
    pub(crate) start_pending: bool,
    pub(crate) restart_attempts: u32,
    /// Application connectors addressed only by number.
    pub connectors: Box<[u16]>,
    pub(crate) jobs: Box<[jig_domain_link::JobBound]>,
    /// Subjects owned by the parent; the child echoes each watch token.
    pub(crate) view_names: skein_lib::Slab<WatchedSubject>,
    pub(crate) changed_views: skein_lib::Set<skein_lib::Id<WatchedSubject>>,
    pub(crate) reads: skein_lib::Slab<reads::Pending>,
    /// Retained dependency results held for preparation.
    pub(crate) dependency_results: Map<u64, Box<[RetainedResult]>>,
    /// Retained result positions waiting to be delivered.
    pub ending_positions: Map<u64, u64>,
    /// Child requests awaiting their typed continuation.
    pub(crate) flight_people_capacity: u32,
    pub(crate) flight_tasks_capacity: u32,
    pub(crate) flights: skein_lib::Slab<flight::Flight>,
    pub(crate) person_task_capacity: u32,
    pub(crate) adoptions: Map<resources::Key, resources::Adoption>,
    pub(crate) resource_roles: Map<resources::Key, effects::connector::ResourceRole>,
    pub(crate) resource_roles_full: bool,
    /// At most one committed word per run awaiting host delivery.
    pub(crate) relaying: Map<u64, PendingRelay>,
    /// Configured project identifiers for bootstrap.
    pub projects: List<u32>,
    pub current: settings::Current,
    pub(crate) held: settings::Held,
    pub(crate) hub_turns: u32,
    pub(crate) settings_restore: settings::restart::Restore,
    pub(crate) restore_failure: Option<start::Failure>,
    /// Names seeded or restored with the project policies (domain/model.md, 6.1).
    pub(crate) names: Map<u32, Box<str>>,
    visibility: admission::visibility::Visibility,
    public_index: public::Index,
    public_reads: skein_lib::Slab<Option<public::reads::Pending>>,
    /// Mutable connector permission mappings approved by policy edits.
    pub permission_roles: Map<u32, Box<[people::PermissionRole]>>,
    /// Template overrides retained with each project policy.
    pub(crate) template_overrides: Map<u32, Box<[people::TemplateOverride]>>,
    pub(crate) seed_projects: Box<[SeedProject]>,
    pub(crate) seed_owners: Box<[people::InitialOwner]>,
    /// Immutable settings for core decisions.
    pub settings: Settings,
    /// Named calls awaiting their routed decision.
    pub pending_calls: Map<CallKey, bool>,
    /// Durable live named-call decisions, with connector payloads retained by
    /// their owner and represented here only by connector number.
    pub call_parts: Map<CallKey, CallPart>,
    /// Settled host answers and opaque delivery evidence under the same retained names.
    pub call_settled: Map<CallKey, SettledCall>,
    /// Live reply rights waiting for a committed effect or its answer deadline.
    pub(crate) effect_replies: Map<u64, effects::Waiting>,
    /// Connectors whose outboxes must finish before a task closes.
    pub(crate) closing_connectors: Map<(u64, u16), ()>,
    pub(crate) projections: Map<u64, Box<Projection>>,
    /// Preparations waiting for notes, or deferring their next activation for load room,
    /// at most one per live task (`jig/docs/design/domain/engine/briefs.md`, 2.2).
    pub(crate) pending_note_briefs: Map<u64, routing::NoteBrief>,
    pub(crate) preferences: routing::run::preferences::State,
    pub(crate) now: routing::run::now::State,
    /// Durable deployment numbers.
    pub counters: Counters,
    /// Live task preparation contexts.
    pub(crate) contexts: Map<u64, Box<tasks::RunContext>>,
    pub(crate) workspace_credentials: Map<u64, Box<[u32]>>,
    pub(crate) job_workspaces: Map<u64, Box<[u8]>>,
    /// Recent transcript windows for runs being prepared.
    pub transcripts: Map<u64, Transcript>,
    /// Current durable claim evidence.
    pub proofs: Map<u64, RunProof>,
    /// Durable proof kept aside until a replacement claim commits.
    pub replaced: Map<u64, RunProof>,
    /// Correlation while current proofs are restored.
    pub(crate) restoring_proofs: Map<u64, RestoringProof>,
    /// Restored claims not yet reported by a host.
    pub(crate) unreported_restored: Map<u64, u64>,
    /// Claims awaiting completion of their route: attempt and prepared offered kinds.
    pub claiming: Map<u64, (u64, tasks::Reasons, Option<tasks::SessionEnd>)>,
    /// Child work due to activation.
    pub(crate) due: Queue<Box<tasks::RunContext>>,
    /// Host adoption events held through startup.
    pub(crate) adopted: Queue<fleet::Event>,
    /// The task hub.
    pub tasks: tasks::Domain,
    /// Policy and authority.
    pub authority: authority::Domain,
    /// Parties and authenticated requests.
    pub people: people::Domain,
    /// Hosts and attempts.
    pub fleet: fleet::Domain,
    /// Secret-free account lifetimes.
    pub accounts: accounts::Domain,
    /// Run brief gathering.
    pub brief: brief::GatherDomain,
    /// Scoped notes.
    pub notes: notes::Domain,
    /// Live views.
    pub views: views::Domain,
}

fn assert_connector_registry(connectors: &[u16]) {
    assert!(!connectors.contains(&RECURRING), "the core reserves its recurring connector number");
    for connector in connectors {
        let mut count = 0_u32;
        for other in connectors {
            if connector == other {
                count = count.checked_add(1).expect("bounded connector registry");
            }
        }
        assert_eq!(count, 1, "connector numbers are unique");
    }
}

impl Core {
    /// The kept project name for a root's read context (domain/model.md, 6.1).
    #[must_use]
    pub fn project_name(&self, project: u32) -> Option<&str> {
        match self.names.get(&project) {
            Some(name) => Some(name),
            None => None,
        }
    }

    /// The templates `project` offers: the current ones under its overrides, as `me` delivers them
    /// (jig/docs/design/domain/engine/settings.md, 2).
    #[must_use]
    pub fn project_templates(&self, project: u32) -> Box<[Template]> {
        let overrides = match self.template_overrides.get(&project) {
            Some(entries) => entries.as_ref(),
            None => &[],
        };
        templates::current_merged(&self.current, overrides)
    }

    /// Templates the active settings decision would offer this project, including its policy
    /// overrides. `owner` is the token from `JudgeSettings`; accepted settings remain unchanged
    /// until every judge accepts (jig/docs/design/domain/engine/settings.md, 1–2).
    #[must_use]
    pub fn settings_project_templates(&self, owner: skein_lib::Token, project: u32) -> Box<[Template]> {
        settings::application::project_templates(self, owner, project)
    }

    /// Allocate the eight children and the bounded live core tables.
    /// # Errors
    /// A configured worker count or name exceeds the priced membership bounds.
    #[expect(clippy::too_many_lines, reason = "construct all bounded children, bindings and subject marks together")]
    pub fn new(mut config: Config, limits: &Limits) -> Result<Core, Problem> {
        let worker_names = limits::problems(core::mem::replace(&mut config.worker_names, Box::new([])), limits)?;
        assert!(limits.people.words <= limits.tasks.message_bytes, "party words fit one run message");
        assert!(limits.tasks.message_bytes >= 24, "projection refusal news fits one run message");
        assert!(config.settings.tools.valid(), "each engine tool has one configured authority bit");

        assert!(
            config.connectors.len() <= skein_lib::length::usize_of(limits.connectors),
            "core prices every configured connector number"
        );
        assert_connector_registry(&config.connectors);
        let mut previous = None;
        for job in &config.jobs {
            if let Some(kind) = previous {
                assert!(kind < job.kind, "job kinds are ascending and distinct");
            }
            assert!(job.spec <= limits.tasks.spec_bytes, "job specs fit the tasks child");
            assert!(job.result <= limits.tasks.result_bytes, "job results fit the tasks child");
            previous = Some(job.kind);
        }
        for grant in &config.grants {
            assert!(config.connectors.contains(&grant.connector), "acts name configured connectors");
        }
        assert!(
            config.settings.resume_bytes > 0 && config.settings.resume_bytes <= limits.resume_bytes,
            "core prices its transcript retention"
        );
        assert!(
            config.settings.tail_bytes <= limits.brief_core_budgets.tail,
            "core prices its transcript text retention"
        );
        if config.defaults.models.is_empty() {
            config.defaults.models = settings::catalogue::defaults(&config.settings);
        }
        config.settings.models = settings::catalogue::rows(&config.defaults.models);
        charters::assert_catalogue(&config.settings);
        assert!(
            config.seed_projects.len() <= skein_lib::length::usize_of(limits.authority.projects),
            "core prices seed projects"
        );
        assert!(config.defaults.policies.capacity() <= limits.authority.projects, "core prices configured policies");
        assert!(
            config.seed_projects.len() < skein_lib::length::usize_of(limits.people.projects),
            "people prices seed rosters and the deployment's"
        );
        for (index, project) in config.seed_projects.iter().enumerate() {
            assert!(project.number != 0, "seed project numbers start at one");
            assert!(people::name_rule(&project.name).is_none(), "seed project names are in shape");
            for previous in config.seed_projects.iter().take(index) {
                assert!(project.number != previous.number, "seed project numbers are unique");
                assert!(!people::same_name(&project.name, &previous.name), "seed project names are unique");
            }
            let value =
                policy::snapshot(&project.policy.policy, &project.policy.permissions, &project.policy.templates)
                    .expect("seed policy snapshot");
            assert!(
                match people::policy_bytes(&value) {
                    Some(bytes) => bytes <= limits.policy_bytes,
                    None => false,
                },
                "core prices seed policies"
            );
        }
        let fleet_needs = fleet_needs::configured(&config, limits);
        let mut current = settings::Current::new(config.defaults, limits, &config.kinds);
        config.settings.models = settings::catalogue::table(&current);
        charters::rebuild(&mut current, limits);
        for project in &config.seed_projects {
            assert!(
                project.policy.templates.is_empty()
                    || templates::valid(&current, &config.settings, &project.policy.templates),
                "merged template choices fit their effective executors"
            );
        }
        let authority = authority::Domain::new(current.rules().clone(), limits.authority).expect("configured rules");
        Ok(Core {
            erasures: erase::Erasures::new(),
            deletions: deletion::Deletions::new(limits.people.projects),
            routes: limits.routes,
            run_inbox: limits.tasks.run_inbox,
            fill: 0,
            person_answers: false,
            workers_in_process: config.workers_in_process,
            stopping: false,
            settling: false,
            draining: None,
            question_activity: Map::with_capacity(inbox::question_capacity(limits).expect("validated question bound")),
            census_progress: Map::with_capacity(limits.people.people),
            census_position: None,
            ended_chats: Map::with_capacity(
                limits
                    .people
                    .people
                    .checked_mul(limits.people.inbox_entries)
                    .expect("validated archive cache product")
                    .checked_add(1)
                    .expect("validated archive cache bound"),
            ),
            proposed_input: None,
            settings_proposals: Map::with_capacity(limits.settings.proposals),
            settings_input: None,
            restoring: settings::Restoring::new(&config.kinds),
            settings_judging: settings::application::Judging::new(&config.kinds),
            kinds: config.kinds,
            key_input: None,
            application_input: None,
            task_capacity_mark: 0,
            settings_holders: 0,
            start: start::Script::new(),
            opened: false,
            start_pending: false,
            restart_attempts: limits.fleet.attempts,
            counters: Counters::bootstrap(config.deployment),
            seed_owners: config.owners.clone(),
            seed_projects: config.seed_projects,
            view_names: skein_lib::Slab::with_capacity(limits.views.watchers),
            changed_views: skein_lib::Set::with_capacity(limits.views.watchers),
            reads: skein_lib::Slab::with_capacity(limits.load_slots.saturating_add(2)),
            dependency_results: Map::with_capacity(limits.tasks.tasks),
            ending_positions: Map::with_capacity(limits.tasks.tasks),
            flight_people_capacity: limits.people.pending,
            flight_tasks_capacity: limits.tasks.tasks,
            flights: skein_lib::Slab::with_capacity(u32::MAX),
            person_task_capacity: limits.people.pending,
            adoptions: Map::uncapped(),
            resource_roles: Map::with_capacity(limits.tasks.resource_reports),
            resource_roles_full: false,
            relaying: Map::with_capacity(limits.tasks.tasks),
            projects: List::with_capacity(limits.authority.projects),

            names: Map::with_capacity(limits.authority.projects),
            visibility: admission::visibility::Visibility::new(limits.authority.projects),
            public_index: public::Index::new(limits.people.projects, limits.tasks.depth),
            public_reads: skein_lib::Slab::with_capacity(limits.load_slots),
            permission_roles: Map::with_capacity(limits.authority.projects),
            template_overrides: Map::with_capacity(limits.authority.projects),
            connectors: config.connectors,
            jobs: config.jobs,
            pending_calls: Map::with_capacity(limits.call_records),
            call_parts: Map::with_capacity(limits.call_records),
            call_settled: Map::with_capacity(limits.call_records),
            effect_replies: Map::with_capacity(limits.call_records),
            closing_connectors: Map::with_capacity(
                limits.tasks.tasks.checked_mul(limits.connectors).expect("bounded closing connectors"),
            ),
            projections: Map::with_capacity(limits.tasks.tasks),
            pending_note_briefs: Map::with_capacity(limits.tasks.tasks),
            preferences: routing::run::preferences::State::new(limits.brief.briefs),
            now: routing::run::now::State::new(limits.brief.briefs),
            run_now: skein_lib::Time::ZERO,
            run_wall: Wall::EPOCH,
            contexts: Map::with_capacity(limits.tasks.tasks),
            workspace_credentials: Map::with_capacity(limits.tasks.tasks),
            job_workspaces: Map::with_capacity(limits.tasks.tasks),
            transcripts: Map::with_capacity(limits.tasks.tasks),
            proofs: Map::with_capacity(limits.tasks.tasks),
            replaced: Map::with_capacity(limits.tasks.tasks),
            restoring_proofs: Map::with_capacity(limits.tasks.tasks),
            unreported_restored: Map::with_capacity(limits.tasks.tasks),
            claiming: Map::with_capacity(limits.tasks.tasks),
            due: Queue::uncapped(),
            adopted: Queue::uncapped(),
            tasks: {
                let mut tasks = tasks::Domain::new(
                    &limits.tasks,
                    tasks::Kept { revisions: limits.settings.revisions, versions: limits.settings.versions },
                    config.seed,
                    current.newest(),
                    charters::models(&current, &config.settings, limits),
                    charters::efforts(&config.settings),
                );
                tasks.set_kept_starts(config.kept_starts);
                tasks
            },
            current,
            held: settings::Held::new(limits),
            hub_turns: config.hub_turns,
            settings_restore: settings::restart::Restore::new(limits),
            restore_failure: None,
            authority,
            people: people::Domain::new(
                &limits.people,
                config.owners,
                config.settings.deployment_provider,
                config.grants,
            ),
            fleet: {
                let mut fleet = fleet::Domain::new(&limits.fleet, limits.settings.versions, fleet_needs);
                fleet.configure_workers(worker_names);
                fleet
            },
            accounts: accounts::Domain::new(&limits.accounts, config.settings.llm_providers.clone()),
            brief: brief::GatherDomain::new(&limits.brief),
            notes: notes::Domain::new(&limits.notes),
            views: views::Domain::new(&limits.views),
            settings: config.settings,
        })
    }
}

pub use effects::connector::PersonSeed;
