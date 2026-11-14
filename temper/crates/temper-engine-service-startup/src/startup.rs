//! The document's shape and its checked startup (protocol/configuration/README.md, 2).
pub mod application;
pub mod members;
mod reference;
pub use crate::limits::{engine_reference, reference};
pub use reference::{configuration_reference, engine_configuration_reference};

mod checkout;

pub mod checks;
mod problems;
pub use temper_engine_service_vocabulary::startup::*;

use crate::{AccountsRuntime, settings};
use alloc::boxed::Box;
pub use application::sign_in::{Endpoints, SignIn, url as sign_in_url};
pub use application::{Forge, Identity, Transport, USER_BYTES, User};
pub use checks::redirect_ports;
use jig_protocol_config::{self as config, Path, Segment, read};
pub use problems::{EngineWhat, Problem, Problems, What, lines};
use skein_lib::List;

use temper_service_core::agent::Declaration;
use temper_service_core::startup::{CallAnswer, nearest_document};
pub use temper_service_core::startup::{Shape, nearest};

/// Refuse a shape the engine service does not serve (protocol/binaries.md, 1, and protocol/cluster.md, 1).
#[must_use]
pub fn unserved(shape: Shape) -> Option<Problem> {
    match shape {
        Shape::Standalone | Shape::WorkerAlone => Some(Problem {
            path: Path::root().member(b"workers"),
            what: What::Half(EngineWhat::UnbuiltWorker),
            file: None,
        }),
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
    pub forges: Box<[Forge]>,
    pub identities: Box<[Identity]>,
    pub clients: config::Address,
    /// `engine.clients.link`: the deployment prints a sign-in link.
    pub link: bool,
    /// `application.clients.public`.
    pub public: Option<application::Origin>,
    pub sign_in: Option<SignIn>,
    pub push: Option<application::push::Push>,
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
pub fn store_limits(document: &[u8], directories: &config::Directories) -> Result<StoreLimits, Problems> {
    let mut parsed =
        load(document, directories, false)?.1.ok_or(problem(Path::root().member(b"engine"), What::Standalone))?;
    let root = engine_limits(&mut parsed.engine.limits, &parsed.application.limits, parsed.engine.memory)?;
    let store = checked_store(&root, &parsed.engine.limits.store, None)?;
    Ok(StoreLimits { store, forge: root.forge })
}

pub fn checked_store(
    root: &temper_engine_domain::Limits,
    profile: &jig_protocol_store::Limits,
    turn: Option<u32>,
) -> Result<jig_protocol_store::Limits, Problems> {
    match crate::store::admit(root, &root.forge, profile, turn) {
        Ok(store) => Ok(store),
        Err(why) => Err(checks::store_refusal(why)),
    }
}

pub fn engine_limits(
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
    fn compose(&mut self, agent: &Declaration) -> Result<(), Problems> {
        let mut problems = Problems::empty();
        if let Err(what) = self.engine_call_answer.compose(
            &mut self.engine.limits,
            agent.tool_payload,
            agent.host_ask_input,
            agent.host_reply_text,
        ) {
            problems.push(call_input_path(Path::root().member(b"engine")), what.extend());
        }
        checks::call_answer(Path::root().member(b"engine"), self.engine.limits.core.call_answer_bytes, &mut problems);
        for (index, worker) in self.workers.iter_mut().enumerate() {
            if let Err(what) = self.worker_call_answers.get(index).expect("one provenance per parsed worker").compose(
                &mut worker.limits,
                agent.tool_payload,
                agent.host_ask_input,
                agent.host_reply_text,
            ) {
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
            What::Against { other: b"representable watch rooms (clients.streams times 31, with two slots per watch)" },
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
    if let Some(found) = unserved(shape) {
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
        push: parsed.application.push,
        bundle: parsed.application.clients.bundle,
        bundle_limits: parsed.application.limits.clients,
        webhooks: parsed.application.webhooks,
        forges: Box::new([]),
        identities: Box::new([]),
        overrides: parsed.application.overrides,
        deployment: parsed.engine.deployment.directory,
        llm: parsed.engine.llm,
        projects: parsed.engine.projects,
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
/// The checked settings and runtime facts handed from startup to the engine's start.
#[derive(Debug)]
pub struct CheckedStart {
    pub warnings: Problems,
    pub not_set_up: List<u16>,
    pub engine: settings::Engine,
    pub io: skein_io::Limits,
    pub forges: skein_forge_connection::Settings,
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
    checked_start(document, directories, files, Some(agent), seeds, forges, setup)?;
    Ok(())
}
/// Check the document and runtime facts into settings for the engine's start.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one checked startup pass preserves diagnostic order beside resolved setup and runtime facts"
)]
pub fn checked_start_with(
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
    let declaration = match agent {
        Some(agent) => Some(agent.clone()),
        None => None,
    };
    if let Some(agent) = &declaration {
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
    checks::turns(Some(&parsed.engine.limits), &parsed.workers, own, declaration.as_ref(), &mut problems);
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
        reds: own.forge.branch_reds,
        red_key_bytes: own.forge.red_key_bytes,
    };
    let worker_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.checkout.repositories,
        name_bytes: own.checkout.name_bytes,
        remote_bytes: own.checkout.name_bytes,
        reds: temper_protocol_worker_wire::v1::CEILINGS.reds_reds,
        red_key_bytes: temper_protocol_worker_wire::v1::CEILINGS.red_key,
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
    if let Some(agent) = &declaration {
        let measured = temper_engine_smith::room::host_part(&agent.run_limits);
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
    if let Some(agent) = &declaration {
        warnings = checks::admission(&core, &parsed.engine, agent, &mut problems);
        checks::agent_answer(agent, &mut problems);
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let not_set_up = List::with_capacity(0);

    let mut engine = settings::Engine {
        host_reply_bytes: match &declaration {
            Some(agent) => Some(agent.host_reply_text),
            None => None,
        },
        agent_charter_bytes: match &declaration {
            Some(agent) => Some(agent.charter_bytes),
            None => None,
        },
        host_part,
        machine: parsed.application.machine,
        directories: directories.clone(),
        jig: parsed.engine,
        clients: parsed.application.clients,
        sign_in: resolved.sign_in,
        push: resolved.push,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        demo: parsed.application.demo,
        welcome: parsed.application.welcome,
        program_environment: Box::new([]),
        kinds: Box::default(),
        period_span: crate::stock::PERIOD_SPAN,
        kinds_stock: Box::default(),
        ways_stock: match given {
            crate::stock::Given::Stock => crate::stock::ways(),
            crate::stock::Given::Own { ways, .. } => ways.clone(),
        },
        landings: parsed.application.landings,
        limits: parsed.application.limits,
    };
    let root = engine_limits(&mut engine.jig.limits, &engine.limits, engine.jig.memory)?;
    if let Some(&connector) = core.connectors.first() {
        engine.kinds = crate::stock::kinds(&root, connector);
    }
    engine.kinds_stock = match given {
        crate::stock::Given::Stock => crate::stock::forge(&root),
        crate::stock::Given::Own { forge, .. } => forge.clone(),
    };
    checks::root(core, &engine, seeds, &parsed.application.landing_projects, &root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut turn = None;
    if let Some(agent) = &declaration {
        turn = Some(agent.turn_body);
    }
    checked_store(&root, &engine.jig.limits.store, turn)?;
    let io = checks::engine_io(&engine.jig.limits.io);
    let prepared_forges = prepare_forges(&engine, forges)?;
    Ok(CheckedStart { warnings, not_set_up, engine, io, forges: prepared_forges })
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
pub fn prepare_forges(
    engine: &settings::Engine,
    forges: List<crate::ForgeRuntime>,
) -> Result<skein_forge_connection::Settings, Problems> {
    match crate::forge::prepare(engine, forges) {
        Ok(settings) => Ok(settings),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}
