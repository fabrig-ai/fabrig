//! The worker's agent: its document read and preflighted (protocol/binaries.md, 5; protocol/configuration/README.md, 4).
use jig_domain_link::{Bounds, EndpointBound};
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_service::{self as service, settings::Agent};
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
pub fn read(configuration: &Path) -> Result<Agent, String> {
    settings(self::configuration(configuration)?)
}
fn settings(configuration: smith_agent_document::config::Configuration) -> Result<Agent, String> {
    let limits = &configuration.service.limits;
    let host_limits =
        smith_agent_document::host_limits(&configuration.profile, limits).map_err(|_| "agent host limits refused")?;
    let channel = limits.channel;
    let mut protocol = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).ok_or("agent output bound")?;
    let endpoints = configuration
        .declarations
        .endpoints
        .iter()
        .map(|endpoint| EndpointBound {
            endpoint: endpoint.number,
            window: endpoint.limits.window,
            output: endpoint.limits.output,
        })
        .collect();
    let policy = configuration.profile.policy;
    let message_bytes = limits
        .domain
        .run
        .message_bytes
        .checked_sub(u32::try_from(temper_worker_domain::SEPARATOR).expect("message separator fits u32"))
        .ok_or("agent message_bytes: too small for the rendered separator")?;
    let programs = configuration
        .program
        .into_iter()
        .map(|program| service::settings::Program {
            adapter: service::settings::Adapter::ClaudeCode,
            path: program.path.as_os_str().as_bytes().into(),
            directory: program.directory.as_os_str().as_bytes().into(),
        })
        .collect();
    let reply_bytes = protocol.bodies.host_reply_text;
    Ok(Agent {
        run_limits: limits.domain.run,
        tool_payload: configuration.profile.declared.tool_payload,
        programs,
        endpoints: configuration.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: Bounds {
            program_endpoints: configuration.program_endpoints.iter().copied().collect::<Vec<_>>().into_boxed_slice(),
            endpoints,
            turns: policy.max_turns,
            spend: policy.max_spend,
            time: policy.max_time,
            waiting: policy.max_waiting,
            checks: policy.check_timeout,
            message_bytes,
            inbox: policy.inbox,
            unacknowledged: policy.unacknowledged,
            reply_bytes: Some(reply_bytes),
        },
    })
}
/// Read declarations and check existing programs directories at their names (skein/docs/design/runtime/io/files.md, 3).
/// Missing directories stay missing; neither this nor smith's check reads any login.
pub fn preflight_worker(worker: &service::startup::NamedWorker) -> Result<Agent, String> {
    Ok(prepare_worker(worker)?.agent)
}
/// One read supplies the host settings, source environment and pinned document.
pub struct Prepared {
    pub agent: Agent,
    pub bytes: Box<[u8]>,
    pub environment: Box<[Box<[u8]>]>,
}
pub fn prepare_worker(worker: &service::startup::NamedWorker) -> Result<Prepared, String> {
    let prepared = prepare(path(&worker.agent)).map_err(|why| format!("workers[0].agent.configuration: {why}"))?;
    for program in &prepared.agent.programs {
        smith_agent_document::program::private_directory(path(&program.directory))
            .map_err(|why| program_directory_refusal(&why))?;
    }
    Ok(prepared)
}
fn prepare(path: &Path) -> Result<Prepared, String> {
    let bytes = smith_agent_document::config::read_bytes(path)?;
    let configuration = decoded_bytes(&bytes)?
        .resolve(&mut std::io::stderr())
        .map_err(smith_agent_document::config::ConfigurationRefusal::detail)?;
    let environment = configuration.service.environment.command_entries();
    Ok(Prepared { agent: settings(configuration)?, bytes, environment })
}
#[must_use]
pub fn program_directory_refusal(why: &smith_agent_document::program::Refusal) -> String {
    format!("workers[0].agent.configuration: programs.claude_code.directory: {why}")
}

/// Claude Code's launcher, beside its programs directory (protocol/setup/owned.md, 6).
#[must_use]
pub fn launcher(directory: &Path) -> Option<PathBuf> {
    temper_launcher::beside(directory)
}
/// agent.json as smith decodes it: a program `"fetched"` names its launcher.
pub fn decoded(configuration: &Path) -> Result<smith_agent_document::config::Decoded, String> {
    let bytes = smith_agent_document::config::read_bytes(configuration)?;
    decoded_bytes(&bytes)
}
fn decoded_bytes(bytes: &[u8]) -> Result<smith_agent_document::config::Decoded, String> {
    let substituted = temper_launcher::substituted(bytes)?;
    smith_agent_document::config::decode(substituted.as_deref().unwrap_or(bytes))
}
/// `decoded`, resolved with stderr's diagnostics, as `config::read` does.
pub fn configuration(path: &Path) -> Result<smith_agent_document::config::Configuration, String> {
    decoded(path)?.resolve(&mut std::io::stderr()).map_err(smith_agent_document::config::ConfigurationRefusal::detail)
}

/// Keep the generic start from reading the agent document again.
#[must_use]
pub fn without_configuration(
    files: Box<[jig_protocol_config::File]>,
    configuration: &[u8],
) -> Box<[jig_protocol_config::File]> {
    files.into_iter().filter(|file| file.path.as_ref() != configuration).collect()
}
/// Supply the already checked bytes to the service's ordinary file checks.
#[must_use]
pub fn with_configuration(
    files: service::settings::Files,
    configuration: &[u8],
    bytes: &[u8],
) -> service::settings::Files {
    let mut entries = skein_lib::List::with_capacity(files.entries.len().checked_add(1).expect("configuration entry"));
    for entry in files.entries.into_boxed() {
        entries.push(entry).expect("existing file capacity");
    }
    entries.push((configuration.into(), skein_lib::Secret::new(bytes.into()))).expect("configuration capacity");
    service::settings::Files { entries }
}
