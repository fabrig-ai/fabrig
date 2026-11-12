use crate::settings;
use jig_domain_link::{Bounds, EndpointBound};
pub const DIALECT: u32 = 3;
mod worker;
mod worker_git;
mod agent;
pub(crate) mod worker_peers;
pub use agent::agent;
pub use worker::*;
pub use worker_git::*;
pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits { crate::worker_settings::payload_limits(&worker_limits()).unwrap() }
pub fn worker_runtime() -> crate::WorkerRuntime { crate::WorkerRuntime { root: skein_io::kernel::Fd::new(9), path: b"/worker".as_slice().into() } }
pub fn answer(ending: jig_domain_host::Ending) -> jig_domain_host::LinkUp { jig_domain_host::LinkUp::Answer { run: skein_lib::Token::new(7), attempt: skein_lib::Token::new(2), answer: jig_domain_host::RunAnswer { read: None, refused: Box::default(), turns: 0, spent: 0, ending } } }
pub(crate) fn assert_worker_only(service: &crate::worker_alone::Service) { let crate::worker_alone::Service { process: _, worker: _, held_listeners: _, listeners_closing: _ } = service; }
fn charter() -> smith_domain_run::Charter {
    use skein_lib::Duration;
    use smith_domain_run::{self as run, charter, outcome};
    run::Charter {
        check_timeout: None,
        mode: run::Mode::Finite,
        resume: false,
        waiting: Duration::from_secs(30),
        instructions: Box::default(),
        brief: run::Brief { sections: Box::default() },
        conventions: None,
        grants: charter::Grants {
            wait: true,
            deliver: None,
            save: false,
            agents: false,
            host_tools: Box::default(),
            tools: charter::Tools { inspect: false, modify: false, shell: false },
        },
        outcome: outcome::OutcomeSpec {
            change: None,
            verdicts: Box::default(),
            failure: None,
            report: Some(outcome::TextSpec { max: 1024, fields: Box::default() }),
        },
        budget: run::Budget { turns: 64, spend: 10_000, time: Duration::from_secs(3600) },
        reserve: charter::Reserve { turns: 0, time: Duration::ZERO },
        llm: charter::Llm {
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
            dialect: 3,
            account: Some(1),
            endpoint: charter::Endpoint(1),
            model: b"model".as_slice().into(),
            effort: None,
            window: 8192,
            output: 512,
        },
        models: Box::default(),
    }
}

mod startup;
pub use startup::{worker_alone_document, owned_worker_alone_document, owned_worker_agent, startup_agent, files, with};
pub use worker::assignment as worker_assignment;
/// Parse the shipped agent with inert destinations and selected payload/output quantities.
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary fixture code embeds the shipped profile and returns the shell's text diagnostic"
)]
pub fn stock_configuration(payload: u32, output: u32) -> Result<smith_agent_shell::config::Configuration, String> {
    let bytes = include_str!("../../../temper-init/template/agent.json")
        .replace("\"tool_payload\": 10662", &format!("\"tool_payload\": {payload}"))
        .replace("\"output\": 8192", &format!("\"output\": {output}"))
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", r#"{"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}}"#)
        .replace("api.anthropic.com:443", "127.0.0.1:8373")
        .replace("\"server_name\": \"api.anthropic.com\",", "")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"")
        .replace("\"server_name\": \"api.openai.com\",", "");
    smith_agent_shell::config::parse(bytes.as_bytes())
}

/// The startup agent derived from the shell configuration, including its hello.
#[expect(clippy::disallowed_methods, reason = "ordinary fixture code traverses the parsed model declarations")]
pub fn parsed_agent(config: smith_agent_shell::config::Configuration) -> settings::Agent {
    use std::os::unix::ffi::OsStrExt;
    let limits = config.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&config.profile, &limits).unwrap();
    let mut protocol = smith_host_protocol::Limits {
        bodies: limits.channel.bodies,
        charter: limits.channel.charter,
        transcript: limits.channel.transcript,
        endpoints: limits.channel.endpoints,
        channel: limits.channel.channel,
        start_bytes: limits.channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).unwrap();
    settings::Agent {
        run_limits: limits.domain.run,
        tool_payload: config.profile.declared.tool_payload,
        programs: config
            .program
            .into_iter()
            .map(|program| settings::Program {
                adapter: settings::Adapter::ClaudeCode,
                path: program.path.as_os_str().as_bytes().into(),
                directory: program.directory.as_os_str().as_bytes().into(),
            })
            .collect(),
        endpoints: config.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: Bounds {
            endpoints: config
                .declarations
                .endpoints
                .iter()
                .map(|endpoint| EndpointBound {
                    endpoint: endpoint.number,
                    window: endpoint.limits.window,
                    output: endpoint.limits.output,
                })
                .collect(),
            program_endpoints: config.program_endpoints.into_iter().collect(),
            turns: config.profile.policy.max_turns,
            spend: config.profile.policy.max_spend,
            time: config.profile.policy.max_time,
            waiting: config.profile.policy.max_waiting,
            checks: config.profile.policy.check_timeout,
            message_bytes: limits.domain.run.message_bytes.checked_sub(2).unwrap(),
            inbox: config.profile.policy.inbox,
            unacknowledged: config.profile.policy.unacknowledged,
            reply_bytes: None,
        },
    }
}

