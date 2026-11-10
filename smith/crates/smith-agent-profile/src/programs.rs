//! Program configuration bounds without backends or grants (smith/docs/design/shell/README.md, section 2.6).
//! Contract: smith/docs/design/protocol/programs.md, sections 2, 3 and 5.

use crate::{Config, Limits};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{Duration, Secret, Token};
use smith_protocol_llm as llm;
use smith_protocol_program::{Programs, backend};

// The program owns the upper private token range below the root-close token.
const OWNER_NAMES: u64 = 1024;

/// Seven backend/MCP tokens and one service-owned directory-open token.
pub const STRIDE: u64 = 8;

/// First token in the private program owner range.
pub const FIRST: Token = Token::new(u64::MAX.checked_sub(OWNER_NAMES).expect("private owner range"));
// Host-result allowance shared with smith_protocol_llm::render_bound.
const HOST_RESULT_OVERHEAD_BYTES: u64 = 7;
const FIXED_RESULT_BYTES: u64 = 4096;
// MCP IDs, names, descriptions and the RPC envelope surround the admitted body.
const MCP_ENVELOPE_BYTES: u32 = 4096;
// Native JSON headers, usage and block envelopes surround semantic payloads.
const STREAM_ENVELOPE_BYTES: u32 = 65_536;
// A JSON byte can be emitted as a six-byte Unicode escape.
const JSON_ESCAPE_BYTES: u32 = 6;
const CONTRACT_SCHEMAS: u32 = 2;
const MCP_AUTHENTICATION: Duration = Duration::from_secs(5);

/// Deadline for starting a program and opening its private directory.
pub const STARTUP: Duration = Duration::from_secs(30);

/// Maximum encoded relay executable path length.
pub const RELAY_PATH_BYTES: usize = 4096;

/// Required entropy bytes for a program backend.
pub const ENTROPY_BYTES: usize = 32;

/// Native IO slots reserved per program conversation.
pub const IO_SLOTS: u32 = 11;

/// File slots for three private directories and one serialized native snapshot load.
pub const FILE_SLOTS: u32 = 4;

/// One configured program selected by the main account's source grant (section 3).
#[derive(Debug)]
pub struct ProgramBackend {
    /// Append the charter to the program's own prompt instead of replacing it.
    pub keep_program_prompt: bool,
    pub program: Programs,
    pub endpoints: Box<[u32]>,
    pub relay: Box<[u8]>,
    pub entropy: Secret,
    /// Effective UID read by the shell before any private root is adopted.
    pub effective_user: u32,
    /// The agent's own process ID, read by the shell, when each program
    /// process is to start through the agent's `--program-child` mode and
    /// die with it (protocol/programs.md, 5.1); `None` where the system
    /// has no parent-death signal, and in simulated worlds.
    pub parent: Option<u32>,
}

/// Derive a program backend's receiving and rendering settings.
#[must_use]
pub fn settings(
    config: &ProgramBackend,
    limits: &Limits,
    environment: &smith_protocol_machine::Environment,
) -> Option<backend::Settings> {
    let mut translation = limits.llm.translation;
    // Built-in results are skipped before rendering; only host text and
    // bounded root feedback use this MCP reply ceiling (smith/docs/design/protocol/programs.md 5.5).
    let result = limits
        .domain
        .session
        .delegated_result_bytes
        .max(u64::from(limits.domain.run.host_reply_bytes).checked_add(HOST_RESULT_OVERHEAD_BYTES)?)
        .max(u64::from(limits.domain.run.message_bytes))
        .max(FIXED_RESULT_BYTES)
        .checked_add(u64::from(llm::CUT_MARKER_BYTES))?;
    translation.rendered_result = u32::try_from(result).ok()?;
    let contract_bytes = u32::try_from(limits.llm.contract_bytes).ok()?;
    // Host schemas count in run::charter::cost against run_bytes (smith/docs/design/domain/run.md, 5.2).
    // schemas_for_contract caps finish/deliver at contract_bytes; the fixed
    // inventory bounds every other generated schema (smith/docs/design/protocol/programs.md, 5.3).
    let schema_bytes = contract_bytes
        .max(u32::try_from(limits.domain.run.run_bytes).ok()?)
        .max(u32::try_from(llm::fixed_schema_bytes(&translation.stated)?).ok()?);
    // A call's JSON input plus its RPC ID/name/envelope. This is a line
    // ceiling, not another allocation beside MCP's priced input buffers.
    let call_bytes = translation.stated.call_input_bytes.checked_add(MCP_ENVELOPE_BYTES)?;
    // Each tool byte can become six JSON escape bytes in tools/list. Finish
    // and delivery schemas each have their own contract-sized encoded ceiling;
    // fixed room covers descriptions, names and the RPC/list envelope.
    let tool_bytes = u32::try_from(translation.tool_bytes)
        .ok()?
        .checked_mul(JSON_ESCAPE_BYTES)?
        .checked_add(contract_bytes.checked_mul(CONTRACT_SCHEMAS)?)?
        .checked_add(MCP_ENVELOPE_BYTES)?;
    // A result byte needs at most six JSON escape bytes. Tool-list and result
    // are alternative messages in the same MCP output slot, hence max, not sum.
    let answer_bytes =
        translation.rendered_result.checked_mul(JSON_ESCAPE_BYTES)?.checked_add(MCP_ENVELOPE_BYTES)?.max(tool_bytes);
    // Stream JSON wraps semantic answer text/arguments (six-byte escapes),
    // plus message headers, IDs, usage and block envelopes.
    let line_bytes = translation.client.answer.checked_mul(JSON_ESCAPE_BYTES)?.checked_add(STREAM_ENVELOPE_BYTES)?;
    Some(backend::Settings {
        keep_program_prompt: config.keep_program_prompt,
        program: Programs { claude: config.program.claude.clone(), directory: config.program.directory.clone() },
        relay: config.relay.clone(),
        parent: config.parent,
        directory: None,
        workspace: None,
        path_bytes: limits.domain.session.tools.host_path_bytes,
        environment: environment.command_entries(),
        mcp: smith_protocol_program::mcp::Limits {
            message_bytes: call_bytes,
            answer_bytes,
            schema_bytes,
            calls: u16::try_from(limits.domain.run.calls).ok()?,
            unauthenticated: skein_lib::Time::from_nanos(MCP_AUTHENTICATION.as_nanos()),
        },
        translation,
        receiving: limits.llm.receiving,
        stream_tokens: backend::Settings::stream_tokens(line_bytes),
        prompt_bytes: u64::from(translation.client.request),
        contract_bytes,
        line_bytes,
        // client.request already bounds encoded JSON, including escaping and
        // all admitted history. This user envelope omits the model's schemas.
        input_bytes: translation.client.request.checked_add(STREAM_ENVELOPE_BYTES)?,
        max_turns: 1,
        grace: limits.io.close_timeout,
        startup: STARTUP,
    })
}

