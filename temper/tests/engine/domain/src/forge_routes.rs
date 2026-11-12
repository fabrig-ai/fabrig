//! Root forge stories with its durable store and the independent fake Forgejo.
use crate as temper_engine_domain_world;
use crate::forge_routes as paths;
use crate::journal_limits as journal;
use jig_domain_authority as authority;
use jig_domain_core::documents;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Key, Record};
use temper_engine_domain_forge as forge_top;
use temper_engine_domain_world::protocol::{self, Protocol};
use temper_engine_domain_world::{store::Store, walking};
use temper_engine_forge_world::{self as forge_world, translate};

// in the connector's fixture output cohorts.
const FORGE_OUTPUT: u32 = 80;

// A live result's label is 17 bytes, its separator 2, and its two task numbers at most 20 each.
const RESULT_MESSAGE_BYTES: u32 = 59;

// Writes for the forge change world with no landing members.
const EMPTY_LANDING_WRITES: u32 = 8_696;

// Writes for the twelve-task forge change and landing worlds.
const CHANGE_WRITES: u32 = 8_698;

// Writes for the twelve-task forge change world with eight fleet slots.
const EIGHT_SLOT_FORGE_WRITES: u32 = 8_730;

// Writes for the forge worlds using the deployment's effective limits.
const DEPLOYMENT_FORGE_WRITES: u32 = 12_288;

// The world's existing bound for a durable round trip (until).
const STEPS: u32 = 200;

// Writes for the stock forge world before deployment limits are applied.
const STOCK_FORGE_WRITES: u32 = 9_754;

// Holds the stock goal agent's instructions, four parts and resolved model.
const STOCK_RUN_BYTES: u32 = 32_768;

/// The settings family's tools run on a world's charter (temper/docs/design/testing/checks.md, 4).
pub fn add_settings_charter(defaults: &mut jig_domain_core::settings::Defaults) {
    const SETTINGS_CHARTER: u32 = 14;
    const SETTINGS_TEMPLATE: u32 = 12;
    let mut charter = defaults.charters[0].clone();
    charter.number = SETTINGS_CHARTER;
    charter.policy.tools = tasks::Tools(1_920);
    charter.policy.instructions = b"Read and propose settings.".as_slice().into();
    let mut charters = defaults.charters.clone().into_vec();
    charters.push(charter);
    defaults.charters = charters.into_boxed_slice();
    let mut template = defaults.templates[0].clone();
    template.number = SETTINGS_TEMPLATE;
    template.name = b"settings".as_slice().into();
    template.executor = people::TemplateExecutor::Charter(SETTINGS_CHARTER);
    let mut templates = defaults.templates.clone().into_vec();
    templates.push(template);
    defaults.templates = templates.into_boxed_slice();
    let policy = &mut defaults.policies.get_mut(&1).expect("world's policy 1").policy;
    for authority in std::iter::once(&mut defaults.rules.ceiling)
        .chain(std::iter::once(&mut policy.ceiling))
        .chain(policy.roles.iter_mut().map(|role| &mut role.authority))
    {
        if authority.delegation.tasks != 0 {
            let mut kinds = authority.delegation.kinds.clone().into_vec();
            // The settings agent takes the standing role's unused authority place in this world.
            kinds.retain(|kind| *kind != authority::Executor::Charter(13));
            kinds.push(authority::Executor::Charter(SETTINGS_CHARTER));
            authority.delegation.kinds = kinds.into_boxed_slice();
        }
    }
}

/// Which outward write’s decoded answer the story retains.
#[derive(Clone, Copy)]
pub enum HoldWrite {
    All,
    Merge,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CallError {
    pub call: Box<[u8]>,
    pub text: Box<[u8]>,
}

#[expect(clippy::struct_excessive_bools, reason = "independent outside fault and observation switches remain explicit")]
pub struct World {
    restart_config: engine::Config,
    oracle_documents: Vec<(Token, forge_top::Document)>,
    oracle_watcher: u64,
    broken: bool,
    oracle_running: bool,
    oracle_peer: Option<(Token, u32, u32)>,
    creating: jig_test_profile::fixtures::Creating,
    pub root: engine::Domain,
    pub env: Env<engine::Limits>,
    pub out: Queue<engine::Request>,
    pub events: VecDeque<engine::Event>,
    pub store: Store,
    pub fake: fake::Domain,
    pub fake_env: Env<fake::Config>,
    fake_out: Queue<fake::Request>,
    pub pending: BTreeMap<Token, skein_forge::api::Op>,
    pending_repositories: BTreeMap<Token, Box<[u8]>>,
    located_names: BTreeMap<u32, Box<[u8]>>,
    run_repositories: BTreeMap<Token, Box<[u8]>>,
    pub forge_calls: u32,
    pub forge_reads: Option<Vec<(Token, skein_forge::api::Read)>>,
    pub adopted: Vec<forge_top::ApplicationOutcome>,
    pub follow_answers: Vec<Option<Box<[u8]>>>,
    pub signed_in: Option<u64>,
    pub web_replies: Vec<people::Reply>,
    pub assigned: Vec<engine::Assignment>,
    pub jobs: Vec<host::JobAssignment>,
    pub job_workspaces: BTreeMap<u64, engine::ForgeWorkspace>,
    /// Inbox messages delivered after an assignment, as a real worker receives them.
    pub delivered_messages: BTreeMap<(u64, u64), Vec<u64>>,
    pub answers: Vec<temper_engine_domain::CallAnswer>,
    /// Error text received by the scripted worker, paired with its call name.
    pub call_errors: Vec<CallError>,
    pub reads: Vec<(ReplyTo, jig_domain_core::reads::Answer)>,
    pub panels: Vec<(Token, forge_top::Panel)>,
    pub approvals: Vec<(Token, engine::Approvals)>,
    pub protocol: Protocol,
    pub immediate: VecDeque<engine::Event>,
    /// The funding period sent to this process's root, once it is ready.
    period: Option<u64>,
    period_budget: u64,
    pub fail_job_reads: bool,
    comparisons_too_large: bool,
    /// Refuse the named section reads while a CI repair is preparing.
    pub fail_brief_reads: Option<(bool, bool, skein_forge::api::Error)>,
    slow_brief_reads: bool,
    /// A branch whose reads take twenty seconds, each decided when it is sent.
    pub slow_branch: Option<Box<[u8]>>,
    /// Delay the review diff independently of the change's fresh checks.
    pub review_files_delay: Option<Duration>,
    gate: paths::Gate,
    pub held_reads: Option<Vec<engine::Request>>,
    /// Source comparisons withheld before their terminal, including across a cold restart.
    pub held_comparisons: Option<Vec<engine::Request>>,
    /// An unusable comparison must conservatively keep a source landing.
    pub fail_comparisons: bool,
    pub merges: u32,
    pub writes: Vec<skein_forge::api::Write>,
    pub repository_writes: Vec<(skein_forge::api::Repository, skein_forge::api::Write)>,
    /// Hold decoded write answers after the independent forge has made the effect.
    pub hold_write_answers: Option<HoldWrite>,
    pub held_write_answers: Vec<engine::Event>,
    /// Timeouts before dispatch: the caller cannot know the write was never made.
    pub lose_writes: u32,
    pub fail_issue_writes: bool,
    pub fail_open_writes: bool,
    pub issue_write_times: Vec<Wall>,
    /// The durable commit number observed when the root drained.
    pub drained: Option<u64>,
    pub unsettled: Vec<engine::Unsettled>,
    pub busy_calls: u32,
}

#[derive(Clone, Copy, Default)]
#[expect(clippy::struct_excessive_bools, reason = "stories independently configure landing rules, kind and authority")]
pub struct StoryLimits {
    /// Tool families granted by the deployment and project for this story.
    pub tools: Option<u64>,
    /// Scalar note scopes and a resource scope held by the parent in narrowing stories.
    pub notes: Option<authority::Scopes>,
    /// The chat may delegate to its owner's role before goal narrowing.
    pub person_executor: bool,
    /// An external procedure makes a change through its committed batch.
    pub procedure_batch: bool,
    pub tier: Option<crate::tier_runs::Tier>,
    pub turn: bool,
    pub across: bool,
    pub library_gate_latency: Option<Duration>,
    pub landing_members: Option<u32>,
    pub slots: Option<u32>,
    pub tasks: Option<u32>,
    pub authority_batch: Option<u32>,
    pub tasks_batch: Option<u32>,
    pub task_delegates: Option<u32>,
    pub tree_tasks: Option<u32>,
    pub inbox_messages: Option<u32>,
    pub inbox_bytes: Option<u32>,
    pub brief: Option<engine::BriefLimits>,
    pub stall: Option<Duration>,
    pub spec_bytes: Option<u32>,
    pub result_bytes: Option<u32>,
    pub forge_charter: Option<u32>,
    pub applications: Option<u32>,
    pub forge_commits: Option<u32>,
    pub kinds: bool,
    pub questions: Option<u32>,
    pub merge_kind: bool,
    pub resolutions: Option<u32>,
    pub merge_authority: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Until {
    Ready,
    SignedIn,
    Adopted,
    Assigned,
    Subscribed,
    News,
    Read,
    Effect,
    Delegated,
    SecondAssignment,
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self::configured(false, false)
    }

    #[must_use]
    pub fn configured(with_read: bool, with_change: bool) -> Self {
        Self::configured_ci(with_read, with_change, false)
    }

    #[must_use]
    pub fn configured_ci(with_read: bool, with_change: bool, silent_ci: bool) -> Self {
        Self::configured_checks(with_read, with_change, silent_ci, 1000)
    }

    #[must_use]
    pub fn configured_checks(with_read: bool, with_change: bool, silent_ci: bool, passes: u32) -> Self {
        Self::configured_policy(with_read, with_change, silent_ci, passes, paths::Gate::default(), None)
    }

    #[must_use]
    pub fn configured_policy(
        with_read: bool,
        with_change: bool,
        silent_ci: bool,
        passes: u32,
        gate: paths::Gate,
        documents: Option<jig_domain_core::documents::DocumentLimits>,
    ) -> Self {
        Self::configured_brief(with_read, with_change, silent_ci, passes, gate, documents, None)
    }

    #[must_use]
    pub fn configured_brief(
        with_read: bool,
        with_change: bool,
        silent_ci: bool,
        passes: u32,
        gate: paths::Gate,
        documents: Option<jig_domain_core::documents::DocumentLimits>,
        brief: Option<engine::BriefLimits>,
    ) -> Self {
        Self::configured_limits(
            with_read,
            with_change,
            silent_ci,
            passes,
            gate,
            documents,
            StoryLimits { brief, ..StoryLimits::default() },
        )
    }

    /// Stock settings and model, with room for B2's advisor and two repaired changes.
    #[must_use]
    pub fn configured_stock(defaults: &jig_domain_core::settings::Defaults, model: &engine::Model) -> Self {
        Self::configured_settings(
            true,
            true,
            true,
            1000,
            paths::Gate { agent: true, ..paths::Gate::default() },
            None,
            &StoryLimits::default(),
            Some((defaults, model, None)),
            &[],
        )
    }

    /// Stock settings under exactly the deployment's effective limits.
    #[must_use]
    pub fn configured_stock_limits(
        defaults: &jig_domain_core::settings::Defaults,
        model: &engine::Model,
        limits: &engine::Limits,
    ) -> Self {
        Self::configured_settings(
            true,
            true,
            true,
            1000,
            paths::Gate { agent: true, ..paths::Gate::default() },
            None,
            &StoryLimits::default(),
            Some((defaults, model, Some(limits))),
            &[],
        )
    }

    #[must_use]
    pub fn configured_limits(
        with_read: bool,
        with_change: bool,
        silent_ci: bool,
        passes: u32,
        gate: paths::Gate,
        documents: Option<jig_domain_core::documents::DocumentLimits>,
        timing: StoryLimits,
    ) -> Self {
        Self::configured_settings(with_read, with_change, silent_ci, passes, gate, documents, &timing, None, &[])
    }

