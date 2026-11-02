//! Program backend admission and lower routing (smith/docs/design/protocol/programs.md, 2, 3, 5).
//! Keeps main and owner-keyed child backends, grants and pipe routes; never reads source logins.
//! The service calls these hooks at the same stages as its direct LLM component.

use super::{Config, ConfigError, Failure, Limits, Owner, Service, Stages, component_owner, credential};
use alloc::boxed::Box;
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::{Duration, List, Map, Queue, Secret, Token, sha256};
use smith_domain::{self as domain};
use smith_protocol_llm as llm;
use smith_protocol_program::{Programs, backend};

// The program owns the upper private token range below the root-close token.
const OWNER_NAMES: u64 = 1024;
// Seven backend/MCP tokens and one service-owned directory-open token.
const STRIDE: u64 = 8;
const FIRST: Token = Token::new(u64::MAX.checked_sub(OWNER_NAMES).expect("private owner range"));
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
const STARTUP: Duration = Duration::from_secs(30);
const RELAY_PATH_BYTES: usize = 4096;
const ENTROPY_BYTES: usize = 32;
const IO_SLOTS: u32 = 11;
// Three private directories and one serialized native snapshot load.
const FILE_SLOTS: u32 = 4;

/// One configured program selected by the main account's source grant (section 3).
#[derive(Debug)]
pub struct ProgramBackend {
    pub program: Programs,
    pub endpoints: Box<[u32]>,
    pub relay: Box<[u8]>,
    pub entropy: Secret,
    /// Effective UID read by the shell before any private root is adopted.
    pub effective_user: u32,
}

/// The shell's programs root cannot be adopted by this service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdoptError {
    /// No endpoint names a program backend.
    Unconfigured,
    /// The checked file table has no free slot.
    Full,
}

#[derive(Debug)]
pub(super) struct Program {
    backend: backend::Backend,
    configuration: ProgramBackend,
    environment: smith_protocol_machine::Environment,
    directory: Option<backend::Root>,
    workspace: Option<backend::Root>,
    mounts: Box<[backend::Mount]>,
    grant: Option<backend::Grant>,
    turns: u32,
    time: Duration,
    owner: Option<Token>,
    children: List<Option<Child>>,
    below: Queue<backend::Below>,
    pipes: Map<Token, Token>,
}

// A slot retains its token range until its backend and directory open settle.
#[derive(Debug)]
struct Child {
    owner: Token,
    first: Token,
    backend: backend::Backend,
    opening: Option<llm::FromDomain>,
    closing: bool,
    yielding: bool,
}

impl Child {
    fn directory_owner(&self) -> Token {
        Token::new(self.first.raw().checked_add(7).expect("child token range"))
    }

    fn opened(
        &mut self,
        env: &skein_lib::Env<llm::ComponentLimits>,
        root: backend::Root,
        request: llm::FromDomain,
        up: &mut Queue<llm::ToDomain>,
        below: &mut Queue<backend::Below>,
    ) {
        self.backend.directory(root);
        if self.closing {
            self.backend.close(env, up, below);
        } else {
            self.backend.from_domain(env, request, up, below);
            if self.yielding {
                self.yielding = false;
                self.backend.from_domain(env, llm::FromDomain::Yield { owner: self.owner }, up, below);
            }
        }
    }

    fn close(
        &mut self,
        service: &skein_lib::Env<llm::ComponentLimits>,
        up: &mut Queue<llm::ToDomain>,
        below: &mut Queue<backend::Below>,
    ) {
        self.closing = true;
        if self.opening.is_none() {
            self.backend.close(service, up, below);
        }
    }
}

