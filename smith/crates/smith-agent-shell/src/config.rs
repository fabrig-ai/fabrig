//! Generated configuration values or JSON read once before the agent opens its channel.
//! It retains endpoint names and destinations, program paths, limits, command environment,
//! and trace settings; no credential is read here.
//! Durations in JSON are milliseconds. Contract: smith/docs/design/protocol/agent.md, sections 4 and 6.

#[cfg(unix)]
use std::path::Path;

use skein_channel::StreamMode;
use skein_lib::{Duration, List};
use smith_agent_service as service;
use smith_domain as domain;
use smith_protocol_llm as llm;
use smith_protocol_machine as machine;

use crate::trace::{Capture, TraceConfig};
use smith_agent_service::profile::{ENDPOINTS, Layer, LimitName, Refusal};

const CONFIG_BYTES: u64 = 1 << 20;

mod document;
mod endpoints;
use endpoints::{PreparedEndpoints, build_endpoints, build_models};
mod native;
#[cfg(test)]
use document::Variable;
use document::{DeclaredOverrides, Document, Endpoint, ModelDocument, PolicyOverrides, ProfileDocument, TraceDocument};
pub use native::TRUST_BYTES;

#[cfg(all(test, unix))]
use native::trust_source;
#[cfg(unix)]
pub use native::{configured_trust, trust};
mod profile;
mod programs;
mod refusal;
use profile::build_profile;
pub use programs::{Program, ProgramName};
mod trace;
pub use refusal::ConfigurationRefusal;

/// Parsed startup configuration and the service it can build.
pub struct Configuration {
    pub programs: std::collections::BTreeMap<ProgramName, Program>,
    /// Program selections keyed by endpoint number; only endpoints naming a program are present.
    #[expect(
        clippy::zero_sized_map_values,
        reason = "Endpoint selections retain the adapter name; Claude Code is currently the sole adapter."
    )]
    pub endpoint_programs: std::collections::BTreeMap<u32, ProgramName>,
    pub service: service::Config,
    pub memory: u64,
    pub trace: Option<TraceConfig>,
    pub profile: service::profile::Profile,
    pub declarations: service::profile::Configuration,
}

/// Validated configuration values awaiting native endpoint resolution and trust loading.
/// Decoding retains names and trust sources; it performs no DNS or filesystem effects.
#[expect(clippy::zero_sized_map_values, reason = "the shell configuration uses a unit selector map")]
pub struct Decoded {
    programs: std::collections::BTreeMap<ProgramName, Program>,
    endpoint_programs: std::collections::BTreeMap<u32, ProgramName>,
    profile: service::profile::Profile,
    declarations: service::profile::Configuration,
    limits: service::Limits,
    domain_programs: Box<[domain::Program]>,
    environment: machine::Environment,
    trace: Option<TraceConfig>,
    endpoints: PreparedEndpoints,
}

/// Read one JSON document and reject unknown keys, oversized input and
/// invalid transport settings before any channel byte is read.
#[cfg(unix)]
pub fn read_decoded(path: &Path) -> Result<Decoded, String> {
    let bytes = skein_shell::read_file(path, u32::try_from(CONFIG_BYTES).expect("configuration bound fits u32"))
        .map_err(|why| match why {
            skein_shell::Unread::Absent => format!("configuration metadata: {}: file absent", path.display()),
            skein_shell::Unread::NotAFile => format!("configuration {}: not a regular file", path.display()),
            skein_shell::Unread::TooLarge { max } => {
                format!("configuration {} is larger than {max} bytes", path.display())
            }
            skein_shell::Unread::Unsafe(reason) => format!("configuration read {}: {reason:?}", path.display()),
            skein_shell::Unread::Kernel(error) => format!("configuration read {}: {error:?}", path.display()),
        })?;
    decode(&bytes)
}

/// Read and resolve one native configuration, reporting trust diagnostics to stderr.
#[cfg(unix)]
pub fn read(path: &Path) -> Result<Configuration, String> {
    read_decoded(path)?.resolve(&mut std::io::stderr()).map_err(ConfigurationRefusal::detail)
}

