//! The job capability beside smith and the checkout: routing and translation
//! only (temper/docs/design/domain/worker.md, 4.4).

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Decimal, Duration, Env, Queue, Token, Writer};

use crate::workspace::{self, Write};
use crate::{Domain, Limits, Request, route, tooling, wire};

/// The engine identity keeps the decoded spec through workspace preparation.
#[derive(Debug)]
pub(crate) struct Spec {
    kind: host::JobKind,
    value: Option<Result<wire::JobSpec, wire::Undecodable>>,
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
    let items = match items {
        Ok(items) => items,
        Err(wire::Undecodable) => return refuse(run, attempt, b"workspace items", env, out),
    };
    let checked = if !items.repositories.is_empty() && job.workspace.is_none() {
        Err(b"workstream".as_slice())
    } else {
        crate::assignment::check(&items, &env.limits)
    };
    if let Err(why) = checked {
        return refuse(run, attempt, why, env, out);
    }
    if domain.job_specs.len() == domain.job_specs.capacity() || workspace::stage_job(domain, &mut job, items).is_none()
    {
        out.push(Request::Link(host::LinkUp::JobAnswer {
            run,
            attempt,
            answer: host::JobEnding::Refused(host::JobRefusal::Busy),
        }));
        return;
    }
    let prior = domain
        .job_specs
        .insert((run, attempt), Spec { kind: job.kind, value: Some(spec) })
        .expect("one staged spec per hosted slot and the entering assignment");
    assert!(prior.is_none(), "wire retries were dropped");
    route::host_step(domain, env, host::Event::Down(host::LinkDown::AssignJob { job }));
}

fn refuse(run: Token, attempt: Token, why: &[u8], env: &Env<Limits>, out: &mut Queue<Request>) {
    let most = usize::try_from(env.limits.host.detail_bytes).expect("a u32 fits usize");
    let why = Box::from(why.get(..why.len().min(most)).expect("bounded refusal"));
    let answer = host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start { why }));
    out.push(Request::Link(host::LinkUp::JobAnswer { run, attempt, answer }));
}

pub(crate) fn run(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    kind: host::JobKind,
    deadline: Duration,
    workspace: Option<Token>,
) {
    let hosting = domain.host.hosting(owner).expect("the hub starts an admitted job");
    let spec = domain.job_specs.get_mut(&(hosting.run, hosting.attempt)).expect("an admitted job has a staged spec");
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
}

fn start_refused(domain: &mut Domain, env: &Env<Limits>, owner: Token, why: &[u8]) {
    route::host_step(domain, env, host::Event::Job(host::FromJob::StartRefused { owner, why: Box::from(why) }));
}

pub(crate) fn step(domain: &mut Domain, env: &Env<Limits>, event: tooling::Event) {
    let env = Env { now: env.now, wall: env.wall, limits: env.limits.tooling.clone() };
    tooling::step(&mut domain.tooling, &env, event, &mut domain.tooling_out);
}

/// The outer owner routes the terminal; the inner owner must name that job.
pub(crate) fn event(domain: &mut Domain, env: &Env<Limits>, owner: Token, event: tooling::Event) {
    let inner = match &event {
        tooling::Event::Run { owner, .. }
        | tooling::Event::Stop { owner }
        | tooling::Event::Written { owner, .. }
        | tooling::Event::Spawned { owner }
        | tooling::Event::Exited { owner, .. }
        | tooling::Event::Read { owner, .. }
        | tooling::Event::Gone { owner } => *owner,
    };
    if owner == inner && domain.jobs.contains_key(&owner) {
        match event {
            tooling::Event::Run { .. } | tooling::Event::Stop { .. } => {}
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
    let kind = domain.job_specs.get(&identity).expect("job metadata lives through delivery").kind;
    let number = Decimal::of(u64::from(kind.0));
    let mut title = Writer::new(b"job ".len().checked_add(number.as_bytes().len()).expect("bounded kind title"));
    title.put(b"job ").expect("job title prefix");
    title.put(number.as_bytes()).expect("job kind");
    workspace::write(domain, env, owner, workspace, Write::Push { title: title.finish(), body: Box::new([]) });
}

pub(crate) fn answered(domain: &mut Domain, run: Token, attempt: Token, answer: &host::JobEnding) {
    domain.job_specs.remove(&(run, attempt));
    match answer {
        host::JobEnding::Refused(_) => workspace::forget(domain, run, attempt),
        host::JobEnding::Finished { .. } | host::JobEnding::Failed { .. } => {}
    }
}
