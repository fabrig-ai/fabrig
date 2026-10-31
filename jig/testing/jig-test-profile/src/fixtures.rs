//! Test application records and request fixtures (`jig/docs/design/domain/testing.md`, 2).
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

pub const WORKSPACE_CREDENTIAL: u32 = 7;

pub const CREDENTIAL_RESOURCE: u64 = 1;

#[must_use]
pub fn bounds() -> jig_domain_fleet::Bounds {
    use skein_lib::Duration;
    jig_domain_fleet::Bounds {
        program_endpoints: Box::new([]),
        models: Box::new([
            jig_domain_fleet::ModelBound { name: b"scripted".as_slice().into(), window: 8192, output: 1024 },
            jig_domain_fleet::ModelBound { name: b"research".as_slice().into(), window: 8192, output: 1024 },
        ]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
    }
}

pub const QUERY_REQUESTER: u64 = 1;

/// A requesting party's task query, shared by the independent watch stories.
pub const OWN_TASKS: jig_domain_core::documents::TaskQuery = jig_domain_core::documents::TaskQuery {
    project: None,
    requester: Some(jig_domain_tasks::Party::Person(QUERY_REQUESTER)),
    root: None,
    parent: None,
    label: None,
    phase: None,
    executor: None,
    tracked: None,
    order: jig_domain_core::documents::QueryOrder::Number,
};
#[must_use]
pub fn templates() -> Box<[jig_domain_core::Template]> {
    use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
    use jig_domain_tasks::{Contract, Flags, Retention};
    Box::new([
        jig_domain_core::Template {
            number: 1,
            name: Box::from(&b"chat"[..]),
            executor: TemplateExecutor::Charter(1),
            contract: Contract::Report { words: 32 },
            shallower: 1,
            budget: TemplateBudget::AsAllowed { cap: None },
            deadline: TemplateDeadline::None,
            tracked: None,
            flags: Flags {
                ended_by: jig_domain_tasks::EndedBy::Result,
                conversation: true,
                listed: false,
                retention: Retention::Kept,
            },
            label: jig_client_test_records::CHAT,
            grace: Some(skein_lib::Duration::from_millis(5000)),
            llm: None,
        },
        jig_domain_core::Template {
            number: 2,
            name: Box::from(&b"goal"[..]),
            executor: TemplateExecutor::Chosen,
            contract: Contract::Report { words: 32 },
            shallower: 1,
            budget: TemplateBudget::Given,
            deadline: TemplateDeadline::Given,
            tracked: Some(1),
            flags: Flags {
                ended_by: jig_domain_tasks::EndedBy::Result,
                conversation: false,
                listed: true,
                retention: Retention::Kept,
            },
            label: jig_client_test_records::GOAL,
            grace: None,
            llm: None,
        },
    ])
}

/// A person's amendment retaining each field.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: reason.into(),
    }
}

/// A task amendment retaining every setting.
#[must_use]
pub fn unchanged(reason: &[u8]) -> tasks::Amendment {
    tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: reason.into(),
    }
}

/// An explicit batch proposal for a goal request.
#[must_use]
pub fn propose_goal(ask: jig_domain_people::Ask) -> jig_domain_people::Ask {
    use jig_domain_people as people;
    let people::Ask::Create { project, batch } = ask else { panic!("creation fixture") };
    people::Ask::Propose {
        project,
        reason: Box::from(b"please fund this goal".as_slice()),
        action: people::Action::Batch(batch),
    }
}

/// One requested root, shared by direct creation and explicit proposal stories.
#[must_use]
pub fn people_member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
        deadline: jig_domain_people::Deadline::Named,
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        named: Box::new([]),
        contract: people::Contract::Report { words: 16 },
        authority: people::Authority {
            tools: 0,
            models: None,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
        llm: None,
    }
}

/// The configured authority and task bound used by a creating party.
#[derive(Clone, Debug)]
pub struct Creating {
    roles: std::collections::BTreeMap<(u32, u32), authority::Authority>,
    pub tasks: u32,
}

