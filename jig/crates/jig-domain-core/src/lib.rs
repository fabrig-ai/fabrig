//! The engine core composes the task hub, policy, parties, fleet, accounts,
//! briefs, notes and views. It keeps work in jig's vocabulary and asks its
//! application's root to route the store, hosts, parties and connectors.
//! See `domain/engine.md`, sections 3 to 5.
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
mod numbers;
pub mod reads;
mod routing;
pub mod runs;
pub mod settings;
pub mod start;
mod stored;
#[cfg(test)]
mod tests;
mod translate;
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
    effects::{
        EffectKey, EffectOrigin, EffectPurpose,
        projections::{
            MilestoneId, Projection, ProjectionFeed, ProjectionGoal, ProjectionKey, ProjectionMilestone,
            ProjectionRecord, ProjectionTask, finish_decision, projection_bytes, projection_record_bytes,
        },
    },
    funding::pools::CreateStart,
    limits::{Problem, policy_owned_bytes, valid},
    numbers::{Counters, Deployment, Family, fresh},
    reads::{SettingsAnswer, SettingsRead},
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
            Transcript, reserve_floor,
        },
        transcript_text::{TranscriptRow, TranscriptText},
    },
    settings::{charters::Charter, templates::Template},
    start::{Census, HeldLimit, ReadResults, Restore, RestoreRange, restore_cursor},
    stored::{
        CallKey, CallPart, CallRecord, CallReplay, CoreKey, CoreRecord, CurrentWindow, EscalationDecisionRecord, Key,
        Made, ProposalDecisionRecord, Record, Restored, RunProof, TerminalRecord, TurnProof, TurnRecord,
    },
    translate::{authority_to_people, pattern_to_people, people_authority, requirement_to_authority, task_authority},
    vocabulary::{
        Ask, Carried, CoreBriefBudgets, EscalationChoice, Event, Held, Limits, NamedAction, Now, PayloadRefusal,
        ProposalChoice, Request, StepRefusal, Timer, ToolKind, TranscriptLoad, WatchRefusal, Write,
    },
};

/// Policy copied into a later project (configuration.md, 3; core.md, 6.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConfiguredPolicy {
    pub policy: authority::Policy,
    pub permissions: Box<[people::PermissionRole]>,
    pub templates: Box<[people::TemplateOverride]>,
}

/// One first-start project seed (configuration.md, 3; core.md, 6.1).
#[derive(Debug)]
pub struct SeedProject {
    pub number: u32,
    pub name: Box<str>,
    pub policy: ConfiguredPolicy,
}

/// Application-supplied initial core state, in jig's vocabulary
/// (`engine.md`, 6; `configuration.md`, 3).
#[derive(Debug)]
pub struct Config {
    /// Configured worker names in authenticated protocol position order.
    pub worker_names: Box<[Box<[u8]>]>,
    /// Identity committed with the first decision.
    pub deployment: [u8; 16],
    /// Deterministic task scheduling seed.
    pub seed: u64,
    /// Workers share this engine's process and cannot outlive it.
    pub workers_in_process: bool,
    /// Normal retained turns per attempt in the deployment's hub.
    pub hub_turns: u32,
    /// Initial authenticated owners.
    pub owners: Box<[people::InitialOwner]>,
    /// Configured deployment acts for each connector (people.md, 5.3).
    pub grants: Box<[people::ActGrant]>,
    /// Configured authority and policy.
    pub defaults: settings::Defaults,
    /// Deployment and run settings retained by the core.
    pub settings: Settings,
    /// Projects used only by the first start (configuration.md, 3).
    pub seed_projects: Box<[SeedProject]>,
    /// Numbered connectors the application routes for task lifecycle handoffs.
    pub connectors: Box<[u16]>,
}

/// Immutable deployment and run settings used by core decisions.
#[derive(Debug)]
pub struct Settings {
    /// Provider number for the application's services.
    pub deployment_provider: u16,
    /// Configured models in configuration's order: model `i` is bit `i` of an authority's
    /// models and the `i` a task's LLM choice names (authority.md, 3; tasks.md, 3).
    pub models: Box<[Model]>,
    /// Maximum committed conversation bytes resumed whole.
    pub resume_bytes: u32,
    /// Maximum transcript text retained for a fresh run's brief.
    pub tail_bytes: u32,
    /// Jig's one-bit authority family for each engine tool.
    pub tools: ToolFamilies,
    /// Configured LLM providers, for the accounts child (accounts.md, 3.1).
    pub llm_providers: accounts::Providers,
    /// Actual client key ceiling, supplied by startup after protocol limits are checked.
    pub key_bytes: u32,
    /// The driver's redirect bound: the most bytes a pasted sign-in address may have.
    pub redirect_bytes: u32,
}

/// Jig's authority bits for the engine's tool families.
#[derive(Clone, Copy, Debug)]
pub struct ToolFamilies {
    pub delegate: authority::Tools,
    pub message: authority::Tools,
    pub control: authority::Tools,
    pub decide: authority::Tools,
    pub propose: authority::Tools,
    pub subscribe: authority::Tools,
    pub effect: authority::Tools,
    pub note: authority::Tools,
    pub recall: authority::Tools,
    pub read: authority::Tools,
    pub settings: u64,
}

