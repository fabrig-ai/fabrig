//! The document's shape and its checked startup (protocol/configuration/README.md, 2).
pub mod application;
pub use temper_service_core::startup::fixed_worker;
pub mod members;
mod reference;
pub use crate::limits::{engine_reference, reference};
pub use reference::{configuration_reference, engine_configuration_reference};

use temper_worker_service::startup::worker;

pub use worker::WorkerPart;
#[cfg(test)]
#[path = "tests/startup_calls.rs"]
mod calls_tests;
#[cfg(test)]
#[path = "tests/figures.rs"]
mod figures_tests;

mod checkout {
    use super::{Problems, settings};
    use crate::limits;
    use jig_protocol_config as config;
    pub(super) fn capacity(
        engine: Option<&config::Limits>,
        workers: &[config::Worker],
        own: &limits::Limits,
        agent: Option<&settings::Agent>,
        problems: &mut Problems,
    ) {
        super::worker::checkout::capacity(engine, workers, &limits::worker(own), agent, problems);
    }
}
pub(crate) mod checks;
mod problems;
pub use temper_engine_service::startup::{EngineWhat, Listener, RedirectPort};
#[cfg(test)]
#[path = "tests/push_configuration.rs"]
mod push_configuration;
#[cfg(test)]
#[path = "tests/welcome_policy.rs"]
mod welcome_policy;
use crate::{AccountsRuntime, Service, WorkerRuntime, settings};
use alloc::boxed::Box;
pub use application::sign_in::{Endpoints, SignIn, url as sign_in_url};
pub use application::{Forge, Identity, Transport, USER_BYTES, User};
pub use checks::redirect_ports;
use jig_protocol_config::{self as config, Path, Segment, read};
use temper_engine_service::startup::{checked_store, engine_limits, prepare_forges};
pub use temper_engine_service::startup::{engine, engine_with, fixed};

pub use problems::{Problem, Problems, What, lines};
use skein_lib::List;

use temper_service_core::agent::Declaration;
use temper_service_core::startup::{CallAnswer, nearest_document};
pub use temper_service_core::startup::{Shape, nearest};

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
/// A forge API token's bound, until the forge component supplies it.
pub const API_TOKEN_BYTES: u32 = 4096;
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
                        .member(b"providers.setting")
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

pub use worker::NamedWorker;

fn named_worker(directories: &config::Directories, own: &application::Worker) -> NamedWorker {
    NamedWorker {
        root: own.root.clone(),
        agent: directories.join(config::Directory::Data, b"agent.json").expect("bounded data path"),
        program: Box::default(),
        arguments: Box::default(),
    }
}

/// The box composes before opening its store (protocol/configuration/figures.md, 4).
pub fn box_worker(
    document: &[u8],
    directories: &config::Directories,
    given: &crate::stock::Given,
    agents: &temper_client_protocol_wire::v1::AgentFigures,
) -> Result<Option<WorkerPart>, Problems> {
    let parsed = parse_with(document, directories, given)?;
    let Some(jig) = parsed.workers.into_iter().next() else { return Ok(None) };
    let own = parsed.application.workers.into_iter().next().expect("one application worker checked");
    let figures =
        temper_engine_service::figures::Composer::new(&parsed.engine, &parsed.application.limits).compose(agents);
    let mut parts = figures.into_parts();
    parts.places = parsed.engine.limits.core.fleet.slots;
    let figures =
        jig_protocol_worker_wire::Figures::new(&jig_protocol_worker_wire::CEILINGS, parts).expect("composed figures");
    let mut part = WorkerPart::in_process(jig, own, directories, crate::limits::worker(&parsed.application.limits));
    match temper_worker_service::figures::apply(&mut part, figures) {
        Ok(_) => {}
        Err(found) => return Err(problems::from_worker(found)),
    }
    Ok(Some(part))
}