    /// Charter documents through the core's plan, root binding and forge gather
    /// (temper/docs/design/testing/checks.md, 4).
    #[must_use]
    pub fn configured_documents(documents: &[jig_domain_core::Document]) -> Self {
        Self::configured_settings(
            true,
            false,
            true,
            1000,
            paths::Gate::default(),
            None,
            &StoryLimits::default(),
            None,
            documents,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one builder derives bootstrap and runtime policy configurations for root forge stories"
    )]
    fn configurations(
        with_read: bool,
        with_change: bool,
        gate: paths::Gate,
        documents: Option<jig_domain_core::documents::DocumentLimits>,
        timing: &StoryLimits,
        stock: Option<(&jig_domain_core::settings::Defaults, &engine::Model, Option<&engine::Limits>)>,
        charter_documents: &[jig_domain_core::Document],
    ) -> [(engine::Config, engine::Limits); 2] {
        // Sizes subscription/hold notifications and workspace-resource bookkeeping
        let StoryLimits {
            tools,
            notes,
            person_executor,
            procedure_batch,
            brief,
            stall,
            spec_bytes,
            result_bytes,
            authority_batch,
            tasks_batch,
            task_delegates,
            tree_tasks,
            inbox_messages,
            inbox_bytes,
            forge_charter,
            applications,
            forge_commits: _,
            kinds,
            questions,
            merge_kind,
            merge_authority,
            resolutions,
            turn,
            landing_members,
            slots,
            tasks,
            across,
            library_gate_latency: _,
            tier,
        } = *timing;
        let paths::Gate { approval, agent: agent_gate, owner: owner_gate } = gate;
        std::array::from_fn(|index| {
            let mut limits = walking::limits();
            if let Some(questions) = questions {
                limits.forge.change_policy.questions = questions;
            }
            limits.people.amendment_bytes =
                jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.people.amendment_bytes;
            limits.people.people = 4;
            limits.people.holdings = 4;
            limits.journal.writes = journal::WALKING_WRITES;
            let mut config = walking::config(71);
            let mut landing_rules =
                engine::LandingRules { deployment: Box::default(), projects: skein_lib::Map::with_capacity(1) };
            if approval.is_some() || agent_gate || owner_gate {
                limits.authority.roles = 3;
            }
            if documents.is_some() {
                limits.authority.roles = 2;
            }
            if with_change {
                limits.authority.requirements = 8;
                limits.authority.facts = 8;
            }
            if with_change {
                // One adoption, three chat creations and two Says survive the restart cut.
                limits.people.requests = 6;
                // Adoption stories ask in 128 bytes of words.
                limits.people.words = 128;
                limits.tasks.message_bytes = 128;
                limits.call_records = 4;
                limits.brief.gather = Duration::from_secs(5);
                limits.brief.read_bytes = 512;
                // Change delegates gather the larger CI and pull fixtures below.
                limits.brief.budgets.task = 384;
                limits.brief.budgets.pull = 384;
                limits.brief.budgets.ci = 384;
                limits.brief.brief_bytes = 768;
                limits.tasks.tasks = tasks.unwrap_or(12);
                limits.forge.tasks = limits.tasks.tasks;
                if tasks.is_some() {
                    // Concurrent chats and changes keep all of their workspace watches.
                    limits.forge.client.resources = limits.tasks.tasks * limits.forge.resources_per_task;
                    limits.forge.output = FORGE_OUTPUT;
                }
                limits.tasks.project_tasks = if authority_batch.is_some() { tasks.unwrap_or(12) } else { 12 };
                limits.tasks.delegates = task_delegates.unwrap_or(4);
                limits.tasks.tree_tasks = tree_tasks.unwrap_or(12);
                limits.tasks.depth = 4;
                limits.tasks.parameters = 8;
                limits.tasks.spec_bytes = 512;
                limits.tasks.executor_kinds = 5;
                limits.tasks.charters = 4;
                let mut producer = walking::charter();
                producer.number = 2;
                let mut reviewer = walking::charter();
                reviewer.number = 4;
                reviewer.policy.modify = false;
                reviewer.policy.agents = false;
                config.core.defaults.charters = Box::new([walking::charter(), producer, reviewer]);
                limits.tasks.authority_grants = 11;
                limits.journal.run_bytes = 8192;
                limits.tasks.authority_segments = 9;
                limits.tasks.authority_bytes = 512;
                limits.tasks.contract_choices = 3;
                limits.tasks.inbox_messages = inbox_messages.unwrap_or(8);
                limits.tasks.inbox_bytes = inbox_bytes.unwrap_or(1024);
                limits.authority.executors = 5;
                limits.authority.batch = authority_batch.unwrap_or(2);
                limits.authority.grants = 11;
                limits.authority.segments = 8;
                limits.authority.segment_bytes = 64;
                limits.authority.writes = 2;
                limits.journal.writes += 64;
                // The tasks child's whole route grew with jig's task families.
                limits.journal.writes = 8192;
                limits.journal.deliveries = 256;
                limits.journal.held = 1024;
                limits.journal.result_bytes = 4096;
                limits.journal.transcript_bytes = 32_768;
                limits.brief.briefs = slots.unwrap_or(2);
                limits.fleet.attempts = slots.unwrap_or(2);
                limits.fleet.slots = slots.unwrap_or(2);
                limits.fleet.workstreams = slots.unwrap_or(2);
                // The core's inbox census and waiting entries cover every live task.
                limits.people.inbox_tasks = if authority_batch.is_some() { limits.tasks.tasks * 2 } else { 24 };
                limits.people.inbox_recipients = 32.max(limits.tasks.inbox_messages * 2 + 8);
            }
            if with_read {
                limits.tasks.inbox_bytes = limits.tasks.inbox_bytes.max(256);
                if !with_change {
                    limits.authority.grants = 4;
                }
                limits.authority.segments = 8;
                limits.authority.segment_bytes = 64;
                if !with_change {
                    limits.tasks.authority_grants = 2;
                }
                limits.tasks.authority_segments = 9;
                if !with_change {
                    limits.tasks.authority_bytes = 64;
                }
                let grant = authority::Grant {
                    connector: 0,
                    kind: 1,
                    pattern: if with_change {
                        authority::Pattern {
                            segments: Box::new([
                                Box::from(b"forge".as_slice()),
                                Box::from(b"forge.example".as_slice()),
                            ]),
                            last: authority::Last::Open(Box::new([])),
                        }
                    } else {
                        authority::Pattern {
                            segments: Box::new([
                                Box::from(&b"forge"[..]),
                                Box::from(&b"forge.example"[..]),
                                Box::from(&b"org"[..]),
                            ]),
                            last: authority::Last::Open(Box::from(&b"repo"[..])),
                        }
                    },
                };
                let effect_grant = authority::Grant { kind: 8, ..grant.clone() };
                let mut rules = config.core.defaults.rules.clone();
                if with_change {
                    // The chat's 100-unit allotment must retain room for its 20-unit delegate
                    // after the run reserves its capped allowance at claim.
                    rules.maximum_run_spend = 60;
                }
                rules.ceiling.tools = authority::Tools(tools.unwrap_or(1023));
                let grants: Box<[authority::Grant]> = if with_change {
                    Box::new([
                        grant.clone(),
                        effect_grant.clone(),
                        authority::Grant { kind: 2, ..grant.clone() },
                        authority::Grant { kind: 3, ..grant.clone() },
                        authority::Grant { kind: 4, ..grant.clone() },
                        authority::Grant { kind: 10, ..grant.clone() },
                        authority::Grant { kind: 11, ..grant.clone() },
                        authority::Grant { kind: 12, ..grant.clone() },
                        authority::Grant { kind: 17, ..grant.clone() },
                    ])
                } else {
                    Box::new([grant.clone(), effect_grant.clone()])
                };
                let mut ceiling_grants = List::with_capacity(limits.authority.grants);
                for grant in &grants {
                    ceiling_grants.push(grant.clone()).expect("configured task grants");
                }
                ceiling_grants.push(authority::Grant { kind: 7, ..grant.clone() }).expect("projection comments");
                ceiling_grants.push(authority::Grant { kind: 13, ..grant.clone() }).expect("projection issue sets");
                rules.ceiling.grants = ceiling_grants.into_boxed();
                if approval.is_some() || agent_gate || owner_gate {
                    let landing: Box<[engine::LandingRule]> = Box::new([engine::LandingRule {
                        sets: Box::new([]),
                        connector: 0,
                        kind: 4,
                        enforce: true,
                        pattern: authority::Pattern {
                            segments: Box::new([
                                Box::from(&b"forge"[..]),
                                Box::from(&b"forge.example"[..]),
                                Box::from(&b"org"[..]),
                                Box::from(&b"repo"[..]),
                                Box::from(&b"branch"[..]),
                            ]),
                            last: authority::Last::Exact(Box::from(&b"main"[..])),
                        },
                        ci: true,
                        up_to_date: true,
                        gates: if agent_gate || owner_gate {
                            Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Exact }])
                        } else {
                            Box::new([])
                        },
                        approvals: match approval {
                            Some(freshness) => Box::new([engine::Approval { role: 2, people: 1, freshness }]),
                            None => Box::new([]),
                        },
                    }]);
                    if !owner_gate {
                        landing_rules.deployment.clone_from(&landing);
                    }
                    assert!(landing_rules.projects.insert(1, landing).is_ok());
                }
                if with_change {
                    rules.ceiling.delegation.kinds = Box::new([
                        authority::Executor::Charter(1),
                        authority::Executor::Charter(2),
                        authority::Executor::Charter(4),
                        authority::Executor::Procedure(2),
                    ]);
                    rules.ceiling.delegation.tasks = tree_tasks.unwrap_or(12);
                    rules.ceiling.delegation.depth = 4;
                }
                let mut policy = config.core.seed_projects[0].policy.policy.clone();
                policy.ceiling.tools = authority::Tools(tools.unwrap_or(1023));
                policy.ceiling.grants.clone_from(&rules.ceiling.grants);
                policy.projections.clone_from(&rules.ceiling.grants);
                if approval.is_some() || agent_gate || owner_gate {
                    let mut maintainer = policy.roles[0].clone();
                    maintainer.number = 1;
                    let mut member = maintainer.clone();
                    member.number = 2;
                    policy.roles = Box::new([policy.roles[0].clone(), maintainer, member]);
                }
                policy.roles[0].authority.tools = authority::Tools(tools.unwrap_or(1023));
                policy.roles[0].authority.grants.clone_from(&grants);
                if with_change {
                    policy.ceiling.delegation = rules.ceiling.delegation.clone();
                    policy.roles[0].authority.delegation = rules.ceiling.delegation.clone();
                }
                if documents.is_some() {
                    paths::add_reader(&mut policy);
                }
                if across {
                    limits.fleet.slots = 3;
                    limits.fleet.attempts = 3;
                    limits.fleet.workstreams = 3;
                }
                if turn || tier.is_some() {
                    limits.authority.grants = 14;
                    limits.tasks.authority_grants = 14;
                    // The closer procedure and close job add two executor places.
                    limits.authority.executors = 9;
                    // The closer procedure and close job add two executor places.
                    limits.tasks.executor_kinds = 9;
                    // Verification evidence has 256 bytes beside the live result label.
                    limits.tasks.result_bytes = 256 + RESULT_MESSAGE_BYTES;
                    limits.tasks.inbox_bytes += limits.tasks.inbox_messages * RESULT_MESSAGE_BYTES;
                    limits.journal.run_bytes = 32768;
                    if let Some(slots) = landing_members {
                        limits.forge.landing_members = slots;
                    }
                    let mut grants = List::with_capacity(14);
                    for grant in &rules.ceiling.grants {
                        grants.push(grant.clone()).expect("ceiling grants");
                    }
                    for kind in [6, 9, 16] {
                        grants.push(authority::Grant { kind, ..grant.clone() }).expect("landing grants");
                    }
                    rules.ceiling.grants = grants.into_boxed();
                    rules.ceiling.delegation.kinds = Box::new([
                        authority::Executor::Charter(1),
                        authority::Executor::Charter(2),
                        authority::Executor::Charter(4),
                        authority::Executor::Procedure(2),
                        authority::Executor::Procedure(3),
                        authority::Executor::Procedure(4),
                        authority::Executor::Procedure(6),
                        authority::Executor::Job(4),
                    ]);
                    policy.ceiling = rules.ceiling.clone();
                    let rule = engine::LandingRule {
                        sets: Box::new([]),
                        connector: 0,
                        kind: 4,
                        enforce: true,
                        pattern: authority::Pattern {
                            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"]
                                .map(Box::from)
                                .into(),
                            last: authority::Last::Exact(b"main".as_slice().into()),
                        },
                        ci: true,
                        up_to_date: true,
                        gates: Box::new([]),
                        approvals: Box::new([]),
                    };
                    if across {
                        let mut lib = rule.clone();
                        lib.pattern.segments[3] = b"lib".as_slice().into();
                        landing_rules.deployment = Box::new([rule.clone(), lib.clone()]);
                        landing_rules.projects.insert(1, Box::new([rule.clone(), lib])).expect("two repository rules");
                    } else {
                        landing_rules.deployment = Box::new([rule.clone()]);
                        landing_rules.projects.insert(1, Box::new([rule])).expect("turn landing requirements");
                    }
                    config
                        .landings
                        .insert(
                            1,
                            engine::ProjectLanding {
                                check_timeout: None,
                                close_job: None,
                                merge_job: None,
                                after: engine::AfterLandingCheck {
                                    tier_retries: 2,
                                    verification_bound_ms: tier.map_or(7_200_000, |tier| tier.bound_ms),
                                },
                                by: if turn { engine::Way::Turn } else { engine::Way::Queue },
                                repositories: Box::new([engine::RepositoryChecks {
                                    tiers: match tier {
                                        Some(tier) => crate::tier_runs::checks(tier),
                                        None => Box::new([]),
                                    },
                                    repository: forge_world::REPO,
                                    gates: if turn {
                                        Box::new([engine::GateCheck {
                                            name: b"gate".as_slice().into(),
                                            workflow: b"gate.yml".as_slice().into(),
                                            ignored: Box::new([b"docs".as_slice().into()]),
                                            blocking: true,
                                        }])
                                    } else {
                                        Box::new([])
                                    },
                                }]),
                            },
                        )
                        .expect("one turn project");
                    if across {
                        let landing = config.landings.get_mut(&1).expect("turn project");
                        let mut lib = landing.repositories[0].clone();
                        lib.repository = skein_forge::api::Repository { forge: 1, repository: 4 };
                        landing.repositories = Box::new([landing.repositories[0].clone(), lib]);
                    }
                }
                if let Some(scopes) = notes {
                    let resources: Box<[authority::ResourceScope]> =
                        Box::new([authority::ResourceScope { connector: 0, pattern: grant.pattern.clone() }]);
                    for authority in [&mut rules.ceiling, &mut policy.ceiling, &mut policy.roles[0].authority] {
                        authority.notes = scopes;
                        authority.note_resources.clone_from(&resources);
                    }
                }
                if person_executor {
                    for authority in [&mut rules.ceiling, &mut policy.ceiling, &mut policy.roles[0].authority] {
                        authority.delegation.kinds =
                            [authority.delegation.kinds.as_ref(), &[authority::Executor::Role(0)]].concat().into();
                    }
                    let executors = u32::try_from(rules.ceiling.delegation.kinds.len()).expect("fixture kinds fit");
                    limits.authority.executors = limits.authority.executors.max(executors);
                    limits.tasks.executor_kinds = limits.tasks.executor_kinds.max(executors);
                }
                if procedure_batch {
                    let grants = rules.ceiling.grants.clone();
                    for authority in [&mut rules.ceiling, &mut policy.ceiling, &mut policy.roles[0].authority] {
                        authority.grants.clone_from(&grants);
                        authority.delegation.kinds =
                            [authority.delegation.kinds.as_ref(), &[authority::Executor::Procedure(1)]].concat().into();
                    }
                    limits.authority.executors = limits.authority.executors.max(10);
                    limits.tasks.executor_kinds = limits.tasks.executor_kinds.max(10);
                }
                let mut domain = authority::Domain::new(rules, limits.authority).expect("read authority rules");
                let mut facts = Queue::uncapped();
                authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut facts);
                assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
                // A chat's authority is its role's, one level shallower.
                config.core.seed_projects[0].policy.policy = domain.policy(1).expect("configured seed policy").clone();
                config.core.defaults.rules = domain.rules().clone();
            }
            if let Some((defaults, model, _)) = stock {
                const TREE_TASKS: u32 = 12; // Stock lifetime ceiling, including the root.
                const LANDED_CHANGES: u32 = 2;
                const RETAINED_HOLDINGS: u32 = 1 + LANDED_CHANGES * 2;
                const TREE_ISSUE_BYTES: u32 = 8192;
                const TREE_MILESTONES: u32 = 64;
                const MILESTONE_BYTES: u32 = 1024; // A 512-byte result plus its lifecycle heading.
                const TREE_FORGE_BYTES: u32 = TREE_ISSUE_BYTES + 1024;
                const RESULT_INBOX_BYTES: u32 = 4096;
                // Keep policy 1, the run limits, charters and templates exactly as supplied.
                config.core.defaults.rules = defaults.rules.clone();
                config.core.defaults.charters.clone_from(&defaults.charters);
                config.core.defaults.templates.clone_from(&defaults.templates);
                config
                    .core
                    .defaults
                    .policies
                    .insert(1, defaults.policies.get(&1).expect("stock policy 1").clone())
                    .expect("one stock policy");
                config.core.seed_projects[0].policy = defaults.policies.get(&1).expect("stock policy 1").clone();
                config.core.settings.models = Box::new([model.clone()]);
                config.core.settings.endpoints = Box::new([jig_domain_core::EndpointFacts {
                    providers: Box::new([]),
                    number: model.endpoint,
                    dialect: model.dialect,
                    program: model.program.clone(),
                }]);

                config.core.connectors = Box::new([1]);
                for rule in &mut landing_rules.deployment {
                    rule.connector = 1;
                    for gate in &mut rule.gates {
                        gate.freshness = engine::Freshness::Clean;
                    }
                }
                landing_rules.projects = skein_lib::Map::with_capacity(1);
                // Count the supplied stock and any charters of the world's own.
                limits.tasks.charters = u32::try_from(defaults.charters.len()).expect("fixture charters fit");
                limits.tasks.executor_kinds =
                    u32::try_from(defaults.rules.ceiling.delegation.kinds.len()).expect("fixture executor kinds fit");
                limits.authority.executors = limits.tasks.executor_kinds;
                // B2 retains eleven tasks (chat, work, advisor, two procedures and six agents).
                limits.tasks.funders = TREE_TASKS;
                limits.tasks.project_goals = TREE_TASKS;
                // The root retains its run branch and both landed changes' branch/pull pairs.
                limits.tasks.holdings = RETAINED_HOLDINGS;
                // Names for every task's permitted workspace resources can coexist in watches.
                limits.forge.client.resources = TREE_TASKS * limits.forge.resources_per_task;
                limits.forge.output = FORGE_OUTPUT;
                // The tracked work's issue renders every descendant's spec and lifecycle.
                // Twelve 512-byte specs plus headings fit in 8 KiB, rather than the tiny
                // fixture's 512-byte body. Milestones are retained for the whole walk:
                // eleven task endings, both changes' state transitions and W's wakes.
                limits.forge.issue_policy.plan_items = TREE_TASKS;
                limits.forge.issue_policy.milestones = TREE_MILESTONES;
                limits.forge.issue_policy.body_bytes = TREE_ISSUE_BYTES;
                limits.forge.issue_policy.comment_bytes = MILESTONE_BYTES;
                // The forge client carries that body plus its title, purpose key and framing.
                limits.forge.client.op_bytes = TREE_FORGE_BYTES;
                limits.forge.client.answer_bytes = TREE_FORGE_BYTES;
                // Reports use jig's deployment result bound.
                limits.tasks.result_bytes =
                    jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.tasks.result_bytes;
                limits.tasks.inbox_bytes = RESULT_INBOX_BYTES;
                limits.loads.bytes = limits.loads.bytes.max(RESULT_INBOX_BYTES);
                limits.loads.reply_bytes = limits.loads.reply_bytes.max(RESULT_INBOX_BYTES);
            }
            if let Some(brief) = brief {
                limits.brief = brief;
                // A shipped brief brings its own room: results hold it, forge sections read it.
                limits.journal.result_bytes = limits.journal.result_bytes.max(brief.brief_bytes);
                limits.forge.brief_bytes = limits.forge.brief_bytes.max(brief.read_bytes);
            }
            if let Some(batch) = tasks_batch {
                limits.tasks.batch = batch;
            }
            if let Some(bytes) = result_bytes {
                // A task row owns three result texts and its inbox beside the fixture's carrier.
                let growth = bytes.saturating_sub(limits.tasks.result_bytes).saturating_mul(4);
                limits.journal.transcript_bytes += growth;
                limits.loads.transcript_bytes = limits.loads.transcript_bytes.max(limits.journal.transcript_bytes);
                limits.loads.bytes = limits.loads.bytes.max(limits.journal.transcript_bytes);
                limits.loads.reply_bytes = limits.loads.reply_bytes.max(limits.journal.transcript_bytes);
                limits.tasks.result_bytes = bytes;
                limits.tasks.inbox_bytes = limits.tasks.inbox_bytes.max(bytes);
                limits.forge.client.op_bytes = limits.forge.client.op_bytes.max(bytes);
                limits.journal.result_bytes = limits.journal.result_bytes.max(bytes);
            }
            if let Some(spec_bytes) = spec_bytes {
                limits.tasks.spec_bytes = spec_bytes;
            }
            if merge_kind {
                let mut bound = protocol::merge_bound();
                bound.spec = bound.spec.min(limits.tasks.spec_bytes);
                bound.result = bound.result.min(limits.tasks.result_bytes);
                config.core.jobs = Box::new([bound]);
            }
            if merge_authority {
                let kinds = &mut config.core.defaults.rules.ceiling.delegation.kinds;
                let mut extended = kinds.to_vec();
                extended.push(authority::Executor::Job(3));
                let executors = u32::try_from(extended.len()).expect("fixture executor count fits");
                limits.tasks.executor_kinds = limits.tasks.executor_kinds.max(executors);
                limits.authority.executors = limits.authority.executors.max(executors);
                *kinds = extended.into_boxed_slice();
                for project in &mut config.core.seed_projects {
                    let policy = &mut project.policy.policy;
                    policy.ceiling.delegation.kinds.clone_from(kinds);
                    policy.roles[0].authority.delegation.kinds.clone_from(kinds);
                }
            }
            if let Some(resolutions) = resolutions {
                limits.forge.change_policy.resolutions = resolutions;
            }
            if let Some(stall) = stall {
                limits.forge.change_policy.stall = stall;
            }
            limits.notes.scopes = 3 + limits.authority.grants;
            if stock.is_some() {
                // The larger resource route needs its full declared journal-write burst.
                limits.journal.writes = STOCK_FORGE_WRITES;
                limits.journal.run_bytes = limits.journal.run_bytes.max(STOCK_RUN_BYTES);
                limits.loads.bytes = limits.loads.bytes.max(STOCK_RUN_BYTES);
                limits.loads.reply_bytes = limits.loads.reply_bytes.max(STOCK_RUN_BYTES);
                limits.loads.transcript_bytes = limits.loads.transcript_bytes.max(STOCK_RUN_BYTES);
            }
            if let Some((_, _, Some(deployment))) = stock {
                limits = *deployment;
            }
            if !charter_documents.is_empty() {
                // Core sections and the two documents admitted by these stories
                // (jig/docs/design/domain/engine/settings.md, 9).
                const CHARTER_DOCUMENTS: u32 = 2;
                const DOCUMENT_SECTIONS: u32 = jig_domain_core::CORE_SECTIONS + CHARTER_DOCUMENTS;
                limits.brief.sections = DOCUMENT_SECTIONS;
                limits.tasks.charter_documents = CHARTER_DOCUMENTS;
                let mut charter = walking::charter();
                charter.documents = charter_documents.into();
                config.core.defaults.charters[0] = charter;
            }
            // A brief override carries its section count into the forge's fetch room.
            if let Some(applications) = applications {
                limits.people.requests = applications;
                limits.forge.applications = applications;
            }
            limits.forge.brief_sections =
                limits.forge.brief_sections.max(limits.brief.briefs * limits.brief.sections * 2);
            config.kinds_stock = engine::landing_stock(
                &landing_rules.deployment,
                &landing_rules.projects,
                if stock.is_some() { 2 } else { forge_charter.unwrap_or(1) },
                4,
            );
            if kinds {
                // The story changes whole rules through the settings entrance.
                limits.people.amendment_bytes = 2048;
            }
            if kinds && index == 1 {
                config.core.kinds = temper_service::stock::kinds(&limits, config.core.connectors[0]);
            }
            (config, limits)
        })
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one fixture configures the policy, fake, and bounds for root forge stories"
    )]
    fn configured_settings(
        with_read: bool,
        with_change: bool,
        silent_ci: bool,
        passes: u32,
        gate: paths::Gate,
        documents: Option<jig_domain_core::documents::DocumentLimits>,
        timing: &StoryLimits,
        stock: Option<(&jig_domain_core::settings::Defaults, &engine::Model, Option<&engine::Limits>)>,
        charter_documents: &[jig_domain_core::Document],
    ) -> Self {
        let [(bootstrap, mut limits), (config, _)] =
            Self::configurations(with_read, with_change, gate, documents, timing, stock, charter_documents);
        let StoryLimits { turn, tier, across, library_gate_latency, tasks, landing_members, slots, .. } = *timing;
        let mut fake_config = forge_world::fake_config();
        fake_config.limits.repositories = 2;
        if let Some(commits) = timing.forge_commits {
            fake_config.limits.commits = commits;
        }
        if stock.is_some() {
            // The peer accepts the same whole issue bodies and retained comments.
            fake_config.limits.body_bytes = limits.forge.issue_policy.body_bytes;
            fake_config.limits.title_bytes = limits.forge.issue_policy.title_bytes;
            fake_config.limits.comments = limits.forge.issue_policy.milestones;
        }
        if turn {
            fake_config.limits.workflows = 1;
            fake_config.limits.workflow_runs = 8;
        }
        if tier.is_some() {
            // The baseline, range, rerun and probes coexist with ordinary CI runs.
            fake_config.limits.workflows = 2;
            fake_config.limits.workflow_runs = 16;
            fake_config.limits.branches = 16;
            // Four live changes retain producer and merge statuses beside the baseline.
            fake_config.limits.statuses = 16;
            // The two failed test cases and JUnit wrapper exceed a source file's 128 bytes.
            fake_config.limits.content_bytes = 512;
        }
        let mut fake = fake::Domain::new(&fake_config, 71);
        let setup = raw::Setup {
            workflows: crate::tier_runs::workflows(
                tier,
                if turn {
                    Box::new([raw::Workflow {
                        fails: Box::new([]),
                        flaky: Box::new([]),
                        report: None,
                        select: None,
                        file: b"gate.yml".as_slice().into(),
                        jobs: Box::new([b"test".as_slice().into()]),
                        latency_min: Duration::from_secs(1),
                        latency_max: Duration::from_secs(1),
                        silent: 0,
                        passes: 1000,
                        cue: None,
                    }])
                } else {
                    Box::new([])
                },
            ),
            id: 2,
            name: Box::from(&b"org/repo"[..]),
            default: Box::from(&b"main"[..]),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                cues: Box::new([]),
                contexts: if turn { Box::new([]) } else { Box::new([Box::from(&b"build"[..])]) },
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: if silent_ci { 1000 } else { 0 },
                passes,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        };
        fake::repository(&mut fake, &fake_config, setup.clone());
        if across {
            let mut setup = setup;
            if let Some(latency) = library_gate_latency {
                for workflow in &mut setup.workflows {
                    workflow.latency_min = latency;
                    workflow.latency_max = latency;
                }
            }
            fake::repository(
                &mut fake,
                &fake_config,
                raw::Setup { id: 4, name: b"org/lib".as_slice().into(), ..setup },
            );
            fake::grant(&mut fake, b"org/lib", 1, raw::Permission::Admin);
            fake::grant(&mut fake, b"org/lib", 2, raw::Permission::Write);
            fake::grant(&mut fake, b"org/lib", 7, raw::Permission::Read);
        }
        fake::grant(&mut fake, b"org/repo", 1, raw::Permission::Admin);
        fake::grant(&mut fake, b"org/repo", 2, raw::Permission::Write);
        fake::grant(&mut fake, b"org/repo", 7, raw::Permission::Read);
        let fake_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fake_config };
        // Match production's derived document bounds to the admitted authorities.
        limits.documents = paths::document_limits(documents.unwrap_or(limits.documents), &limits);
        // Keep each forge scenario's complete decision in one journal cohort.
        limits.journal.writes = if stock.is_some_and(|(_, _, deployment)| deployment.is_some()) {
            DEPLOYMENT_FORGE_WRITES
        } else if stock.is_some() {
            STOCK_FORGE_WRITES
        } else if tasks == Some(8) {
            journal::TREE_WRITES
        } else if !with_change {
            journal::WALKING_WRITES
        } else if landing_members == Some(0) {
            EMPTY_LANDING_WRITES
        } else if slots == Some(8) {
            EIGHT_SLOT_FORGE_WRITES
        } else {
            CHANGE_WRITES
        };
        // Keep the world's broad page bounds and align its message codec with the root.
        let mut store_limits = match stock {
            Some((_, _, Some(deployment))) => *deployment,
            Some((_, _, None)) | None => temper_engine_domain_world::store::widest(),
        };
        if turn || tier.is_some() {
            store_limits.tasks.executor_kinds = limits.tasks.executor_kinds;
            store_limits.tasks.authority_grants = limits.tasks.authority_grants;
            store_limits.authority.grants = limits.authority.grants;
            store_limits.authority.executors = limits.authority.executors;
            store_limits.tasks.result_bytes = limits.tasks.result_bytes;
        }
        store_limits.tasks.authority_grants = store_limits.tasks.authority_grants.max(limits.tasks.authority_grants);
        store_limits.authority.grants = store_limits.authority.grants.max(limits.authority.grants);
        store_limits.people.words = limits.people.words;
        store_limits.tasks.message_bytes = limits.tasks.message_bytes;
        store_limits.tasks.spec_bytes = limits.tasks.spec_bytes;
        store_limits.people.spec_bytes = limits.tasks.spec_bytes;
        store_limits.tasks.result_bytes = limits.tasks.result_bytes;
        store_limits.tasks.inbox_bytes = limits.tasks.inbox_bytes;
        store_limits.tasks.executor_kinds = limits.tasks.executor_kinds;
        store_limits.authority.executors = limits.authority.executors;
        store_limits.tasks.charters = limits.tasks.charters;
        store_limits.tasks.contract_choices = limits.tasks.contract_choices;
        if timing.authority_batch.is_some() {
            store_limits.authority.batch = limits.authority.batch;
            store_limits.tasks.tasks = limits.tasks.tasks;
            store_limits.tasks.project_tasks = limits.tasks.project_tasks;
            store_limits.tasks.tree_tasks = limits.tasks.tree_tasks;
            store_limits.tasks.delegates = limits.tasks.delegates;
            store_limits.tasks.batch = limits.tasks.batch;
            store_limits.tasks.inbox_messages = limits.tasks.inbox_messages;
        }
        if timing.result_bytes.is_some() {
            store_limits.forge.client.op_bytes = limits.forge.client.op_bytes;
        }
        if stock.is_some() {
            store_limits.forge.issue_policy = limits.forge.issue_policy;
            store_limits.forge.client.op_bytes = limits.forge.client.op_bytes;
            store_limits.forge.client.answer_bytes = limits.forge.client.answer_bytes;
            store_limits.forge.client.resources = limits.forge.client.resources;
            store_limits.forge.output = limits.forge.output;
            store_limits.tasks.holdings = limits.tasks.holdings;
            store_limits.journal.writes = limits.journal.writes;
        }
        let store =
            temper_engine_domain_world::fixtures::restored_over(Store::with_limits(&store_limits), bootstrap, &limits);
        let period_budget = config.core.defaults.rules.period_spend;
        Self {
            restart_config: crate::oracle::config::copy(&config),
            oracle_documents: Vec::new(),
            oracle_watcher: 0x4e33,
            broken: false,
            oracle_running: false,
            oracle_peer: None,
            creating: jig_test_profile::fixtures::Creating::new(&config.core, limits.tasks.tree_tasks),
            root: engine::Domain::new(config, &limits).expect("valid engine configuration"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
            events: VecDeque::from([engine::Event::Restart { pending: store.pending_generation }]),
            store,
            fake,
            fake_env,
            fake_out: Queue::uncapped(),
            pending: BTreeMap::new(),
            pending_repositories: BTreeMap::new(),
            located_names: BTreeMap::new(),
            run_repositories: BTreeMap::new(),
            forge_calls: 0,
            forge_reads: None,
            adopted: Vec::new(),
            follow_answers: Vec::new(),
            signed_in: None,
            web_replies: Vec::new(),
            assigned: Vec::new(),
            jobs: Vec::new(),
            job_workspaces: BTreeMap::new(),
            delivered_messages: BTreeMap::new(),
            answers: Vec::new(),
            call_errors: Vec::new(),
            reads: Vec::new(),
            panels: Vec::new(),
            approvals: Vec::new(),
            protocol: Protocol::default(),
            immediate: VecDeque::new(),
            period: None,
            period_budget,
            fail_job_reads: false,
            comparisons_too_large: false,
            fail_brief_reads: None,
            slow_brief_reads: false,
            slow_branch: None,
            review_files_delay: None,
            gate,
            held_reads: None,
            held_comparisons: None,
            fail_comparisons: false,
            hold_write_answers: None,
            held_write_answers: Vec::new(),
            lose_writes: 0,
            fail_issue_writes: false,
            fail_open_writes: false,
            issue_write_times: Vec::new(),
            drained: None,
            unsettled: Vec::new(),
            busy_calls: 0,
            merges: 0,
            writes: Vec::new(),
            repository_writes: Vec::new(),
        }
    }

    /// The configured pages base, before a multi-project fixture rebuilds startup.
    #[must_use]
    pub fn with_forge_pages(mut self, pages: &[u8]) -> Self {
        assert_eq!(self.env.now, Time::ZERO);
        self.restart_config.forges[0].web = Some(pages.into());
        self
    }

    /// A second project with the same owner and policy, before startup.
    #[must_use]
    pub fn with_second_project(mut self) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "project configuration precedes startup");
        self.env.limits.authority.projects = 2;
        self.env.limits.people.projects = 3;
        self.env.limits.people.initial_owners = 2;
        let first = &self.restart_config.core.seed_projects[0];
        let second = jig_domain_core::SeedProject { number: 2, name: "Project 2".into(), policy: first.policy.clone() };
        let mut projects = self.restart_config.core.seed_projects.into_vec();
        projects.push(second);
        self.restart_config.core.seed_projects = projects.into_boxed_slice();
        let identity = self.restart_config.core.owners[0].identity.clone();
        let mut owners = self.restart_config.core.owners.into_vec();
        owners.push(people::InitialOwner { project: 2, identity });
        self.restart_config.core.owners = owners.into_boxed_slice();
        self.creating =
            jig_test_profile::fixtures::Creating::new(&self.restart_config.core, self.env.limits.tasks.tree_tasks);
        let mut bootstrap = crate::oracle::config::copy(&self.restart_config);
        bootstrap.core.kinds = Box::new([]);
        self.store = crate::fixtures::restored_over(Store::with_limits(&self.env.limits), bootstrap, &self.env.limits);
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid two-project configuration");
        self
    }

    /// Configure an additional project's verification policy before startup.
    #[must_use]
    pub fn with_project_landing(mut self, project: u32, landing: engine::ProjectLanding) -> Self {
        assert_eq!(self.env.now, Time::ZERO);
        let mut landings = skein_lib::Map::with_capacity(self.env.limits.authority.projects);
        for (number, value) in &self.restart_config.landings {
            landings.insert(*number, value.clone()).expect("configured project");
        }
        landings.insert(project, landing).expect("additional project");
        self.restart_config.landings = landings;
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("project verification configuration");
        self
    }

    /// Choose the settings owner's identity and admit settings kinds before the world's first input.
    #[must_use]
    pub fn with_settings_owner(mut self, identity: &people::IdentityKey) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "owner identity precedes startup");
        for owner in &mut self.restart_config.core.owners {
            owner.identity = identity.clone();
        }
        self.restart_config.core.kinds =
            temper_service::stock::kinds(&self.env.limits, self.restart_config.core.connectors[0]);
        let mut bootstrap = crate::oracle::config::copy(&self.restart_config);
        bootstrap.core.kinds = Box::new([]);
        self.store = crate::fixtures::restored_over(Store::with_limits(&self.env.limits), bootstrap, &self.env.limits);
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid owner identity");
        self
    }

    /// An owned forge on a box, before bootstrap (temper/docs/design/testing/checks.md, 4).
    #[must_use]
    pub fn with_box_setup(mut self) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "box setup precedes startup");
        self.restart_config.under_box = true;
        self.restart_config.forges[0].mode = forge_top::Mode::Owned;
        let owner = self.restart_config.core.owners[0].identity.clone();
        let mut owners = self.restart_config.core.owners.into_vec();
        owners.push(people::InitialOwner { project: people::DEPLOYMENT, identity: owner });
        self.restart_config.core.owners = owners.into_boxed_slice();
        let mut bootstrap = crate::oracle::config::copy(&self.restart_config);
        bootstrap.core.kinds = Box::new([]);
        self.store = crate::fixtures::restored_over(Store::with_limits(&self.env.limits), bootstrap, &self.env.limits);
        self.store.seed(
            Box::new([Record::Forge {
                row: Box::new(forge_top::Stored::Forge(forge_top::ForgeRecord {
                    number: 1,
                    kind: forge_top::Kind::Forgejo,
                    mode: forge_top::Mode::Owned,
                    state: forge_top::ForgeState::Kept,
                    place: forge_top::Place {
                        api: forge_top::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
                        git: Box::new([]),
                        web: None,
                    },
                    read_only: false,
                    bot: forge_top::Bot { login: b"temper".as_slice().into(), name: Box::new([]), email: Box::new([]) },
                    asked: false,
                    first: None,
                    git_program: None,
                    person: None,
                })),
            }]),
            Box::new([]),
        );
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid box setup");
        self
    }

    /// Replace the forge client's admission bounds before the world's first input.
    /// The bootstrap rows and scripted worker retain their configured policy.
    #[must_use]
    pub fn with_forge_client(mut self, limits: skein_forge_client::Limits) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "client limits precede world startup");
        assert!(!self.root.ready(), "client limits precede bootstrap");
        self.env.limits.forge.client = limits;
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid client admission bounds");
        self
    }

    /// Give history reads the same settings bounds as the root before startup.
    #[must_use]
    pub fn with_settings_history(mut self) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "history bounds precede world startup");
        assert!(!self.root.ready(), "history bounds precede bootstrap");
        let rows = self.store.rows.values().cloned().collect();
        self.store = Store::with_limits(&self.env.limits);
        self.store.seed(rows, Box::new([]));
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self
    }

    /// Configure a second model, allowed by charter 2 alone, before the world's first input.
    #[must_use]
    pub fn with_producer_model(mut self, name: &[u8]) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "models precede world startup");
        assert!(!self.root.ready(), "models precede bootstrap");
        let settings = &mut self.restart_config.core.settings;
        let mut second = settings.models[0].clone();
        second.name = name.into();
        settings.models = Box::new([settings.models[0].clone(), second.clone()]);

        let producer = self
            .restart_config
            .core
            .defaults
            .charters
            .iter_mut()
            .find(|charter| charter.number == 2)
            .expect("producer charter 2");
        producer.policy.alternatives = Box::new([second]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid producer model configuration");
        self
    }

    /// Size a goal's question story for its changes and their foremen before startup
    /// (temper/docs/design/domain/forge/change/questions.md, 1, and temper/docs/design/domain/forge/branches.md, 1).
    #[must_use]
    pub fn with_goal_question_capacity(mut self, tasks: u32, inbox_bytes: u32) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "question capacity precedes startup");
        assert!(!self.root.ready(), "question capacity precedes bootstrap");
        // Keep the question story's report room beside its live result label.
        self.env.limits.tasks.result_bytes += RESULT_MESSAGE_BYTES;
        self.env.limits.forge.client.op_bytes =
            self.env.limits.forge.client.op_bytes.max(self.env.limits.tasks.result_bytes);
        self.env.limits.tasks.tasks = tasks;
        self.env.limits.tasks.project_tasks = tasks;
        self.env.limits.tasks.tree_tasks = tasks;
        self.env.limits.tasks.inbox_bytes = inbox_bytes;
        self.env.limits.tasks.batch = 2;
        self.env.limits.people.inbox_tasks = tasks * 2;
        self.env.limits.forge.tasks = tasks;
        self.env.limits.forge.client.resources = tasks * self.env.limits.forge.resources_per_task;
        self.env.limits.brief.budgets.task = 512;
        // Task, Waiting, transcript tail and Home fit beside one another.
        self.env.limits.brief.sections = 4;
        self.env.limits.brief.brief_bytes = 1024;
        self.restart_config.core.defaults.rules.ceiling.delegation.tasks = tasks;
        let policy = &mut self.restart_config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.tasks = tasks;
        policy.roles[0].authority.delegation.tasks = tasks;
        self.creating = jig_test_profile::fixtures::Creating::new(&self.restart_config.core, tasks);
        let mut codec = crate::store::widest();
        codec.tasks = self.env.limits.tasks;
        codec.forge.client.op_bytes = self.env.limits.forge.client.op_bytes;
        codec.people.spec_bytes = self.env.limits.tasks.spec_bytes;
        codec.authority = self.env.limits.authority;
        codec.people.words = self.env.limits.people.words;
        self.store = crate::fixtures::restored_over(
            Store::with_limits(&codec),
            crate::oracle::config::copy(&self.restart_config),
            &self.env.limits,
        );
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid goal question capacity");
        self
    }

    /// Configure the project's delivery checks' bound before startup.
    #[must_use]
    pub fn with_check_timeout(mut self, bound: Option<Duration>) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "check bound precedes startup");
        self.set_check_timeout(bound);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid check bound configuration");
        self
    }

    fn set_check_timeout(&mut self, bound: Option<Duration>) {
        if let Some(landing) = self.restart_config.landings.get_mut(&1) {
            landing.check_timeout = bound;
        } else {
            self.restart_config
                .landings
                .insert(
                    1,
                    engine::ProjectLanding {
                        check_timeout: bound,
                        by: engine::Way::Queue,
                        merge_job: None,
                        close_job: None,
                        repositories: Box::new([]),
                        after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                    },
                )
                .expect("one project's landing");
        }
    }

    /// Restart from durable rows with a changed project bound.
    pub fn restart_with_check_timeout(&mut self, bound: Option<Duration>) {
        self.set_check_timeout(bound);
        self.restart_same();
    }

    /// Configure the project's merge command before the first input.
    #[must_use]
    pub fn with_merge_job(mut self, command: &[u8]) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "merge command precedes world startup");
        if let Some(landing) = self.restart_config.landings.get_mut(&1) {
            landing.merge_job = Some(command.into());
        } else {
            self.restart_config
                .landings
                .insert(
                    1,
                    engine::ProjectLanding {
                        check_timeout: None,
                        close_job: None,
                        merge_job: Some(command.into()),
                        by: engine::Way::Queue,
                        repositories: Box::new([]),
                        after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                    },
                )
                .expect("one project's landing");
        }
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid merge job configuration");
        self
    }

    /// Configure the close kind and its project's command before startup.
    #[must_use]
    pub fn with_close_job(mut self, command: Option<&[u8]>, authority: bool) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "close configuration precedes startup");
        // Retain the close request and both whole change word fields in the fixture carrier.
        self.env.limits.tasks.spec_bytes = 16_384;
        self.env.limits.journal.transcript_bytes = self.env.limits.journal.transcript_bytes.max(65_536);
        self.env.limits.loads.transcript_bytes = self.env.limits.loads.transcript_bytes.max(65_536);
        self.env.limits.loads.bytes = self.env.limits.loads.bytes.max(65_536);
        self.env.limits.loads.reply_bytes = self.env.limits.loads.reply_bytes.max(self.env.limits.loads.bytes);
        self.restart_config.core.jobs = Box::new([jig_domain_link::JobBound {
            kind: jig_domain_link::JobKind(forge_top::CLOSE_KIND),
            spec: 16_384,
            result: 512,
            deadline: Duration::from_secs(600),
        }]);
        if authority {
            let kinds = &mut self.restart_config.core.defaults.rules.ceiling.delegation.kinds;
            let mut extended = kinds.to_vec();
            if !extended.contains(&authority::Executor::Job(forge_top::CLOSE_KIND)) {
                extended.push(authority::Executor::Job(forge_top::CLOSE_KIND));
            }
            let count = u32::try_from(extended.len()).expect("fixture executor count");
            self.env.limits.tasks.executor_kinds = self.env.limits.tasks.executor_kinds.max(count);
            self.env.limits.authority.executors = self.env.limits.authority.executors.max(count);
            *kinds = extended.into_boxed_slice();
            for project in &mut self.restart_config.core.seed_projects {
                project.policy.policy.ceiling.delegation.kinds.clone_from(kinds);
                project.policy.policy.roles[0].authority.delegation.kinds.clone_from(kinds);
            }
        }
        let mut landing = self.restart_config.landings.get(&1).cloned().unwrap_or(engine::ProjectLanding {
            check_timeout: None,
            by: engine::Way::Queue,
            merge_job: None,
            close_job: None,
            repositories: Box::new([]),
            after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
        });
        landing.close_job = command.map(Into::into);
        self.restart_config.landings.insert(1, landing).expect("one project's landing");
        self.env.limits.documents = paths::document_limits(self.env.limits.documents, &self.env.limits);
        self.creating =
            jig_test_profile::fixtures::Creating::new(&self.restart_config.core, self.env.limits.tasks.tree_tasks);
        let mut codec = self.env.limits;
        codec.tasks = self.env.limits.tasks;
        codec.authority = self.env.limits.authority;
        self.store = crate::fixtures::restored_over(
            Store::with_limits(&codec),
            crate::oracle::config::copy(&self.restart_config),
            &self.env.limits,
        );
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid close job configuration");
        self
    }

    /// Give these stories the goal closer's procedure without changing other fixtures.
    #[must_use]
    pub fn with_goal_closer(mut self) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "closer authority precedes startup");
        // The closer's report and new work reserve words beside the goal's landing news.
        self.env.limits.tasks.inbox_bytes = self.env.limits.tasks.inbox_bytes.max(24_576);
        self.env.limits.journal.transcript_bytes = self.env.limits.journal.transcript_bytes.max(131_072);
        self.env.limits.loads.transcript_bytes = self.env.limits.loads.transcript_bytes.max(131_072);
        self.env.limits.loads.bytes = self.env.limits.loads.bytes.max(131_072);
        self.env.limits.loads.reply_bytes = self.env.limits.loads.reply_bytes.max(131_072);
        let kinds = &mut self.restart_config.core.defaults.rules.ceiling.delegation.kinds;
        let mut extended = kinds.to_vec();
        if !extended.contains(&authority::Executor::Procedure(6)) {
            extended.push(authority::Executor::Procedure(6));
        }
        let count = u32::try_from(extended.len()).expect("fixture executor count");
        self.env.limits.tasks.executor_kinds = self.env.limits.tasks.executor_kinds.max(count);
        self.env.limits.authority.executors = self.env.limits.authority.executors.max(count);
        *kinds = extended.into_boxed_slice();
        for project in &mut self.restart_config.core.seed_projects {
            project.policy.policy.ceiling.delegation.kinds.clone_from(kinds);
            project.policy.policy.roles[0].authority.delegation.kinds.clone_from(kinds);
        }
        self.env.limits.documents = paths::document_limits(self.env.limits.documents, &self.env.limits);
        self.creating =
            jig_test_profile::fixtures::Creating::new(&self.restart_config.core, self.env.limits.tasks.tree_tasks);
        let mut codec = self.env.limits;
        codec.tasks = self.env.limits.tasks;
        codec.authority = self.env.limits.authority;
        self.store = crate::fixtures::restored_over(
            Store::with_limits(&codec),
            crate::oracle::config::copy(&self.restart_config),
            &self.env.limits,
        );
        self.events = VecDeque::from([engine::Event::Restart { pending: self.store.pending_generation }]);
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid closer configuration");
        self
    }

    /// Replace the deployment's landing rules before the world's first input.
    #[must_use]
    #[expect(clippy::needless_pass_by_value, reason = "the world builder takes ownership of its startup rule inputs")]
    pub fn with_landing_deployment(mut self, rules: Box<[engine::LandingRule]>) -> Self {
        assert_eq!(self.env.now, Time::ZERO, "landing rules precede world startup");
        assert!(!self.root.ready(), "landing rules precede bootstrap");
        let replacement = engine::landing_stock(&rules, &skein_lib::Map::with_capacity(0), 1, 4);
        let mut stock = skein_lib::List::uncapped();
        for item in replacement {
            if item.kind == 1 {
                stock.push(item).expect("uncapped replacement landing rules");
            }
        }
        for item in &self.restart_config.kinds_stock {
            if item.kind != 1 {
                stock.push(item.clone()).expect("uncapped project rules and charters");
            }
        }
        self.restart_config.kinds_stock = stock.into_boxed();
        self.root = engine::Domain::new(crate::oracle::config::copy(&self.restart_config), &self.env.limits)
            .expect("valid deployment landing rules");
        self
    }

    #[must_use]
    pub fn chat(&self, project: u32, words: &[u8]) -> people::Ask {
        walking::configured_chat(&self.creating, &self.store.rows, self.env.limits.tasks.tree_tasks, project, words)
    }

    pub fn send(&mut self, event: engine::Event) {
        if crate::waits_for_start(&self.root, &event) {
            self.events.push_back(event);
            return;
        }
        if let engine::Event::Worker(engine::WorkerInput::Link {
            channel,
            link: engine::Link::Hello { worker, hello },
        }) = &event
        {
            self.oracle_peer = Some((*channel, *worker, hello.slots));
        }
        if let engine::Event::Worker(engine::WorkerInput::Link { channel, link: engine::Link::Lost }) = &event
            && self.oracle_peer.is_some_and(|(peer, _, _)| peer == *channel)
        {
            self.oracle_peer = None;
        }
        self.immediate.push_back(event);
        self.flush();
    }

    /// Step every input waiting at once: the one sent, then what the
    /// protocol layer answered at once, as the root's door gives them.
    pub fn flush(&mut self) {
        while let Some(event) = self.immediate.pop_front() {
            let restart = matches!(&event, engine::Event::Restart { .. });
            if let Err(event) = engine::try_step(&mut self.root, &self.env, event) {
                self.events.push_back(event);
            } else if restart {
                for configured in &self.restart_config.forges {
                    if let Some(event) = crate::fixtures::connection(&self.restart_config, configured.number) {
                        self.events.push_back(event);
                    }
                }
            }
            engine::release(&mut self.root, &self.env, &mut self.out);
            self.collect();
            self.root.reclaim();
        }
    }

    /// Lose the process while retaining only the store, external forge and
    /// worker's hosted attempt. Outstanding API terminals belong to the old
    /// process; startup has to rediscover the outbox by its durable key.
    pub fn restart(&mut self, with_read: bool, with_change: bool) {
        self.restart_with_charter(with_read, with_change, None);
    }

    /// Restart with a different deployment charter while retaining the change rows.
    pub fn restart_with_charter(&mut self, with_read: bool, with_change: bool, forge_charter: Option<u32>) {
        assert!(self.store.pending.is_empty(), "the chosen cut resolves store uncertainty first");
        assert!(self.pending.is_empty(), "old API terminals are drained at this cut");
        let [_bootstrap, (config, _limits)] = Self::configurations(
            with_read,
            with_change,
            self.gate,
            None,
            &StoryLimits {
                brief: Some(self.env.limits.brief),
                forge_charter,
                questions: Some(self.env.limits.forge.change_policy.questions),
                applications: Some(self.env.limits.forge.applications),
                stall: Some(self.env.limits.forge.change_policy.stall),
                ..StoryLimits::default()
            },
            None,
            &[],
        );
        self.restart_domain(config, with_change);
    }

    /// Choose whether the next restart constructs the connector under a box.
    #[must_use]
    pub fn with_restart_under_box(mut self, under_box: bool) -> Self {
        self.restart_config.under_box = under_box;
        self
    }

    /// Restart with the same tier and project configuration, retaining only durable rows.
    pub fn restart_same(&mut self) {
        self.restart_domain(crate::oracle::config::copy(&self.restart_config), true);
    }

    fn restart_domain(&mut self, config: engine::Config, with_change: bool) {
        assert!(self.store.pending.is_empty(), "store answers precede a restart");
        assert!(self.pending.is_empty(), "API answers precede a restart");
        self.root =
            engine::Domain::new(crate::oracle::config::copy(&config), &self.env.limits).expect("restart config");
        self.restart_config = config;
        self.out = Queue::uncapped();
        self.events.clear();
        self.held_write_answers.clear();
        self.hold_write_answers = None;
        self.drained = None;
        self.unsettled.clear();
        self.answers.clear();
        self.call_errors.clear();
        self.adopted.clear();
        self.period = None;
        self.events.push_back(engine::Event::Restart { pending: self.store.pending_generation });
        if !self.assigned.is_empty() {
            self.events.push_back(protocol::hello_slots(
                Token::new(7),
                self.oracle_peer.map_or(if with_change { 2 } else { 1 }, |(_, _, slots)| slots),
                self.assigned
                    .iter()
                    .filter(|assignment| matches!(self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task)))), Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == assignment.attempt)))
                    .map(|assignment| {
                        protocol::hosting(assignment.task, assignment.attempt, jig_domain_link::Phase::Active)
                    })
                    .collect(),
            ));
        }
    }

    /// Refuse full comparisons at the protocol body bound; ancestry reads still reach the fake.
    pub fn reject_comparisons(&mut self) {
        self.comparisons_too_large = true;
    }

    pub fn external(&mut self, user: u64, op: raw::Op) -> raw::Answer {
        self.external_at(b"org/repo", user, op)
    }

    pub fn external_at(&mut self, repository: &[u8], user: u64, op: raw::Op) -> raw::Answer {
        let mut config = self.fake_env.limits;
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        let env = self.env.with(config);
        fake::step(
            &mut self.fake,
            &env,
            fake::Event::Call {
                reply_to: ReplyTo::new(Token::new(10_000)),
                user,
                repository: Box::from(repository),
                op,
            },
            &mut self.fake_out,
        );
        for _ in 0..8 {
            if self.fake.is_due(env.now) {
                fake::fire(&mut self.fake, &env, &mut self.fake_out);
            }
            while let Some(output) = self.fake_out.pop() {
                if let fake::Request::Reply { result, .. } = output {
                    return result.expect("external fake call");
                }
            }
        }
        panic!("external fake call did not return")
    }

    #[expect(clippy::too_many_lines, reason = "one exhaustive root output table")]
    pub fn collect(&mut self) {
        for _ in 0..self.out.len() {
            let request = self.out.pop().expect("counted root output");
            let Some(request) = self.stored_read(request) else { continue };
            let request = match self.protocol.play(request) {
                // The protocol layer answers at once (testing-strategy.md, 4).
                Ok(events) => {
                    self.immediate.extend(events);
                    continue;
                }
                Err(request) => *request,
            };
            match request {
                engine::Request::ToChild(released) => self.events.push_back(engine::Event::Released(released)),
                engine::Request::Store(engine::StoreRequest::Catalogue { names }) => self.store.catalogue(names),
                engine::Request::Store(engine::StoreRequest::Switch) => {
                    let answer = self.store.switch();
                    self.events.push_back(engine::Event::Store(answer));
                }
                engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                    self.store.pending.push_back((number, writes));
                }
                engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. }) => {
                    let (rows, next) = self.store.page(&range, after.as_ref(), most);
                    self.events.extend(self.store.take_missing(owner).into_iter().map(engine::Event::Store));
                    self.events.push_back(engine::Event::Store(engine::StoreInput::Loaded {
                        owner,
                        rows,
                        next,
                        texts: None,
                    }));
                }
                engine::Request::Forge { call, repository, op } => {
                    if matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Compare { .. })) {
                        if let Some(reads) = &mut self.held_comparisons {
                            reads.push(engine::Request::Forge { call, repository, op });
                            continue;
                        }
                        if self.fail_comparisons {
                            self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::Client(
                                skein_forge_client::Event::Answered {
                                    free: 0,
                                    rate: None,
                                    call,
                                    cost: 1,
                                    result: Err(skein_forge::api::Error::Forbidden),
                                },
                            ))));
                            continue;
                        }
                    }
                    if let Some(reads) = &mut self.held_reads
                        && matches!(op, skein_forge::api::Op::Read(_))
                        && self.store.rows.values().any(|row| matches!(row,
                            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(row)
                                if matches!(row.change.state, temper_engine_domain_forge_change::State::Queued { .. } | temper_engine_domain_forge_change::State::First { .. }))))
                    {
                        reads.push(engine::Request::Forge { call, repository, op });
                        continue;
                    }
                    if matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::Merge { .. })) {
                        self.merges += 1;
                    }
                    if let skein_forge::api::Op::Write(write) = &op {
                        self.writes.push(write.clone());
                        self.repository_writes.push((repository, write.clone()));
                    }
                    if let (Some(reads), skein_forge::api::Op::Read(read)) = (&mut self.forge_reads, &op) {
                        reads.push((call, read.clone()));
                    }
                    self.forge_calls += 1;
                    let issue_write = match &op {
                        skein_forge::api::Op::Write(skein_forge::api::Write::CreateIssue { .. }) => true,
                        skein_forge::api::Op::Write(
                            skein_forge::api::Write::Edit { number, .. }
                            | skein_forge::api::Write::Post { number, .. }
                            | skein_forge::api::Write::Close { number },
                        ) => self.store.rows.values().any(|row| {
                            matches!(row,
                                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Issue(issue)
                                    if issue.number == Some(*number)))
                        }),
                        _ => false,
                    };
                    if issue_write {
                        self.issue_write_times.push(self.env.wall);
                    }
                    let fail_open = self.fail_open_writes
                        && matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::OpenPull { .. }));
                    if (self.fail_issue_writes && issue_write) || fail_open {
                        self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::Client(
                            skein_forge_client::Event::Answered {
                                free: 0,
                                rate: None,
                                call,
                                cost: 1,
                                result: Err(if fail_open {
                                    skein_forge::api::Error::Forbidden
                                } else {
                                    skein_forge::api::Error::Refused
                                }),
                            },
                        ))));
                        continue;
                    }

                    if self.lose_writes > 0 && matches!(op, skein_forge::api::Op::Write(_)) {
                        self.lose_writes -= 1;
                        self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::Client(
                            skein_forge_client::Event::Answered {
                                free: 0,
                                rate: None,
                                call,
                                cost: 1,
                                result: Err(skein_forge::api::Error::Timeout),
                            },
                        ))));
                        continue;
                    }
                    // Nonparticipating keep-up reads Item after u64::MAX. The change
                    // finishes its fresh Checks before its repair enters Preparing.
                    if let Some((pull, checks, error)) = self.fail_brief_reads
                        && self.store.rows.values().any(|row| {
                            matches!(row,
                            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                                if row.spec.words.starts_with(b"Repair CI at ")
                                    && row.phase == tasks::Phase::Active(tasks::Active::Preparing))
                        })
                        && ((pull
                            && matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Item { after: 0, .. })))
                            || (checks
                                && matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Checks { .. }))))
                    {
                        self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::Client(
                            skein_forge_client::Event::Answered {
                                free: 0,
                                rate: None,
                                call,
                                cost: 0,
                                result: Err(error),
                            },
                        ))));
                        continue;
                    }
                    if self.comparisons_too_large
                        && matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Compare { .. }))
                    {
                        self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::Client(
                            skein_forge_client::Event::Answered {
                                free: 0,
                                rate: None,
                                call,
                                cost: 1,
                                result: Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed())),
                            },
                        ))));
                        continue;
                    }
                    if self.fail_job_reads
                        && matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Job { .. }))
                    {
                        self.events.push_back(engine::Event::Forge(Box::new(
                            temper_engine_domain_forge::Event::Client(skein_forge_client::Event::Answered {
                                free: 0,
                                rate: None,
                                call,
                                cost: 0,
                                result: Err(skein_forge::api::Error::Refused),
                            }),
                        )));
                        continue;
                    }
                    let located = translate::locate_name(&op);
                    let name: &[u8] = if let Some(name) = &located {
                        name
                    } else if let Some(name) = self.located_names.get(&repository.repository) {
                        name
                    } else if repository == forge_world::REPO {
                        b"org/repo"
                    } else if repository == (skein_forge::api::Repository { forge: 1, repository: 4 }) {
                        b"org/lib"
                    } else if repository == (skein_forge::api::Repository { forge: 1, repository: 3 }) {
                        b"org/extra"
                    } else {
                        panic!("unknown test repository {repository:?}");
                    };
                    let review_delay =
                        if matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::PullFiles { .. })) {
                            self.review_files_delay
                        } else {
                            None
                        };
                    let slow = (self.slow_brief_reads
                        && matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Job { .. })))
                        || matches!((&op, &self.slow_branch),
                            (skein_forge::api::Op::Read(skein_forge::api::Read::Branch { branch }), Some(slow))
                                if branch == slow);
                    let raw = translate::op(&op, &self.env.limits.forge.client);
                    if matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Run { .. })) {
                        self.run_repositories.insert(call, name.into());
                    }
                    self.pending.insert(call, op);
                    self.pending_repositories.insert(call, Box::from(name));
                    let mut limits = self.fake_env.limits;
                    if slow {
                        limits.latency_min = Duration::from_secs(20);
                        limits.latency_max = Duration::from_secs(20);
                    }
                    if let Some(delay) = review_delay {
                        limits.latency_min = delay;
                        limits.latency_max = delay;
                    }
                    let fake_env = self.fake_env.with(limits);
                    fake::step(
                        &mut self.fake,
                        &fake_env,
                        fake::Event::Call {
                            reply_to: ReplyTo::new(call),
                            user: 1,
                            repository: Box::from(name),
                            op: raw,
                        },
                        &mut self.fake_out,
                    );
                }
                engine::Request::Worker(engine::WorkerRequest::Deliver(delivery))
                | engine::Request::Party(engine::PartyRequest::Deliver(delivery)) => match delivery {
                    Delivery::WebReply { reply, .. } if !matches!(reply, people::Reply::SignedIn { .. }) => {
                        self.web_replies.push(reply);
                    }
                    Delivery::WebReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } => {
                        self.signed_in = Some(sign_in);
                    }
                    Delivery::Assigned { assignment, .. } => self.assigned.push(assignment),
                    Delivery::AssignedJob { job, workspace, .. } => {
                        if let Some(workspace) = workspace {
                            self.job_workspaces.insert(job.run.raw(), workspace);
                        }
                        self.jobs.push(job);
                    }
                    Delivery::Read { reply, answer } => self.reads.push((reply, answer)),
                    Delivery::Link { down: host::LinkDown::Message { run, attempt, name, .. }, .. } => {
                        self.delivered_messages.entry((run.raw(), attempt.raw())).or_default().push(name.raw());
                    }
                    Delivery::Panel { watcher, panel } => self.panels.push((watcher, *panel)),
                    Delivery::Approvals { watcher, document } => {
                        self.approvals.push((watcher, *document));
                        self.events.push_back(engine::Event::Party(engine::PartyInput::Delivered { watcher }));
                    }
                    Delivery::ForgeDocument { watcher, document } => self.oracle_documents.push((watcher, *document)),
                    Delivery::Link {
                        down: host::LinkDown::Relayed { call, answer: host::Relayed::Answer { error: true, text }, .. },
                        ..
                    } => self.call_errors.push(CallError { call, text }),
                    // The run has its answer once the hub relays it down; the
                    // world renders an answer as its first call's name, and a
                    // replayed call carries that same text.
                    Delivery::Link {
                        down: host::LinkDown::Relayed { answer: host::Relayed::Answer { text, .. }, .. },
                        ..
                    } => {
                        let (_, answer) = self
                            .protocol
                            .answers
                            .iter()
                            .find(|(name, _)| *name == text)
                            .expect("the world rendered every relayed answer");
                        self.answers.push(answer.clone());
                    }
                    Delivery::Link { down: host::LinkDown::Relayed { answer: host::Relayed::Busy, .. }, .. } => {
                        self.busy_calls += 1;
                    }
                    Delivery::Application { outcome, .. } => match outcome {
                        Some(engine::ApplicationAnswer::Adopt(outcome)) => self.adopted.push(outcome),
                        Some(engine::ApplicationAnswer::Follow(outcome)) => self.follow_answers.push(outcome),
                        Some(engine::ApplicationAnswer::Looked(_)) => {}
                        answer => panic!("retained application answer: {answer:?}"),
                    },
                    _ => {}
                },
                engine::Request::AdoptAccess { forge, request, .. } => {
                    self.events.push_back(engine::Event::Forge(Box::new(forge_top::Event::AccessAnswered {
                        forge,
                        request,
                        answer: Ok(()),
                    })));
                }
                engine::Request::Account(_)
                | engine::Request::ForgeKept(_)
                | engine::Request::ForgeGone { .. }
                | engine::Request::BoxGroup { .. }
                | engine::Request::PlaygroundActs { .. }
                | engine::Request::ResetAct { .. }
                | engine::Request::BoxRemove { .. }
                | engine::Request::Connect { .. }
                | engine::Request::Replace { .. }
                | engine::Request::Manifest { .. }
                | engine::Request::Reachable { .. }
                | engine::Request::Install { .. }
                | engine::Request::Accept { .. }
                | engine::Request::EraseCarried { .. }
                | engine::Request::MakingCopied { .. }
                | engine::Request::SetUp { .. }
                | engine::Request::Bound { .. }
                | engine::Request::Setup { .. }
                | engine::Request::Party(_)
                | engine::Request::Store(_)
                | engine::Request::Busy(_) => {}
                engine::Request::Unsettled(write) => {
                    assert!(self.drained.is_none(), "unfinished entries precede Drained");
                    self.unsettled.push(write);
                }
                engine::Request::Drained => {
                    assert!(self.drained.is_none(), "drained once");
                    self.drained = Some(self.store.header().commits);
                }
                engine::Request::Worker(other) => panic!("forge fixture received an unexpected request {other:?}"),
                engine::Request::Stop => panic!("root startup or store failure: {:?}", self.root.start_stop()),
            }
        }
    }

    /// Release the terminals this world held; the provider operation is not repeated.
    pub fn release_write_answers(&mut self) {
        self.hold_write_answers = None;
        self.events.extend(self.held_write_answers.drain(..));
    }

    /// Read the messages offered to this run before its next scripted action.
    pub fn scripted_read_inbox(&mut self, run: &engine::Assignment) {
        let proof_key = Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(run.task)));
        let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
            self.store.rows.get(&proof_key)
        else {
            panic!("scripted run proof")
        };
        let Some(read) = proof.outstanding.last().copied() else { return };
        let task_key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(run.task)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            self.store.rows.get(&task_key)
        else {
            panic!("scripted reader is live")
        };
        let turn = row.turn + 1;
        self.send(protocol::turn(
            Token::new(7),
            run.task,
            run.attempt,
            turn,
            0,
            Some(read),
            b"Read the offered messages",
        ));
        for _ in 0..STEPS {
            self.tick();
            if let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
                self.store.rows.get(&task_key)
                && row.turn >= turn
                && self.store.pending.is_empty()
            {
                return;
            }
        }
        panic!("scripted reader commits turn {turn}");
    }

    /// Send a scripted agent's tool call and commit its turn's call evidence.
    pub fn scripted_call(&mut self, run: &engine::Assignment, number: u64, tool: engine::Tool) -> engine::CallAnswer {
        const CHANNEL: Token = Token::new(7);
        self.scripted_read_inbox(run);
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(run.task)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) = self.store.rows.get(&key)
        else {
            panic!("scripted caller is live")
        };
        let turn = row.turn + 1;
        let before = self.answers.len();
        let errors = self.call_errors.len();
        let event = self.protocol.call(
            CHANNEL,
            run.task,
            run.attempt,
            number,
            engine::Call { completion: turn, position: 1, tool },
        );
        self.send(event);
        for _ in 0..STEPS {
            self.tick();
            assert_eq!(self.call_errors.len(), errors, "scripted call {number}: {:?}", self.call_errors);
            if self.answers.len() > before && self.store.pending.is_empty() {
                let answer = self.answers[before].clone();
                let proof_key = Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(run.task)));
                let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
                    self.store.rows.get(&proof_key)
                else {
                    panic!("scripted run proof")
                };
                let read = proof.turn.as_ref().and_then(|turn| turn.read);
                self.send(protocol::turn(CHANNEL, run.task, run.attempt, turn, 0, read, b"Continue the agreed work"));
                for _ in 0..STEPS {
                    self.tick();
                    if let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
                        self.store.rows.get(&key)
                        && row.turn >= turn
                        && self.store.pending.is_empty()
                    {
                        return answer;
                    }
                }
                panic!("scripted call {number} turn committed");
            }
        }
        panic!("scripted call {number} answered: {:?}", self.answers);
    }

    /// A producer asks its change through the worker's message call.
    pub fn producer_question(&mut self, producer: &engine::Assignment, number: u64, words: &[u8]) -> u64 {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(producer.task)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) = self.store.rows.get(&key)
        else {
            panic!("live producer")
        };
        let tasks::Party::Task(target) = row.requester else { panic!("producer's change") };
        let engine::CallAnswer::Sent { message } = self.scripted_call(
            producer,
            number,
            engine::Tool::Message { target, form: engine::MessageForm::Question, words: words.into() },
        ) else {
            panic!("producer's question sent")
        };
        message
    }

    /// The goal answers the change's own question through its message call.
    pub fn goal_answer(&mut self, goal: &engine::Assignment, number: u64, change: u64, question: u64, words: &[u8]) {
        let answer = self.scripted_call(
            goal,
            number,
            engine::Tool::Message {
                target: change,
                form: engine::MessageForm::Answer { question },
                words: words.into(),
            },
        );
        assert!(matches!(answer, engine::CallAnswer::Sent { .. }), "goal answers question {question}: {answer:?}");
    }

    /// Drain the current terminals before choosing a committed restart cut.
    pub fn drain_cut(&mut self) {
        for _ in 0..STEPS {
            if self.store.pending.is_empty() && self.pending.is_empty() && self.events.is_empty() {
                return;
            }
            self.tick();
        }
        assert!(
            self.store.pending.is_empty() && self.pending.is_empty() && self.events.is_empty(),
            "restart cut drains within the fixture's bound"
        );
    }

    /// Restore a delegate question with its reserved answer room at a committed cut.
    pub fn restore_delegate_question(&mut self, delegate: u64, words: &[u8]) -> u64 {
        self.restore_delegate_message(delegate, words, tasks::MessageKind::Question)
    }

    /// Restore a delegate's plain words in its change's inbox at a committed cut.
    pub fn restore_delegate_words(&mut self, delegate: u64, words: &[u8]) -> u64 {
        self.restore_delegate_message(delegate, words, tasks::MessageKind::Words)
    }

    fn restore_delegate_message(&mut self, delegate: u64, words: &[u8], kind: tasks::MessageKind) -> u64 {
        self.drain_cut();
        assert!(self.store.pending.is_empty());
        let mut header = self.store.header();
        header.messages += 1;
        header.commits += 1;
        let question = header.messages;
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(delegate)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(mut sender)))) =
            self.store.rows.get(&key).cloned()
        else {
            panic!("live delegate")
        };
        let tasks::Party::Task(change) = sender.requester else { panic!("delegate's requester") };
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(change)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(mut target)))) =
            self.store.rows.get(&key).cloned()
        else {
            panic!("live change")
        };
        assert!(target.delegates.contains(&delegate));
        sender.activity += 1;
        if kind == tasks::MessageKind::Question {
            let mut credits = sender.questions.into_vec();
            credits.push(tasks::QuestionCredit {
                number: question,
                answerer: tasks::Party::Task(change),
                asked: tasks::Party::Task(change),
                line: sender.activity,
            });
            sender.questions = credits.into_boxed_slice();
        }
        let mut inbox = target.inbox.into_vec();
        inbox.push(tasks::Word {
            number: question,
            from: tasks::Party::Task(delegate),
            kind: kind.clone(),
            words: words.into(),
            at: self.env.wall,
            hits: 1,
            eligible: true,
        });
        target.inbox = inbox.into_boxed_slice();
        target.last_message = question;
        target.phase = tasks::Phase::Active(tasks::Active::Due);
        let writes = Box::new([
            engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(
                tasks::ActivityLine {
                    task: delegate,
                    position: sender.activity,
                    at: self.env.wall,
                    line: tasks::Line::Words {
                        message: question,
                        by: tasks::Party::Task(delegate),
                        kind,
                        words: words.into(),
                    },
                },
            )))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(sender)))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(target)))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                header,
            )))),
        ]);
        self.store.pending.push_back((header.commits, writes));
        self.store.apply();
        self.restart(true, true);
        question
    }

    pub fn tick(&mut self) {
        self.tick_with_reply_cut(None);
    }

    /// Apply the step saving a reply to `target`, losing its commit terminal.
    /// Other decisions and the clock follow an ordinary world tick.
    pub fn tick_losing_reply_commit(&mut self, target: u64) -> bool {
        self.tick_with_reply_cut(Some(target))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one tick preserves reply cuts, located sources and runner job answers together"
    )]
    fn tick_with_reply_cut(&mut self, target: Option<u64>) -> bool {
        let mut lost = false;
        if self.oracle_running {
            if !self.pending.is_empty()
                && let Some(due) = self.fake.next_deadline()
            {
                self.env.now = self.env.now.max(due);
            }
        } else {
            self.env.now = self.env.now.saturating_add(Duration::from_millis(100));
        }
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        self.fake_env.now = self.env.now;
        self.fake_env.wall = self.env.wall;
        if let Some(index) = self.events.iter().position(|event| !crate::waits_for_start(&self.root, event)) {
            let event = self.events.remove(index).expect("selected eligible input");
            self.send(event);
        }
        if let Some((_, writes)) = self.store.pending.front() {
            lost = target.is_some_and(|target| {
                writes.iter().any(|write| {
                    matches!(write,
                engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))))
                    if row.number == target && row.inbox.iter().any(|word| matches!(word.from, tasks::Party::Task(_))))
                })
            });
            let number = self.store.apply();
            if !lost {
                self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            }
        }
        if self.period.is_none() && self.root.ready() {
            self.period = Some(1);
            let period = engine::PartyInput::Period { project: 1, period: 1, budget: self.period_budget, next: None };
            self.send(engine::Event::Party(period));
            // The repository's git identity, an application credential from
            // temper's own source (domain/root.md, 6.1).
            self.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
        }
        if self.fake.is_due(self.fake_env.now) {
            fake::fire(&mut self.fake, &self.fake_env, &mut self.fake_out);
        }
        for _ in 0..self.fake_out.len() {
            match self.fake_out.pop().expect("counted fake output") {
                fake::Request::Reply { to, result } => {
                    let call = to.into_token();
                    let op = self.pending.remove(&call).expect("one pending forge call");
                    let repository = self.pending_repositories.remove(&call).expect("pending call's repository");
                    let result = match result {
                        Ok(answer) => {
                            let mut answer = translate::answer(&op, answer, &self.env.limits.forge.client);
                            if let Ok(skein_forge::api::Answer::Run(run)) = &mut answer {
                                let raw::Answer::Jobs(jobs) = self
                                    .fake
                                    .inspect(&self.fake_env.limits, &repository, &raw::Read::Jobs { run: run.id })
                                    .expect("dispatched run jobs")
                                else {
                                    panic!("run jobs answer")
                                };
                                run.jobs = translate::run_jobs(jobs);
                            }
                            answer
                        }
                        Err(error) => {
                            Err(translate::error(error, fake::time(&self.fake_env.limits, self.fake_env.now)))
                        }
                    };
                    if let Ok(skein_forge::api::Answer::Located { repository }) = &result
                        && let Some(name) = translate::locate_name(&op)
                    {
                        self.located_names.insert(*repository, name);
                    }
                    self.run_repositories.remove(&call);
                    let event = engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
                        skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result },
                    )));
                    let held = match self.hold_write_answers {
                        Some(HoldWrite::All) => matches!(op, skein_forge::api::Op::Write(_)),
                        Some(HoldWrite::Merge) => {
                            matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::Merge { .. }))
                        }
                        None => false,
                    };
                    if held {
                        self.held_write_answers.push(event);
                    } else {
                        self.events.push_back(event);
                    }
                }
                fake::Request::Hook { .. } => {}
            }
        }
        temper_engine_domain_world::poll(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.flush();
        if !self.oracle_running {
            engine::fire(&mut self.root, &self.env);
        }
        engine::release(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.flush();
        self.root.reclaim();
        self.fake.reclaim();
        lost
    }

    pub fn until(&mut self, target: Until) {
        for _ in 0..200 {
            self.tick();
            let done = match target {
                Until::Ready => self.root.quiescent() && self.events.is_empty() && self.store.pending.is_empty(),
                Until::SignedIn => self.signed_in.is_some(),
                Until::Adopted => !self.adopted.is_empty(),
                Until::Assigned => !self.assigned.is_empty(),
                Until::Subscribed => self
                    .answers
                    .iter()
                    .any(|answer| matches!(answer, temper_engine_domain::CallAnswer::Subscribed { .. })),
                Until::News => {
                    let task = self.assigned.first().expect("assigned subscriber").task;
                    match self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
                        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => {
                            row.inbox.iter().any(|word| matches!(word.kind, tasks::MessageKind::News { .. }))
                        }
                        _ => false,
                    }
                }
                Until::Read => {
                    self.answers.iter().any(|answer| matches!(answer, temper_engine_domain::CallAnswer::ForgeRead(_)))
                }
                Until::Effect => self
                    .answers
                    .iter()
                    .any(|answer| matches!(answer, temper_engine_domain::CallAnswer::ForgeEffect { .. })),
                Until::Delegated => {
                    self.answers.iter().any(|answer| matches!(answer, temper_engine_domain::CallAnswer::Delegated(_)))
                }
                Until::SecondAssignment => self.assigned.len() >= 2,
            };
            if done {
                return;
            }
        }
        panic!(
            "forge root did not reach {target:?}; answers={:?}, assignments={:?}, task2={:?}, task3={:?}, forge={:?}",
            self.answers,
            self.assigned,
            self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)))),
            self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(3)))),
            self.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
        );
    }

    pub fn adopt(&mut self) {
        self.adopt_with_checks(true, Box::new([]));
    }

    pub fn sign_in(&mut self) {
        self.until(Until::Ready);
        self.send(engine::Event::Party(engine::PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(90)),
            identity: people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: Box::from(&b"owner"[..]),
                name: Box::from(&b"Owner"[..]),
            },
        }));
        self.until(Until::SignedIn);
    }

    pub fn adopt_with_checks(&mut self, ci: bool, checks: Box<[u32]>) {
        self.sign_in();
        self.send(engine::Event::Party(engine::PartyInput::Application {
            scope: jig_domain_people::ApplicationScope::Project(1),
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: self.signed_in.expect("signed in"),
            key: [91; 16],
            digest: [5; 32],
            carried: None,
            request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
                project: 1,
                home: true,
                forge: forge_world::REPO.forge,
                host: Box::from(&b"forge.example"[..]),
                owner: Box::from(&b"org"[..]),
                name: Box::from(&b"repo"[..]),
                prefix: Box::from(&b"temper/"[..]),
                role: forge_top::Role::Owned,
                landing: Box::from(&b"main"[..]),
                ci,
                checks,
            })),
        }));
        self.until(Until::Adopted);
        assert!(matches!(self.adopted.as_slice(), [forge_top::ApplicationOutcome::Adopted(_)]), "{:?}", self.adopted);
    }
}

