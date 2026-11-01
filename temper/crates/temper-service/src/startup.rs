//! The document's shape and its checked startup (protocol/configuration.md, 2).
pub(crate) mod application;
#[cfg(test)]
mod calls_tests;
pub(crate) mod checks;
mod problems;
#[cfg(test)]
#[path = "tests/welcome_policy.rs"]
mod welcome_policy;
use crate::{AccountsRuntime, Service, Worker, WorkerRuntime, settings};
use alloc::boxed::Box;
pub use application::sign_in::{Endpoints, SignIn, url as sign_in_url};
pub use checks::redirect_ports;
use jig_protocol_config::{self as config, Path, Segment, read};
pub use problems::{Problem, Problems, What, lines};
use skein_lib::List;

/// The services selected by the document's members (protocol/README.md, 15.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Standalone,
    EngineAlone,
    WorkerAlone,
}
fn member(document: &skein_json::Document, at: u32, name: &[u8]) -> Option<u32> {
    read::Object::new(document, at).ok()?.member(document, name)
}
fn nearest(document: &skein_json::Document) -> Shape {
    let Some(engine) = member(document, 0, b"engine") else { return Shape::WorkerAlone };
    let Some(hosts) = member(document, engine, b"hosts") else { return Shape::Standalone };
    let Some(at) = member(document, hosts, b"listener") else { return Shape::Standalone };
    let Ok(listeners) = read::list(document, at, document.len()) else { return Shape::Standalone };
    let Some(first) = listeners.get(0) else { return Shape::Standalone };
    if member(document, *first, b"socket").is_some() { Shape::EngineAlone } else { Shape::Standalone }
}
/// Read and check the shape before opening any named file.
pub fn shape_with(
    input: &[u8],
    directories: &config::Directories,
    given: &crate::stock::Given,
) -> Result<Shape, Problems> {
    Ok(load_with(input, directories, true, given)?.0)
}
pub fn shape(input: &[u8], directories: &config::Directories) -> Result<Shape, Problems> {
    shape_with(input, directories, &crate::stock::Given::Stock)
}
fn finish_shape(document: &skein_json::Document, object: read::Object, path: &Path, problems: &mut Problems) {
    let mut found = config::Problems::empty();
    object.finish(document, path, &mut found);
    for found in &found.problems {
        problems.push(
            found.path.clone(),
            if found.what == config::What::Unknown { What::WorkerAlone } else { What::Jig(found.what) },
        );
    }
    problems.past = problems.past.saturating_add(found.past);
}
/// A forge API token's bound, until the forge component supplies it.
pub const API_TOKEN_BYTES: u32 = 4096;
/// A provider's fixed sign-in redirect, with its diagnostic path; no credentials.
#[derive(Debug)]
pub struct RedirectPort {
    pub port: u16,
    pub path: Path,
    pub redirect_path: Box<[u8]>,
}
/// The standalone listeners whose fixed ports a redirect must avoid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Listener {
    Clients,
    Webhooks,
    Owned(u32),
    Fake(u32),
    Demo,
}
fn redirects(engine: &config::Engine) -> Box<[RedirectPort]> {
    let mut redirects = List::with_capacity(u32::try_from(engine.llm.providers.len()).expect("bounded providers"));
    for (index, provider) in engine.llm.providers.iter().enumerate() {
        if let Some(registration) = &provider.sign_in
            && let Some(port) = registration.redirect_port
        {
            redirects
                .push(RedirectPort {
                    port,
                    path: Path::root()
                        .member(b"engine")
                        .member(b"llm")
                        .member(b"providers")
                        .child(Segment::Index(u32::try_from(index).expect("bounded providers")))
                        .member(b"sources")
                        .member(b"sign_in")
                        .member(b"redirect_port"),
                    redirect_path: registration.redirect_path.clone(),
                })
                .expect("one redirect per provider");
        }
    }
    redirects.into_boxed()
}
fn check_redirects(engine: &config::Engine, application: &application::Members, problems: &mut Problems) {
    let redirects = redirects(engine);
    redirect_ports(&redirects, checks::port(&application.clients.listener), Listener::Clients, problems);
    let port = match &application.webhooks {
        Some(hooks) => checks::port(&hooks.listener),
        None => None,
    };
    redirect_ports(&redirects, port, Listener::Webhooks, problems);
}
/// The files and directories the binary reads and opens before startup.
#[derive(Debug)]
pub struct Named {
    pub files: Box<[config::File]>,
    pub redirects: Box<[RedirectPort]>,
    pub deployment: Box<[u8]>,
    pub llm: config::Llm,
    pub projects: Box<[config::Project]>,
    pub worker: Option<NamedWorker>,
    pub machine: temper_engine_domain_forge::Machine,
    pub clients: config::Address,
    pub sign_in: Option<SignIn>,
    /// `application.clients.bundle`, for the binary's read.
    pub bundle: Option<Box<[u8]>>,
    /// The bundle's limits, overrides applied.
    pub bundle_limits: crate::limits::Clients,
    pub webhooks: Option<application::Webhooks>,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
}
#[derive(Debug)]
pub struct NamedWorker {
    pub root: Box<[u8]>,
    pub agent: Box<[u8]>,
}
/// The limits serve opens the deployment's store at, for a reader of a copy of it.
#[derive(Debug)]
pub struct StoreLimits {
    pub store: jig_protocol_store::Limits,
    pub forge: temper_engine_domain_forge::Limits,
}
/// Derive the store's opening limits without reading named files or checking runtime facts.
pub fn store_limits(
    document: &[u8],
    directories: &config::Directories,
    agent: Option<&settings::Agent>,
) -> Result<StoreLimits, Problems> {
    let mut parsed =
        load(document, directories, false)?.1.ok_or(problem(Path::root().member(b"engine"), What::Standalone))?;
    if !parsed.workers.is_empty() {
        let agent = agent.ok_or(problem(indexed(0).member(b"agent"), What::Agent))?;
        parsed.compose(agent)?;
        let mut problems = Problems::empty();
        checks::agent_answer(agent, &mut problems);
        if !problems.is_empty() {
            return Err(problems);
        }
    }
    let root = engine_limits(&mut parsed.engine.limits, &parsed.application.limits, parsed.engine.memory)?;
    let store = checked_store(&root, &parsed.engine.limits.store, None)?;
    Ok(StoreLimits { store, forge: root.forge })
}

