//! The job capability beside smith and the checkout: routing and translation
//! and temper's own one-member prepare (temper/docs/design/domain/worker.md, 4.4).

mod prepare;

use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::JobKind;
use skein_lib::{Decimal, Duration, Env, Queue, Token, Writer};

use crate::workspace::{self, Write};
use crate::{Domain, Limits, Request, route, tooling, wire};

/// The engine identity keeps the decoded spec through workspace preparation.
#[derive(Debug)]
pub(crate) struct Spec {
    title: Box<[u8]>,
    own: Option<Box<prepare::Request>>,
    result: Option<(Box<[u8]>, u8)>,
    value: Option<Result<wire::JobSpec, wire::Undecodable>>,
}

/// Bounded by hosted slots and the entering assignment; consumed by the service, never a fact.
#[derive(Debug)]
pub(crate) struct Evidence {
    pub(crate) kind: JobKind,
    pub(crate) why: Option<Box<[u8]>>,
}

pub(crate) fn assign(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut job: host::JobAssignment,
    items: Result<wire::Workspace, wire::Undecodable>,
    spec: Result<wire::JobSpec, wire::Undecodable>,
    out: &mut Queue<Request>,
) {
    let (run, attempt) = (job.run, job.attempt);
    if domain.host.is_hosting(run, attempt) {
        return;
    }
    // Unhosted refusals from a preceding entry point no longer own a slot.
    let mut stale = skein_lib::List::with_capacity(domain.job_evidence.capacity());
    for (identity, _) in &domain.job_evidence {
        if !domain.host.is_hosting(identity.0, identity.1) {
            stale.push(*identity).expect("bounded evidence identities");
        }
    }
    for identity in &stale {
        domain.job_evidence.remove(identity);
    }
    domain
        .job_evidence
        .insert((run, attempt), Evidence { kind: job.kind, why: None })
        .expect("hosted slots and the entering assignment bound operator evidence");
    let items = match items {
        Ok(items) => items,
        Err(wire::Undecodable) => return refuse(domain, run, attempt, b"workspace items", env, out),
    };
    let checked = if !items.repositories.is_empty() && job.workspace.is_none() {
        Err(b"workstream".as_slice())
    } else {
        crate::assignment::check(&items, &env.limits)
    };
    if let Err(why) = checked {
        return refuse(domain, run, attempt, why, env, out);
    }
    let own = match &spec {
        Ok(spec) if job.kind == JobKind(1) && spec.command.is_empty() => {
            let Some(request) = prepare::decode(&spec.request) else {
                return refuse(domain, run, attempt, b"prepare request", env, out);
            };
            if !prepare::check(&request, &items, &env.limits) {
                return refuse(domain, run, attempt, b"prepare items", env, out);
            }
            Some(Box::new(request))
        }
        Ok(_) if job.kind == JobKind(1) => return refuse(domain, run, attempt, b"prepare command", env, out),
        Ok(_) | Err(wire::Undecodable) => None,
    };
    if domain.job_specs.len() == domain.job_specs.capacity() || workspace::stage_job(domain, &mut job, items).is_none()
    {
        refused(domain, run, attempt, host::JobRefusal::Busy, out);
        return;
    }
    let request = match &spec {
        Ok(spec) => spec.request.as_ref(),
        Err(wire::Undecodable) => b"",
    };
    let title = commit_title(job.kind, request);
    let prior = domain
        .job_specs
        .insert((run, attempt), Spec { title, own, result: None, value: Some(spec) })
        .expect("one staged spec per hosted slot and the entering assignment");
    assert!(prior.is_none(), "wire retries were dropped");
    route::host_step(domain, env, host::Event::Down(host::LinkDown::AssignJob { job }));
}

fn refuse(domain: &mut Domain, run: Token, attempt: Token, why: &[u8], env: &Env<Limits>, out: &mut Queue<Request>) {
    let most = usize::try_from(env.limits.host.detail_bytes).expect("a u32 fits usize");
    let why = Box::from(why.get(..why.len().min(most)).expect("bounded refusal"));
    refused(domain, run, attempt, host::JobRefusal::Invalid(host::JobInvalid::Start { why }), out);
}

fn refused(domain: &mut Domain, run: Token, attempt: Token, refusal: host::JobRefusal, out: &mut Queue<Request>) {
    let answer = host::JobEnding::Refused(refusal);
    domain.job_evidence.get_mut(&(run, attempt)).expect("the entering assignment keeps its kind").why =
        Some(job_reason(&answer));
    out.push(Request::JobAnswer {
        run,
        attempt,
        answer,
        work: wire::Work { landed: Box::new([]), saved: None },
        preparation: None,
    });
}