fn settings(
    config: &ProgramBackend,
    limits: &Limits,
    environment: &smith_protocol_machine::Environment,
) -> Option<backend::Settings> {
    let mut adapter = limits.llm.adapter;
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
    adapter.rendered_result = u32::try_from(result).ok()?;
    let contract_bytes = u32::try_from(limits.llm.contract_bytes).ok()?;
    // Host schemas count in run::charter::cost against run_bytes (smith/docs/design/domain/run.md, 5.2).
    // schemas_for_contract caps finish/deliver at contract_bytes; the fixed
    // inventory bounds every other generated schema (smith/docs/design/protocol/programs.md, 5.3).
    let schema_bytes = contract_bytes
        .max(u32::try_from(limits.domain.run.run_bytes).ok()?)
        .max(u32::try_from(llm::fixed_schema_bytes(&adapter.stated)?).ok()?);
    // A call's JSON input plus its RPC ID/name/envelope. This is a line
    // ceiling, not another allocation beside MCP's priced input buffers.
    let call_bytes = adapter.stated.call_input_bytes.checked_add(MCP_ENVELOPE_BYTES)?;
    // Each tool byte can become six JSON escape bytes in tools/list. Finish
    // and delivery schemas each have their own contract-sized encoded ceiling;
    // fixed room covers descriptions, names and the RPC/list envelope.
    let tool_bytes = u32::try_from(adapter.tool_bytes)
        .ok()?
        .checked_mul(JSON_ESCAPE_BYTES)?
        .checked_add(contract_bytes.checked_mul(CONTRACT_SCHEMAS)?)?
        .checked_add(MCP_ENVELOPE_BYTES)?;
    // A result byte needs at most six JSON escape bytes. Tool-list and result
    // are alternative messages in the same MCP output slot, hence max, not sum.
    let answer_bytes =
        adapter.rendered_result.checked_mul(JSON_ESCAPE_BYTES)?.checked_add(MCP_ENVELOPE_BYTES)?.max(tool_bytes);
    // Stream JSON wraps semantic answer text/arguments (six-byte escapes),
    // plus message headers, IDs, usage and block envelopes.
    let line_bytes = adapter.client.answer.checked_mul(JSON_ESCAPE_BYTES)?.checked_add(STREAM_ENVELOPE_BYTES)?;
    Some(backend::Settings {
        program: Programs { claude: config.program.claude.clone(), directory: config.program.directory.clone() },
        relay: config.relay.clone(),
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
        adapter,
        receiving: limits.llm.receiving,
        stream_tokens: backend::Settings::stream_tokens(line_bytes),
        prompt_bytes: u64::from(adapter.client.request),
        contract_bytes,
        line_bytes,
        // client.request already bounds encoded JSON, including escaping and
        // all admitted history. This user envelope omits the model's schemas.
        input_bytes: adapter.client.request.checked_add(STREAM_ENVELOPE_BYTES)?,
        max_turns: 1,
        grace: limits.io.close_timeout,
        startup: STARTUP,
    })
}

fn reserved(limits: &Limits, settings: &backend::Settings) -> Option<Limits> {
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
    reserved.routes = reserved.routes.max(operations.checked_add(super::ROUTE_HEADROOM)?);
    Some(reserved)
}

