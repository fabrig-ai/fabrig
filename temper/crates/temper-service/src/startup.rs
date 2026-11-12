//! The document's shape and its checked startup (protocol/configuration/README.md, 2).
pub(crate) mod application;
pub mod members;
mod reference;
#[cfg(feature = "worker")]
mod worker;
pub use crate::limits::{engine_reference, reference};
pub use reference::{configuration_reference, engine_configuration_reference};
#[cfg(feature = "worker")]
pub use worker::WorkerPart;
#[cfg(test)]
#[path = "tests/startup_calls.rs"]
mod calls_tests;
#[cfg(not(feature = "worker"))]
mod checkout;
#[cfg(feature = "worker")]
mod checkout {
 use super::{Problems, settings};
 use crate::limits;
 use jig_protocol_config as config;
 pub(super) fn capacity(engine: Option<&config::Limits>, workers: &[config::Worker], own: &limits::Limits, agent: Option<&settings::Agent>, problems: &mut Problems) {
 super::worker::checkout::capacity(engine, workers, &limits::worker(own), agent, problems);
 }
}
pub(crate) mod checks;
mod problems;
#[cfg(test)]
#[path = "tests/welcome_policy.rs"]
mod welcome_policy;
use crate::{AccountsRuntime, Service, WorkerRuntime, settings};
use alloc::boxed::Box;
pub use application::sign_in::{Endpoints, SignIn, url as sign_in_url};
pub use application::{Forge, Identity, Transport, USER_BYTES, User};
pub use checks::redirect_ports;
use jig_protocol_config::{self as config, Path, Segment, read};
pub(crate) use problems::admission_words;
pub use problems::{EngineWhat, Problem, Problems, What, lines};
use skein_lib::List;

use temper_service_core::startup::{nearest_document, CallAnswer};
pub use temper_service_core::startup::{Shape, nearest};