/// Only a goal's supplied terms differ from ordinary creation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Goal {
    pub budget: u64,
    pub priority: u32,
    pub deadline: Option<skein_lib::Wall>,
}

#[must_use]
pub fn create_chat(creating: &Creating, project: u32, role: u32, words: Box<[u8]>) -> people::Ask {
    let role =
        creating.roles.get(&(project, role)).or_else(|| creating.roles.get(&(project, 0))).expect("fixture role");
    let mut member = people_member(&[], 0);
    member.words = words;
    member.authority = core::authority_to_people(role).expect("fixture authority");
    member.authority.spend = 0;
    member.authority.delegation.tasks = member.authority.delegation.tasks.min(creating.tasks).saturating_sub(1);
    member.authority.delegation.depth = member.authority.delegation.depth.saturating_sub(1);
    member.budget = people::Budget::AsAllowed { cap: None };
    member.flags = people::Flags {
        ended_by: people::EndedBy::Result,
        conversation: true,
        listed: false,
        retention: people::Retention::Kept,
    };
    member.label = jig_client_test_records::CHAT;
    member.contract = people::Contract::Report { words: 128 };
    member.wake = people::WakePolicy {
        words: people::WakeRule::Immediate,
        notices: people::WakeRule::Immediate,
        news: people::WakeRule::Immediate,
        results: people::ResultsWake::LastOrFailure,
        questions: true,
        answers: true,
        timers: true,
    };
    people::Ask::Create { project, batch: Box::new([member]) }
}

/// Explicit durable key accounts used by the lower domain world; zero is a real account row.
#[must_use]
pub fn account_rows(providers: &jig_domain_accounts::Providers) -> Box<[root::Record]> {
    use jig_domain_accounts::{Holder, Kind, Stored};
    let mut rows = Vec::new();
    for (index, entry) in providers.providers.iter().filter(|p| !p.offers.none()).enumerate() {
        let index = u32::try_from(index).expect("bounded fixture value");
        let (account, source) = if index == 0 { (0, 1) } else { (2 * index + 1, 2 * index) };
        let provider = entry.number;
        {
            for row in [
                Stored::Account {
                    number: account,
                    provider,
                    holder: Holder::Deployment,
                    place: index,
                    uses: None,
                    subject: Box::new([]),
                    display: Box::new([]),
                },
                Stored::Source { account, number: source, place: 0, kind: Kind::Key, name: Box::new([]), held: None },
            ] {
                rows.push(root::Record::Core(core::Record::Accounts(row)));
            }
        }
    }
    if !rows.is_empty() {
        rows.push(root::Record::Core(core::Record::Accounts(Stored::Next {
            number: u32::try_from(rows.len()).expect("bounded fixture value"),
        })));
    }
    rows.into_boxed_slice()
}

/// What a first start leaves for its seed projects, for worlds that hand-bootstrap rosters without
/// one: the deployment header marking every seeded number, then each project's policy row.
#[must_use]
pub fn seeded_project_rows(config: &core::Config) -> Vec<core::Record> {
    let mut deployment = core::Counters::bootstrap(config.deployment).deployment();
    for project in &config.seed_projects {
        deployment.projects = deployment.projects.max(u64::from(project.number));
    }
    let mut rows = vec![core::Record::Core(core::CoreRecord::Deployment(deployment))];
    for row in project_policy_rows(config) {
        let root::Record::Core(row) = row else { panic!("a policy row is the core's") };
        rows.push(row);
    }
    rows
}

