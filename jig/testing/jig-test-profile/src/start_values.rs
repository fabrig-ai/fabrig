//! Core values for held-start fixtures with one project and one connector.
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use skein_lib::Duration;

#[must_use]
pub fn authority_limits() -> authority::Limits {
    authority::Limits {
        projects: 1,
        roles: 1,
        requirements: 2,
        facts: 2,
        grants: 1,
        executors: 1,
        segments: 1,
        segment_bytes: 32,
        implications: 0,
        batch: 1,
        accounts: 2,
        writes: 1,
    }
}

fn authority_value(spend: u64, kinds: Box<[authority::Executor]>) -> authority::Authority {
    authority::Authority {
        tools: authority::Tools(1023),
        models: authority::Models::ALL,
        grants: Box::new([]),
        delegation: authority::Delegation { kinds, tasks: 2, depth: 1 },
        budget: authority::Budget { spend, deadline: None },
        notes: authority::Scopes(0),
        note_resources: Box::new([]),
    }
}

#[must_use]
pub fn core_config(seed: u64, projects: u32) -> jig_domain_core::Config {
    let ceiling = authority_value(1000, Box::new([authority::Executor::Charter(1)]));
    let rules = authority::Rules {
        ceiling: ceiling.clone(),
        period_spend: 1000,
        minimum_run_spend: 1,
        maximum_run_spend: 100,
        implies: authority::Implies::new(Box::new([]), 0).expect("empty implication configuration"),
        requirements: Box::new([]),
    };
    let mut policy_limits = authority_limits();
    policy_limits.projects = projects;
    // Check the same fixture bounds without allocating a discarded live policy table.
    assert!(authority::valid_rules(&rules, &policy_limits), "valid authority configuration");
    let policy = authority::Policy {
        projections: ceiling.grants.clone(),
        escalation_role: Some(0),
        ceiling,
        period_spend: 1000,
        roles: Box::new([authority::Role {
            number: 0,
            authority: authority_value(500, Box::new([authority::Executor::Charter(1)])),
            period_spend: 500,
            requests: authority::Requests::ALL,
            decides: authority::Proposals::ALL,
        }]),
        requirements: Box::new([]),
    };
    jig_domain_core::Config {
        fixed_providers: Box::new([]),
        parts: Box::new([]),
        deployment: [31; 16],
        seed,
        // The worlds' workers run apart from the engine (domain/root.md, 2).
        workers_in_process: false,
        jobs: Box::new([]),
        kept_starts: Box::new([]),

        hub_turns: HUB_TURNS,
        owners: Box::new([people::InitialOwner {
            project: 1,
            identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        }]),
        grants: Box::new([]),
        kinds: Box::default(),
        defaults: jig_domain_core::settings::Defaults {
            limits: Box::default(),
            providers: Box::new([provider_setting()]),
            workers: Box::new([jig_domain_core::settings::WorkerSetting {
                number: 1,
                name: Box::from(b"worker".as_slice()),
                places: 1,
                digest: Some([7; 32]),
                withdrawn: false,
            }]),
            models: Box::default(),
            rules,
            charters: Box::new([charter()]),
            templates: Box::new([]),
            policies: skein_lib::Map::with_capacity(authority_limits().projects),
        },
        settings: jig_domain_core::Settings {
            endpoints: Box::new([jig_domain_core::EndpointFacts {
                direct: Box::new([]),
                providers: Box::new([]),
                number: 1,
                dialect: 3,
                program: None,
            }]),
            key_bytes: 256,
            redirect_bytes: 256,
            deployment_provider: 1,
            resume_bytes: 256,
            tail_bytes: 2048,
            private_tools: Box::new([]),
            tools: jig_domain_core::ToolFamilies::JIG,
            models: models(),
            llm_providers: accounts::Providers {
                providers: Box::new([accounts::Provider {
                    list: None,
                    number: 1,
                    name: Box::from(b"Scripted".as_slice()),
                    signed_in_plan: false,
                    manage_usage: None,
                    offers: accounts::Offers { sign_in: true, key: true, programs: Box::new([]) },
                }]),
            },
        },
        seed_projects: Box::new([jig_domain_core::SeedProject {
            number: 1,
            name: "Project 1".into(),
            policy: jig_domain_core::ConfiguredPolicy {
                policy,
                permissions: permission_roles().remove(&1).expect("project permissions"),
                templates: Box::new([]),
            },
        }]),
        // This profile has one connector.
        connectors: Box::new([0]),
    }
}