pub(crate) fn run(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    kind: JobKind,
    deadline: Duration,
    workspace: Option<Token>,
) {
    let hosting = domain.host.hosting(owner).expect("the hub starts an admitted job");
    let spec = domain.job_specs.get_mut(&(hosting.run, hosting.attempt)).expect("an admitted job has a staged spec");
    let own = spec.own.is_some();
    let result = spec.result.take();
    if own && result.is_none() {
        return start_refused(domain, env, owner, b"prepare result");
    }
    let spec = spec.value.take().expect("a job starts once");
    let spec = match spec {
        Ok(spec) => spec,
        Err(wire::Undecodable) => return start_refused(domain, env, owner, b"job spec"),
    };
    let Some(workspace) = workspace else {
        return start_refused(domain, env, owner, b"job workspace");
    };
    let (directory, _) = workspace::job_directory(domain, workspace);
    let number = Decimal::of(directory.raw());
    let mut path = Writer::new(b"workspaces/".len().checked_add(number.as_bytes().len()).expect("bounded directory"));
    path.put(b"workspaces/").expect("directory prefix");
    path.put(number.as_bytes()).expect("directory token");
    domain.jobs.insert(owner, workspace).expect("one job per hosted slot");
    step(
        domain,
        env,
        tooling::Event::Run {
            owner,
            kind,
            spec: tooling::Spec { command: spec.command, request: spec.request },
            deadline,
            directory: path.finish(),
        },
    );
    if let Some((result, code)) = result {
        step(domain, env, tooling::Event::Prepared { owner, result, code });
    }
}

fn start_refused(domain: &mut Domain, env: &Env<Limits>, owner: Token, why: &[u8]) {
    route::host_step(domain, env, host::Event::Job(host::FromJob::StartRefused { owner, why: Box::from(why) }));
}

pub(crate) fn step(domain: &mut Domain, env: &Env<Limits>, event: tooling::Event) {
    let env = env.with(env.limits.tooling.clone());
    tooling::step(&mut domain.tooling, &env, event, &mut domain.tooling_out);
}

/// The outer owner routes the terminal; the inner owner must name that job.
pub(crate) fn event(domain: &mut Domain, env: &Env<Limits>, owner: Token, event: tooling::Event) {
    let inner = match &event {
        tooling::Event::Run { owner, .. }
        | tooling::Event::Prepared { owner, .. }
        | tooling::Event::Stop { owner }
        | tooling::Event::Written { owner, .. }
        | tooling::Event::Spawned { owner }
        | tooling::Event::Exited { owner, .. }
        | tooling::Event::Read { owner, .. }
        | tooling::Event::Gone { owner } => *owner,
    };
    if owner == inner && domain.jobs.contains_key(&owner) {
        match event {
            tooling::Event::Run { .. } | tooling::Event::Prepared { .. } | tooling::Event::Stop { .. } => {}
            event @ (tooling::Event::Written { .. }
            | tooling::Event::Spawned { .. }
            | tooling::Event::Exited { .. }
            | tooling::Event::Read { .. }
            | tooling::Event::Gone { .. }) => step(domain, env, event),
        }
    }
}

pub(crate) fn up(domain: &mut Domain, env: &Env<Limits>, request: tooling::Request, out: &mut Queue<Request>) {
    let event = match request {
        op @ (tooling::Request::Write { owner, .. }
        | tooling::Request::Spawn { owner, .. }
        | tooling::Request::Kill { owner }
        | tooling::Request::Read { owner, .. }) => return out.push(Request::Job { owner, op }),
        tooling::Request::Started { owner } => host::FromJob::Started { owner },
        tooling::Request::StartRefused { owner, why } => {
            domain.jobs.remove(&owner);
            host::FromJob::StartRefused { owner, why }
        }
        tooling::Request::Ended { owner, end } => {
            let end = match end {
                tooling::End::Finished { result, deliver } => {
                    let delivery = if deliver { delivery(domain, owner) } else { None };
                    if deliver && delivery.is_none() {
                        host::JobEnd::Failed { detail: Box::from(b"no unique writable repository".as_slice()) }
                    } else {
                        host::JobEnd::Finished { result, delivery }
                    }
                }
                tooling::End::Failed { detail } => host::JobEnd::Failed { detail },
                tooling::End::Deadline => host::JobEnd::Deadline,
            };
            host::FromJob::Ended { owner, end }
        }
        tooling::Request::Gone { owner, detail } => {
            domain.jobs.remove(&owner);
            host::FromJob::Gone { owner, detail }
        }
    };
    route::host_step(domain, env, host::Event::Job(event));
}

