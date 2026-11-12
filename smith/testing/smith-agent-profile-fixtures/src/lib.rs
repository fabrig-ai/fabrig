//! Shared fixtures (smith/docs/design/testing.md, section 7; smith/docs/design/shell/README.md, section 2.6).
//! Small admitted configuration values and one model declaration for composition tests.
//! No service is constructed here (smith/docs/design/shell/README.md, section 2.6).

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use skein_channel::StreamMode;
use skein_lib::{List, Secret};
use smith_agent_profile::{Config, ProgramBackend};
use smith_domain as domain;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;
use smith_protocol_program::Programs;

/// One native model and its environment declaration for profile scenarios.
#[must_use]
pub fn declarations(environment_bytes: u32) -> smith_agent_profile::profile::Configuration {
    use smith_agent_profile::profile::{Configuration, Endpoint, Limits as DeclaredLimits, OversizedReasoning};
    Configuration {
        role_bound: None,
        environment_bytes,
        endpoints: Box::new([Endpoint {
            number: 0,
            provider: skein_llm::Provider::OpenAi,
            option_bytes: 0,
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            limits: DeclaredLimits {
                window: 2_097_152,
                output: 65_536,
                reasoning_item: 2048,
                oversized_reasoning: OversizedReasoning::Fail,
                head: skein_lib::Duration::from_secs(1),
                idle: skein_lib::Duration::from_secs(1),
            },
        }]),
    }
}

/// Derive a small service with the same receiving relationships as deployment.
#[must_use]
pub fn small() -> smith_agent_profile::Limits {
    let mut profile = smith_agent_profile::profile::standard();
    profile.declared.memory = u64::MAX;
    profile.declared.conversations = 2;
    profile.declared.calls_per_response = 4;
    profile.declared.jobs = 4;
    profile.declared.tool_payload = 65_536;
    profile.declared.file_bytes = 65_536;
    profile.declared.read_window = 4096;
    profile.declared.shell_output = 512;
    profile.declared.shell_head = 256;
    profile.declared.shell_tail = 256;
    profile.declared.search_hits = 16;
    profile.declared.search_bytes = 1024;
    profile.declared.list_entries = 64;
    profile.policy.parallel_tools = 4;
    profile.policy.check_timeout = skein_lib::Duration::from_secs(30);
    let configuration = declarations(0);
    smith_agent_profile::profile::derive(&profile, &configuration)
        .expect("small source declarations satisfy receiving relationships")
}

/// Build a native configuration with empty endpoint and domain declarations.
#[must_use]
pub fn configuration() -> Config {
    let limits = small();
    let channel_endpoints = smith_protocol_translate::Endpoints::new(List::with_capacity(1));
    let llm_endpoints = llm::Endpoints::new(Box::new([]), 1).expect("empty endpoint table");
    Config {
        program: None,
        limits,
        domain: domain::Config { programs: Box::new([]), endpoints: Box::new([]), limits: Box::new([]) },
        channel_endpoints,
        llm_endpoints,
        environment: machine::Environment::encode(&[], &limits.machine).expect("empty environment fits"),
        stream_mode: StreamMode::Two,
    }
}

/// Build one configured program backend for admission checks.
#[must_use]
pub fn program_configuration() -> Config {
    let mut config = configuration();
    config.program = Some(ProgramBackend {
        keep_program_prompt: false,
        program: Programs { claude: Box::from(b"/claude".as_slice()), directory: Box::from(b"/programs".as_slice()) },
        endpoints: Box::new([0]),
        relay: Box::from(b"/agent".as_slice()),
        entropy: Secret::new(Box::new([7; 32])),
        effective_user: 1000,
        parent: None,
    });
    config
}