/// Connector number reserved for the core's recurring procedure (tasks.md, 10).
pub const RECURRING: u16 = u16::MAX;

impl ToolFamilies {
    /// Every offered family has one distinct authority bit.
    #[must_use]
    pub fn valid(&self) -> bool {
        let mut seen = 0_u64;
        for family in [
            self.delegate,
            self.message,
            self.control,
            self.decide,
            self.propose,
            self.subscribe,
            self.effect,
            self.note,
            self.recall,
            self.read,
            authority::Tools(self.settings),
        ] {
            if family.0.count_ones() != 1 || seen & family.0 != 0 {
                return false;
            }
            seen |= family.0;
        }
        true
    }

    /// The eleven engine tool families, numbered alike in every deployment.
    pub const JIG: Self = Self {
        delegate: authority::Tools(1),
        message: authority::Tools(2),
        control: authority::Tools(4),
        decide: authority::Tools(8),
        propose: authority::Tools(16),
        subscribe: authority::Tools(32),
        effect: authority::Tools(64),
        note: authority::Tools(128),
        recall: authority::Tools(256),
        read: authority::Tools(512),
        settings: 1024,
    };
}

/// The engine's children, routing and durable live state (domain/engine.md,
/// section 3). Configuration limits stay immutable (programming-model.md,
/// section 10.1); the hand-off bound also validates restored projection history.
#[derive(Debug)]
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
    /// Last reported store map bytes (protocol/store.md, 11.2).
    pub fill: u64,
    pub(crate) workers_in_process: bool,
    pub(crate) stopping: bool,
    pub(crate) draining: Option<bool>,
    pub(crate) question_activity: Map<(u64, u64), tasks::ActivityLine>,
    pub(crate) census_progress: Map<u64, inbox::Progress>,
    pub(crate) census_position: Option<(u64, u64, u64)>,
    pub(crate) ended_chats: Map<u64, inbox::EndedChat>,
    pub(crate) proposed_input: Option<Result<Box<settings::Proposal>, people::Malformed>>,
    pub(crate) settings_proposals: Map<(people::Proposer, u32), settings::Proposal>,
    pub(crate) settings_input: Option<(Result<Box<settings::Change>, settings::Failure>, u32)>,
    pub(crate) key_input: Option<Carried>,
    pub(crate) application_input: Option<routing::application_routes::Input>,
    pub(crate) task_capacity_mark: u64,
    pub(crate) settings_holders: u32,
    pub(crate) opened: bool,
    pub(crate) start_pending: bool,
    pub(crate) restart_attempts: u32,
    /// Application connectors addressed only by number.
    pub connectors: Box<[u16]>,
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
    /// Names seeded or restored with the project policies (core.md, 6.1).
    pub(crate) names: Map<u32, Box<str>>,
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
    /// Preparations waiting for notes, or deferring their next activation for load room.
    pub(crate) pending_note_briefs: Map<u64, routing::NoteBrief>,
    /// Durable deployment numbers.
    pub counters: Counters,
    /// Live task preparation contexts.
    pub(crate) contexts: Map<u64, Box<tasks::RunContext>>,
    pub(crate) workspace_credentials: Map<u64, Box<[u32]>>,
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
    /// Claims awaiting completion of their route.
    pub claiming: Map<u64, u64>,
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
        let current = settings::Current::new(config.defaults, limits);
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
            permission_roles: Map::with_capacity(limits.authority.projects),
            template_overrides: Map::with_capacity(limits.authority.projects),
            connectors: config.connectors,
            pending_calls: Map::with_capacity(limits.call_records),
            call_parts: Map::with_capacity(limits.call_records),
            call_settled: Map::with_capacity(limits.call_records),
            effect_replies: Map::with_capacity(limits.call_records),
            closing_connectors: Map::with_capacity(
                limits.tasks.tasks.checked_mul(limits.connectors).expect("bounded closing connectors"),
            ),
            projections: Map::with_capacity(limits.tasks.tasks),
            pending_note_briefs: Map::with_capacity(limits.brief.briefs),
            run_now: skein_lib::Time::ZERO,
            run_wall: Wall::EPOCH,
            contexts: Map::with_capacity(limits.tasks.tasks),
            workspace_credentials: Map::with_capacity(limits.tasks.tasks),
            transcripts: Map::with_capacity(limits.tasks.tasks),
            proofs: Map::with_capacity(limits.tasks.tasks),
            replaced: Map::with_capacity(limits.tasks.tasks),
            restoring_proofs: Map::with_capacity(limits.tasks.tasks),
            unreported_restored: Map::with_capacity(limits.tasks.tasks),
            claiming: Map::with_capacity(limits.tasks.tasks),
            due: Queue::uncapped(),
            adopted: Queue::uncapped(),
            tasks: tasks::Domain::new(
                &limits.tasks,
                config.seed,
                current.newest(),
                charters::models(&current, &config.settings, limits),
                charters::efforts(&config.settings),
            ),
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
                let mut fleet = fleet::Domain::new(&limits.fleet, fleet_needs, {
                    let mut names = List::with_capacity(tasks::MODELS);
                    for model in &config.settings.models {
                        names.push(model.name.clone()).expect("catalogue bound");
                    }
                    names.into_boxed()
                });
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
