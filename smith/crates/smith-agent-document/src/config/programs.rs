//! Program declarations and endpoint bindings (smith/docs/design/protocol/agent.md, section 4).
//! Keeps launch choices and adapter names only; it never opens or runs a program.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use serde::Deserialize;
use smith_agent_profile::profile::{Layer, LimitName, Per, Refusal};

use super::{ConfigurationRefusal, Endpoint};

/// A program the agent may run as a backend (smith/docs/design/protocol/agent.md, section 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramName {
    /// Claude Code, selected by the configuration for program sources.
    ClaudeCode,
}

/// A program's launch choices and directory of source configurations (smith/docs/design/protocol/programs.md, 4.2 and 5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub path: PathBuf,
    pub directory: PathBuf,
    /// Append the charter to the program's own prompt instead of replacing it.
    pub keep_program_prompt: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProgramDocument {
    path: Option<String>,
    directory: Option<String>,
    #[serde(default)]
    keep_program_prompt: bool,
}

fn name(value: &str, field: &str) -> Result<ProgramName, ConfigurationRefusal> {
    match value {
        "claude_code" => Ok(ProgramName::ClaudeCode),
        _ => Err(format!("{field}: unknown program {value}").into()),
    }
}

fn path(value: Option<String>, field: &'static str) -> Result<PathBuf, ConfigurationRefusal> {
    let value = value.ok_or_else(|| ConfigurationRefusal::Input(format!("{field}: required path")))?;
    if value.len() > 4096 || value.contains('\0') || !std::path::Path::new(&value).is_absolute() {
        return Err(format!("{field}: must be an absolute path of at most 4096 bytes without NUL").into());
    }
    Ok(value.into())
}

pub(super) fn build(documents: BTreeMap<String, ProgramDocument>) -> Result<Option<Program>, ConfigurationRefusal> {
    let mut program = None;
    for (key, document) in documents {
        name(&key, &format!("programs.{key}"))?;
        program = Some(Program {
            path: path(document.path, "programs.claude_code.path")?,
            directory: path(document.directory, "programs.claude_code.directory")?,
            keep_program_prompt: document.keep_program_prompt,
        });
    }
    Ok(program)
}

pub(super) fn endpoints(
    endpoints: &[Endpoint],
    program: Option<&Program>,
) -> Result<BTreeSet<u32>, ConfigurationRefusal> {
    let mut selections = BTreeSet::new();
    for endpoint in endpoints {
        match &endpoint.program {
            None => {}
            Some(value) => {
                name(value, &format!("endpoints[{}].program", endpoint.number))?;
                if program.is_none() {
                    return Err(ConfigurationRefusal::Profile(Refusal::Field {
                        limit: LimitName {
                            layer: Layer::Declared,
                            field: "program",
                            per: Per::Endpoint(endpoint.number),
                        },
                    }));
                }
                selections.insert(endpoint.number);
            }
        }
    }
    Ok(selections)
}

/// Prepare the service's program declaration; the shell later supplies entropy and relay resources.
#[cfg(unix)]
pub(super) fn backend(
    program: Option<&Program>,
    endpoints: &BTreeSet<u32>,
) -> Option<smith_agent_profile::ProgramBackend> {
    use std::os::unix::ffi::OsStrExt;
    let program = program?;
    Some(smith_agent_profile::ProgramBackend {
        keep_program_prompt: program.keep_program_prompt,
        program: smith_protocol_program::Programs {
            claude: program.path.as_os_str().as_bytes().into(),
            directory: program.directory.as_os_str().as_bytes().into(),
        },
        endpoints: endpoints.iter().copied().collect(),
        relay: Box::default(),
        entropy: skein_lib::Secret::new(Box::new([0; 32])),
        effective_user: skein_shell::effective_user(),
        parent: None,
    })
}

#[cfg(not(unix))]
pub(super) fn backend(
    _program: Option<&Program>,
    _endpoints: &BTreeSet<u32>,
) -> Option<smith_agent_profile::ProgramBackend> {
    None
}

/// Charge program declarations to the same startup pool as model declarations.
pub(super) fn domain(
    endpoints: &BTreeSet<u32>,
    limits: &mut smith_domain::Limits,
) -> Result<Box<[smith_domain::Program]>, ConfigurationRefusal> {
    let mut programs = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let name: Box<[u8]> = b"claude_code".as_slice().into();
        let owned = u64::try_from(size_of::<smith_domain::Program>())
            .ok()
            .and_then(|cell| cell.checked_add(skein_lib::len_u64(name.len())))
            .and_then(|owned| limits.configured_endpoint_bytes.checked_add(owned))
            .ok_or_else(|| {
                ConfigurationRefusal::Profile(Refusal::Derived {
                    limit: LimitName::deployment(Layer::Service, "configured_endpoint_bytes"),
                })
            })?;
        limits.configured_endpoint_bytes = owned;
        programs.push(smith_domain::Program { endpoint: smith_domain::run::charter::Endpoint(*endpoint), name });
    }
    Ok(programs.into_boxed_slice())
}