/// Refuse a shape this build does not serve (protocol/binaries.md, 1, and protocol/cluster.md, 1).
#[must_use]
pub fn unbuilt(shape: Shape, worker: bool) -> Option<Problem> {
    if worker {
        return None;
    }
    match shape {
        Shape::Standalone | Shape::WorkerAlone => {
            Some(Problem { path: Path::root().member(b"workers"), what: What::Half(EngineWhat::UnbuiltWorker), file: None })
        }
        Shape::EngineAlone => None,
    }
}
fn member(document: &skein_json::Document, at: u32, name: &[u8]) -> Option<u32> {
    read::Object::new(document, at).ok()?.member(document, name)
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
fn finish_shape(document: &skein_json::Document, object: members::Object, path: &Path, problems: &mut Problems) {
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
#[cfg(feature = "worker")]
pub use worker::NamedWorker;
#[cfg(not(feature = "worker"))]
#[derive(Debug)]
pub struct NamedWorker { pub root: Box<[u8]>, pub agent: Box<[u8]>, pub program: Box<[u8]> }
fn named_worker(worker: &config::Worker, own: &application::Worker) -> NamedWorker {
    NamedWorker {
        root: own.root.clone(),
        agent: worker.agent.configuration.clone(),
        program: worker.agent.program.clone(),
    }
}

/// The files and directories the binary reads and opens before startup.
#[derive(Debug)]
pub struct Named {
    pub machine: temper_forge_setup::machine::Machine,
    pub files: Box<[config::File]>,
    pub asked_files: Box<[config::File]>,
    pub redirects: Box<[RedirectPort]>,
    pub deployment: Box<[u8]>,
    pub llm: config::Llm,
    pub projects: Box<[config::Project]>,
    pub worker: Option<NamedWorker>,
    pub forges: Box<[Forge]>,
    pub identities: Box<[Identity]>,
    pub clients: config::Address,
    /// `engine.clients.link`: the deployment prints a sign-in link.
    pub link: bool,
    /// `application.clients.public`.
    pub public: Option<application::Origin>,
    pub sign_in: Option<SignIn>,
    /// `application.clients.bundle`, for the binary's read.
    pub bundle: Option<Box<[u8]>>,
    /// The bundle's limits, overrides applied.
    pub bundle_limits: crate::limits::Clients,
    pub webhooks: Option<application::Webhooks>,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
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
        let agent = agent.ok_or(problem(indexed(0).member(b"agent"), What::Half(EngineWhat::Agent)))?;
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
        if let Err(what) = self.engine_call_answer.compose(&mut self.engine.limits, agent.tool_payload(), agent.host_ask_input(), agent.host_reply_text()) {
            problems.push(call_input_path(Path::root().member(b"engine")), what.extend());
        }
        checks::call_answer(Path::root().member(b"engine"), self.engine.limits.core.call_answer_bytes, &mut problems);
        for (index, worker) in self.workers.iter_mut().enumerate() {
            if let Err(what) = self
                .worker_call_answers
                .get(index)
                .expect("one provenance per parsed worker")
                .compose(&mut worker.limits, agent.tool_payload(), agent.host_ask_input(), agent.host_reply_text())
            {
                problems.push(call_input_path(indexed(index)), what.extend());
            }
            checks::call_answer(indexed(index), worker.limits.core.call_answer_bytes, &mut problems);
        }
        if problems.is_empty() { Ok(()) } else { Err(problems) }
    }
}
use temper_service_core::startup::{call_input_path, indexed};
fn problem(path: Path, what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}
/// Parse jig's engine member and pass the supplied defaults, refusing document
/// settings members (protocol/configuration/settings.md, 4 and 5).
/// # Errors
/// The document's problems at its paths under `engine`.
pub fn engine_with(
    document: &skein_json::Document,
    at: u32,
    directories: &config::Directories,
    given: &crate::stock::Given,
) -> Result<config::Engine, config::Problems> {
    let mut engine = config::engine(document, at, directories, config::Defaults::Passed)?;
    let stock = match given {
        crate::stock::Given::Stock => true,
        crate::stock::Given::Own { .. } => false,
    };
    if stock && crate::stock::models(&engine.llm).is_empty() {
        let mut problems = config::Problems::empty();
        problems.push(Path::root().member(b"llm").member(b"providers"), config::What::Bound);
        return Err(problems);
    }
    let Some(defaults) = given.defaults(&engine.llm, engine.limits.core.tasks.result_bytes) else {
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
// Refuse obsolete room settings and diagnose unrepresentable rooms before transport or store sizing.
fn watch_rooms(document: &skein_json::Document, engine: u32, problems: &mut Problems) {
    let Some(limits) = member(document, engine, b"limits") else { return };
    let Some(overrides) = member(document, limits, b"overrides") else { return };
    for name in [b"views.watchers".as_slice(), b"views.watchers_per_party"] {
        if member(document, overrides, name).is_some() {
            problems.push(
                Path::root().member(b"engine").member(b"limits").member(b"overrides").member(name),
                What::Against { other: b"engine.limits.overrides.clients.streams, which sizes both watch rooms" },
            );
        }
    }
    let Some(at) = member(document, overrides, b"clients.streams") else { return };
    let Ok(streams) = read::integer(document, at, u64::from(u32::MAX)) else { return };
    let streams = u32::try_from(streams).expect("checked stream count");
    if !jig_domain_views::valid(&temper_engine_domain::views_for(streams)) {
        problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"overrides").member(b"clients.streams"),
            What::Against { other: b"representable watch rooms (clients.streams times 22, with two slots per watch)" },
        );
    }
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
                What::Half(EngineWhat::InlineSeedPolicy),
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
    let mut whole = match members::object(document, 0, "document") {
        Ok(object) => object,
        Err(what) => return Err(problem(Path::root(), What::Jig(what))),
    };
    let shape = nearest_document(document);
    if let Some(found) = unbuilt(shape, cfg!(feature = "worker")) {
        return Err(problem(found.path, found.what));
    }
    let mut problems = Problems::empty();
    let mut found = config::Problems::empty();
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
        watch_rooms(document, at, &mut problems);
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
            #[cfg(feature = "worker")]
            if let Err(found) = worker::application(document, at, directories) {
                let found = problems::from_worker(found);
                for found in found.problems.into_boxed() { problems.push_file(found.path, found.what, found.file); }
                problems.past = problems.past.saturating_add(found.past);
            }
        }
        return if problems.is_empty() { Ok((shape, None)) } else { Err(problems) };
    }
    if check_start && let Some(engine) = &engine {
        match given {
            crate::stock::Given::Stock => checks::stock(engine, &mut problems),
            crate::stock::Given::Own { .. } => {}
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
        problems.push(root.member(b"workers"), What::Half(EngineWhat::EngineAlone));
    }
    checks::standalone(
        engine.as_ref().expect("required engine checked"),
        &application.clients,
        &workers,
        &mut problems,
    );
    if shape == Shape::EngineAlone && !application.workers.is_empty() {
        problems.push(root.member(b"application").member(b"workers"), What::Half(EngineWhat::EngineAlone));
    } else if workers.len() != application.workers.len() {
        problems.push(root.member(b"application").member(b"workers"), What::Jig(config::What::Bound));
    }
    checks::hub(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::resumed_start(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::provider(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::credentials(
        engine.as_ref().expect("required engine checked"),
        &workers,
        application.limits.forge.forges,
        &mut problems,
    );
    checks::webhooks(&application, &mut problems);
    check_redirects(engine.as_ref().expect("required engine checked"), &application, &mut problems);
    checks::records(engine.as_ref().expect("required engine checked"), &application.limits, &mut problems);
    checks::limits(&application.limits, &application.limits.git_component, &mut problems);
    checks::names(
        checks::machine_remote(&application.machine, application.limits.forge.name_bytes),
        &application.limits,
        &mut problems,
    );
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
        machine: parsed.application.machine,
        files,
        asked_files: Box::new([]),
        redirects: redirects(&parsed.engine),
        clients: parsed.application.clients.listener,
        link: parsed.engine.clients.link,
        public: parsed.application.clients.public,
        sign_in: parsed.application.sign_in,
        bundle: parsed.application.clients.bundle,
        bundle_limits: parsed.application.limits.clients,
        webhooks: parsed.application.webhooks,
        forges: Box::new([]),
        identities: Box::new([]),
        overrides: parsed.application.overrides,
        deployment: parsed.engine.deployment.directory,
        llm: parsed.engine.llm,
        projects: parsed.engine.projects,
        worker: match parsed.workers.first() {
            Some(worker) => {
                Some(named_worker(worker, parsed.application.workers.first().expect("one application worker checked")))
            }
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
pub(crate) struct CheckedStart {
    warnings: Problems,
    not_set_up: List<u16>,
    pub(crate) engine: settings::Engine,
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
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<(), Problems> {
    #[cfg(feature = "worker")]
    if shape(document, directories)? == Shape::WorkerAlone {
        let mut part = worker_part(document, directories)?;
        if let Err(found) = worker::check_worker_part(&mut part, files, agent) { return Err(problems::from_worker(found)); }
        return Ok(());
    }
    checked_start(document, directories, files, Some(agent), seeds, forges, setup)?;
    Ok(())
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one checked startup pass preserves diagnostic order beside resolved setup and runtime facts"
)]
pub(crate) fn checked_start_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    _setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
) -> Result<CheckedStart, Problems> {
    let mut parsed = parse_with(document, directories, given)?;
    let agent = match parsed.workers.first() {
        Some(_) => Some(agent.ok_or(problem(indexed(0).member(b"agent"), What::Half(EngineWhat::Agent)))?),
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
    checkout::capacity(Some(&parsed.engine.limits), &parsed.workers, own, agent, &mut problems);
    checks::turns(Some(&parsed.engine.limits), &parsed.workers, own, agent, &mut problems);
    let remote =
        u32::try_from(checks::longest_remote(&parsed.application.machine, own.forge.name_bytes).unwrap_or(u64::MAX))
            .ok();
    let engine_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.checkout.repositories,
        name_bytes: own.forge.name_bytes,
        remote_bytes: remote.unwrap_or(u32::MAX).min(own.checkout.name_bytes),
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
        Some(&parsed.application.machine),
        own,
        &mut problems,
    );
    let mut host_part = None;
    if let Some(agent) = agent {
        let measured = temper_engine_smith::room::host_part(agent.run_limits());
        checks::agent(&parsed.engine, agent, &mut problems);
        checks::room(&parsed.engine, measured, &mut problems);
        host_part = Some(measured);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut core = config::core(&parsed.engine, seeds.deployment, seeds.seed);
    core.jobs = crate::limits::jobs(&parsed.engine.limits);
    let mut warnings = Problems::empty();
    if let Some(agent) = agent {
        warnings = checks::admission(&core, &parsed.engine, agent, &mut problems);
        checks::agent_answer(agent, &mut problems);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let not_set_up = List::with_capacity(0);
    #[cfg(feature = "worker")]
    let worker =
        checked_worker(parsed.workers, parsed.application.workers, &parsed.engine, &parsed.application.limits, agent)?;
    #[cfg(not(feature = "worker"))]
    let worker: Option<settings::Worker> = None;
    let mut engine = settings::Engine {
        host_reply_bytes: match agent {
            Some(agent) => Some(agent.host_reply_text()),
            None => None,
        },
        agent_charter_bytes: match agent {
            Some(agent) => Some(agent.charter_bytes()),
            None => None,
        },
        host_part,
        machine: parsed.application.machine,
        directories: directories.clone(),
        jig: parsed.engine,
        clients: parsed.application.clients,
        sign_in: resolved.sign_in,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        demo: parsed.application.demo,
        welcome: parsed.application.welcome,
        program_environment: Box::new([]),
        kinds_stock: Box::default(),
        ways_stock: match given {
            crate::stock::Given::Stock => crate::stock::ways(),
            crate::stock::Given::Own { ways, .. } => ways.clone(),
        },
        landings: parsed.application.landings,
        limits: parsed.application.limits,
    };
    let root = engine_limits(&mut engine.jig.limits, &engine.limits, engine.jig.memory)?;
    engine.kinds_stock = match given {
        crate::stock::Given::Stock => crate::stock::forge(&root),
        crate::stock::Given::Own { forge, .. } => forge.clone(),
    };
    checks::root(core, &engine, seeds, &parsed.application.landing_projects, &root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut turn = None;
    if let Some(agent) = agent {
        turn = Some(agent.turn_body());
    }
    checked_store(&root, &engine.jig.limits.store, turn)?;
    let io = match &worker {
        Some(worker) => checks::process_io(&engine.jig.limits.io, Some(&worker.jig.limits.io)),
        None => checks::process_io(&engine.jig.limits.io, None),
    }
    .ok_or(problem(Path::root().member(b"engine").member(b"limits").member(b"io"), What::Jig(config::What::Bound)))?;
    let prepared_forges = prepare_forges(&engine, forges)?;
    Ok(CheckedStart { warnings, not_set_up, engine, worker, io, forges: prepared_forges })
}
fn checked_start(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<CheckedStart, Problems> {
    checked_start_with(document, directories, files, agent, seeds, forges, setup, &crate::stock::Given::Stock)
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
    forges: List<crate::ForgeRuntime>,
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
        forges,
        None,
        None,
        None,
        given,
        Box::new([]),
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
    forges: List<crate::ForgeRuntime>,
) -> Result<Service, Problems> {
    startup_with(document, directories, files, agent, seeds, accounts, runtime, forges, &crate::stock::Given::Stock)
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
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
    box_line_bytes: Option<u32>,
    program_login: Option<Box<[u8]>>,
    given: &crate::stock::Given,
    program_environment: Box<[u8]>,
) -> Result<Service, Problems> {
    let CheckedStart { mut engine, worker, io, forges, not_set_up, warnings } =
        checked_start_with(document, directories, files, agent, seeds, forges, setup, given)?;
    engine.program_environment = program_environment;
    #[cfg(feature = "worker")]
    let worker = match worker::build(worker, agent, &crate::limits::worker(&engine.limits), files, runtime, seeds.seed) { Ok(worker) => worker, Err(found) => return Err(problems::from_worker(found)) };
    #[cfg(not(feature = "worker"))]
    let worker = {
        drop(worker);
        match runtime {
            Some(runtime) => match runtime {},
            None => None,
        }
    };
    match Service::prepared(engine, seeds, accounts, forges, worker, io, box_line_bytes, program_login) {
        Ok(mut service) => {
            for warning in &warnings.problems {
                let mut found = Problems::empty();
                found.push(warning.path.clone(), warning.what);
                let line = lines(&found);
                service.notes().push(crate::Note::StartupWarning {
                    line: line.strip_suffix(b"\n").expect("one warning line").into(),
                });
            }
            for number in &not_set_up {
                crate::setup::lifecycle::state(
                    service.engine.as_mut().expect("engine startup"),
                    *number,
                    temper_engine_domain_forge::Connection::NotSetUp,
                );
            }
            Ok(service)
        }
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
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
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
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
        forges,
        setup,
        box_line_bytes,
        None,
        &crate::stock::Given::Stock,
        Box::new([]),
    )
}

fn prepare_forges(
    engine: &settings::Engine,
    forges: List<crate::ForgeRuntime>,
) -> Result<skein_forge_connection::Settings, Problems> {
    match crate::forge::prepare(engine, forges) {
        Ok(settings) => Ok(settings),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

#[cfg(feature = "worker")]
fn checked_worker(
    workers: Box<[config::Worker]>,
    own: Box<[application::Worker]>,
    engine: &config::Engine,
    limits: &crate::limits::Limits,
    agent: Option<&settings::Agent>,
) -> Result<Option<settings::Worker>, Problems> {
    let mut problems = Problems::empty();
    let limits = &crate::limits::worker(limits);
    if let Some(agent) = agent {
        let worker = workers.into_iter().next().expect("one worker checked");
        let own = own.into_iter().next().expect("one application worker checked");
        let worker = settings::Worker { jig: worker, root: own.root, git: own.git };
        checks::worker(&crate::worker_settings::root_limits(&worker, agent, limits), &mut problems);
        checks::grants(engine, &worker.jig.limits, agent, &mut problems);
        if !problems.is_empty() {
            return Err(problems);
        }
        if let Err(found) = crate::worker_settings::prepare(&worker, agent, limits) {
            return Err(Problems::jig(found, &Path::root()));
        }
        Ok(Some(worker))
    } else {
        Ok(None)
    }
}


#[cfg(feature = "worker")]
pub fn worker_part(input: &[u8], directories: &config::Directories) -> Result<WorkerPart, Problems> {
 match worker::worker_part(input, directories) { Ok(part) => Ok(part), Err(found) => Err(problems::from_worker(found)) }
}
#[cfg(feature = "worker")]
pub fn worker_alone(part: WorkerPart, files: &settings::Files, agent: &settings::Agent, runtime: WorkerRuntime, seed: u64) -> Result<Service, Problems> {
 match worker::worker_alone(part, files, agent, runtime, seed) { Ok(service) => Ok(service), Err(found) => Err(problems::from_worker(found)) }
}