pub(crate) fn bound(config: Option<&ProgramBackend>) -> Option<&ProgramBackend> {
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
        && super::valid(&config.limits, &config.llm_endpoints)
        && super::valid(&reserved, &config.llm_endpoints)
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

#[expect(
    clippy::disallowed_methods,
    reason = "protocol/programs.md, section 5.1: the MCP command path must be JSON text"
)]
fn new(
    config: ProgramBackend,
    limits: &mut Limits,
    environment: &smith_protocol_machine::Environment,
) -> Result<Box<Program>, ConfigError> {
    let settings = settings(&config, limits, environment).ok_or(ConfigError::Limits)?;
    if config.entropy.len() != ENTROPY_BYTES
        || !config.relay.starts_with(b"/")
        || config.relay.len() > RELAY_PATH_BYTES
        || config.relay.contains(&0)
        || core::str::from_utf8(&config.relay).is_err()
        || settings.line_bytes == 0
    {
        return Err(ConfigError::Io);
    }
    let conversations = limits.domain.run.conversations;
    let mut children = List::with_capacity(conversations.checked_sub(1).ok_or(ConfigError::Limits)?);
    for _ in 1..conversations {
        children.push(None).expect("admitted child slots");
    }
    *limits = reserved(limits, &settings).ok_or(ConfigError::Limits)?;
    let process_slots = limits.machine.processes.checked_mul(3).ok_or(ConfigError::Limits)?;
    let minimum = limits
        .llm
        .connection
        .connections
        .checked_add(3)
        .ok_or(ConfigError::Limits)?
        .checked_add(process_slots)
        .ok_or(ConfigError::Limits)?
        .checked_add(IO_SLOTS.checked_mul(conversations).ok_or(ConfigError::Limits)?)
        .ok_or(ConfigError::Limits)?;
    if limits.io.sockets < minimum
        || limits.file_slots
            < limits
                .machine
                .roots
                .checked_add(FILE_SLOTS.checked_mul(conversations).ok_or(ConfigError::Limits)?)
                .ok_or(ConfigError::Limits)?
    {
        return Err(ConfigError::Io);
    }
    Ok(Box::new(Program {
        backend: backend::Backend::new(FIRST, settings, &config.entropy),
        configuration: config,
        environment: environment.clone(),
        directory: None,
        workspace: None,
        mounts: Box::default(),
        grant: None,
        turns: 1,
        time: Duration::ZERO,
        owner: None,
        children,
        below: Queue::uncapped(),
        pipes: Map::with_capacity(5_u32.checked_mul(conversations).ok_or(ConfigError::Limits)?),
    }))
}

/// Native construction shared with `Stages::parts`; private roots need the UID first.
#[derive(Debug)]
pub(super) struct Parts {
    pub program: Option<Box<Program>>,
    pub files: file_layer::FileIo,
}

/// Build and reserve only a program an endpoint names; private roots need its UID first.
pub(super) fn parts(
    config: Option<ProgramBackend>,
    limits: &mut Limits,
    environment: &smith_protocol_machine::Environment,
) -> Result<Parts, ConfigError> {
    let config = match bound(config.as_ref()) {
        Some(_) => config,
        None => None,
    };
    let effective_user = match &config {
        Some(program) => Some(program.effective_user),
        None => None,
    };
    let program = match config {
        Some(program) => Some(new(program, limits, environment)?),
        None => None,
    };
    let files = file_layer::FileIo::with_whole_limit(
        limits.file_slots,
        limits.file_read,
        limits.file_entries,
        limits.file_bytes,
        limits.file_timeout,
    );
    let files = match effective_user {
        Some(user) => files.with_effective_user(user),
        None => files,
    };
    Ok(Parts { program, files })
}

impl Service {
    /// Adopt the shell's private programs root, closing it with the backend (5.1).
    pub fn adopt_program_root(&mut self, descriptor: kernel::Fd) -> Result<(), AdoptError> {
        let program = self.stages.program.as_mut().ok_or(AdoptError::Unconfigured)?;
        let file = self.stages.files.adopt_root(descriptor).ok_or(AdoptError::Full)?;
        let root = backend::Root { file, descriptor };
        program.directory = Some(root);
        program.backend.directory(root);
        Ok(())
    }
}