fn checked_store(
    root: &temper_engine_domain::Limits,
    profile: &jig_protocol_store::Limits,
    turn: Option<u32>,
) -> Result<jig_protocol_store::Limits, Problems> {
    match crate::store::admit(root, &root.forge, profile, turn) {
        Ok(store) => Ok(store),
        Err(why) => Err(checks::store_refusal(why)),
    }
}

fn engine_limits(
    jig: &mut config::Limits,
    own: &crate::limits::Limits,
    memory_bytes: u64,
) -> Result<temper_engine_domain::Limits, Problems> {
    check_memory_bytes(memory_bytes)?;
    jig.clients.wire.keyed_value_some = crate::limits::client_value_bytes(jig, own);
    let mut root = crate::limits::engine_candidate(jig, own);
    root.memory_bytes = memory_bytes;
    let mut problems = Problems::empty();
    checks::call_answer(Path::root().member(b"engine"), jig.core.call_answer_bytes, &mut problems);
    checks::engine_envelope(&root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    if !temper_engine_domain::valid(&root) {
        return Err(problem(Path::root().member(b"engine").member(b"limits"), What::Jig(config::What::Bound)));
    }
    Ok(root)
}
fn check_memory_bytes(memory_bytes: u64) -> Result<(), Problems> {
    if memory_bytes == 0 {
        return Err(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Jig(config::What::Bound)));
    }
    Ok(())
}
#[derive(Debug)]
struct Parsed {
    engine: config::Engine,
    workers: Box<[config::Worker]>,
    application: application::Members,
    engine_call_answer: CallAnswer,
    worker_call_answers: Box<[CallAnswer]>,
}
impl Parsed {
    fn compose(&mut self, agent: &settings::Agent) -> Result<(), Problems> {
        let mut problems = Problems::empty();
        if let Err(what) = self.engine_call_answer.compose(&mut self.engine.limits, agent) {
            problems.push(call_input_path(Path::root().member(b"engine")), what);
        }
        checks::call_answer(Path::root().member(b"engine"), self.engine.limits.core.call_answer_bytes, &mut problems);
        for (index, worker) in self.workers.iter_mut().enumerate() {
            if let Err(what) = self
                .worker_call_answers
                .get(index)
                .expect("one provenance per parsed worker")
                .compose(&mut worker.limits, agent)
            {
                problems.push(call_input_path(indexed(index)), what);
            }
            checks::call_answer(indexed(index), worker.limits.core.call_answer_bytes, &mut problems);
        }
        if problems.is_empty() { Ok(()) } else { Err(problems) }
    }
}
/// Raw membership preserves an explicit profile-default override (protocol/configuration.md, 7).
#[derive(Clone, Copy, Debug)]
enum CallAnswer {
    Profile,
    Explicit,
}
impl CallAnswer {
    fn read(document: &skein_json::Document, at: u32) -> Self {
        let Some(limits) = member(document, at, b"limits") else { return Self::Profile };
        let Some(overrides) = member(document, limits, b"overrides") else { return Self::Profile };
        if member(document, overrides, b"core.call_answer_bytes").is_some() { Self::Explicit } else { Self::Profile }
    }
    fn compose(self, limits: &mut config::Limits, agent: &settings::Agent) -> Result<(), What> {
        match self {
            Self::Profile => limits.core.call_answer_bytes = limits.core.call_answer_bytes.max(agent.tool_payload),
            Self::Explicit => {
                if limits.core.call_answer_bytes < agent.tool_payload {
                    return Err(What::CallInput {
                        bytes: u64::from(limits.core.call_answer_bytes),
                        input: u64::from(agent.tool_payload),
                        ask: u64::from(agent.limits.bodies.host_ask_input),
                        reply: u64::from(agent.limits.bodies.host_reply_text),
                    });
                }
            }
        }
        Ok(())
    }
}
fn call_input_path(path: Path) -> Path {
    path.member(b"limits").member(b"overrides").member(b"core.call_answer_bytes")
}
fn problem(path: Path, what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}
fn indexed(index: usize) -> Path {
    Path::root().member(b"workers").child(Segment::Index(u32::try_from(index).expect("bounded workers")))
}
/// Parse jig's engine member and pass the supplied defaults, refusing document
/// settings members (protocol/configuration.md, 13.5 and 13.6).
/// # Errors
/// The document's problems at its paths under `engine`.
pub fn engine_with(
    document: &skein_json::Document,
    at: u32,
    directories: &config::Directories,
    given: &crate::stock::Given,
) -> Result<config::Engine, config::Problems> {
    let mut engine = config::engine(document, at, directories, config::Defaults::Passed)?;
    if engine.llm.models.is_empty() {
        let mut problems = config::Problems::empty();
        problems.push(Path::root().member(b"llm").member(b"models"), config::What::Bound);
        return Err(problems);
    }
    let Some(defaults) = given.defaults(&engine.llm) else {
        let mut problems = config::Problems::empty();
        problems.push(
            Path::root().member(b"llm").member(b"models").child(Segment::Index(0)).member(b"prices"),
            config::What::Bound,
        );
        return Err(problems);
    };
    config::pass(&mut engine, defaults)?;
    Ok(engine)
}
pub fn engine(
    document: &skein_json::Document,
    at: u32,
    directories: &config::Directories,
) -> Result<config::Engine, config::Problems> {
    engine_with(document, at, directories, &crate::stock::Given::Stock)
}
fn parse_with(
    input: &[u8],
    directories: &config::Directories,
    given: &crate::stock::Given,
) -> Result<Parsed, Problems> {
    load_with(input, directories, true, given)?.1.ok_or(problem(Path::root().member(b"engine"), What::Standalone))
}

