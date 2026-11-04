//! Agent configuration inputs (temper/docs/design/playground.md, 3).
use skein_channel::StreamMode;
use skein_io::kernel;
use skein_lib::{Duration, List};
use smith_agent_service as agent;
use smith_protocol_llm as llm;
pub(super) fn declarations(environment_bytes: u32) -> agent::profile::Configuration {
    use agent::profile::{Configuration, Endpoint, Model, OversizedReasoning};
    Configuration {
        role_bound: None,
        environment_bytes,
        endpoints: Box::new([Endpoint {
            number: 0,
            provider: skein_llm::Provider::OpenAi,
            option_bytes: 0,
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            models: Box::new([Model {
                efforts: Box::new([]),
                name: Box::from(b"fake".as_slice()),
                window: 1_048_576,
                output: 512,
                reasoning_item: 64,
                oversized_reasoning: OversizedReasoning::Fail,
                head: skein_lib::Duration::from_secs(1),
                idle: skein_lib::Duration::from_secs(1),
            }]),
        }]),
    }
}
pub(super) fn profile() -> agent::profile::Profile {
    let mut profile = agent::profile::standard();
    profile.declared.conversations = 6;
    profile.declared.calls_per_response = 16;
    profile.declared.tool_payload = 256;
    profile.declared.file_bytes = 65_536;
    profile.declared.shell_output = 512;
    profile.declared.read_window = 16;
    profile.declared.shell_head = 64;
    profile.declared.shell_tail = 64;
    profile.declared.search_hits = 8;
    profile.declared.search_bytes = 256;
    profile.declared.list_entries = 2;
    profile.declared.guide = 128;
    profile.declared.memory = 2_147_483_648;
    profile.declared.llm_pool = agent::profile::standard().declared.llm_pool;
    profile.policy.max_turns = 512;
    profile.policy.max_time = Duration::from_secs(3600);
    profile.policy.inbox = 8;
    profile.policy.sections = 4;
    profile.policy.host_tools = 2;
    profile.policy.verdicts = 2;
    profile.policy.check_timeout = Duration::from_secs(60);
    profile.policy.unacknowledged = 8;
    profile.policy.group_stop = Duration::from_millis(10);
    profile
}
pub(super) fn derive_limits() -> agent::Limits {
    let profile = profile();
    let configuration = declarations(0);
    let mut limits = agent::profile::derive(&profile, &configuration).expect("tiny profile derives");
    // Worlds may lower or specialize a derived limit (smith/docs/design/protocol/limits.md, section 2.2).
    limits.domain.run.models = 2;
    limits.domain.configured_model_bytes = 4096;
    limits.llm.connection.endpoints = 1;
    limits.llm.connection.connections = 2;
    limits.llm.connection.calls = 2;
    limits.llm.connection.per_endpoint = 2;
    limits.llm.accounts = 1;
    limits.llm.grant_value_bytes = 128;
    limits.machine.file_bytes = 16;
    limits.file_timeout = Duration::from_secs(1);
    limits.queue = agent::profile::DEFAULT_PACING;
    limits
}
pub(super) fn configuration_with_limits(provider: skein_llm::Provider, limits: &agent::Limits) -> agent::Config {
    let mut endpoints = List::with_capacity(1);
    endpoints
        .push(smith_protocol_translate::Endpoint { name: Box::default(), number: 0, dialect: 3 })
        .expect("one endpoint");
    let llm_endpoints = llm::Endpoints::new(
        Box::new([llm::ConfiguredEndpoint {
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            name: smith_domain::llm::Endpoint(0),
            destination: skein_llm_connection::Endpoint {
                limits: limits.llm.adapter.client,
                credential: skein_llm::client::CredentialLimits { value: limits.llm.grant_value_bytes },
                address: kernel::Addr::from((std::net::Ipv4Addr::LOCALHOST, 443)),
                transport: skein_llm_connection::Transport::Plaintext,
                llm: match provider {
                    skein_llm::Provider::Anthropic => skein_llm::Endpoint::anthropic(),
                    skein_llm::Provider::OpenAi => skein_llm::Endpoint::openai(),
                },
            },
            models: Box::new([]),
        }]),
        1,
    )
    .expect("one LLM endpoint");
    agent::Config {
        program: None,
        limits: *limits,
        domain: smith_domain::Config {
            programs: Box::new([]),
            endpoints: Box::new([smith_domain::run::charter::Endpoint(0)]),
            models: Box::new([smith_domain::ConfiguredModel {
                efforts: Box::new([]),
                prices: smith_domain::run::Prices {
                    input: 1,
                    cached: 1,
                    cache_write: None,
                    output: 1,
                    unit: 1_000_000,
                },
                protocol_allowance: agent::profile::endpoint_allowance(
                    &profile(),
                    &declarations(0).endpoints[0],
                    &limits.domain.session.tools,
                )
                .expect("derived endpoint framing"),
                endpoint: smith_domain::run::charter::Endpoint(0),
                model: Box::from(&b"fake"[..]),
                window: 1_048_576,
                output: 512,
                head: Duration::from_secs(2),
                idle: Duration::from_secs(3),
            }]),
        },
        channel_endpoints: smith_protocol_translate::Endpoints::new(endpoints),
        llm_endpoints,
        environment: smith_protocol_machine::Environment::encode(&[], &limits.machine).expect("empty environment fits"),
        stream_mode: StreamMode::Two,
    }
}