pub(super) fn install(
    service: &mut Stages,
    start: &smith_protocol_translate::DecodedStart,
    name: domain::GrantName,
    value: &smith_protocol_translate::GrantValue,
    lapses: skein_lib::Time,
    workspace: Option<&domain::run::Workspace>,
) -> Result<(), ()> {
    match value {
        smith_protocol_translate::GrantValue::Program(source) => {
            let program = service.program.as_mut().ok_or(())?;
            if start.charter.llm.account != Some(name.account)
                || !program.configuration.endpoints.contains(&start.charter.llm.endpoint.0)
            {
                return Err(());
            }
            if let Some(file) = service.roots.get(0) {
                let descriptor = service.files.descriptor(*file).ok_or(())?;
                let root = backend::Root { file: *file, descriptor };
                program.workspace = Some(root);
                program.backend.workspace(root);
            }
            let mut mounts = List::with_capacity(service.limits.machine.roots);
            if let Some(workspace) = workspace {
                for directory in &workspace.directories {
                    if directory.host.len()
                        > usize::try_from(service.limits.domain.session.tools.host_path_bytes).or(Err(()))?
                    {
                        return Err(());
                    }
                    mounts
                        .push(backend::Mount {
                            root: directory.root,
                            host: directory.host.clone(),
                            writable: directory.writable,
                        })
                        .or(Err(()))?;
                }
            }
            program.mounts = mounts.into_boxed();
            program.backend.mounts(program.mounts.clone());
            program.turns = start.charter.budget.turns;
            program.time = start.charter.budget.time;
            program.backend.turn_limit(start.charter.budget.turns);
            program.backend.run_time(start.charter.budget.time);
            program.backend.grant(backend::Grant { name, source: source.clone(), lapses }).or(Err(()))?;
            program.grant = Some(backend::Grant { name, source: source.clone(), lapses });
            Ok(())
        }
        smith_protocol_translate::GrantValue::Bearer(_)
        | smith_protocol_translate::GrantValue::Key(_)
        | smith_protocol_translate::GrantValue::Withdrawn { .. } => {
            if let Some(program) = &service.program {
                for model in core::iter::once(&start.charter.llm).chain(start.charter.models.iter()) {
                    if model.account == Some(name.account)
                        && program.configuration.endpoints.contains(&model.endpoint.0)
                    {
                        return Err(());
                    }
                }
            }
            service.llm.grant(name, credential(value).ok_or(())?, lapses).or(Err(()))
        }
    }
}

pub(super) fn renew(service: &mut Stages, grant: domain::Grant, value: &smith_protocol_translate::GrantValue) -> bool {
    let program = match &mut service.program {
        Some(program) if program.backend.holds(grant.name) => program,
        Some(_) | None => return false,
    };
    let accepted = match value {
        smith_protocol_translate::GrantValue::Program(source) => {
            let lapses = service.domain_env.now.saturating_add(grant.valid);
            let mut accepted =
                program.backend.grant(backend::Grant { name: grant.name, source: source.clone(), lapses }).is_ok();
            for index in 0..program.children.len() {
                if let Some(child) = program.children.get_mut(index).expect("child slot") {
                    accepted &= child
                        .backend
                        .grant(backend::Grant { name: grant.name, source: source.clone(), lapses })
                        .is_ok();
                }
            }
            if accepted {
                program.grant = Some(backend::Grant { name: grant.name, source: source.clone(), lapses });
            }
            accepted
        }
        smith_protocol_translate::GrantValue::Bearer(_)
        | smith_protocol_translate::GrantValue::Key(_)
        | smith_protocol_translate::GrantValue::Withdrawn { .. } => false,
    };
    if accepted {
        service.domain_events.push(domain::Event::Grant { grant });
    } else {
        service.mark_failed(Failure::Credential);
    }
    true
}

fn failed(up: &mut Queue<llm::ToDomain>, owner: Token, detail: &[u8]) {
    up.push(llm::ToDomain::Failed {
        owner,
        failure: domain::llm::Failure::Program { exit: None },
        evidence: domain::llm::Evidence::Unknown,
        detail: detail.into(),
        report: domain::llm::Report::none(),
    });
}

impl Program {
    fn child_index(&self, owner: Token) -> Option<u32> {
        for index in 0..self.children.len() {
            if let Some(child) = self.children.get(index).expect("child slot")
                && child.owner == owner
            {
                return Some(index);
            }
        }
        None
    }

    fn io_index(&self, owner: Token) -> Option<u32> {
        for index in 0..self.children.len() {
            if let Some(child) = self.children.get(index).expect("child slot")
                && child.backend.owns(owner)
            {
                return Some(index);
            }
        }
        None
    }

    fn route_io(
        &mut self,
        env: &skein_lib::Env<llm::ComponentLimits>,
        owner: Token,
        event: backend::BelowEvent,
        up: &mut Queue<llm::ToDomain>,
    ) {
        match self.io_index(owner) {
            Some(index) => {
                let child = self.children.get_mut(index).expect("child slot").as_mut().expect("child backend");
                child.backend.from_io(env, event, up, &mut self.below);
            }
            None => self.backend.from_io(env, event, up, &mut self.below),
        }
    }