fn delivery(domain: &Domain, owner: Token) -> Option<Box<[u8]>> {
    let workspace = *domain.jobs.get(&owner)?;
    workspace::job_directory(domain, workspace).1
}

pub(crate) fn deliver(domain: &mut Domain, env: &Env<Limits>, owner: Token, workspace: Token, delivery: &[u8]) {
    let (_, writable) = workspace::job_directory(domain, workspace);
    if writable.as_deref() != Some(delivery) {
        route::host_step(
            domain,
            env,
            host::Event::Delivered {
                owner,
                delivery: host::Delivery { outcome: host::DeliveryOutcome::Refused, changed: false, left: workspace },
            },
        );
        return;
    }
    let identity = workspace::job_run(domain, workspace);
    let title = domain.job_specs.get(&identity).expect("job metadata lives through delivery").title.clone();
    workspace::write(domain, env, owner, workspace, Write::Push { title, body: Box::new([]) });
}

/// Close titles name the goal in the canonical request (temper/docs/design/protocol/tooling.md, 4.2).
pub(crate) fn commit_title(kind: JobKind, request: &[u8]) -> Box<[u8]> {
    if kind == JobKind(4)
        && let Some(goal) = close_goal(request)
    {
        return skein_lib::bytes::concat(&[b"Settle goal ", goal, b"'s status"]);
    }
    let number = Decimal::of(u64::from(kind.0));
    skein_lib::bytes::concat(&[b"job ", number.as_bytes()])
}

fn close_goal(request: &[u8]) -> Option<&[u8]> {
    let rest = request.strip_prefix(b"{\"version\":1,\"goal\":")?;
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b',' && (1..=20).contains(&index) {
            return rest.get(..index);
        }
        if index == 20 || !byte.is_ascii_digit() {
            return None;
        }
    }
    None
}

pub(crate) fn answered(
    domain: &mut Domain,
    run: Token,
    attempt: Token,
    mut answer: host::JobEnding,
    detail_bytes: u32,
    out: &mut Queue<Request>,
) {
    let (work, preparation) = workspace::job_parts(domain, &answer);
    if domain.job_specs.remove(&(run, attempt)).is_some() {
        let evidence = domain.job_evidence.get_mut(&(run, attempt)).expect("an admitted job keeps its kind");
        evidence.why = match &answer {
            host::JobEnding::Failed { failure: host::JobFailure::Undelivered(_), .. } => {
                let outcome = job_reason(&answer);
                let why = match evidence.why.take() {
                    Some(why) => {
                        let size = outcome
                            .len()
                            .checked_add(2)
                            .expect("closed delivery outcome")
                            .checked_add(why.len())
                            .expect("bounded push diagnostic");
                        let mut text = Writer::new(size);
                        for part in [outcome.as_ref(), b": ", why.as_ref()] {
                            text.put(part).expect("measured worker delivery why");
                        }
                        text.finish()
                    }
                    None => outcome,
                };
                Some(why)
            }
            host::JobEnding::Failed { .. } | host::JobEnding::Refused(_) => Some(job_reason(&answer)),
            host::JobEnding::Finished { .. } => None,
        };
    }
    if let host::JobEnding::Failed { detail, .. } = &mut answer
        && let Some(evidence) = domain.job_evidence.get(&(run, attempt))
        && let Some(why) = evidence.why.as_deref()
    {
        let start = why.len().saturating_sub(usize::try_from(detail_bytes).expect("u32 bound"));
        *detail = Box::from(why.get(start..).expect("bounded job cause"));
    }
    match &answer {
        host::JobEnding::Refused(_) => workspace::forget(domain, run, attempt),
        host::JobEnding::Finished { .. } | host::JobEnding::Failed { .. } => {}
    }
    out.push(Request::JobAnswer { run, attempt, answer, work, preparation });
}

/// Keep only the job's delivery evidence; agent deliveries already have their own feedback.
pub(crate) fn delivered(domain: &mut Domain, owner: Token, outcome: &crate::checkout::Outcome) {
    let Some(hosting) = domain.host.hosting(owner) else { return };
    let Some(evidence) = domain.job_evidence.get_mut(&(hosting.run, hosting.attempt)) else { return };
    evidence.why = delivery_why(outcome);
}