#[cfg(test)]
fn parse(input: &[u8], directories: &config::Directories) -> Result<Parsed, Problems> {
    parse_with(input, directories, &crate::stock::Given::Stock)
}

// Readers need the typed document alone. Startup also checks cross-part and
// raw-path constraints here, preserving the order of its accumulated problems.
fn load_with(
    input: &[u8],
    directories: &config::Directories,
    check_start: bool,
    given: &crate::stock::Given,
) -> Result<(Shape, Option<Parsed>), Problems> {
    let length = u32::try_from(input.len()).or(Err(problem(Path::root(), What::Jig(config::What::Bound))))?;
    let document =
        match config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: length, depth: 32 }) {
            Ok(document) => document,
            Err(found) => return Err(problem(found.path, What::Jig(found.what))),
        };
    load_document(&document, directories, check_start, given)
}
// Temper seeds name settings; jig also serves callers that accept inline policies.
fn inline_seed_policies(document: &skein_json::Document, engine: u32, problems: &mut Problems) {
    let Some(projects) = member(document, engine, b"projects") else { return };
    let Ok(projects) = read::list(document, projects, document.len()) else { return };
    for (index, project) in projects.iter().enumerate() {
        if let Some(policy) = member(document, *project, b"policy")
            && document.token(policy).expect("policy token").kind == skein_json::document::Kind::ObjectStart
        {
            problems.push(
                Path::root()
                    .member(b"engine")
                    .member(b"projects")
                    .child(Segment::Index(u32::try_from(index).expect("bounded projects")))
                    .member(b"policy"),
                What::InlineSeedPolicy,
            );
        }
    }
}
#[expect(clippy::too_many_lines, reason = "one load pass accumulates all cross-part diagnostics")]
fn load_document(
    document: &skein_json::Document,
    directories: &config::Directories,
    check_start: bool,
    given: &crate::stock::Given,
) -> Result<(Shape, Option<Parsed>), Problems> {
    let shape = nearest(document);
    let mut problems = Problems::empty();
    let mut found = config::Problems::empty();
    let mut whole = match read::Object::new(document, 0) {
        Ok(object) => object,
        Err(what) => return Err(problem(Path::root(), What::Jig(what))),
    };
    let root = Path::root();
    // Keep the empty object's established missing-member diagnostics.
    let engine_at = if document.len() == 2 {
        whole.required(document, b"engine", &root, &mut found)
    } else {
        whole.member(document, b"engine")
    };
    let worker_at = whole.required(document, b"workers", &root, &mut found);
    let application_at = whole.required(document, b"application", &root, &mut found);
    if shape == Shape::WorkerAlone {
        finish_shape(document, whole, &root, &mut problems);
    } else {
        whole.finish(document, &root, &mut found);
    }
    problems.append(found, &root);
    if let Some(at) = engine_at {
        let before = problems.problems.len();
        inline_seed_policies(document, at, &mut problems);
        if problems.problems.len() != before {
            return Err(problems);
        }
    }
    let engine = match engine_at {
        Some(at) => match engine_with(document, at, directories, given) {
            Ok(mut engine) => {
                engine.limits.clients.watches_per_stream = temper_engine_domain::WATCHES_PER_STREAM;
                Some(engine)
            }
            Err(found) => {
                problems.append(found, &root.member(b"engine"));
                None
            }
        },
        None => None,
    };
    let ats = match worker_at {
        Some(at) => match read::list(document, at, document.len()) {
            Ok(ats) => ats,
            Err(what) => {
                problems.push(root.member(b"workers"), What::Jig(what));
                List::with_capacity(0)
            }
        },
        None => List::with_capacity(0),
    };
    let mut workers = List::with_capacity(ats.len());
    let mut worker_call_answers = List::with_capacity(ats.len());
    for (index, at) in ats.iter().enumerate() {
        if shape == Shape::EngineAlone {
            continue;
        }
        if check_start {
            checks::raw_worker(document, *at, &indexed(index), &mut problems);
        }
        match config::worker(document, *at, directories) {
            Ok(worker) => {
                workers.push(worker).expect("bounded workers");
                worker_call_answers.push(CallAnswer::read(document, *at)).expect("bounded workers");
            }
            Err(found) => problems.append(found, &indexed(index)),
        }
    }
    if shape == Shape::WorkerAlone {
        if worker_at.is_some() && ats.len() != 1 {
            problems.push(root.member(b"workers"), What::WorkerAlone);
        }
        for (index, worker) in workers.iter().enumerate() {
            if !checks::loopback(&worker.engine) {
                problems.push(indexed(index).member(b"engine"), What::WorkerAlone);
            }
        }
        if let Some(at) = application_at {
            let _part = application::worker_alone(document, at, directories, &mut problems);
        }
        return if problems.is_empty() { Ok((shape, None)) } else { Err(problems) };
    }
    if check_start && let Some(engine) = &engine {
        match given {
            crate::stock::Given::Stock => checks::stock(engine, &mut problems),
            crate::stock::Given::Own(_) => {}
        }
    }
    let application = match &engine {
        Some(engine) => match application_at {
            Some(at) => application::parse(document, at, engine, directories, &mut problems),
            None => None,
        },
        None => None,
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    let application = application.expect("required application checked");
    let workers = workers.into_boxed();
    let worker_call_answers = worker_call_answers.into_boxed();
    let engine_call_answer = CallAnswer::read(document, engine_at.expect("required engine checked"));
    if !check_start {
        return Ok((
            shape,
            Some(Parsed {
                engine: engine.expect("required engine checked"),
                workers,
                application,
                engine_call_answer,
                worker_call_answers,
            }),
        ));
    }
    if shape == Shape::EngineAlone && !ats.is_empty() {
        problems.push(root.member(b"workers"), What::EngineAlone);
    }
    checks::standalone(
        engine.as_ref().expect("required engine checked"),
        &application.clients,
        &workers,
        &mut problems,
    );
    if shape == Shape::EngineAlone && !application.workers.is_empty() {
        problems.push(root.member(b"application").member(b"workers"), What::EngineAlone);
    } else if workers.len() != application.workers.len() {
        problems.push(root.member(b"application").member(b"workers"), What::Jig(config::What::Bound));
    }
    checks::hub(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::provider(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::credentials(
        engine.as_ref().expect("required engine checked"),
        &workers,
        &application.machine,
        &mut problems,
    );
    checks::webhooks(&application, &mut problems);
    check_redirects(engine.as_ref().expect("required engine checked"), &application, &mut problems);
    checks::records(
        engine.as_ref().expect("required engine checked"),
        &application.limits,
        &application.machine,
        &mut problems,
    );
    checks::limits(&application.limits, &application.limits.git_component, &mut problems);
    checks::names(checks::longest_remote(&application.machine, application.limits.forge.name_bytes), &application.limits, &mut problems);
    if let Some(worker) = workers.first() {
        checks::sockets(
            engine.as_ref().expect("required engine checked"),
            worker,
            &application.limits.git_component,
            &application,
            &mut problems,
        );
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok((
        shape,
        Some(Parsed {
            engine: engine.expect("required engine checked"),
            workers,
            application,
            engine_call_answer,
            worker_call_answers,
        }),
    ))
}

fn load(
    input: &[u8],
    directories: &config::Directories,
    check_start: bool,
) -> Result<(Shape, Option<Parsed>), Problems> {
    load_with(input, directories, check_start, &crate::stock::Given::Stock)
}
fn all_files(parsed: &Parsed) -> Box<[config::File]> {
    let engine = config::files(&parsed.engine);
    let worker = match parsed.workers.first() {
        Some(worker) => config::worker_files(worker),
        None => Box::new([]),
    };
    let application = application::files(&parsed.application);
    let count = engine
        .len()
        .checked_add(worker.len())
        .expect("bounded files")
        .checked_add(application.len())
        .expect("bounded files");
    let mut files = List::with_capacity(u32::try_from(count).expect("bounded file count"));
    for (named, prefix) in [(engine, Path::root().member(b"engine")), (worker, indexed(0)), (application, Path::root())]
    {
        for mut file in named {
            let mut path = prefix.clone();
            for segment in &file.at.segments {
                path = path.child(segment.clone());
            }
            file.at = path;
            files.push(file).expect("capacity includes every named file");
        }
    }
    files.into_boxed()
}
/// Parse every member before the binary reads any named file.
pub fn named_with(
    document: &[u8],
    directories: &config::Directories,
    given: &crate::stock::Given,
) -> Result<Named, Problems> {
    let parsed = parse_with(document, directories, given)?;
    check_memory_bytes(parsed.engine.memory)?;
    let files = all_files(&parsed);
    Ok(Named {
        files,
        redirects: redirects(&parsed.engine),
        clients: parsed.application.clients.listener,
        sign_in: parsed.application.sign_in,
        bundle: parsed.application.clients.bundle,
        bundle_limits: parsed.application.limits.clients,
        webhooks: parsed.application.webhooks,
        machine: parsed.application.machine,
        overrides: parsed.application.overrides,
        deployment: parsed.engine.deployment.directory,
        llm: parsed.engine.llm,
        projects: parsed.engine.projects,
        worker: match parsed.workers.first() {
            Some(worker) => Some(NamedWorker {
                root: parsed.application.workers.first().expect("one application worker checked").root.clone(),
                agent: worker.agent.configuration.clone(),
            }),
            None => None,
        },
    })
}
pub fn named(document: &[u8], directories: &config::Directories) -> Result<Named, Problems> {
    named_with(document, directories, &crate::stock::Given::Stock)
}
fn check_files(parsed: &Parsed, files: &settings::Files) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    for file in all_files(parsed) {
        match files.contents(&file.path) {
            Some(contents) if contents.len() <= usize::try_from(file.max).expect("file bound") => {}
            Some(_) => problems.push(file.at, What::Jig(config::What::Bound)),
            None => problems.push(file.at, What::Jig(config::What::Unreadable)),
        }
    }
    if problems.is_empty() { Ok(()) } else { Err(problems) }
}
struct CheckedStart {
    engine: settings::Engine,
    worker: Option<settings::Worker>,
    io: skein_io::Limits,
    forges: skein_forge_connection::Settings,
}
/// Every check startup makes, without constructing a service.
/// # Errors
/// The problems startup would refuse with, in its order.
#[expect(clippy::too_many_arguments, reason = "the same inputs as startup, including its checked setup")]
pub fn check(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    _accounts: Option<&AccountsRuntime>,
    _setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<(), Problems> {
    if shape(document, directories)? == Shape::WorkerAlone {
        let mut part = worker_part(document, directories)?;
        check_worker_part(&mut part, files, agent)?;
        return Ok(());
    }
    checked_start(document, directories, files, Some(agent), seeds, _setup)?;
    Ok(())
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one checked startup pass preserves diagnostic order beside resolved setup and runtime facts"
)]
fn checked_start_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    _setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
) -> Result<CheckedStart, Problems> {
    let mut parsed = parse_with(document, directories, given)?;
    let agent = match parsed.workers.first() {
        Some(_) => Some(agent.ok_or(problem(indexed(0).member(b"agent"), What::Agent))?),
        None => None,
    };
    if let Some(agent) = agent {
        parsed.compose(agent)?;
    }
    check_files(&parsed, files)?;
    let mut problems = Problems::empty();
    let resolved = application::resolve(&parsed.application, files, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let resolved = resolved.expect("resolved files checked");
    let own = &parsed.application.limits;
    checks::turns(Some(&parsed.engine.limits), &parsed.workers, own, agent, &mut problems);
    let remote = u32::try_from(checks::longest_remote(&parsed.application.machine, own.forge.name_bytes).unwrap_or(u64::MAX)).ok();
    let engine_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.forge.resources_per_task,
        name_bytes: own.forge.name_bytes,
        remote_bytes: remote.unwrap_or(u32::MAX),
        conflicts: own.checkout.conflicts,
        path_bytes: own.checkout.path_bytes,
        token_bytes: temper_protocol_worker_wire::v1::CEILINGS.token_token,
        entries: own.forges.installations,
        entry_token_bytes: own.forge_component.token_bytes,
        text_bytes: temper_protocol_worker_wire::v1::CEILINGS.identity_name,
    };
    let worker_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.checkout.repositories,
        name_bytes: own.checkout.name_bytes,
        remote_bytes: own.checkout.name_bytes,
        ..engine_bounds
    };
    checks::channel(
        Some(&engine_bounds),
        if agent.is_some() { Some(&worker_bounds) } else { None },
        &parsed.application.machine,
        own,
        &mut problems,
    );
    if let Some(wire) = crate::translate::limits(own) {
        checks::identities(&parsed.application.machine, &wire, &mut problems);
    }
    if let Some(agent) = agent {
        checks::agent(&parsed.engine, agent, &mut problems);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let core = config::core(&parsed.engine, seeds.deployment, seeds.seed);
    if let Some(agent) = agent {
        checks::admission(&core, &parsed.engine, agent, &mut problems);
        checks::agent_answer(agent, &mut problems);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let worker = if let Some(agent) = agent {
        let worker = parsed.workers.into_iter().next().expect("one worker checked");
        let own = parsed.application.workers.into_iter().next().expect("one application worker checked");
        let worker = settings::Worker { jig: worker, root: own.root, git: own.git };
        checks::worker(&crate::worker_settings::root_limits(&worker, agent, &parsed.application.limits), &mut problems);
        checks::grants(&parsed.engine, &worker.jig.limits, agent, &mut problems);
        if !problems.is_empty() {
            return Err(problems);
        }
        if let Err(found) = crate::worker_settings::prepare(&worker, agent, &parsed.application.limits) {
            return Err(Problems::jig(found, &Path::root()));
        }
        Some(worker)
    } else {
        None
    };
    let mut engine = settings::Engine {
        directories: directories.clone(),

        jig: parsed.engine,
        clients: parsed.application.clients,
        sign_in: resolved.sign_in,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        machine: parsed.application.machine,
        demo: parsed.application.demo,
        landing: parsed.application.landing,
        charter: parsed.application.charter,
        limits: parsed.application.limits,
    };
    let root = engine_limits(&mut engine.jig.limits, &engine.limits, engine.jig.memory)?;
    checks::root(core, &engine, seeds, &parsed.application.landing_projects, &root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut turn = None;
    if let Some(agent) = agent {
        turn = Some(agent.limits.bodies.turn_body);
    }
    checked_store(&root, &engine.jig.limits.store, turn)?;
    let io = match &worker {
        Some(worker) => checks::process_io(&engine.jig.limits.io, Some(&worker.jig.limits.io)),
        None => checks::process_io(&engine.jig.limits.io, None),
    }
    .ok_or(problem(Path::root().member(b"engine").member(b"limits").member(b"io"), What::Jig(config::What::Bound)))?;
    let prepared_forges = match crate::forge::prepare(&engine) { Ok(forges) => forges, Err(found) => return Err(Problems::jig(found, &Path::root())) };
    Ok(CheckedStart { engine, worker, io, forges: prepared_forges })
}
fn checked_start(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    _setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<CheckedStart, Problems> {
    checked_start_with(document, directories, files, agent, seeds, _setup, &crate::stock::Given::Stock)
}
/// Build the engine and worker together from the parsed document and read files.
#[expect(clippy::too_many_arguments, reason = "startup receives the same directories as named plus the runtime facts")]
pub fn startup_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: WorkerRuntime,
    given: &crate::stock::Given,
) -> Result<Service, Problems> {
    with_setup_given(
        document,
        directories,
        files,
        Some(agent),
        seeds,
        accounts,
        Some(runtime),
        None,
        None,
        given,
    )
}
#[expect(clippy::too_many_arguments, reason = "the Stock entry retains the existing startup inputs")]
pub fn startup(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: WorkerRuntime,
) -> Result<Service, Problems> {
    startup_with(document, directories, files, agent, seeds, accounts, runtime, &crate::stock::Given::Stock)
}
/// Build the standalone service with the binary's checked, resolved forge setup.
#[expect(clippy::too_many_arguments, reason = "startup receives the checked setup beside the same runtime facts")]
pub fn with_setup_given(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: Option<WorkerRuntime>,
    _setup: Option<&temper_forge_setup::check::Checked>,
    box_line_bytes: Option<u32>,
    given: &crate::stock::Given,
) -> Result<Service, Problems> {
    let CheckedStart { engine, worker, io, forges } =
        checked_start_with(document, directories, files, agent, seeds, _setup, given)?;
    let worker = match worker {
        Some(worker) => match Worker::new(
            &worker,
            agent.expect("checked local agent"),
            &engine.limits,
            files,
            runtime.ok_or(problem(indexed(0), What::Standalone))?,
            seeds.seed,
        ) {
            Ok(worker) => Some(worker),
            Err(found) => return Err(Problems::jig(found, &Path::root())),
        },
        None => None,
    };
    match Service::prepared(engine, seeds, accounts, forges, worker, io, box_line_bytes) { Ok(service) => Ok(service), Err(found) => Err(Problems::jig(found, &Path::root())) }
}
#[expect(clippy::too_many_arguments, reason = "the Stock entry retains the existing startup inputs")]
pub fn with_setup(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: Option<WorkerRuntime>,
    _setup: Option<&temper_forge_setup::check::Checked>,
    box_line_bytes: Option<u32>,
) -> Result<Service, Problems> {
    with_setup_given(
        document,
        directories,
        files,
        agent,
        seeds,
        accounts,
        runtime,
        _setup,
        box_line_bytes,
        &crate::stock::Given::Stock,
    )
}

/// The worker-only document, with no deployment or engine declarations.
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the shell reads the worker handover; raw override provenance stays service-private"
)]
pub struct WorkerPart {
    call_answer: CallAnswer,
    pub worker: settings::Worker,
    pub limits: crate::limits::Limits,
    pub named: NamedWorker,
    pub files: Box<[config::File]>,
}
/// Read exactly a worker's part for serve and program-login (protocol/README.md, 15.1).
pub fn worker_part(input: &[u8], directories: &config::Directories) -> Result<WorkerPart, Problems> {
    if shape(input, directories)? != Shape::WorkerAlone {
        return Err(problem(Path::root(), What::WorkerAlone));
    }
    let document = config::document(
        input,
        &config::DocumentLimits {
            bytes: 1_048_576,
            tokens: u32::try_from(input.len()).expect("checked document"),
            depth: 32,
        },
    )
    .expect("checked shape document");
    let at = member(&document, 0, b"workers").expect("checked workers");
    let ats = read::list(&document, at, document.len()).expect("checked worker list");
    let jig = config::worker(&document, *ats.get(0).expect("one worker"), directories).expect("checked worker");
    let mut problems = Problems::empty();
    let (own, limits) = application::worker_alone(
        &document,
        member(&document, 0, b"application").expect("checked application"),
        directories,
        &mut problems,
    )
    .expect("checked worker part");
    let mut files = config::worker_files(&jig);
    for file in &mut files {
        let mut path = indexed(0);
        for segment in &file.at.segments {
            path = path.child(segment.clone());
        }
        file.at = path;
    }
    Ok(WorkerPart {
        call_answer: CallAnswer::read(&document, *ats.get(0).expect("one worker")),
        named: NamedWorker { root: own.root.clone(), agent: jig.agent.configuration.clone() },
        worker: settings::Worker { jig, root: own.root, git: own.git },
        limits,
        files,
    })
}
/// Construct a worker-only service.
pub fn worker_alone(
    mut part: WorkerPart,
    files: &settings::Files,
    agent: &settings::Agent,
    runtime: WorkerRuntime,
    seed: u64,
) -> Result<Service, Problems> {
    check_worker_part(&mut part, files, agent)?;
    match Service::worker_alone(&part.worker, &part.limits, agent, files, runtime, seed) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

fn check_worker_part(part: &mut WorkerPart, files: &settings::Files, agent: &settings::Agent) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    if let Err(what) = part.call_answer.compose(&mut part.worker.jig.limits, agent) {
        problems.push(call_input_path(indexed(0)), what);
    }
    checks::call_answer(indexed(0), part.worker.jig.limits.core.call_answer_bytes, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    for file in &part.files {
        match files.contents(&file.path) {
            Some(bytes) if bytes.len() <= usize::try_from(file.max).expect("file bound") => {}
            Some(_) => problems.push(file.at.clone(), What::Jig(config::What::Bound)),
            None => problems.push(file.at.clone(), What::Jig(config::What::Unreadable)),
        }
    }
    checks::worker(&crate::worker_settings::root_limits(&part.worker, agent, &part.limits), &mut problems);
    checks::turns(None, core::slice::from_ref(&part.worker.jig), &part.limits, Some(agent), &mut problems);
    checks::agent_answer(agent, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(())
}