#[must_use]
pub fn change_world(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    change_world_with_gate(silent_ci, passes, approval, false)
}

#[must_use]
pub fn change_world_with_gate(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
    agent_gate: bool,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    change_world_with_policy(silent_ci, passes, approval, agent_gate, false, false, false)
}

#[expect(clippy::fn_params_excessive_bools, reason = "independent CI, gate and policy settings select test worlds")]
#[must_use]
pub fn change_world_with_policy(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
    agent_gate: bool,
    no_ci: bool,
    owner_gate: bool,
    tracked: bool,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    let world = World::configured_policy(
        true,
        true,
        silent_ci,
        passes,
        paths::Gate { approval, agent: agent_gate, owner: owner_gate },
        None,
    );
    change_world_in(world, paths::Change { no_ci, owner_gate, tracked })
}

#[must_use]
pub fn change_world_in(
    world: World,
    policy: paths::Change,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    change_world_funded(world, policy, 40)
}

#[expect(clippy::too_many_lines, reason = "the root change fixture builds both producer and review routes")]
#[must_use]
pub fn change_world_funded(
    mut world: World,
    policy: paths::Change,
    spend: u64,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    let paths::Change { no_ci, owner_gate: _, tracked } = policy;
    if no_ci {
        world.adopt_with_checks(false, Box::new([1]));
    } else {
        world.adopt();
    }
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = if tracked {
        walking::goal_from(
            world.chat(1, b"make the fix"),
            people::TaskExecutor::Agent { charter: 1 },
            jig_test_profile::fixtures::Goal { budget: 200, priority: 3, deadline: None },
        )
    } else {
        world.chat(1, &b"make the fix"[..])
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask,
    }));
    world.until(Until::Assigned);
    let chat = world.assigned[0].clone();
    // Tracked stories put the owner below another tracked task, as in the
    // director's chat. Both goals must gather the change in their subtree.
    let chat = if tracked { delegated_goal(&mut world, &chat) } else { chat };
    let producer_index = world.assigned.len();
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(owner)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(chat.task))))
    else {
        panic!("change owner remains live")
    };
    let root = owner.root;
    let grant = tasks::Grant {
        connector: 0,
        kind: 2,
        pattern: tasks::Pattern {
            segments: Box::new([Box::from(&b"forge"[..]), Box::from(&b"forge.example"[..]), Box::from(&b"org"[..])]),
            last: tasks::Last::Open(Box::from(&b"repo"[..])),
        },
    };
    let child_authority = tasks::Authority {
        tools: tasks::Tools(1023),
        grants: [1, 3, 4, 10, 11, 12].map(|kind| tasks::Grant { kind, ..grant.clone() }).into(),
        // The change's six tasks: a producer and five review attempts.
        delegation: tasks::Delegation {
            kinds: Box::new([
                tasks::AuthorityExecutor::Charter(1),
                tasks::AuthorityExecutor::Charter(2),
                tasks::AuthorityExecutor::Charter(4),
            ]),
            tasks: 6,
            depth: 1,
        },
        // Each run's spend must fund its finishing reserve, six with the
        // world's charter: room for six attempts of five.
        budget: tasks::Budget { spend, deadline: None },
        notes: tasks::Scopes(0),
        note_resources: Box::new([]),
        models: tasks::Models::NONE,
    };
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Delegate {
                batch: Box::new([engine::Delegate {
                    executor: tasks::Executor::Procedure { connector: 0, code: 2 },
                    spec: tasks::Spec {
                        job: None,
                        asks: false,
                        words: Box::from(&b"Small fix"[..]),
                        parameters: Box::new([
                            tasks::Parameter::Resource {
                                name: 1,
                                connector: 0,
                                resource: temper_engine_domain_forge::tag(
                                    forge_world::REPO.forge,
                                    forge_world::REPO.repository,
                                ),
                            },
                            tasks::Parameter::Bytes { name: 2, value: Box::from(&b"main"[..]) },
                            tasks::Parameter::Bytes { name: 6, value: Box::from(&b"file"[..]) },
                        ]),
                        inputs: Box::new([]),
                    },
                    // Reserve 32 bytes for landing identity beside its live result label.
                    contract: tasks::Contract::Change { connector: 0, kind: 1, words: 32 + RESULT_MESSAGE_BYTES },
                    authority: child_authority,
                    symbolic_grants: [2, 17]
                        .map(|kind| tasks::Grant {
                            connector: 0,
                            kind,
                            pattern: tasks::Pattern {
                                segments: Box::new([
                                    Box::from(&b"forge"[..]),
                                    Box::from(&b"forge.example"[..]),
                                    Box::from(&b"org"[..]),
                                    Box::from(&b"repo"[..]),
                                    Box::from(&b"branch"[..]),
                                    Box::from(&b"temper"[..]),
                                    root.to_string().into_bytes().into_boxed_slice(),
                                ]),
                                last: tasks::Last::Open(Box::from(&b"c"[..])),
                            },
                        })
                        .into(),
                    dependencies: Box::new([]),
                    wake: Some(tasks::WakePolicy::DEFAULT),
                    ..temper_engine_domain_world::fixtures::report_delegate(b"fixture", Box::new([]))
                }]),
            },
        },
    );
    world.send(event);
    world.until(Until::Delegated);
    for _ in 0..200 {
        if world.assigned.len() > producer_index {
            break;
        }
        world.tick();
    }
    let producer = world.assigned.get(producer_index).expect("change producer assigned").clone();
    assert_eq!(
        producer.workspace.repositories[0].landing.as_deref(),
        Some(b"main".as_slice()),
        "initial producer starts from Base"
    );
    let change = world
        .answers
        .iter()
        .rev()
        .find_map(|answer| match answer {
            temper_engine_domain::CallAnswer::Delegated(numbers) => numbers.first().map(|made| made.task),
            _ => None,
        })
        .expect("change task number");
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                if task.number == producer.task && task.authority.grants.iter().any(|grant|
                    grant.connector == 0 && grant.kind == 2
                        && grant.pattern.segments.last().map(AsRef::as_ref)
                            == Some(root.to_string().as_bytes())
                        && matches!(&grant.pattern.last, tasks::Last::Exact(prefix)
                            if prefix.as_ref() == format!("c{change}").as_bytes()))
        )),
        "producer inherits the change task's numbered branch grant"
    );
    let branch = producer.workspace.repositories[0].push.clone().expect("writable change branch");
    (world, chat, producer, branch)
}