    fn close(&mut self, env: &skein_lib::Env<llm::ComponentLimits>, up: &mut Queue<llm::ToDomain>) {
        for index in 0..self.children.len() {
            if let Some(child) = self.children.get_mut(index).expect("child slot") {
                child.close(env, up, &mut self.below);
            }
        }
        self.backend.close(env, up, &mut self.below);
    }

    fn all_closed(&self) -> bool {
        if !self.backend.closed() {
            return false;
        }
        for child in self.children.iter().flatten() {
            if child.opening.is_some() || !child.backend.closed() {
                return false;
            }
        }
        true
    }
}

pub(super) fn complete(service: &mut Stages, request: llm::FromDomain) {
    let Some(program) = &mut service.program else {
        service.llm.from_domain(&service.llm_env, request, &mut service.llm_events, &mut service.llm_below);
        return;
    };
    let owner = match &request {
        llm::FromDomain::Complete { owner, grant, prompt, .. } => {
            if !match grant {
                Some(name) => program.backend.holds(*name),
                None => false,
            } {
                service.llm.from_domain(&service.llm_env, request, &mut service.llm_events, &mut service.llm_below);
                return;
            }
            match prompt.asker {
                domain::llm::Asker::Main => {
                    program.owner = Some(*owner);
                    program.backend.from_domain(&service.llm_env, request, &mut service.llm_events, &mut program.below);
                    return;
                }
                domain::llm::Asker::Child => {}
            }
            *owner
        }
        llm::FromDomain::Cancel { owner } | llm::FromDomain::Yield { owner } | llm::FromDomain::Close { owner } => {
            *owner
        }
    };
    if let Some(index) = program.child_index(owner) {
        let child = program.children.get_mut(index).expect("child slot").as_mut().expect("child owner");
        match request {
            llm::FromDomain::Close { .. } => child.close(&service.llm_env, &mut service.llm_events, &mut program.below),
            llm::FromDomain::Cancel { .. } if child.opening.is_some() => {
                // The directory terminal still owns the slot; do not start a cancelled completion.
                if !child.closing {
                    service.llm_events.push(llm::ToDomain::Cancelled { owner });
                }
                child.closing = true;
            }
            llm::FromDomain::Yield { .. } if child.opening.is_some() => child.yielding = true,
            request @ (llm::FromDomain::Complete { .. }
            | llm::FromDomain::Cancel { .. }
            | llm::FromDomain::Yield { .. }) => {
                child.backend.from_domain(&service.llm_env, request, &mut service.llm_events, &mut program.below);
            }
        }
        return;
    }
    match request {
        request @ llm::FromDomain::Complete { .. } => {
            start_child(program, &service.limits, &service.llm_env, owner, request, &mut service.llm_events);
        }
        llm::FromDomain::Close { .. } => {}
        request @ (llm::FromDomain::Cancel { .. } | llm::FromDomain::Yield { .. }) => {
            if program.owner == Some(owner) {
                program.backend.from_domain(&service.llm_env, request, &mut service.llm_events, &mut program.below);
            } else {
                service.llm.from_domain(&service.llm_env, request, &mut service.llm_events, &mut service.llm_below);
            }
        }
    }
}

