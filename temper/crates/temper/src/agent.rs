//! Checked agent metadata and effect-free programs-directory preflight (configuration.md, 5).
use super::json::{self, Json};
use jig_domain_link::{Bounds, EndpointBound};
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_service::{self as service, settings::Agent};
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
pub(crate) fn read(configuration: &Path) -> Result<Agent, String> {
    let configuration = self::configuration(configuration)?;
    let limits = &configuration.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&configuration.profile, limits)
        .map_err(|_| "agent host limits refused")?;
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
        },
    })
}
/// Read declarations and check existing programs directories at their names (io.md, 5.3).
/// Missing directories stay missing; neither this nor smith's check reads any login.
pub(crate) fn preflight(named: &service::Named) -> Result<Agent, String> {
    let worker = named.worker.as_ref().ok_or("workers: standalone takes one worker")?;
    preflight_worker(worker)
}
pub(crate) fn preflight_worker(worker: &service::startup::NamedWorker) -> Result<Agent, String> {
    let agent = read(path(&worker.agent)).map_err(|why| format!("workers[0].agent.configuration: {why}"))?;
    for program in &agent.programs {
        smith_agent_shell::program::private_directory(path(&program.directory))
            .map_err(|why| program_directory_refusal(&why))?;
    }
    Ok(agent)
}
pub(crate) fn program_directory_refusal(why: &smith_agent_shell::program::Refusal) -> String {
    format!("workers[0].agent.configuration: programs.claude_code.directory: {why}")
}

/// Claude Code's launcher, beside its programs directory (protocol/setup.md, 6.6).
pub(crate) fn launcher(directory: &Path) -> Option<PathBuf> {
    directory.parent().map(|parent| parent.join("claude-code/claude"))
}
/// agent.json as smith decodes it: a program `"fetched"` names its launcher.
pub(crate) fn decoded(configuration: &Path) -> Result<smith_agent_shell::config::Decoded, String> {
    let bytes = smith_agent_shell::config::read_bytes(configuration)?;
    let bytes = substituted(&bytes)?.unwrap_or(bytes);
    smith_agent_shell::config::decode(&bytes)
}
/// `decoded`, resolved with stderr's diagnostics, as `config::read` does.
pub(crate) fn configuration(path: &Path) -> Result<smith_agent_shell::config::Configuration, String> {
    decoded(path)?.resolve(&mut std::io::stderr()).map_err(smith_agent_shell::config::ConfigurationRefusal::detail)
}
fn substituted(bytes: &[u8]) -> Result<Option<Box<[u8]>>, String> {
    // Inspect the bounded document first; smith keeps its own diagnostics for other documents.
    let Ok(document) = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: json::CONFIG_BYTES, tokens: 65_536, depth: 32 },
    ) else {
        return Ok(None);
    };
    let fetched = document
        .member(0, b"programs")
        .and_then(|programs| document.member(programs, b"claude_code"))
        .and_then(|program| document.member(program, b"path"))
        .and_then(|at| document.token(at))
        .filter(|token| token.kind == skein_json::document::Kind::String)
        .and_then(|token| document.text(token))
        == Some(temper_init::FETCHED);
    if !fetched {
        return Ok(None);
    }
    let mut document = Json::decode(bytes)?;
    let directory = document
        .string("programs/claude_code/directory")
        .map_err(|why| format!("programs.claude_code.directory: {why}"))?;
    let launcher =
        launcher(path(directory)).ok_or("programs.claude_code.directory: expected a directory with a parent")?;
    document.set("programs/claude_code/path", Json::String(launcher.as_os_str().as_bytes().into()))?;
    document.encode().map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(program: &serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"programs":{"claude_code":program}})).unwrap()
    }
    #[test]
    fn fetched_names_the_launcher_beside_the_programs_directory() {
        let bytes = document(&serde_json::json!({"path":"fetched","directory":"/data/programs"}));
        let bytes = substituted(&bytes).unwrap().unwrap();
        let expected = document(&serde_json::json!({"path":"/data/claude-code/claude","directory":"/data/programs"}));
        json::Json::decode(&bytes).unwrap().same(&json::Json::decode(&expected).unwrap(), "agent.json").unwrap();
    }
    #[test]
    fn an_absolute_program_document_goes_to_smith_byte_for_byte() {
        let bytes =
            b"{ \"programs\": {\"claude_code\": {\"path\": \"/bin/claude\", \"directory\": \"/data/programs\"}}}\n";
        assert_eq!(substituted(bytes).unwrap(), None);
    }
    #[test]
    fn fetched_requires_a_programs_directory_with_a_parent() {
        for program in [serde_json::json!({"path":"fetched"}), serde_json::json!({"path":"fetched","directory":"/"})] {
            let why = substituted(&document(&program)).unwrap_err();
            assert!(why.contains("programs.claude_code.directory"), "{why}");
        }
    }
}