fn delegated_goal(world: &mut World, parent: &engine::Assignment) -> engine::Assignment {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(parent.task))))
    else {
        panic!("tracked parent remains live")
    };
    let mut authority = row.authority.clone();
    authority.budget.spend = 100;
    authority.delegation.depth = 2;
    authority.delegation.tasks = 8;
    let event = world.protocol.call(
        Token::new(7),
        parent.task,
        parent.attempt,
        90,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Delegate {
                batch: Box::new([engine::Delegate {
                    tracked: Some(3),
                    authority,
                    ..temper_engine_domain_world::fixtures::report_delegate(b"delegated goal", Box::new([]))
                }]),
            },
        },
    );
    world.send(event);
    world.until(Until::SecondAssignment);
    let goal = world.assigned[1].clone();
    let event =
        world.protocol.answer(Token::new(7), parent.task, parent.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    for _ in 0..20 {
        world.tick();
    }
    goal
}

#[derive(Clone, Copy, Default)]
pub struct Gate {
    pub approval: Option<engine::Freshness>,
    pub agent: bool,
    pub owner: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Change {
    pub no_ci: bool,
    pub owner_gate: bool,
    pub tracked: bool,
}

/// Two live procedures with distinct landing branches. The scripted root lets
/// stories observe `WakeProcedure` before offering each fresh-read step.
#[must_use]
pub fn hint_world() -> forge_world::World {
    hint_world_for_goal(false)
}

#[must_use]
pub fn shared_goal_hint_world() -> forge_world::World {
    hint_world_for_goal(true)
}

fn hint_world_for_goal(shared_goal: bool) -> forge_world::World {
    use temper_engine_domain_forge_change as change;
    let mut world = forge_world::World::new(71);
    world.adopt();
    world.produce(b"release");
    let first_base = if shared_goal { b"release".as_slice() } else { b"main".as_slice() };
    for (task, branch, base) in [(51, b"temper/a".as_slice(), first_base), (52, b"temper/b", b"release")] {
        world.produce_on(branch, base);
        let pull = world.open_pull(branch, base);
        let head = translate::commit(world.branch(branch));
        let mut row = forge_world::changes::change_row();
        row.task = task;
        row.change.task = task;
        row.branch = branch.into();
        row.base = base.into();
        row.goal = if shared_goal { Some(70) } else { None };
        row.pull = Some(pull);
        row.change.last_head = Some(head);
        row.change.since = world.wall();
        row.change.state = change::State::Gating { head, asked: Box::new([9]) };
        row.change.gates = Box::new([change::Gate {
            number: 9,
            kind: change::GateKind::Agent,
            blocking: true,
            freshness: change::Freshness::Exact,
            eager: false,
        }]);
        world.event(forge_top::Event::Change { row });
    }
    world.run_for(3);
    for task in [51, 52] {
        hint_step(&mut world, task, task);
    }
    world.take_seen();
    world.take_calls();
    world
}

pub fn hint_step(world: &mut forge_world::World, task: u64, entry: u64) {
    use temper_engine_domain_forge_change as change;
    world.event(forge_top::Event::StepChange {
        merge_job: false,
        task,
        entry,
        heard: change::Heard {
            delegate_running: false,
            delegate_live: false,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: true,
    });
    world.run_for(1);
}

pub fn add_reader(policy: &mut authority::Policy) {
    let mut reader = policy.roles[0].clone();
    reader.number = people::Role::Observer.number();
    reader.authority.grants = Box::new([]);
    reader.requests = authority::Requests(0);
    reader.decides = authority::Proposals(0);
    policy.roles = Box::new([policy.roles[0].clone(), reader]);
}

impl World {
    /// A deliberately broken fixture; the story names its writer finding.
    pub fn broken(&mut self) {
        self.broken = true;
    }

    /// Settle, read the public documents, crash the same store, and compare.
    pub fn close(&mut self) {
        if self.broken {
            return;
        }
        self.oracle_running = true;
        self.oracle_settle();
        let forges = self.root.forges();
        let mut snapshot = crate::oracle::Snapshot::new(&self.store.rows, self.env.wall);
        for (sign_in, read) in &snapshot.queries {
            snapshot.answers.push(self.oracle_read(*sign_in, read.clone()));
        }
        let subjects = crate::oracle::forge_subjects(&self.store.rows);
        let mut documents = Vec::new();
        for (sign_in, project, subject) in &subjects {
            documents.push(self.oracle_document(*sign_in, *project, *subject));
        }
        self.oracle_settle();
        self.store.crash(0);
        snapshot.compare_rows(&self.store.rows);
        let store = std::mem::take(&mut self.store);
        let mut fresh =
            crate::direct::Driver::cold(store, crate::oracle::config::copy(&self.restart_config), &self.env.limits);
        std::mem::swap(&mut self.root, &mut fresh.root);
        std::mem::swap(&mut self.out, &mut fresh.out);
        std::mem::swap(&mut self.events, &mut fresh.events);
        std::mem::swap(&mut self.store, &mut fresh.store);
        fresh.broken = crate::oracle::Fixture::Broken;
        self.drained = None;
        self.period = Some(1);
        self.events.push_back(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
        if let Some((channel, worker, slots)) = self.oracle_peer {
            self.events.push_back(protocol::hello_at(
                channel, worker, slots,
                self.assigned.iter().filter(|assignment| {
                    matches!(self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task)))),
                        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))))
                        if matches!(task.phase, tasks::Phase::Active(tasks::Active::Claimed { attempt } | tasks::Active::Running { attempt }) if attempt == assignment.attempt))
                }).map(|assignment| protocol::hosting(assignment.task, assignment.attempt, jig_domain_link::Phase::Active)).collect(),
            ));
        }
        self.oracle_settle();
        snapshot.compare_rows(&self.store.rows);
        crate::oracle::compare_forges(forges, self.root.forges());
        for (index, (sign_in, read)) in snapshot.queries.iter().enumerate() {
            let answer = self.oracle_read(*sign_in, read.clone());
            snapshot.compare_answer(index, answer);
        }
        for ((sign_in, project, subject), before) in subjects.iter().zip(documents) {
            let after = self.oracle_document(*sign_in, *project, *subject);
            crate::oracle::compare_forge(*project, *subject, before, after);
        }
        self.oracle_settle();
        self.oracle_running = false;
    }

    fn oracle_settle(&mut self) {
        // These are the world's held terminals, not new actions from surviving peers.
        self.release_write_answers();
        if let Some(reads) = self.held_reads.take() {
            for read in reads {
                self.out.push(read);
            }
            self.collect();
        }
        let mut quiet = 0;
        for _ in 0..STEPS {
            let commit = self.store.applied;
            self.tick();
            // A hosted attempt may keep a future fleet deadline armed. No
            // timer is fired merely to close a story; all immediate terminals drain.
            if self.root.ready()
                && self.root.held_bytes() == 0
                && self.events.is_empty()
                && self.immediate.is_empty()
                && self.pending.is_empty()
                && self.store.idle()
                && self.store.held.is_empty()
                && self.store.applied == commit
            {
                quiet += 1;
                if quiet == 2 {
                    return;
                }
            } else {
                quiet = 0;
            }
        }
        panic!(
            "restart oracle forge world did not settle: ready={} held={} events={} immediate={:?} pending={:?} store_idle={} held_commits={} phase={:?} stop={:?} first={:?}",
            self.root.ready(),
            self.root.held_bytes(),
            self.events.len(),
            self.immediate,
            self.pending,
            self.store.idle(),
            self.store.held.len(),
            self.root.start_phase(),
            self.root.start_stop(),
            self.events.front()
        );
    }

    fn oracle_read(&mut self, sign_in: u64, read: jig_domain_core::reads::Read) -> jig_domain_core::reads::Answer {
        const REPLY: Token = Token::new(0x4e32);
        self.reads.clear();
        self.send(engine::Event::Party(engine::PartyInput::Read { reply_to: ReplyTo::new(REPLY), sign_in, read }));
        for _ in 0..STEPS {
            if let Some(index) = self.reads.iter().position(|(reply, _)| *reply == ReplyTo::new(REPLY)) {
                return self.reads.remove(index).1;
            }
            self.tick();
        }
        panic!("restart oracle party read did not answer");
    }

    /// One authenticated forge document, using a fresh watch identity for each subject.
    #[must_use]
    pub fn oracle_document(&mut self, sign_in: u64, project: u32, subject: u64) -> Option<forge_top::Document> {
        let watch = Token::new(self.oracle_watcher);
        self.oracle_watcher = self.oracle_watcher.checked_add(1).expect("fixture watch identities");
        self.oracle_documents.clear();
        self.panels.retain(|(watcher, _)| *watcher != watch);
        self.send(engine::Event::Party(engine::PartyInput::Watch {
            watcher: watch,
            sign_in,
            subject: documents::Watched::Application {
                scope: people::ApplicationScope::Project(project),
                connector: self.restart_config.core.connectors[0],
                subject: Token::new(subject),
            },
            window: documents::Window { rows: self.env.limits.documents.rows, after: None },
        }));
        self.oracle_settle();
        let answer = self
            .oracle_documents
            .iter()
            .rev()
            .find_map(|(watcher, doc)| (*watcher == watch).then(|| doc.clone()))
            .or_else(|| {
                self.panels.iter().rev().find_map(|(watcher, panel)| {
                    (*watcher == watch).then(|| forge_top::Document::Change(panel.clone()))
                })
            });
        if answer.is_some() {
            self.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: watch }));
            self.oracle_settle();
        }
        self.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: watch }));
        self.oracle_settle();
        self.panels.retain(|(watcher, _)| *watcher != watch);
        answer
    }

    /// One authenticated read through the root and its real store routing.
    #[must_use]
    pub fn party_read(&mut self, read: jig_domain_core::reads::Read) -> jig_domain_core::reads::Answer {
        const REPLY: Token = Token::new(0x4e31);
        self.reads.clear();
        self.send(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(REPLY),
            sign_in: self.signed_in.expect("reader session"),
            read,
        }));
        for _ in 0..STEPS {
            if let Some(index) = self.reads.iter().position(|(reply, _)| *reply == ReplyTo::new(REPLY)) {
                return self.reads.remove(index).1;
            }
            self.tick();
        }
        panic!("party read did not finish");
    }

    /// Inspect the transport evidence under the same party read caps, also for live tasks.
    #[must_use]
    pub fn party_load(
        &mut self,
        party: u64,
        scope: jig_domain_core::reads::Scope,
        after: Option<jig_domain_core::reads::Position>,
    ) -> jig_domain_core::reads::Loaded {
        self.store.read(Box::new(jig_domain_core::reads::Range {
            party,
            scope,
            after,
            backwards: false,
            rows: self.env.limits.documents.rows,
            bytes: self.env.limits.loads.read_bytes,
        }))
    }

    pub(super) fn stored_read(&mut self, request: engine::Request) -> Option<engine::Request> {
        if let engine::Request::Store(engine::StoreRequest::Read { load, range }) = request {
            let result = self.store.read(range);
            self.immediate.push_back(engine::Event::Store(engine::StoreInput::ReadLoaded { load, result }));
            None
        } else {
            Some(request)
        }
    }
}

