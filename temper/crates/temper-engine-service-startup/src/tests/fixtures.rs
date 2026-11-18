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
fn jig_limits_over(base: &jig_protocol_config::Limits) -> jig_protocol_config::Limits {
    let mut limits = engine_figures::over(base);
    limits.core.authority.requirements = 3;
    limits
}
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
        jig_protocol_config::Defaults::Passed,
        &crate::startup::fixed(&jig_protocol_config::Directories::working()).unwrap(),
        &jig_limits(),
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

pub fn people_member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
        symbols: Box::new([]),
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        deadline: people::Deadline::Named,
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
        wake: Some(people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        }),
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
        llm: None,
    }
}

pub fn chat(project: u32, words: &[u8]) -> jig_domain_people::Ask {
    use jig_domain_people as people;
    let mut member = people_member(words, 0);
    member.budget = people::Budget::AsAllowed { cap: None };
    member.authority.tools = 1;
    member.authority.notes = 0;
    member.contract = people::Contract::Report { words: 32 };
    member.flags = people::Flags {
        ended_by: people::EndedBy::Result,
        conversation: true,
        listed: false,
        retention: people::Retention::Kept,
    };
    member.label = 1;
    people::Ask::Create { project, batch: Box::new([member]) }
}

#[expect(
    clippy::disallowed_types,
    reason = "ordinary fixture code embeds the shipped profile and returns the shell's text diagnostic"
)]
pub fn stock_configuration(payload: u32, output: u32) -> Result<smith_agent_shell::config::Configuration, String> {
    let mut agent = temper_agent_fixtures::stock(true);
    agent["profile"]["declared"]["tool_payload"] = payload.into();
    for endpoint in agent["endpoints"].as_array_mut().expect("stock endpoints") {
        endpoint["limits"]["output"] = output.into();
    }
    let bytes = serde_json::to_string_pretty(&agent)
        .expect("stock agent JSON")
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", r#"{"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}}"#)
        .replace("api.anthropic.com:443", "127.0.0.1:8373")
        .replace("\"server_name\": \"api.anthropic.com\",", "")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"")
        .replace("\"server_name\": \"api.openai.com\",", "");
    smith_agent_shell::config::parse(bytes.as_bytes())
}

mod http;
pub use temper_worker_service_fixtures::agent;
mod startup;
pub use startup::{APPLICATION, document, engine_part, startup_limits, value, with};