/// Validate generated child configuration before the local host spawns it.
pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    if u64::try_from(bytes.len()).expect("byte length fits u64") > CONFIG_BYTES {
        return Err(format!("configuration is larger than {CONFIG_BYTES} bytes"));
    }
    let document: Document = serde_json::from_slice(bytes).map_err(|error| format!("configuration JSON: {error}"))?;
    decode_document(document).map_err(ConfigurationRefusal::detail)
}

/// Decode and resolve generated child settings with native effects and stderr diagnostics.
pub fn parse(bytes: &[u8]) -> Result<Configuration, String> {
    decode(bytes)?.resolve(&mut std::io::stderr()).map_err(ConfigurationRefusal::detail)
}

// The exact encoded count supplies machine.env_bytes before the shared encoder
// validates names. Contract: smith/docs/design/protocol/limits.md, sections 3.5 and 4.
fn environment_size_refusal() -> ConfigurationRefusal {
    ConfigurationRefusal::Profile(Refusal::Derived { limit: LimitName::deployment(Layer::Machine, "env_bytes") })
}

fn environment_refusal(error: machine::Unencodable) -> ConfigurationRefusal {
    match error {
        machine::Unencodable::Variable { index } | machine::Unencodable::Repeated { index } => {
            ConfigurationRefusal::Profile(Refusal::Environment { index })
        }
        // This configuration's exact count derives the receiving bound; a
        // smaller bound would mean the derived field cannot hold that count.
        machine::Unencodable::TooLarge { .. } => environment_size_refusal(),
    }
}

fn event_limits(
    limits: &service::Limits,
    capture: Capture,
    header: &[smith_events::EffectiveLimit],
    write_deadline: Duration,
) -> Result<smith_protocol_events::EventLimits, ConfigurationRefusal> {
    let events = smith_agent_service::event_limits(
        limits,
        capture,
        1,
        smith_charter::CEILINGS.llm_effort_some,
        header,
        write_deadline,
    )
    .ok_or("event ownership overflow")?;
    Ok(events)
}

pub(crate) fn effective_header(
    profile: &service::profile::Profile,
    declarations: &service::profile::Configuration,
    limits: &service::Limits,
    names: &smith_protocol_translate::Endpoints,
    endpoints: &llm::Endpoints,
) -> Result<List<smith_events::EffectiveLimit>, ConfigurationRefusal> {
    let inventory =
        service::profile::effective(profile, declarations, limits, endpoints).ok_or("effective limits overflow")?;
    let header = service::profile::header(&inventory, names, declarations).ok_or("effective names overflow")?;
    Ok(header)
}

#[cfg(test)]
fn build(document: Document) -> Result<Configuration, String> {
    decode_document(document)
        .and_then(|decoded| decoded.resolve(&mut std::io::stderr()))
        .map_err(ConfigurationRefusal::detail)
}

/// Build the agent from generated settings values, without a JSON file or byte round trip.
pub fn from_value(value: serde_json::Value) -> Result<Configuration, ConfigurationRefusal> {
    let document = serde_json::from_value(value)
        .map_err(|why| ConfigurationRefusal::Input(format!("configuration JSON: {why}")))?;
    decode_document(document)?.resolve(&mut std::io::stderr())
}

/// Resolve profile overrides before destinations or credentials are consulted.
pub fn profile_from_value(value: serde_json::Value) -> Result<service::profile::Profile, ConfigurationRefusal> {
    let document =
        serde_json::from_value(value).map_err(|why| ConfigurationRefusal::Input(format!("profile JSON: {why}")))?;
    build_profile(&document)
}