/// The files and directories the binary reads and opens before startup.
#[derive(Debug)]
pub struct Named {
    pub provider: bool,
    pub test_github: Option<temper_forge_setup::machine::GitHubMachine>,
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
pub fn store_limits(
    document: &[u8],
    directories: &config::Directories,
    agent: Option<&Declaration>,
) -> Result<StoreLimits, Problems> {
    let sources = match agent {
        Some(agent) => temper_service_core::derived::Sources::agent(agent),
        None => temper_service_core::derived::STOCK,
    };
    let mut parsed = load_with_sources(document, directories, false, &crate::stock::Given::Stock, sources)?
        .1
        .ok_or(problem(Path::root().member(b"engine"), What::Standalone))?;
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
        if !problems.is_empty() {
            return Err(problems);
        }
        for worker in &mut self.workers {
            worker.limits = Some(self.engine.limits);
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
    load_with_sources(input, directories, check_start, given, temper_service_core::derived::STOCK)
}
fn load_with_sources(
    input: &[u8],
    directories: &config::Directories,
    check_start: bool,
    given: &crate::stock::Given,
    sources: temper_service_core::derived::Sources,
) -> Result<(Shape, Option<Parsed>), Problems> {
    let length = u32::try_from(input.len()).or(Err(problem(Path::root(), What::Jig(config::What::Bound))))?;
    let document =
        match config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: length, depth: 32 }) {
            Ok(document) => document,
            Err(found) => return Err(problem(found.path, What::Jig(found.what))),
        };
    // A remote engine takes the stock agent's sources, not an unused supplied agent
    // (protocol/configuration/derived.md, 1 and 2).
    let workers = match member(&document, 0, b"workers") {
        Some(at) => read::list(&document, at, u32::MAX).ok(),
        None => None,
    };
    let sources = match workers {
        Some(workers) => {
            if workers.is_empty() {
                temper_service_core::derived::STOCK
            } else {
                sources
            }
        }
        None => temper_service_core::derived::STOCK,
    };
    load_document_over(&document, directories, check_start, given, sources)
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
#[cfg(test)]
fn load_document(
    document: &skein_json::Document,
    directories: &config::Directories,
    check_start: bool,
    given: &crate::stock::Given,
) -> Result<(Shape, Option<Parsed>), Problems> {
    load_document_over(document, directories, check_start, given, temper_service_core::derived::STOCK)
}
#[expect(clippy::too_many_lines, reason = "one load pass accumulates all cross-part diagnostics")]
fn load_document_over(
    document: &skein_json::Document,
    directories: &config::Directories,
    check_start: bool,
    given: &crate::stock::Given,
    sources: temper_service_core::derived::Sources,
) -> Result<(Shape, Option<Parsed>), Problems> {
    let mut whole = match members::object(document, 0, "document") {
        Ok(object) => object,
        Err(what) => return Err(problem(Path::root(), What::Jig(what))),
    };
    let shape = nearest_document(document);
    if shape == Shape::WorkerAlone && document.len() != 2 {
        return match worker::shape_document(document, directories) {
            Ok(shape) => Ok((shape, None)),
            Err(found) => Err(problems::from_worker(found)),
        };
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
    whole.finish(document, &root, &mut found);
    problems.append(found, &root);
    if let Some(at) = engine_at {
        let before = problems.problems.len();
        watch_rooms(document, at, &mut problems);
        if problems.problems.len() != before {
            return Err(problems);
        }
    }
    let engine = match engine_at {
        Some(at) => {
            match temper_engine_service::startup::engine_with_sources(document, at, directories, given, sources) {
                Ok(mut engine) => {
                    temper_engine_service::startup::check_fleet_grace(&engine, &mut problems);
                    if check_start {
                        checks::figures(&engine, &mut problems);
                    }
                    engine.limits.clients.watches_per_stream = temper_engine_domain::WATCHES_PER_STREAM;
                    Some(engine)
                }
                Err(found) => {
                    problems.append(found, &root.member(b"engine"));
                    None
                }
            }
        }
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
    for (index, at) in ats.iter().enumerate() {
        if shape == Shape::EngineAlone {
            continue;
        }
        let mut moved = false;
        for (name, words) in [(b"limits".as_slice(), worker::MOVED_LIMITS), (b"agent".as_slice(), worker::MOVED_AGENT)]
        {
            if member(document, *at, name).is_some() {
                problems.push(indexed(index).member(name), What::Jig(config::What::Because(words)));
                moved = true;
            }
        }
        if moved {
            continue;
        }
        let defaults = match &engine {
            Some(engine) => engine.limits,
            None => config::stock(),
        };
        match config::worker_over(document, *at, directories, &fixed_worker(), &defaults) {
            Ok(mut worker) => {
                if let Some(engine) = &engine {
                    worker.limits = Some(engine.limits);
                }
                worker.agent = Some(config::Agent {
                    program: Box::default(),
                    arguments: Box::default(),
                    configuration: directories.join(config::Directory::Data, b"agent.json").expect("bounded data path"),
                });
                workers.push(worker).expect("bounded workers");
            }
            Err(found) => problems.append(found, &indexed(index)),
        }
    }
    if check_start && let Some(engine) = &engine {
        match given.source() {
            crate::stock::Given::Running { .. } => unreachable!("unwrapped source"),
            crate::stock::Given::Stock => checks::stock(engine, &mut problems),
            crate::stock::Given::Own { .. } => {}
        }
    }
    let application = match &engine {
        Some(engine) => match application_at {
            Some(at) => application::parse_over(document, at, engine, directories, sources, &mut problems),
            None => None,
        },
        None => None,
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    let application = application.expect("required application checked");
    let workers = workers.into_boxed();
    let engine_call_answer = CallAnswer::read(document, engine_at.expect("required engine checked"));
    if !check_start {
        return Ok((
            shape,
            Some(Parsed { engine: engine.expect("required engine checked"), workers, application, engine_call_answer }),
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
        checks::machine_remote(application.test_github.as_ref(), application.limits.forge.name_bytes),
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
        Some(Parsed { engine: engine.expect("required engine checked"), workers, application, engine_call_answer }),
    ))
}

#[cfg(test)]
fn load(
    input: &[u8],
    directories: &config::Directories,
    check_start: bool,
) -> Result<(Shape, Option<Parsed>), Problems> {
    load_with(input, directories, check_start, &crate::stock::Given::Stock)
}
fn all_files(parsed: &Parsed) -> Box<[config::File]> {
    let engine = config::files(&parsed.engine);
    let worker: Box<[config::File]> = match parsed.workers.first() {
        Some(worker) => {
            let configuration = Path::root().member(b"agent").member(b"configuration");
            let mut files = List::uncapped();
            for file in config::worker_files(worker) {
                if file.at != configuration {
                    files.push(file).expect("uncapped named files");
                }
            }
            files.into_boxed()
        }
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
            if prefix == Path::root().member(b"engine")
                && file.at.segments.as_slice().first() == Some(&Segment::Member(b"llm".as_slice().into()))
            {
                path = Path::root().member(b"providers.setting");
                for segment in file.at.segments.iter().skip(1) {
                    path = path.child(segment.clone());
                }
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
        provider: parsed.engine.clients.provider.is_some(),
        test_github: parsed.application.test_github,
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
        worker: match parsed.workers.first() {
            Some(_) => Some(named_worker(
                directories,
                parsed.application.workers.first().expect("one application worker checked"),
            )),
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
#[derive(Debug)]
pub struct CheckedStart {
    warnings: Problems,
    not_set_up: List<u16>,
    pub(crate) engine: settings::Engine,
    worker: Option<(settings::Worker, crate::worker_settings::Prepared)>,
    io: skein_io::Limits,
    forges: skein_forge_connection::Settings,
    figures: Option<jig_protocol_worker_wire::Figures>,
}
impl CheckedStart {
    /// The typed configuration, before a service is constructed.
    pub fn engine_mut(&mut self) -> &mut settings::Engine {
        &mut self.engine
    }
}
/// Check the box with its already composed worker figures.
#[expect(clippy::too_many_arguments, reason = "the box hands over its composed worker beside startup inputs")]
pub fn checked_start_with_worker(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
    git: Option<&[u8]>,
    part: Option<WorkerPart>,
) -> Result<CheckedStart, Problems> {
    checked_start_using(document, directories, files, agent, seeds, forges, setup, given, git, part)
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
    if shape(document, directories)? == Shape::WorkerAlone {
        let mut part = worker_part(document, directories, None)?;
        if let Err(found) = worker::check_worker_part(&mut part, files, agent) {
            return Err(problems::from_worker(found));
        }
        return Ok(());
    }
    checked_start(document, directories, files, Some(agent), seeds, forges, setup)?;
    Ok(())
}
#[expect(
    clippy::too_many_arguments,
    reason = "one checked startup pass preserves diagnostic order beside resolved setup and runtime facts"
)]
pub fn checked_start_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
    git: Option<&[u8]>,
) -> Result<CheckedStart, Problems> {
    checked_start_using(document, directories, files, agent, seeds, forges, setup, given, git, None)
}
#[expect(clippy::too_many_arguments, reason = "the box supplies its applied worker beside the checked engine inputs")]
#[expect(clippy::too_many_lines, reason = "one checked startup pass preserves diagnostic order")]
fn checked_start_using(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    _setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
    git: Option<&[u8]>,
    part: Option<WorkerPart>,
) -> Result<CheckedStart, Problems> {
    let sources = match agent {
        Some(agent) => temper_service_core::derived::Sources::agent(&agent.declaration()),
        None => temper_service_core::derived::STOCK,
    };
    let mut parsed = load_with_sources(document, directories, true, given, sources)?
        .1
        .ok_or(problem(Path::root().member(b"engine"), What::Standalone))?;
    if !parsed.workers.is_empty() {
        let mut found = temper_worker_service::startup::Problems::empty();
        let own = worker::application::fixed(directories, git, &mut found);
        if !found.is_empty() {
            return Err(problems::from_worker(found));
        }
        parsed.application.workers = Box::new([own.expect("fixed paths checked")]);
    }
    let figures = match &part {
        Some(part) => part.figures.clone(),
        None => None,
    };
    if let Some(part) = part {
        parsed.workers = Box::new([part.worker.jig]);
    }

    let agent = match parsed.workers.first() {
        Some(_) => Some(agent.ok_or(problem(indexed(0).member(b"agent"), What::Half(EngineWhat::Agent)))?),
        None => None,
    };
    let declaration = match agent {
        Some(agent) => Some(agent.declaration()),
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
    let remote = u32::try_from(
        checks::longest_remote(parsed.application.test_github.as_ref(), own.forge.name_bytes).unwrap_or(u64::MAX),
    )
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
        parsed.application.test_github.as_ref(),
        own,
        &mut problems,
    );
    let host_part = settings::host_part(&parsed.engine, declaration.as_ref());
    if let Some(agent) = &declaration {
        checks::agent(&parsed.engine, agent, &mut problems);
    }
    checks::room(&parsed.engine, host_part, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut core = temper_engine_service::startup::core(&parsed.engine, seeds.deployment, seeds.seed);
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

    let worker =
        checked_worker(parsed.workers, parsed.application.workers, &parsed.engine, &parsed.application.limits, agent)?;

    let mut engine = settings::Engine {
        landing_held: false,
        setups: Box::new([]),
        host_reply_bytes: match &declaration {
            Some(agent) => Some(agent.host_reply_text),
            None => None,
        },
        agent_charter_bytes: match &declaration {
            Some(agent) => Some(agent.charter_bytes),
            None => None,
        },
        host_part,
        test_github: parsed.application.test_github,
        directories: directories.clone(),
        jig: parsed.engine,
        clients: parsed.application.clients,
        sign_in: resolved.sign_in,
        push: resolved.push,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        demo: parsed.application.demo,
        program_environment: Box::new([]),
        kinds: Box::default(),
        period_span: crate::stock::PERIOD_SPAN,
        kinds_stock: Box::default(),
        ways_stock: match given.source() {
            crate::stock::Given::Running { .. } => unreachable!("unwrapped source"),
            crate::stock::Given::Stock => crate::stock::ways(),
            crate::stock::Given::Own { ways, .. } => ways.clone(),
        },
        limits: parsed.application.limits,
    };
    let root = engine_limits(&mut engine.jig.limits, &engine.limits, engine.jig.memory)?;
    if let Some(&connector) = core.connectors.first() {
        engine.kinds = crate::stock::kinds(&root, connector);
    }
    engine.kinds_stock = match given.source() {
        crate::stock::Given::Running { .. } => unreachable!("unwrapped source"),
        crate::stock::Given::Stock => crate::stock::forge(&root),
        crate::stock::Given::Own { forge, .. } => forge.clone(),
    };
    checks::root(core, &engine, seeds, &[], &root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut turn = None;
    if let Some(agent) = &declaration {
        turn = Some(agent.turn_body);
    }
    checked_store(&root, &engine.jig.limits.store, turn)?;
    let io = match &worker {
        Some((worker, _)) => checks::process_io(
            &engine.jig.limits.io,
            Some(&worker.jig.limits.as_ref().expect("checked worker limits").io),
        ),
        None => checks::process_io(&engine.jig.limits.io, None),
    }
    .ok_or(problem(Path::root().member(b"engine").member(b"limits").member(b"io"), What::Jig(config::What::Bound)))?;
    let prepared_forges = prepare_forges(&engine, forges)?;
    Ok(CheckedStart { warnings, not_set_up, engine, worker, io, forges: prepared_forges, figures })
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
    checked_start_with(document, directories, files, agent, seeds, forges, setup, &crate::stock::Given::Stock, None)
}
/// Check serve's configuration, building only the engine for an upgrade tool (protocol/binaries.md).
pub fn restoring_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    given: &crate::stock::Given,
) -> Result<Service, Problems> {
    let CheckedStart { engine, io, forges, .. } =
        checked_start_with(document, directories, files, agent, seeds, List::with_capacity(0), None, given, None)?;
    match Service::prepared(engine, seeds, accounts, forges, None, io, None, None) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

/// Check the standalone document for an upgrade, returning only the engine adapter.
pub fn restoring_engine_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    given: &crate::stock::Given,
) -> Result<temper_engine_service::Service, Problems> {
    let CheckedStart { engine, io, forges, .. } =
        checked_start_with(document, directories, files, agent, seeds, List::with_capacity(0), None, given, None)?;
    temper_engine_service::startup::restoring_prepared(engine, seeds, accounts, forges, io)
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
    let checked = checked_start_with(document, directories, files, agent, seeds, forges, setup, given, None)?;
    from_checked(checked, files, agent, seeds, accounts, runtime, box_line_bytes, program_login, program_environment)
}
/// Construct the box with its worker already composed from engine figures.
#[expect(clippy::too_many_arguments, reason = "startup accepts the composed worker beside its runtime facts")]
pub fn with_setup_worker_given(
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
    part: Option<WorkerPart>,
) -> Result<Service, Problems> {
    let checked = checked_start_using(document, directories, files, agent, seeds, forges, setup, given, None, part)?;
    from_checked(checked, files, agent, seeds, accounts, runtime, box_line_bytes, program_login, program_environment)
}
/// Construct from typed startup configuration (protocol/configuration/moved.md, 2).
#[expect(clippy::too_many_arguments, reason = "the prepared start takes the same runtime facts as its document entry")]
pub fn from_checked(
    checked: CheckedStart,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: Option<WorkerRuntime>,
    box_line_bytes: Option<u32>,
    program_login: Option<Box<[u8]>>,
    program_environment: Box<[u8]>,
) -> Result<Service, Problems> {
    let CheckedStart { mut engine, worker, io, forges, not_set_up, warnings, figures } = checked;
    engine.program_environment = program_environment;

    let mut worker =
        match worker::build(worker, agent, &crate::limits::worker(&engine.limits), files, runtime, seeds.seed) {
            Ok(worker) => worker,
            Err(found) => return Err(problems::from_worker(found)),
        };
    if let (Some(worker), Some(figures)) = (&mut worker, figures) {
        worker.built_from(figures);
    }
    match Service::prepared(engine, seeds, accounts, forges, worker, io, box_line_bytes, program_login) {
        Ok(mut service) => {
            for warning in &warnings.problems {
                let mut found = Problems::empty();
                found.push(warning.path.clone(), warning.what);
                let line = lines(&found);
                service.notes().push(crate::Note::Engine(temper_engine_service::Note::StartupWarning {
                    line: line.strip_suffix(b"\n").expect("one warning line").into(),
                }));
            }
            for number in &not_set_up {
                service
                    .engine
                    .as_mut()
                    .expect("engine startup")
                    .setup_state(*number, temper_engine_domain_forge::Connection::NotSetUp);
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

fn checked_worker(
    workers: Box<[config::Worker]>,
    own: Box<[application::Worker]>,
    engine: &config::Engine,
    limits: &crate::limits::Limits,
    agent: Option<&settings::Agent>,
) -> Result<Option<(settings::Worker, crate::worker_settings::Prepared)>, Problems> {
    let mut problems = Problems::empty();
    let limits = &crate::limits::worker(limits);
    if let Some(agent) = agent {
        let worker = workers.into_iter().next().expect("one worker checked");
        let own = own.into_iter().next().expect("one application worker checked");
        let mut worker = settings::Worker { jig: worker, root: own.root, git: own.git };
        worker.jig.limits.as_mut().expect("checked worker limits").hub.source_interval = worker.jig.check_interval;
        checks::worker(&crate::worker_settings::root_limits(&worker, agent, limits), &mut problems);
        checks::grants(engine, worker.jig.limits.as_ref().expect("checked worker limits"), agent, &mut problems);
        if !problems.is_empty() {
            return Err(problems);
        }
        let prepared = match crate::worker_settings::prepare(&worker, agent, limits) {
            Ok(prepared) => prepared,
            Err(found) => return Err(Problems::jig(found, &Path::root())),
        };
        Ok(Some((worker, prepared)))
    } else {
        Ok(None)
    }
}

pub fn worker_part(
    input: &[u8],
    directories: &config::Directories,
    git: Option<&[u8]>,
) -> Result<WorkerPart, Problems> {
    match worker::worker_part(input, directories, git) {
        Ok(part) => Ok(part),
        Err(found) => Err(problems::from_worker(found)),
    }
}

pub fn worker_alone(
    part: WorkerPart,
    files: &settings::Files,
    agent: &settings::Agent,
    runtime: WorkerRuntime,
    seed: u64,
) -> Result<crate::worker_alone::Service, Problems> {
    match worker::worker_alone(part, files, agent, runtime, seed) {
        Ok(service) => Ok(service),
        Err(found) => Err(problems::from_worker(found)),
    }
}

impl Named {
    /// The checked engine member, without the local worker declaration.
    #[must_use]
    pub fn into_engine(self) -> temper_engine_service::startup::Named {
        temper_engine_service::startup::Named {
            provider: self.provider,
            test_github: self.test_github,
            files: self.files,
            asked_files: self.asked_files,
            redirects: self.redirects,
            deployment: self.deployment,
            llm: self.llm,
            projects: self.projects,
            forges: self.forges,
            identities: self.identities,
            clients: self.clients,
            link: self.link,
            public: self.public,
            sign_in: self.sign_in,
            push: self.push,
            bundle: self.bundle,
            bundle_limits: self.bundle_limits,
            webhooks: self.webhooks,
            overrides: self.overrides,
        }
    }
}
