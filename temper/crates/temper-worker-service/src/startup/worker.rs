//! The worker's start and checks (protocol/services.md, 5).
pub mod application;
pub mod checkout;
pub mod checks;
use crate::worker_settings::Limits;
use crate::{Worker, WorkerRuntime, settings, worker_alone::Service};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_lib::List;
use temper_service_core::limits;
use temper_service_core::startup::problems::{Problems as SharedProblems, What as SharedWhat};
use temper_service_core::startup::problems::{SharedProblems as Problems, SharedWhat as What};
use temper_service_core::startup::{
    CallAnswer, Shape, against, at, call_answer, call_input_path, indexed, member, nearest_document, own, short,
    worker_limit,
};
use temper_worker_domain as worker_domain;
/// The paths opened for a local worker (protocol/services.md, 5).
#[derive(Debug)]
pub struct NamedWorker {
    pub root: Box<[u8]>,
    pub agent: Box<[u8]>,
    pub program: Box<[u8]>,
}
fn problem(path: Path, what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}
pub fn build(
    worker: Option<settings::Worker>,
    agent: Option<&settings::Agent>,
    limits: &Limits,
    files: &settings::Files,
    runtime: Option<WorkerRuntime>,
    seed: u64,
) -> Result<Option<Worker>, Problems> {
    match worker {
        Some(worker) => match Worker::new(
            &worker,
            agent.expect("checked local agent"),
            limits,
            files,
            runtime.ok_or(problem(indexed(0), What::Standalone))?,
            seed,
        ) {
            Ok(worker) => Ok(Some(worker)),
            Err(found) => Err(Problems::jig(found, &Path::root())),
        },
        None => Ok(None),
    }
}

/// Git's local rules are checked independently; the checkout's capacity
/// reports stdout with the other carriers once both links' rooms are known.
pub fn git_limits<E>(limits: &Limits, git: &limits::GitComponent, problems: &mut SharedProblems<SharedWhat<E>>) {
    use temper_worker_protocol_git::{Unfit, check};
    let mut checked = crate::worker_settings::git_component(*git);
    // Repair just the reported rule in the local copy, then ask for the next.
    for _ in 0_u8..7 {
        match check(&checked, &crate::worker_settings::checkout(limits.checkout)) {
            Ok(()) => break,
            Err(Unfit::Operations) => {
                against(problems, b"git_component.operations", b"checkout.workspaces");
                checked.operations = limits.checkout.workspaces;
            }
            Err(Unfit::StdoutBytes) => {
                let Some(bytes) = temper_worker_protocol_git::limits::largest_output(
                    &crate::worker_settings::checkout(limits.checkout),
                ) else {
                    break;
                };
                checked.stdout_bytes = bytes;
            }
            Err(Unfit::StderrBytes) => {
                against(problems, b"git_component.stderr_bytes", b"checkout.push_diagnostic");
                checked.stderr_bytes = u32::try_from(temper_worker_domain_checkout::git::PushDiagnostic::CAPACITY)
                    .expect("diagnostic bound");
            }
            Err(Unfit::ReadBytes) => {
                against(problems, b"git_component.read_bytes", b"a nonzero read");
                checked.read_bytes = 1;
            }
            Err(Unfit::SweepEntries) => {
                against(problems, b"git_component.sweep_entries", b"a nonempty sweep page");
                checked.sweep_entries = 1;
            }
            Err(Unfit::SweepBytes) => {
                against(problems, b"git_component.sweep_bytes", b"the kernel's longest name");
                checked.sweep_bytes = u32::try_from(skein_io::kernel::LONGEST_NAME).expect("kernel name bound");
            }
            Err(Unfit::Memory) => {
                problems.push(own(b"git_component.operations"), SharedWhat::Jig(config::What::Bound));
                break;
            }
        }
    }
}