fn start_child(
    program: &mut Program,
    limits: &Limits,
    env: &skein_lib::Env<llm::ComponentLimits>,
    owner: Token,
    request: llm::FromDomain,
    up: &mut Queue<llm::ToDomain>,
) {
    let mut free = None;
    for index in 0..program.children.len() {
        if program.children.get(index).expect("child slot").is_none() && free.is_none() {
            free = Some(index);
        }
    }
    let Some(index) = free else {
        failed(up, owner, b"smith: program child could not start: conversations - 1 backend slots are still occupied");
        return;
    };
    let Some(mut settings) = settings(&program.configuration, limits, &program.environment) else {
        unreachable!("admitted program settings");
    };
    settings.workspace = program.workspace;
    settings.max_turns = program.turns;
    let first = Token::new(
        FIRST
            .raw()
            .checked_add(
                u64::from(index).checked_add(1).expect("child index").checked_mul(STRIDE).expect("child stride"),
            )
            .expect("child range"),
    );
    let mut hash = sha256::Sha256::new();
    hash.update(program.configuration.entropy.expose());
    hash.update(b"program-child");
    hash.update(&owner.raw().to_be_bytes());
    let entropy = Secret::new(Box::from(hash.finish().as_slice()));
    let mut backend = backend::Backend::new(first, settings, &entropy);
    backend.mounts(program.mounts.clone());
    backend.run_time(program.time);
    let grant = program.grant.as_ref().expect("installed program grant");
    backend
        .grant(backend::Grant { name: grant.name, source: grant.source.clone(), lapses: grant.lapses })
        .expect("same program grant");
    let child = Child { owner, first, backend, opening: Some(request), closing: false, yielding: false };
    program.below.push(backend::Below::File {
        request: file::Request::OpenDirectory {
            owner: child.directory_owner(),
            root: program.directory.expect("adopted program root").descriptor,
            name: b".".as_slice().into(),
            no_follow: true,
        },
        deadline: env.now.saturating_add(STARTUP),
    });
    *program.children.get_mut(index).expect("child slot") = Some(child);
}

pub(super) fn pending_measures(service: &Stages) -> u32 {
    let mut count = 0_u32;
    if let Some(program) = &service.program {
        count = program.backend.pending_measures();
        for child in program.children.iter().flatten() {
            count = count.checked_add(child.backend.pending_measures()).expect("admitted program measures");
        }
    }
    count
}

pub(super) fn pop_measure(service: &mut Stages) -> Option<llm::Measure> {
    let program = service.program.as_mut()?;
    if let Some(measure) = program.backend.pop_measure() {
        return Some(measure);
    }
    for index in 0..program.children.len() {
        if let Some(child) = program.children.get_mut(index).expect("child slot")
            && let Some(measure) = child.backend.pop_measure()
        {
            return Some(measure);
        }
    }
    None
}

pub(super) fn backlog(service: &Stages) -> u32 {
    match &service.program {
        Some(program) => program.below.len(),
        None => 0,
    }
}

pub(super) fn fire(service: &mut Stages) {
    if let Some(program) = &mut service.program {
        match service.teardown {
            super::Teardown::Live => {}
            super::Teardown::Answered { .. } | super::Teardown::Closing { .. } | super::Teardown::Closed { .. } => {
                program.close(&service.llm_env, &mut service.llm_events);
            }
        }
        program.backend.fire(&service.llm_env, &mut service.llm_events, &mut program.below);
        for index in 0..program.children.len() {
            if let Some(child) = program.children.get_mut(index).expect("child slot") {
                child.backend.fire(&service.llm_env, &mut service.llm_events, &mut program.below);
                if child.opening.is_none() && child.backend.closed() && child.backend.pending_measures() == 0 {
                    *program.children.get_mut(index).expect("closed child slot") = None;
                }
            }
        }
    }
}

pub(super) fn deadline(service: &Stages) -> Option<skein_lib::Time> {
    let mut deadline = None;
    if let Some(program) = &service.program {
        deadline = program.backend.next_deadline();
        for child in program.children.iter().flatten() {
            if let Some(child_deadline) = child.backend.next_deadline() {
                deadline = Some(match deadline {
                    Some(at) => at.min(child_deadline),
                    None => child_deadline,
                });
            }
        }
    }
    deadline
}

// After the domain's terminal, program deadlines only settle owned I/O
// (child close grace and MCP sockets); they do not await policy work.
pub(super) fn policy_deadline(service: &Stages) -> Option<skein_lib::Time> {
    match service.teardown {
        super::Teardown::Live if answer_head(service) => None,
        super::Teardown::Live => deadline(service),
        super::Teardown::Answered { .. } | super::Teardown::Closing { .. } | super::Teardown::Closed { .. } => None,
    }
}