/// Reserve the program conversations' IO, file and routing capacities.
#[must_use]
pub fn reserved(limits: &Limits, settings: &backend::Settings) -> Option<Limits> {
    let mut reserved = *limits;
    let conversations = limits.domain.run.conversations;
    if conversations == 0 || u64::from(conversations).checked_mul(STRIDE)? > OWNER_NAMES {
        return None;
    }
    reserved.io.sockets = reserved.io.sockets.checked_add(IO_SLOTS.checked_mul(conversations)?)?;
    reserved.io.output = reserved.io.output.max(settings.mcp.answer_bytes);
    reserved.llm.connection.io.output = reserved.io.output;
    reserved.io.intake = reserved.io.intake.max(settings.mcp.message_bytes);
    reserved.llm.connection.io.intake = reserved.io.intake;
    reserved.file_slots = reserved.file_slots.checked_add(FILE_SLOTS.checked_mul(conversations)?)?;
    let operations = io::operations(&reserved.io)?;
    reserved.routes = reserved.routes.max(operations.checked_add(crate::ROUTE_HEADROOM)?);
    Some(reserved)
}

/// Select a configured backend only when at least one endpoint names it.
#[must_use]
pub fn bound(config: Option<&ProgramBackend>) -> Option<&ProgramBackend> {
    match config {
        Some(program) if !program.endpoints.is_empty() => Some(program),
        Some(_) | None => None,
    }
}

/// Whether a named program's settings and reserved service fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn program_valid(config: &Config) -> bool {
    let Some(program) = bound(config.program.as_ref()) else { return true };
    let Some(settings) = settings(program, &config.limits, &config.environment) else { return false };
    let Some(reserved) = reserved(&config.limits, &settings) else { return false };
    backend::backend_valid(&settings)
        && crate::valid(&config.limits, &config.llm_endpoints)
        && crate::valid(&reserved, &config.llm_endpoints)
}

/// Native operation route capacity after reserving the configured program's I/O.
#[must_use]
pub fn program_operations(config: &Config) -> Option<u32> {
    match bound(config.program.as_ref()) {
        Some(program) => {
            Some(reserved(&config.limits, &settings(program, &config.limits, &config.environment)?)?.routes)
        }
        None => Some(config.limits.routes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::program_configuration as configuration;
    use smith_domain as domain;

    #[test]
    fn configured_and_native_programs_fit_their_limits() {
        assert!(program_valid(&crate::fixture::configuration()));
        assert!(program_valid(&configuration()));
    }

    #[test]
    fn invalid_limits_are_refused() {
        let base = configuration().limits;
        for (name, limits) in [
            ("contract bytes", Limits { llm: llm::ComponentLimits { contract_bytes: u64::MAX, ..base.llm }, ..base }),
            ("socket overflow", Limits { io: io::Limits { sockets: u32::MAX, ..base.io }, ..base }),
            ("file overflow", Limits { file_slots: u32::MAX, ..base }),
            (
                "host path",
                Limits {
                    domain: domain::Limits {
                        session: domain::session::Limits {
                            tools: domain::tools::Limits { host_path_bytes: u32::MAX, ..base.domain.session.tools },
                            ..base.domain.session
                        },
                        ..base.domain
                    },
                    ..base
                },
            ),
            (
                "conversations",
                Limits {
                    domain: domain::Limits {
                        run: domain::run::Limits { conversations: u32::MAX, ..base.domain.run },
                        ..base.domain
                    },
                    ..base
                },
            ),
            (
                "receive mismatch",
                Limits { io: io::Limits { receive: base.io.receive.checked_add(1).unwrap(), ..base.io }, ..base },
            ),
            ("socket reserve", Limits { io: io::Limits { sockets: u32::MAX / 8 - 10, ..base.io }, ..base }),
        ] {
            let mut config = configuration();
            config.limits = limits;
            assert!(!program_valid(&config), "{name}");
        }
    }
}
