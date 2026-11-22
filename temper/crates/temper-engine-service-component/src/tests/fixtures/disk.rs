//! Engine and store fixtures shared by the step tests and the real loop.
use crate::{limits, settings};
use skein_lib::{Secret, Token};

pub const ADMIN: &[u8] = b"admin-token-placeholder-0123";

// smith's protocol/agent.md, section 4: OpenAI's public Responses dialect.
const DIALECT: u32 = 3;

// The stock catalogue holds twelve charters.
// The closer procedure and close job make seventeen executor places in both limits.
pub const ENGINE: &[u8] = br#"{
"hosts": {
"listener": [{"in_process": "workers"}]
},
"clients": {
},
"owners": [{"provider": 1, "subject": "owner-a"}]
}"#;

// Jig's inline input is retained only to exercise policy decoding and typed guards.

pub fn engine() -> settings::Engine {
    static ENGINE_SETTINGS: std::sync::OnceLock<jig_protocol_config::Engine> = std::sync::OnceLock::new();
    let jig = ENGINE_SETTINGS
        .get_or_init(|| {
            let mut jig = parse_engine(ENGINE);
            let policy = jig.policies.first().unwrap();
            assert_eq!(policy.number, 1);
            jig.projects = Box::new([jig_protocol_config::Project {
                number: 1,
                name: "Project 1".into(),
                policy: policy.policy.clone(),
                roles: policy.roles.clone(),
                templates: policy.templates.clone(),
            }]);
            for owner in &mut jig.owners {
                owner.project = 1;
            }
            jig
        })
        .clone();
    // This typed fixture reaches the core's start; policy 1 needs 6074 bytes with the standing charter executors.

    let own = small_limits();
    let root = limits::engine(&jig.limits, &own).unwrap();
    let connector = *jig.connectors.first().unwrap();
    settings::Engine {
        store_limits: None,
        kinds: crate::stock::kinds(&root, connector),
        period_span: crate::stock::PERIOD_SPAN,
        host_reply_bytes: None,
        agent_charter_bytes: None,
        setups: Box::new([]),
        owned_forgejo: None,
        owned_runner: None,
        test_github: None,
        directories: jig_protocol_config::Directories::working(),
        jig,
        webhooks: None,
        sign_in: None,
        push: None,
        clients: temper_engine_service_startup::startup::application::Clients {
            listener: jig_protocol_config::Address::Addr(address()),
            bundle: None,
            public: None,
        },
        files: settings::Files { entries: skein_lib::List::with_capacity(0) },
        demo: None,
        program_environment: Box::new([]),
        host_part: temper_engine_domain::room::HostPart {
            allowance: u64::MAX,
            ..temper_engine_domain::room::HostPart::EMPTY
        },
        ways_stock: crate::stock::ways(),
        kinds_stock: temper_engine_domain::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
        landing_held: false,
        limits: own,
    }
}

// The small worlds admit the stock's larger goal charter while keeping Small's store page.
fn small_limits() -> limits::Limits {
    let mut own = temper_limits_fixtures::small();
    own.root.journal.run_bytes = 32_768;
    own.root.transcript_memory = 1_152_921_504_606_846_976;
    own
}

pub fn address() -> skein_io::Addr {
    skein_io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}

pub fn small_model() -> jig_domain_core::settings::CatalogueModel {
    jig_domain_core::settings::CatalogueModel {
        first_read: None,
        number: 1,
        model: jig_domain_core::Model {
            provider: 1,
            dialect: DIALECT,
            endpoint: 1,
            program: None,
            name: b"small".as_slice().into(),
            cache_life_ms: 0,
            window: 8192,
            output: 1024,
            input_price: 1,
            cached_price: 1,
            cache_write_price: None,
            output_price: 1,
            price_unit: 1000,
            effort: None,
            efforts: Box::default(),
        },
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
    }
}

/// A parsed engine passed the stock, before any component is made.
#[path = "../../../../../testing/engine_limit_figures.rs"]
mod engine_figures;
pub(super) use engine_figures::over as jig_limits_over;
fn jig_limits() -> jig_protocol_config::Limits {
    jig_limits_over(&jig_protocol_config_fixtures::small())
}

pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut jig = jig_protocol_config::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::PassedProviders,
        &temper_engine_service_startup::startup::fixed(&jig_protocol_config::Directories::working()).unwrap(),
        &jig_limits(),
    )
    .unwrap();
    let running = temper_worker_service_fixtures::providers();
    let mut defaults =
        crate::stock::checked_with(Box::new([small_model()]), jig.limits.core.tasks.result_bytes).unwrap();
    defaults.providers = running.clone();
    jig_protocol_config::pass_providers(&mut jig, defaults, &running, Box::new([])).unwrap();
    jig
}

pub fn store_limits() -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let settings = engine();
    store_limits_for(&settings)
}
pub(super) fn store_limits_for(
    settings: &settings::Engine,
) -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let root = limits::engine(&settings.jig.limits, &settings.limits).expect("root limits");
    (root, settings.jig.limits.core, settings.jig.limits.store)
}

pub fn commit(store: &mut crate::store::Store, number: u64, writes: Box<[temper_engine_domain::Write]>) {
    // Appending a fixture row must retain the real first-start counters, as well
    // as its settings rows. Empty stores still receive the minimal test header.
    let owner = Token::new(993);
    store.ask(jig_protocol_store::Request::Get {
        owner,
        key: jig_protocol_store::Get::Jig(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
    });
    let mut header = None;
    for _ in 0..store.events().len() {
        let event = store.events().pop().expect("store event");
        if let jig_protocol_store::Event::Got { owner: found, .. } = &event
            && *found == owner
        {
            let jig_protocol_store::Event::Got { row, .. } = event else { unreachable!("matched get") };
            if let Some(jig_protocol_store::Row::Jig(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(found),
            ))) = row
            {
                header = Some(found);
            }
        } else {
            store.events().push(event);
        }
    }
    let number = match header {
        Some(header) => number.max(header.commits.checked_add(1).unwrap()),
        None => number,
    };
    let mut header = header.unwrap_or(jig_domain_core::Deployment {
        id: [19; 16],
        restart_wait_ms: None,
        projects: 0,
        tasks: 0,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 0,
        applications: 0,
        commits: number,
    });
    header.commits = number;
    let mut all = skein_lib::List::with_capacity(
        u32::try_from(writes.len().checked_mul(2).unwrap().checked_add(1).unwrap()).unwrap(),
    );
    all.push(temper_engine_domain::Write::Save(temper_engine_domain::Record::Core(jig_domain_core::Record::Core(
        jig_domain_core::CoreRecord::Deployment(header),
    ))))
    .unwrap();
    for write in writes {
        if let temper_engine_domain::Write::Save(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
            jig_domain_tasks::Stored::Live(row),
        ))) = &write
        {
            all.push(temper_engine_domain::Write::Save(temper_engine_domain::Record::Core(
                jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::LiveCold {
                    number: row.number,
                    spec: Box::new(row.spec.clone()),
                    contract: row.contract.clone(),
                    authority: row.authority.clone(),
                    instructions: row.instructions.clone(),
                }),
            )))
            .unwrap();
        }
        all.push(write).unwrap();
    }
    assert!(store.down(temper_engine_domain::StoreRequest::Commit { number, writes: all.into_boxed() }));
}

pub fn seeds() -> settings::Seeds {
    settings::Seeds {
        deployment: [7; 16],
        installation: [9; 16],
        seed: 7,
        crypto: Secret::new(Box::new([7; 32])),
        link_secret: None,
        local_secret: None,
    }
}

/// Seed low application rows with Keep and carried rows through the store's allocator.
pub fn application_request(number: u64, value: Secret) -> jig_protocol_store::Secrets {
    if number >= temper_protocol_records::CARRIED {
        jig_protocol_store::Secrets::Carry { kind: jig_protocol_store::CarriedKind::Value, value }
    } else {
        jig_protocol_store::Secrets::Keep {
            owner: jig_protocol_store::SecretOwner::Application,
            number,
            value: jig_protocol_store::SecretValue::Application(value),
        }
    }
}