fn decode_document(document: Document) -> Result<Decoded, ConfigurationRefusal> {
    let profile = build_profile(&document.profile)?;
    let programs = programs::build(document.programs)?;
    let endpoint_programs = programs::endpoints(&document.endpoints, &programs)?;
    if document.endpoints.is_empty() {
        return Err(ConfigurationRefusal::Profile(Refusal::Field {
            limit: LimitName::deployment(Layer::Declared, "endpoints"),
        }));
    }
    if document.endpoints.len() > usize::try_from(ENDPOINTS).expect("endpoint count fits usize") {
        return Err("too many endpoints".into());
    }
    let trace = trace::build(document.trace)?;
    let mut environment = Vec::with_capacity(document.environment.len());
    let mut environment_bytes = 0_u64;
    for variable in document.environment {
        let entry_bytes = variable
            .name
            .len()
            .checked_add(variable.value.len())
            .and_then(|bytes| bytes.checked_add(1))
            .ok_or_else(environment_size_refusal)?;
        environment_bytes = environment_bytes
            .checked_add(u64::try_from(entry_bytes).map_err(|_| environment_size_refusal())?)
            .ok_or_else(environment_size_refusal)?;
        environment.push(machine::Variable {
            name: variable.name.into_bytes().into(),
            value: variable.value.into_bytes().into(),
        });
    }
    let mut declarations = Vec::with_capacity(document.endpoints.len());
    for endpoint in &document.endpoints {
        declarations.push(build_models(endpoint)?);
    }
    let declarations = service::profile::Configuration {
        role_bound: None,
        environment_bytes: u32::try_from(environment_bytes).map_err(|_| environment_size_refusal())?,
        endpoints: declarations.into_boxed_slice(),
    };
    let mut limits = service::profile::derive(&profile, &declarations).map_err(ConfigurationRefusal::Profile)?;
    let domain_programs = programs::domain(&endpoint_programs, &mut limits.domain)?;
    let environment = machine::Environment::encode(&environment, &limits.machine).map_err(environment_refusal)?;
    let endpoints = build_endpoints(document.endpoints, &declarations, &limits, &profile)?;
    Ok(Decoded {
        programs,
        endpoint_programs,
        profile,
        declarations,
        limits,
        domain_programs,
        environment,
        trace,
        endpoints,
    })
}

impl Decoded {
    /// Resolve names and load trust after the caller blocks termination signals.
    /// Trust diagnostics go only to the supplied operator writer.
    pub fn resolve(self, errors: &mut dyn std::io::Write) -> Result<Configuration, ConfigurationRefusal> {
        let Self {
            programs,
            endpoint_programs,
            profile,
            declarations,
            mut limits,
            domain_programs,
            environment,
            trace,
            endpoints,
        } = self;
        let llm_endpoints =
            llm::Endpoints::new(native::endpoints(endpoints.llm, &limits.llm.connection, errors)?, ENDPOINTS)
                .map_err(|error| format!("LLM endpoints: {error:?}"))?;
        if let Some(trace) = &trace {
            // First admit the sink, then include its own limits in the receiving header.
            for _ in 0..2 {
                let header = effective_header(&profile, &declarations, &limits, &endpoints.channel, &llm_endpoints)?;
                limits.events =
                    Some(event_limits(&limits, trace.capture, header.as_slice(), profile.policy.write_deadline)?);
            }
        }
        let config = service::Config {
            program: programs::backend(&programs, &endpoint_programs),
            limits,
            domain: domain::Config { programs: domain_programs, endpoints: endpoints.domain, models: endpoints.models },
            channel_endpoints: endpoints.channel,
            llm_endpoints,
            environment,
            stream_mode: StreamMode::Two,
        };
        check_limits(&config)?;
        Ok(Configuration {
            service: config,
            memory: profile.declared.memory,
            trace,
            profile,
            declarations,
            programs,
            endpoint_programs,
        })
    }
}

fn check_limits(config: &service::Config) -> Result<(), ConfigurationRefusal> {
    if !service::valid(&config.limits, &config.llm_endpoints) || !service::program_valid(config) {
        return Err(ConfigurationRefusal::Profile(Refusal::Derived {
            limit: LimitName::deployment(Layer::Service, "limits"),
        }));
    }
    Ok(())
}

fn duration(millis: u64, name: LimitName) -> Result<Duration, ConfigurationRefusal> {
    let nanos = millis.checked_mul(1_000_000).ok_or(ConfigurationRefusal::Profile(Refusal::Derived { limit: name }))?;
    Ok(Duration::from_nanos(nanos))
}

pub(crate) fn profile_refusal(error: Refusal) -> String {
    String::from_utf8(service::profile::words(&error).into_vec()).expect("refusal words are UTF-8")
}

#[cfg(test)]
mod tests;
