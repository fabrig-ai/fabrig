//! The test application profile (`jig/docs/design/domain/testing.md`, 2).

#![forbid(unsafe_code)]

use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::Duration;

// Four people hand-offs allow five requests apiece: one bootstrap owner plus
// four sign-in/terminal outputs. This is the fixture's store-write allowance.
const PEOPLE_WRITE_ALLOWANCE: u32 = 20;

#[must_use]
pub fn limits() -> root::Limits {
    static LIMITS: std::sync::OnceLock<root::Limits> = std::sync::OnceLock::new();
    *LIMITS.get_or_init(profile)
}

#[expect(clippy::too_many_lines, reason = "the walking fixture states every child bound")]
fn profile() -> root::Limits {
    let retry = tasks::Retry { retries: 1, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    let tasks = tasks::Limits {
        placed_holds: 1,
        project_placed_holds: 1,
        party_placed_holds: 1,
        exemptions: 1,
        circle: 8,
        tasks: 2,
        funders: 4,
        project_tasks: 2,
        project_goals: 2,
        tree_tasks: 2,
        depth: 1,
        delegates: 1,
        references: 2,
        subscriptions: 2,
        batch: 1,
        dependencies: 1,
        holdings: 2,
        hold_kinds: 2,
        resource_reports: 64,
        pools: 2,
        hold_segments: 4,
        hold_bytes: 128,
        hold_waiters: 2,
        grace: skein_lib::Duration::from_secs(600),
        hold_wait: Duration::from_secs(10),
        inputs: 1,
        spec_bytes: 64,
        parameters: 1,
        result_bytes: 128,
        run_inbox: 4,
        inbox_messages: 4,
        inbox_bytes: 512,
        message_bytes: 64,
        proposal_stall: Duration::from_millis(10),
        escalation_stall: Duration::from_secs(3600),
        person_stall: Duration::from_secs(3600),
        saved_resources: 2,
        contract_choices: 1,
        charters: 1,
        revisions: 4,
        versions: 8,
        authority_grants: 1,
        authority_segments: 1,
        authority_bytes: 32,
        executor_kinds: 2,
        retries: tasks::Retries {
            transient: retry,
            permanent: retry,
            run: retry,
            agent: retry,
            lost: retry,
            invalid: retry,
        },
        facts: 2,
        room_refusals: 2,
    };
    let people = people::Limits {
        operations: 2,
        batch: 2,
        people: 2,
        inbox_entries: 2,
        inbox_tasks: 16,
        inbox_recipients: 64,
        sign_ins: 2,
        // The configured project and the deployment's roles.
        projects: 2,
        holdings: 2,
        initial_owners: 1,
        requests: 4,
        pending: 2,
        waiters: 2,
        program_hosts: 65,
        program_host_name_bytes: 32,
        act_grants: 2,
        identity_bytes: 32,
        words: 64,
        amendment_bytes: 512,
        sign_in_lifetime: Duration::from_secs(60),
        request_retention: Duration::from_secs(120),
        room_refusals: 2,
        facts: 2,
    };
    let fleet = fleet::Limits {
        configured_workers: 64,
        worker_name_bytes: 32,
        program_sources: 12,
        source_detail_bytes: 4096,
        needs_bytes: 4096,
        versions: 8,
        workers: 1,
        engine_slots: 0,
        slots: 1,
        workstreams: 1,
        attempts: 2,
        grants: 8,
        calls: 1,
        call_name_bytes: 64,
        progress_bytes: 64,
        turns: 2,
        grace: Duration::from_secs(5),
        facts: 2,
        history: 2,
    };
    let writes = PEOPLE_WRITE_ALLOWANCE + people.pending * 2 + jig_test_connector::OUTPUTS_BOUND * 2 + tasks.tasks + 6;
    let journal = root::JournalSizes { commits: 3, writes: writes.max(2000), door: 32, release: 32 };
    let mut limits = root::Limits {
        journal,
        routes: 200,
        watermark: root::default_watermark(&journal),
        connector: connector::LIMITS,
        core: core::Limits {
            routes: 20_783,
            settings: core::settings::Limits {
                revisions: 4,
                versions: 8,
                history: 64,
                proposals: 4,
                alternatives: 16,
                permissions: 32,
            },
            templates: 8,
            fill_mark: 1000,
            documents: core::documents::DocumentLimits {
                grants: 1,
                executors: 2,
                implications: 0,
                roles: authority_limits().roles,
                segments: 4,
                rows: 32,
                text_bytes: 256,
                chips: 32,
                lines: 32,
                versions: core::documents::Versions { minimum: 19, maximum: 19 },
            },
            connectors: 2,
            resume_bytes: 256,
            run_bytes: 512,
            policy_bytes: 8192,
            escalation_reason_bytes: 128,
            load_slots: 2,
            load_bytes: 65_536,
            call_records: 2,
            call_answer_bytes: 256,
            tasks,
            authority: authority_limits(),
            people,
            fleet,
            brief: jig_domain_brief::Limits { briefs: 2, sections: 3, read_bytes: 256, brief_bytes: 384 },
            brief_parts: 2,
            brief_core_budgets: core::CoreBriefBudgets {
                tail: 128,
                task: 128,
                dependencies: 128,
                attempts: 128,
                plan: 128,
                notes: 64,
            },
            accounts: accounts::Limits {
                accounts: 1,
                holder_accounts: 1,
                program_names: 12,
                program_hosts: 65,
                sources: 3,
                sign_ins: 1,
                credentials: 2,
                backoff_base: Duration::from_secs(2),
                backoff_max: Duration::from_secs(8),
                retry: Duration::from_secs(30),
                move_deadline: Duration::from_secs(60),
                spent_attention: Duration::from_secs(1),
                facts: 2,
            },
            views: jig_domain_views::Limits {
                runs: 2,
                watchers: 4,
                watchers_per_party: 8,
                subjects_per_change: 64,
                text_bytes: 64,
                facts: 8,
            },
            notes: jig_domain_notes::Limits {
                scopes: 4,
                entries_per_scope: 1,
                pattern_bytes: 64,
                description_bytes: 64,
                body_bytes: 256,
                references: 1,
                load_rows: 1,
                lines: 1,
                recalled: 1,
            },
        },
    };
    let room = limits.core.routes;
    let connectors = jig_test_connector::OUTPUTS_BOUND.checked_mul(2).expect("two connectors");
    limits.journal.writes = limits.journal.writes.max(room.checked_add(connectors).expect("fixture writes"));
    limits
}

fn authority_limits() -> authority::Limits {
    authority::Limits {
        projects: 1,
        roles: 1,
        requirements: 1,
        facts: 2,
        grants: 1,
        executors: 2,
        segments: 1,
        segment_bytes: 32,
        implications: 0,
        batch: 1,
        accounts: 3,
        writes: 1,
    }
}

fn authority_value(spend: u64, kinds: Box<[authority::Executor]>) -> authority::Authority {
    authority::Authority {
        tools: authority::Tools(1023),
        models: authority::Models::ALL,
        grants: Box::new([]),
        delegation: authority::Delegation { kinds, tasks: 3, depth: 2 },
        budget: authority::Budget { spend, deadline: None },
        notes: authority::Scopes(0),
        note_resources: Box::new([]),
    }
}

/// The one configured model, which the charter names and which declares two efforts
/// (`../protocol/configuration.md`, section 3; domain/tasks.md, section 3).
#[must_use]
pub fn scripted_model() -> core::Model {
    core::Model {
        dialect: 3,
        endpoint: 1,
        program: None,
        name: b"scripted".as_slice().into(),
        window: 8192,
        output: 1024,
        input_price: 1,
        cached_price: 1,
        cache_write_price: None,
        output_price: 1,
        price_unit: 1_000_000,
        efforts: Box::new([b"low".as_slice().into(), b"high".as_slice().into()]),
        effort: None,
    }
}

/// Authority policy and an authenticated owner.
#[must_use]
#[expect(clippy::too_many_lines, reason = "the fixture states every deployment field")]
pub fn config(seed: u64) -> root::Config {
    let ceiling = authority_value(1000, Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1)]));
    let rules = authority::Rules {
        ceiling: ceiling.clone(),
        period_spend: 1000,
        minimum_run_spend: 1,
        maximum_run_spend: 100,
        implies: authority::Implies::new(Box::new([]), 0).expect("empty implication configuration"),
        requirements: Box::new([]),
    };
    let domain = authority::Domain::new(rules, authority_limits()).expect("valid authority configuration");
    let policy = authority::Policy {
        projections: Box::new([]),
        escalation_role: Some(0),
        ceiling,
        period_spend: 1000,
        roles: Box::new([authority::Role {
            number: 0,
            authority: {
                let mut given = authority_value(
                    100,
                    Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1)]),
                );
                given.delegation.tasks = 2;
                given.delegation.depth = 2;
                given
            },
            period_spend: 500,
            requests: authority::Requests::ALL,
            decides: authority::Proposals::ALL,
        }]),
        requirements: Box::new([]),
    };
    let mut first = connector::config(1);
    first.deployment = [31; 16];
    let mut second = connector::config(2);
    second.deployment = [31; 16];
    first.procedures = Box::new([jig_test_connector::ProcedureSpec {
        number: 1,
        actions: Box::new([jig_test_connector::ProcedureAction::Finish {
            result: jig_test_connector::ProcedureResult::Succeeded,
        }]),
        max_steps: 1,
        stall: Duration::from_secs(1),
    }]);
    root::Config {
        first,
        second,
        core: core::Config {
            jobs: Box::new([]),
            worker_names: Box::new([b"alpha".as_slice().into(), b"beta".as_slice().into()]),
            deployment: [31; 16],
            seed,
            workers_in_process: false,
            hub_turns: 2,
            grants: Box::new([]),
            owners: Box::new([people::InitialOwner {
                project: 1,
                identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            }]),
            seed_projects: Box::new([core::SeedProject {
                number: 1,
                name: Box::from("Project 1"),
                policy: core::ConfiguredPolicy {
                    policy,
                    permissions: Box::new([
                        people::PermissionRole { connector: 1, permission: 3, role: 1 },
                        people::PermissionRole { connector: 1, permission: 2, role: 2 },
                        people::PermissionRole { connector: 1, permission: 1, role: 3 },
                    ]),
                    templates: Box::new([]),
                },
            }]),
            connectors: Box::new([1, 2]),
            settings: core::Settings {
                deployment_provider: 3,
                models: Box::new([scripted_model()]),
                resume_bytes: 256,
                tail_bytes: 128,
                tools: core::ToolFamilies::JIG,
                key_bytes: 256,
                redirect_bytes: 4096,
                llm_providers: accounts::Providers {
                    providers: Box::new([accounts::Provider {
                        name: b"Fake".as_slice().into(),
                        signed_in_plan: true,
                        manage_usage: Some(b"https://usage.example/".as_slice().into()),
                        number: 1,
                        models: Box::new([b"scripted".as_slice().into()]),
                        offers: accounts::Offers { sign_in: true, key: true, programs: Box::new([]) },
                    }]),
                },
            },
            defaults: core::settings::Defaults {
                charters: Box::new([core::Charter {
                    number: 1,
                    hosts: fleet::Kinds::Workers,
                    spend: 100,
                    policy: core::RunPolicy {
                        instructions: b"Complete the task within its authority.".as_slice().into(),
                        waiting: Duration::from_secs(1),
                        resume: true,
                        turns: 8,
                        time: Duration::from_secs(60),
                        reserve_turns: 1,
                        reserve_time: Duration::from_secs(10),
                        model: scripted_model(),
                        alternatives: Box::new([]),
                        inspect: true,
                        modify: true,
                        shell: true,
                        agents: true,
                        call_timeout: Duration::from_secs(10),
                    },
                }]),
                templates: crate::fixtures::templates(),
                policies: skein_lib::Map::with_capacity(0),
                rules: (domain).rules().clone(),
            },
        },
    }
}