fn delivery_why(outcome: &crate::checkout::Outcome) -> Option<Box<[u8]>> {
    use crate::checkout::{Landing, Outcome, Refusal};
    let landings = match outcome {
        Outcome::Refused { refusal: Refusal::Full } => return Some(Box::from(b"checkout full".as_slice())),
        Outcome::Refused { refusal: Refusal::Busy } => return Some(Box::from(b"checkout busy".as_slice())),
        Outcome::Refused { refusal: Refusal::Invalid } => return Some(Box::from(b"invalid push".as_slice())),
        Outcome::Pushed { landings } => landings,
    };
    for landing in landings {
        let why = match landing {
            Landing::Landed { .. } | Landing::DesignMerged { .. } | Landing::Unchanged => continue,
            Landing::Moved => Box::from(b"branch moved".as_slice()),
            Landing::Conflicted { .. } | Landing::DesignConflict { .. } => {
                Box::from(b"unresolved conflict markers".as_slice())
            }
            Landing::Failed => Box::from(b"push failed".as_slice()),
            Landing::DesignCode { .. } | Landing::DesignMoving | Landing::DesignUnchecked | Landing::Refused => {
                Box::from(b"push refused".as_slice())
            }
            Landing::Aborted => Box::from(b"push aborted".as_slice()),
            Landing::Explained { fault, diagnostic } => {
                let prefix = b"push: ".as_slice();
                let size = prefix
                    .len()
                    .checked_add(fault.name().len())
                    .expect("fixed fault words")
                    .checked_add(2)
                    .expect("separator")
                    .checked_add(diagnostic.output().len())
                    .expect("bounded git output");
                let mut text = Writer::new(size);
                for part in [prefix, fault.name(), b": ", diagnostic.output()] {
                    text.put(part).expect("measured delivery diagnostic");
                }
                text.finish()
            }
        };
        return Some(why);
    }
    None
}

/// Operator-only fallback for endings whose hub answer carries no detail (worker.md, 4.4).
fn job_reason(answer: &host::JobEnding) -> Box<[u8]> {
    use host::{DeliveryOutcome, JobFailure, JobInvalid, JobRefusal, Reason};
    let why: &[u8] = match answer {
        host::JobEnding::Failed { detail, .. } if !detail.is_empty() => return detail.clone(),
        host::JobEnding::Failed { failure, .. } => match failure {
            JobFailure::Unprepared(_) => b"workspace preparation failed",
            JobFailure::Failed => b"command failed",
            JobFailure::Deadline => b"deadline passed",
            JobFailure::Result => b"result missing or past its bound",
            JobFailure::Unstarted => b"process could not be started",
            JobFailure::Undelivered(DeliveryOutcome::Stale) => b"delivery Stale",
            JobFailure::Undelivered(DeliveryOutcome::Refused) => b"delivery Refused",
            JobFailure::Undelivered(DeliveryOutcome::Failed) => b"delivery Failed",
            JobFailure::Undelivered(DeliveryOutcome::Delivered | DeliveryOutcome::Nothing) => {
                unreachable!("successful delivery does not fail a job")
            }
            JobFailure::Cancelled(Reason::Engine) => b"cancelled by engine",
            JobFailure::Cancelled(Reason::Contact) => b"contact lost",
            JobFailure::Cancelled(Reason::Shutdown) => b"worker shutting down",
        },
        host::JobEnding::Refused(JobRefusal::Busy) => b"host busy",
        host::JobEnding::Refused(JobRefusal::Invalid(invalid)) => match invalid {
            JobInvalid::Kind => b"kind not declared",
            JobInvalid::Spec => b"spec past its bound",
            JobInvalid::Deadline => b"deadline past its bound",
            JobInvalid::Grants => b"invalid grants",
            JobInvalid::Start { why } => return why.clone(),
        },
        host::JobEnding::Finished { .. } => unreachable!("finished jobs have no operator failure line"),
    };
    Box::from(why)
}

/// The own request supplies the checkout's one-member combine, after item admission.
pub(crate) fn checkout_spec(domain: &Domain, owner: Token, spec: &mut crate::checkout::Spec) {
    let Some(hosting) = domain.host.hosting(owner) else { return };
    let Some(job) = domain.job_specs.get(&(hosting.run, hosting.attempt)) else { return };
    if let Some(request) = &job.own {
        spec.repositories.first_mut().expect("own prepare has one repository").start = prepare::start(request);
    }
}

pub(crate) fn candidate(domain: &mut Domain, owner: Token, outcome: &crate::checkout::Candidate) {
    let hosting = domain.host.hosting(owner).expect("a job prepared a candidate");
    let job = domain.job_specs.get_mut(&(hosting.run, hosting.attempt)).expect("own request lives through preparation");
    let request = job.own.as_ref().expect("a candidate belongs to temper's prepare");
    job.result =
        Some(prepare::result(request.member, outcome, 4_096).expect("an empty conflict result fits the prepare bound"));
}