/// Byte relationships retain the quantities the worker's admission check compared.
fn worker_payload<E>(
    limits: &worker_domain::Limits,
    unfit: worker_domain::Unfit,
    problems: &mut SharedProblems<SharedWhat<E>>,
) -> bool {
    use worker_domain::Unfit;
    const MESSAGE: &[u8] = b"agent.json profile.declared.tool_payload less the 2-byte separator";
    match unfit {
        Unfit::Events => problems.push(
            worker_limit(b"hub.event_bytes"),
            SharedWhat::Within {
                bytes: limits.host.event_bytes,
                limit: MESSAGE,
                bound: limits.agent.message_bytes.saturating_sub(worker_domain::SEPARATOR),
            },
        ),
        Unfit::BoundsMessage => short(
            problems,
            worker_limit(b"hub.event_bytes"),
            limits.host.event_bytes,
            MESSAGE,
            Some(u64::from(limits.bounds.message_bytes)),
        ),
        Unfit::CallBytes => short(
            problems,
            own(b"checkout.message_bytes"),
            u64::from(limits.checkout.message_bytes),
            b"the agent's call_bytes (agent.json profile.declared.tool_payload)",
            Some(limits.agent.call_bytes),
        ),
        Unfit::TurnQueue => short(
            problems,
            worker_limit(b"hub.turn_queue_bytes"),
            limits.host.turn_queue_bytes,
            b"the agent's turns * workers[0].limits.overrides.hub.turn_bytes",
            u64::from(limits.agent.turns).checked_mul(limits.host.turn_bytes),
        ),
        Unfit::Agent
        | Unfit::Unacknowledged
        | Unfit::Directories
        | Unfit::Conflicts
        | Unfit::Paths
        | Unfit::Names
        | Unfit::Key
        | Unfit::Accounts
        | Unfit::Workspaces
        | Unfit::Turns
        | Unfit::Calls
        | Unfit::Held
        | Unfit::Inbox
        | Unfit::Saved
        | Unfit::BoundsInbox => return false,
    }
    true
}
/// The worker's rules against the agent's configuration, each naming both
/// limits (temper-worker-domain's `unfit`).
pub fn worker<E>(limits: &worker_domain::Limits, problems: &mut SharedProblems<SharedWhat<E>>) {
    use worker_domain::Unfit;
    let worker = at(0, Path::root().member(b"workers"));
    for unfit in &worker_domain::unfit(limits) {
        if worker_payload(limits, *unfit, problems) {
            continue;
        }
        let (path, other): (Path, &'static [u8]) = match unfit {
            Unfit::Agent | Unfit::Unacknowledged => {
                problems.push(worker.member(b"agent").member(b"configuration"), SharedWhat::Jig(config::What::Bound));
                continue;
            }
            Unfit::Directories => (own(b"checkout.repositories"), b"the agent's directories"),
            Unfit::Conflicts => (own(b"checkout.conflicts"), b"the agent's conflicts"),
            Unfit::Paths => (own(b"checkout.path_bytes"), b"the agent's path_bytes"),
            Unfit::Names => (own(b"checkout.name_bytes"), b"the agent's name_bytes"),
            Unfit::Key => (own(b"checkout.name_bytes"), b"a workstream's 8-byte key"),
            Unfit::Accounts => (worker_limit(b"hub.accounts"), b"the agent's accounts"),
            Unfit::Workspaces => {
                (worker.member(b"slots"), b"application.limits.overrides.checkout.workspaces, at least the slots")
            }
            Unfit::Turns => (worker_limit(b"hub.turns"), b"the agent's turns"),
            Unfit::Calls => (worker_limit(b"hub.run_calls"), b"the agent's calls"),
            Unfit::Held => (worker_limit(b"hub.held"), b"the agent's messages"),
            Unfit::Inbox => (worker_limit(b"hub.inbox"), b"the agent's messages"),
            Unfit::Saved => (own(b"checkout.message_bytes"), b"the save's commit message"),
            Unfit::BoundsInbox => (worker_limit(b"hub.inbox"), b"the agent's hello's inbox"),
            Unfit::TurnQueue | Unfit::Events | Unfit::CallBytes | Unfit::BoundsMessage => {
                unreachable!("payload rule reported above")
            }
        };
        problems.push(path, SharedWhat::Against { other });
    }
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
    pub limits: Limits,
    pub named: NamedWorker,
    pub files: Box<[config::File]>,
}
/// Read exactly a worker's part for serve and program-login (protocol/cluster.md, 1).
pub fn worker_part(input: &[u8], directories: &config::Directories) -> Result<WorkerPart, Problems> {
    let document = document(input, directories, true)?;
    let at = member(&document, 0, b"workers").expect("checked workers");
    let ats = read::list(&document, at, document.len()).expect("checked worker list");
    let jig = config::worker(&document, *ats.get(0).expect("one worker"), directories).expect("checked worker");
    let mut problems = Problems::empty();
    let (own, limits) = application::read(
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
        named: NamedWorker {
            root: own.root.clone(),
            agent: jig.agent.configuration.clone(),
            program: jig.agent.program.clone(),
        },
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
    match Service::new(&part.worker, &part.limits, agent, files, runtime, seed) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

pub fn check_worker_part(
    part: &mut WorkerPart,
    files: &settings::Files,
    agent: &settings::Agent,
) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    if let Err(what) = part.call_answer.compose(
        &mut part.worker.jig.limits,
        agent.tool_payload(),
        agent.host_ask_input(),
        agent.host_reply_text(),
    ) {
        problems.push(call_input_path(indexed(0)), what);
    }
    call_answer(indexed(0), part.worker.jig.limits.core.call_answer_bytes, &mut problems);
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
    checkout::capacity(None, core::slice::from_ref(&part.worker.jig), &part.limits, Some(agent), &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    worker(&crate::worker_settings::root_limits(&part.worker, agent, &part.limits), &mut problems);
    checks::turns(core::slice::from_ref(&part.worker.jig), agent, &mut problems);
    checks::agent_answer(agent, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(())
}

fn loopback(address: &config::Address) -> bool {
    match address {
        config::Address::Addr(skein_io::Addr::Ip(socket)) => socket.ip().is_loopback(),
        config::Address::Addr(skein_io::Addr::InProcess(_)) | config::Address::Host { .. } => false,
    }
}
fn document(
    input: &[u8],
    directories: &config::Directories,
    check_start: bool,
) -> Result<skein_json::Document, Problems> {
    let document = match config::document(
        input,
        &config::DocumentLimits { bytes: 1_048_576, tokens: u32::try_from(input.len()).unwrap_or(u32::MAX), depth: 32 },
    ) {
        Ok(document) => document,
        Err(found) => return Err(problem(found.path, What::Jig(found.what))),
    };
    if nearest_document(&document) != Shape::WorkerAlone {
        return Err(problem(Path::root(), What::WorkerAlone));
    }
    validate(&document, directories, check_start)?;
    Ok(document)
}
fn validate(
    document: &skein_json::Document,
    directories: &config::Directories,
    check_start: bool,
) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    let root = Path::root();
    let Some(mut whole) = application::object(document, 0, &root, &mut problems) else {
        return Err(problems);
    };
    let worker_at = application::required(&mut whole, document, b"workers", &root, &mut problems);
    let application_at = application::required(&mut whole, document, b"application", &root, &mut problems);
    application::finish_shape(document, whole, &root, &mut problems);
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
        if check_start {
            checks::raw_worker(document, *at, &indexed(index), &mut problems);
        }
        match config::worker(document, *at, directories) {
            Ok(worker) => {
                workers.push(worker).expect("bounded workers");
            }
            Err(found) => problems.append(found, &indexed(index)),
        }
    }
    if worker_at.is_some() && ats.len() != 1 {
        problems.push(root.member(b"workers"), What::WorkerAlone);
    }
    for (index, worker) in workers.iter().enumerate() {
        if !loopback(&worker.engine) {
            problems.push(indexed(index).member(b"engine"), What::WorkerAlone);
        }
    }
    if let Some(at) = application_at {
        let _part = application::read(document, at, directories, &mut problems);
    }
    if problems.is_empty() { Ok(()) } else { Err(problems) }
}
pub fn application(
    document: &skein_json::Document,
    at: u32,
    directories: &config::Directories,
) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    let _part = application::read(document, at, directories, &mut problems);
    if problems.is_empty() { Ok(()) } else { Err(problems) }
}

/// Validate the worker document before any named file is opened.
pub fn shape(input: &[u8], directories: &config::Directories) -> Result<Shape, Problems> {
    document(input, directories, true)?;
    Ok(Shape::WorkerAlone)
}
/// Check a worker alone without constructing it.
pub fn check(
    input: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
) -> Result<(), Problems> {
    let mut part = worker_part(input, directories)?;
    check_worker_part(&mut part, files, agent)
}