mod charters;
mod connector;
pub mod fixtures;
pub use charters::{research_charter, two_charters};
mod credentials;
pub use credentials::with_workspace_credential;

mod funding;
pub use funding::edit_role;

fn grant() -> authority::Grant {
    authority::Grant {
        connector: 1,
        kind: 5,
        pattern: authority::Pattern { segments: Box::new([]), last: authority::Last::Open(Box::new([])) },
    }
}

#[must_use]
pub fn effect_fixture(seed: u64, requirement: bool) -> (root::Config, root::Limits) {
    let limits = effect_limits();
    let mut configuration = crate::config(seed);
    let mut rules = configuration.core.defaults.rules.clone();
    rules.ceiling.grants = Box::new([grant()]);
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.grants = Box::new([grant()]);
    policy.roles[0].authority.grants = Box::new([grant()]);
    policy.roles[0].authority.budget.spend = 300;
    if requirement {
        policy.requirements = Box::new([authority::Requirement {
            connector: 1,
            kind: 5,
            pattern: grant().pattern,
            judge: authority::Judge { connector: 2, requirement: 9, parameters: 0 },
            guard: authority::Guard::Observed { freshness: Duration::from_secs(60) },
            must_be_guarded: false,
        }]);
    }
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded effect rules");
    let mut out = skein_lib::Queue::uncapped();
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.seed_projects[0].policy.policy = domain.policy(1).expect("checked seed policy").clone();
    configuration.core.defaults.rules = (jig_domain_authority::Domain::new(domain.rules().clone(), *domain.limits())
        .expect("checked rules"))
    .rules()
    .clone();
    configuration.first.procedures = Box::new([jig_test_connector::ProcedureSpec {
        number: 1,
        actions: Box::new([
            jig_test_connector::ProcedureAction::Effect { kind: 5, purpose: 19, target: 13 },
            jig_test_connector::ProcedureAction::Finish { result: jig_test_connector::ProcedureResult::Succeeded },
        ]),
        max_steps: 3,
        stall: Duration::from_secs(1),
    }]);
    // Both connectors name the same resource; their facts come from distinct
    // systems. Connector two is the observer, connector one is the writer.
    configuration.second.prefix = configuration.first.prefix.clone();
    configuration.second.resources.clone_from(&configuration.first.resources);
    configuration.second.pools.clone_from(&configuration.first.pools);
    configuration.second.requirements = Box::new([jig_test_connector::RequirementSpec {
        number: 9,
        guarded: false,
        freshness: Duration::from_secs(60),
    }]);
    (configuration, limits)
}

fn effect_limits() -> root::Limits {
    let mut limits = crate::limits();
    limits.core.authority.segments = 2;
    limits.core.people.requests = 16;
    limits.connector.write_lifetime = Duration::from_secs(1);
    limits
}
