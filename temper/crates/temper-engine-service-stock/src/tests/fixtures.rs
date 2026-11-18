use crate::{limits, settings};
pub const DIALECT: u32 = 3;

pub const ENGINE: &[u8] = br#"{
"llm": {
"providers":[{"name":"fake-provider","number":1,"endpoint":"fake","sources":{"keys":true}}],
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 3,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}]
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"clients": {
},
"owners": [{"provider": 1, "subject": "owner-a"}]
}"#;

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
        kinds: crate::stock::kinds(&root, connector),
        period_span: crate::stock::PERIOD_SPAN,
        host_reply_bytes: None,
        agent_charter_bytes: None,
        setups: Box::new([]),
        test_github: None,
        directories: jig_protocol_config::Directories::working(),
        jig,
        webhooks: None,
        sign_in: None,
        push: None,
        clients: crate::startup::application::Clients {
            listener: jig_protocol_config::Address::Addr(http::address()),
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

fn small_limits() -> limits::Limits {
    let mut own = temper_limits_fixtures::small();
    own.root.journal.run_bytes = 32_768;
    own
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

#[path = "../../../../testing/engine_limit_figures.rs"]
mod engine_figures;
use engine_figures::over as jig_limits_over;
fn jig_limits() -> jig_protocol_config::Limits {
    jig_limits_over(&jig_protocol_config_fixtures::small())
}

pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    parse_engine_over(bytes, &jig_limits())
}
pub fn parse_engine_over(bytes: &[u8], base: &jig_protocol_config::Limits) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut jig = jig_protocol_config::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
        &temper_engine_service_startup::startup::fixed(&jig_protocol_config::Directories::working()).unwrap(),
        base,
    )
    .unwrap();
    let own = match jig.llm.endpoints.first() {
        Some(endpoint) => endpoint.name.as_ref() == b"fake",
        None => false,
    };
    let defaults = if own {
        crate::stock::checked_with(Box::new([small_model()]), jig.limits.core.tasks.result_bytes).unwrap()
    } else {
        crate::stock::defaults(&jig.llm, jig.limits.core.tasks.result_bytes)
    };
    jig_protocol_config::pass(&mut jig, defaults).unwrap();
    jig
}

mod http;
pub use temper_worker_service_fixtures::agent;
mod startup;
pub use startup::{Json, value};
