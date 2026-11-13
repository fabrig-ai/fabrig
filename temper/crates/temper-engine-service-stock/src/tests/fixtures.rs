use crate::{limits, settings};
pub const DIALECT: u32 = 3;

pub const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"core.run_bytes":32768,"tasks.charters":13,"core.templates":6,"tasks.result_bytes":32,"tasks.run_inbox":2,"hub.turns":1,"people.amendment_bytes":64,"core.policy_bytes":8192,"authority.grants":16,"authority.executors":21,"tasks.executor_kinds":21,"notes.scopes":32,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
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
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1],
"memory_bytes": 1152921504606846976,
"projects": [{"number": 1, "policy": 1}]
}"#;

pub fn engine() -> settings::Engine {
    static ENGINE_SETTINGS: std::sync::OnceLock<jig_protocol_config::Engine> = std::sync::OnceLock::new();
    let jig = ENGINE_SETTINGS.get_or_init(|| parse_engine(ENGINE)).clone();
    // This typed fixture reaches the core's start; policy 1 needs 6074 bytes with the standing charter executors.

    let own = small_limits();
    let root = limits::engine(&jig.limits, &own).unwrap();
    let connector = *jig.connectors.first().unwrap();
    settings::Engine {
        kinds: crate::stock::kinds(&root, connector),
        period_span: crate::stock::PERIOD_SPAN,
        host_reply_bytes: None,
        agent_charter_bytes: None,
        machine: machine(),
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
        welcome: false,
        program_environment: Box::new([]),
        host_part: None,
        ways_stock: crate::stock::ways(),
        kinds_stock: temper_engine_domain::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
        landings: skein_lib::Map::with_capacity(0),
        limits: own,
    }
}

fn small_limits() -> limits::Limits {
    let mut own = limits::profile(limits::Profile::Small);
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

pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut jig = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
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

pub fn machine() -> temper_forge_setup::machine::Machine {
    temper_forge_setup::machine::Machine { owned: None, fake: None, github: None }
}

mod http;
pub use temper_worker_service_fixtures::agent;
mod startup;
pub use startup::{Json, value, with};