/// Retained turns per attempt in the workers' hub, as jig's smallest
/// profile sets them (`jig-protocol-config`'s `small.rs`, hub `turns`).
pub const HUB_TURNS: u32 = 8;

/// Project one's connector permissions mapped to its seeded roles.
fn permission_roles() -> skein_lib::Map<u32, Box<[people::PermissionRole]>> {
    let mut projects: skein_lib::Map<u32, Box<[people::PermissionRole]>> = skein_lib::Map::with_capacity(1);
    projects
        .insert(
            1,
            Box::new([
                people::PermissionRole { connector: 0, permission: 3, role: 1 },
                people::PermissionRole { connector: 0, permission: 2, role: 2 },
                people::PermissionRole { connector: 0, permission: 1, role: 3 },
            ]),
        )
        .expect("one configured permission policy");
    projects
}

/// The deployment's configured models: its one charter's.
#[must_use]
pub fn models() -> Box<[jig_domain_core::Model]> {
    Box::new([charter().policy.model])
}

/// The deployment's one charter for its agent work.
/// Its default spend holds its reserve floor (jig's `domain/authority.md`, 7).
#[must_use]
pub fn charter() -> jig_domain_core::Charter {
    jig_domain_core::Charter {
        parts: Box::new([]),
        rest: false,
        wake: jig_domain_tasks::WakePolicy::DEFAULT,
        documents: Box::new([]),
        number: 1,
        policy: jig_domain_core::RunPolicy {
            instructions: b"Complete the task within its authority.".as_slice().into(),
            waiting: Duration::from_secs(1),
            resume: true,
            turns: 8,
            time: Duration::from_secs(60),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(10),
            model: jig_domain_core::Model {
                provider: 1,
                program: None,
                dialect: 3,
                endpoint: 1,
                name: b"scripted".as_slice().into(),
                cache_life_ms: 0,
                window: 1024,
                output: 1024,
                input_price: 1,
                cached_price: 1,
                cache_write_price: None,
                output_price: 1,
                price_unit: 1000,
                efforts: Box::new([]),
                effort: None,
            },
            alternatives: Box::new([]),
            inspect: true,
            modify: true,
            shell: true,
            agents: true,
            call_timeout: Duration::from_secs(10),
            tools: jig_domain_core::RunPolicy::EVERY_FAMILY,
        },
        hosts: fleet::Kinds::Workers,
        spend: 100,
    }
}

fn provider_setting() -> jig_domain_core::settings::ProviderSetting {
    use jig_domain_core::settings::{Endpoint, EndpointAddress, Provider, ProviderSetting, Transport};
    ProviderSetting {
        provider: Provider {
            number: 1,
            name: b"Scripted".as_slice().into(),
            endpoint: Some(b"scripted".as_slice().into()),
            direct: None,
            list: None,
            signed_in_plan: false,
            manage_usage: None,
            sign_in: None,
            keys: false,
            programs: Box::new([]),
        },
        endpoints: Box::new([Endpoint {
            number: 1,
            name: b"scripted".as_slice().into(),
            provider: b"Scripted".as_slice().into(),
            dialect: 3,
            program: None,
            address: EndpointAddress::Socket(::core::net::SocketAddr::from(([127, 0, 0, 1], 8080))),
            transport: Transport::Plaintext,
            headers: Box::new([]),
            connect: Duration::from_millis(10_000),
            handshake: Duration::from_millis(10_000),
        }]),
        withdrawn: false,
    }
}