/// Complete policy rows for worlds that hand-bootstrap rosters before cold restore.
pub fn project_policy_rows(config: &core::Config) -> Vec<root::Record> {
    config
        .seed_projects
        .iter()
        .map(|project| {
            let policy = &project.policy.policy;
            let grant = |value: &authority::Grant| people::Grant {
                connector: value.connector,
                kind: value.kind,
                pattern: core::pattern_to_people(&value.pattern),
            };
            let value = people::PolicyValue {
                ceiling: core::authority_to_people(&policy.ceiling).expect("bounded ceiling"),
                escalation_role: policy.escalation_role,
                period_spend: policy.period_spend,
                projections: policy.projections.iter().map(grant).collect(),
                roles: policy
                    .roles
                    .iter()
                    .map(|role| people::PolicyRole {
                        number: role.number,
                        authority: core::authority_to_people(&role.authority).expect("bounded role"),
                        period_spend: role.period_spend,
                        requests: role.requests.0,
                        decides: role.decides.0,
                    })
                    .collect(),
                requirements: policy
                    .requirements
                    .iter()
                    .map(|requirement| people::Requirement {
                        connector: requirement.connector,
                        kind: requirement.kind,
                        pattern: core::pattern_to_people(&requirement.pattern),
                        judge: people::Judge {
                            connector: requirement.judge.connector,
                            requirement: requirement.judge.requirement,
                            parameters: requirement.judge.parameters,
                        },
                        guard: match requirement.guard {
                            authority::Guard::Guarded => people::Guard::Guarded,
                            authority::Guard::Observed { freshness } => people::Guard::Observed { freshness },
                        },
                        must_be_guarded: requirement.must_be_guarded,
                    })
                    .collect(),
                permissions: project.policy.permissions.clone(),
                templates: project.policy.templates.clone(),
            };
            root::Record::Core(core::Record::People(people::Stored::Policy {
                project: project.number,
                name: project.name.clone(),
                value,
            }))
        })
        .collect()
}

/// Names for the standard fixture's two projects.
#[must_use]
pub fn project_names() -> skein_lib::Map<u32, Box<str>> {
    let mut names = skein_lib::Map::with_capacity(2);
    names.insert(1, Box::from("Project 1")).expect("first project name");
    names.insert(2, Box::from("Project 2")).expect("second project name");
    names
}

/// The shared requesting party's exact proposal inbox identity.
#[must_use]
pub fn proposal_object(proposal: u64) -> jig_domain_core::documents::InboxObject {
    jig_domain_core::documents::InboxObject::Proposal {
        proposer: jig_domain_people::Proposer::Party(QUERY_REQUESTER),
        number: proposal,
    }
}

impl Creating {
    #[must_use]
    pub fn new(config: &core::Config, tasks: u32) -> Self {
        let roles = config
            .seed_projects
            .iter()
            .flat_map(|project| {
                project
                    .policy
                    .policy
                    .roles
                    .iter()
                    .map(move |role| ((project.number, role.number), role.authority.clone()))
            })
            .collect();
        Self { roles, tasks }
    }
}
pub use jig_test_domain::initial_settings_current;

/// The initial authenticated conversation request, shared by read stories.
#[must_use]
pub fn forgettable_conversation() -> people::Ask {
    people::Ask::Create { project: 1, batch: Box::new([conversation_member(true)]) }
}

/// One explicitly flagged conversation under the world's agent charter.
#[must_use]
pub fn conversation_member(forgettable: bool) -> people::Member {
    people::Member {
        deadline: jig_domain_people::Deadline::Named,
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: b"make an effect".as_slice().into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        named: Box::new([]),
        budget: people::Budget::Named,
        contract: people::Contract::Report { words: 128 },
        authority: people::Authority {
            tools: core::ToolFamilies::JIG.delegate.0,
            models: None,
            grants: Box::new([]),
            delegation: people::Delegation {
                kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1)]),
                tasks: 1,
                depth: 1,
            },
            spend: 100,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Immediate,
            news: people::WakeRule::Immediate,
            results: people::ResultsWake::LastOrFailure,
            questions: true,
            answers: true,
            timers: true,
        },
        dependencies: Box::new([]),
        tracked: None,
        label: jig_client_test_records::CHAT,
        flags: people::Flags {
            ended_by: people::EndedBy::Result,
            conversation: true,
            listed: false,
            retention: if forgettable { people::Retention::ForgottenOnceRead } else { people::Retention::Kept },
        },
        llm: None,
    }
}