pub(super) fn closed(service: &Stages) -> bool {
    match &service.program {
        Some(program) => program.all_closed() && program.below.is_empty() && program.pipes.is_empty(),
        None => true,
    }
}

pub(super) fn pending(service: &Stages) -> bool {
    match &service.program {
        Some(program) => !program.below.is_empty(),
        None => false,
    }
}

pub(super) fn file_event(service: &mut Stages, event: file::Event) -> Option<file::Event> {
    let descriptor = match &event {
        file::Event::Opened { file, .. } => service.files.descriptor(*file),
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Closed { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Failed { .. }
        | file::Event::Cancelled { .. } => None,
    };
    let Some(program) = &mut service.program else { return Some(event) };
    let owner = event.owner();
    for index in 0..program.children.len() {
        let opening = match program.children.get(index).expect("child slot") {
            Some(child) => child.opening.is_some() && child.directory_owner() == owner,
            None => false,
        };
        if opening {
            let child = program.children.get_mut(index).expect("child slot").as_mut().expect("opening child");
            let request = child.opening.take().expect("pending first completion");
            match &event {
                file::Event::Opened { file, .. } => {
                    child.opened(
                        &service.llm_env,
                        backend::Root { file: *file, descriptor: descriptor.expect("opened directory") },
                        request,
                        &mut service.llm_events,
                        &mut program.below,
                    );
                }
                file::Event::Locked { .. }
                | file::Event::Busy { .. }
                | file::Event::Made { .. }
                | file::Event::Refused { .. }
                | file::Event::Loaded { .. }
                | file::Event::Scanned { .. }
                | file::Event::Stored { .. }
                | file::Event::Conflict { .. }
                | file::Event::TooLarge { .. }
                | file::Event::Stated { .. }
                | file::Event::Written { .. }
                | file::Event::Read { .. }
                | file::Event::Cut { .. }
                | file::Event::Synced { .. }
                | file::Event::Closed { .. }
                | file::Event::Renamed { .. }
                | file::Event::Removed { .. }
                | file::Event::Listed { .. }
                | file::Event::Failed { .. }
                | file::Event::Cancelled { .. } => {
                    if !child.closing {
                        failed(&mut service.llm_events, child.owner, b"smith: child program directory could not open");
                    }
                    child.close(&service.llm_env, &mut service.llm_events, &mut program.below);
                }
            }
            return None;
        }
    }
    if program.backend.owns(owner) || program.io_index(owner).is_some() {
        program.route_io(
            &service.llm_env,
            owner,
            backend::BelowEvent::File { event, descriptor },
            &mut service.llm_events,
        );
        None
    } else {
        Some(event)
    }
}

pub(super) fn pipe_event(service: &mut Stages, event: io::Event) -> Option<io::Event> {
    let owner = event.owner()?;
    let Some(program) = service.program.as_mut() else { return Some(event) };
    let Some(process_owner) = program.pipes.get(&owner).copied() else { return Some(event) };
    if event.is_closed() {
        program.pipes.remove(&owner);
    }
    program.route_io(&service.llm_env, process_owner, backend::BelowEvent::Io(event), &mut service.llm_events);
    None
}

pub(super) fn io_event(service: &mut Stages, event: io::Event) {
    let program = service.program.as_mut().expect("program route retains its backend");
    let owner = event.owner().expect("routed program event");
    match &event {
        io::Event::Spawned { pipes, .. } => {
            for pipe in pipes {
                program.pipes.insert(*pipe, owner).expect("five pipes per admitted program");
            }
        }
        io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Stream { .. }
        | io::Event::Output { .. }
        | io::Event::Exited { .. }
        | io::Event::Usage { .. }
        | io::Event::Window { .. }
        | io::Event::Failed { .. }
        | io::Event::Closed { .. }
        | io::Event::Resized
        | io::Event::Shutdown { .. } => {}
    }
    program.route_io(&service.llm_env, owner, backend::BelowEvent::Io(event), &mut service.llm_events);
}

pub(super) fn down(service: &mut Stages) {
    let Some(program) = &mut service.program else { return };
    for _ in 0..service.limits.queue {
        let waits_for_file = match program.below.iter().next() {
            Some(backend::Below::File { .. }) => !service.files.takes(),
            Some(backend::Below::Io(_)) | None => false,
        };
        if waits_for_file {
            break;
        }
        let Some(request) = program.below.pop() else { break };
        match request {
            backend::Below::File { request, deadline } => {
                service.files.seed_randomness(service.next_operation);
                file_layer::down_until(
                    &mut service.files,
                    deadline,
                    request,
                    &mut service.file_events,
                    &mut service.file_submissions,
                );
            }
            backend::Below::Io(request) => {
                let request = match request {
                    io::Request::Listen { owner, addr } => {
                        let route = service.owners.insert(Owner::Program(owner)).expect("program listener route");
                        io::Request::Listen { owner: component_owner(route.token()), addr }
                    }
                    io::Request::Bind { socket, owner } => {
                        let route = service.owners.insert(Owner::Program(owner)).expect("program MCP connection route");
                        io::Request::Bind { socket, owner: component_owner(route.token()) }
                    }
                    io::Request::Spawn { owner, spawn } => {
                        let route = service.owners.insert(Owner::Program(owner)).expect("program process route");
                        io::Request::Spawn { owner: component_owner(route.token()), spawn }
                    }
                    other @ (io::Request::Connect { .. }
                    | io::Request::Reject { .. }
                    | io::Request::Stream { .. }
                    | io::Request::Output { .. }
                    | io::Request::Signal { .. }
                    | io::Request::Window { .. }
                    | io::Request::Usage { .. }
                    | io::Request::Close { .. }
                    | io::Request::Abort { .. }) => other,
                };
                service.io_requests.push(request);
            }
        }
    }
}

// The queued Answer already owns and prices its payload. Hold that request
// until the program has acknowledged finish and settled its child/files.
fn answer_head(service: &Stages) -> bool {
    match service.domain_requests.iter().next() {
        Some(domain::Request::Answer { .. }) => true,
        Some(
            domain::Request::Lower(
                smith_domain::Lower::Complete { .. }
                | smith_domain::Lower::Cancel { .. }
                | smith_domain::Lower::Yield { .. }
                | smith_domain::Lower::Close { .. }
                | smith_domain::Lower::Io { .. }
                | smith_domain::Lower::Job { .. }
                | smith_domain::Lower::CancelIo { .. }
                | smith_domain::Lower::Read { .. }
                | smith_domain::Lower::Probe { .. }
                | smith_domain::Lower::Check { .. }
                | smith_domain::Lower::Abort { .. },
            )
            | domain::Request::MessageRefused { .. }
            | domain::Request::Waiting { .. }
            | domain::Request::Turn { .. }
            | domain::Request::HostCall { .. }
            | domain::Request::WithdrawHost { .. }
            | domain::Request::Admitted { .. }
            | domain::Request::Long { .. }
            | domain::Request::LongDone { .. }
            | domain::Request::Deliver { .. }
            | domain::Request::Unauthorized { .. }
            | domain::Request::Refused { .. }
            | domain::Request::Limited { .. },
        )
        | None => false,
    }
}

pub(super) fn requests_ready(service: &Stages) -> bool {
    if !answer_head(service) {
        return true;
    }
    match &service.program {
        Some(program) => program.all_closed(),
        None => true,
    }
}

pub(super) fn settled(service: &mut Stages, turn: &domain::Turn) {
    if let Some(program) = &mut service.program {
        program.backend.settled(&turn.messages, &mut program.below);
        for index in 0..program.children.len() {
            if let Some(child) = program.children.get_mut(index).expect("child slot") {
                child.backend.settled(&turn.messages, &mut program.below);
            }
        }
    }
}

pub(super) fn wait_for_close(service: &mut Stages) -> bool {
    if !answer_head(service) {
        return false;
    }
    match &mut service.program {
        Some(program) if !program.all_closed() => {
            program.close(&service.llm_env, &mut service.llm_events);
            !program.all_closed()
        }
        Some(_) | None => false,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod validity_tests;