pub(super) fn document_limits(
    mut documents: documents::DocumentLimits,
    limits: &engine::Limits,
) -> documents::DocumentLimits {
    // jig-protocol-config's overrides::read derives this after all name overrides.
    documents.segments = limits.authority.segments.max(limits.tasks.hold_segments).max(limits.tasks.authority_segments);
    documents.grants = limits.authority.grants.max(limits.tasks.authority_grants);
    documents.executors = limits.authority.executors.max(limits.tasks.executor_kinds);
    documents.implications = limits.authority.implications;
    documents
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn failed_ci_with_delayed_brief() -> (World, u64, u64) {
    let (mut world, chat, producer, branch) = change_world(false, 0, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    world.slow_brief_reads = true;
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    (world, chat.task, producer.task)
}

/// Effective box limits, read through the shipped init template.
#[must_use]
pub fn box_limits() -> engine::Limits {
    let (jig, temper) = box_profiles();
    temper_service::limits::engine(&jig, &temper).expect("coherent box limits")
}

/// The store join allowance from the same init template projection as `box_limits`.
#[must_use]
pub fn box_joins() -> u32 {
    box_profiles().0.store.joins
}

fn box_profiles() -> (jig_protocol_config::Limits, temper_service::limits::Limits) {
    // A story reads the same template for its defaults, root limits and store joins.
    static PROFILES: std::sync::OnceLock<(jig_protocol_config::Limits, temper_service::limits::Limits)> =
        std::sync::OnceLock::new();
    *PROFILES.get_or_init(read_box_profiles)
}

fn read_box_profiles() -> (jig_protocol_config::Limits, temper_service::limits::Limits) {
    use jig_protocol_config as config;
    let document = config::document(
        include_bytes!("../../../../crates/temper-init/template/temper.json"),
        &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("init template JSON");
    let mut root = config::read::Object::new(&document, 0).expect("template object");
    let engine = root.member(&document, b"engine").expect("engine member");
    let mut engine = config::read::Object::new(&document, engine).expect("engine object");
    let at = engine.member(&document, b"limits").expect("engine limits");
    let parsed = config::overrides::parse(&document, at, &config::Path::root().member(b"engine").member(b"limits"));
    assert!(parsed.problems.is_empty(), "init limits: {:?}", parsed.problems);
    let application = root.member(&document, b"application").expect("application member");
    let mut application = config::read::Object::new(&document, application).expect("application object");
    let at = application.member(&document, b"limits").expect("application limits");
    let mut limits = config::read::Object::new(&document, at).expect("application limits object");
    let at = limits.member(&document, b"overrides").expect("application overrides");
    let mut overrides = config::read::Object::new(&document, at).expect("application overrides object");
    let end = config::read::after(&document, at).expect("application overrides end");
    let mut temper = temper_service::limits::profile(temper_service::limits::Profile::Standard);
    for index in at..end {
        let token = document.token(index).expect("application override token");
        let Some(name) = document.text(token) else { continue };
        let Some(value_at) = overrides.member(&document, name) else { continue };
        let value = config::read::integer(&document, value_at, u64::MAX).expect("numeric application override");
        temper_service::limits::apply(&mut temper, name, value)
            .unwrap_or_else(|what| panic!("init application override {}: {what:?}", String::from_utf8_lossy(name)));
    }
    (parsed.limits, temper)
}
