//! Worker construction and compatibility checks (protocol/README.md, 5, and protocol/cluster.md, 1).
use super::checks::{against, at, own, short, worker_limit};
use super::{
    CallAnswer, NamedWorker, Problems, Shape, What, application, call_input_path, checks, indexed, member, problem,
    shape,
};
use crate::{Service, Worker, WorkerRuntime, limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use temper_worker_domain as worker_domain;

pub(super) fn checked(
    workers: Box<[config::Worker]>,
    own: Box<[application::Worker]>,
    engine: &config::Engine,
    limits: &limits::Limits,
    agent: Option<&settings::Agent>,
) -> Result<Option<settings::Worker>, Problems> {
    let mut problems = Problems::empty();
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

pub(super) fn build(
    worker: Option<settings::Worker>,
    agent: Option<&settings::Agent>,
    limits: &limits::Limits,
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
pub(super) fn git_limits(limits: &limits::Limits, git: &limits::GitComponent, problems: &mut Problems) {
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
                problems.push(own(b"git_component.operations"), What::Jig(config::What::Bound));
                break;
            }
        }
    }
}

/// Byte relationships retain the quantities the worker's admission check compared.
fn worker_payload(limits: &worker_domain::Limits, unfit: worker_domain::Unfit, problems: &mut Problems) -> bool {
    use worker_domain::Unfit;
    const MESSAGE: &[u8] = b"agent.json profile.declared.tool_payload less the 2-byte separator";
    match unfit {
        Unfit::Events => problems.push(
            worker_limit(b"hub.event_bytes"),
            What::Within {
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
pub fn worker(limits: &worker_domain::Limits, problems: &mut Problems) {
    use worker_domain::Unfit;
    let worker = at(0, Path::root().member(b"workers"));
    for unfit in &worker_domain::unfit(limits) {
        if worker_payload(limits, *unfit, problems) {
            continue;
        }
        let (path, other): (Path, &'static [u8]) = match unfit {
            Unfit::Agent | Unfit::Unacknowledged => {
                problems.push(worker.member(b"agent").member(b"configuration"), What::Jig(config::What::Bound));
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
        problems.push(path, What::Against { other });
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
    pub limits: limits::Limits,
    pub named: NamedWorker,
    pub files: Box<[config::File]>,
}
/// Read exactly a worker's part for serve and program-login (protocol/cluster.md, 1).
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
    match Service::worker_alone(&part.worker, &part.limits, agent, files, runtime, seed) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

pub(super) fn check_worker_part(
    part: &mut WorkerPart,
    files: &settings::Files,
    agent: &settings::Agent,
) -> Result<(), Problems> {
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
    super::checkout::capacity(None, core::slice::from_ref(&part.worker.jig), &part.limits, Some(agent), &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    worker(&crate::worker_settings::root_limits(&part.worker, agent, &part.limits), &mut problems);
    checks::turns(None, core::slice::from_ref(&part.worker.jig), &part.limits, Some(agent), &mut problems);
    checks::agent_answer(agent, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(())
}
