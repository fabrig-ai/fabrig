//! Hosted jobs share the run slab, names and kept answers (jig/docs/design/domain/hosts/hub.md, 9).
use crate::hosted::Hosted;
use crate::{
    Delivery, DeliveryOutcome, FromJob, Hub, JobAssignment, JobEnd, JobEnding, JobFailure, JobInvalid, JobKind,
    JobRefusal, Limits, LinkUp, Phase, Preparation, Reason, Request, ToJob, Work,
};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{Duration, Env, Id, Queue, Token};

#[derive(Debug)]
pub(crate) struct Job {
    result_bound: u32,
    stopped: bool,
    /// Once delivery begins, shutdown cannot promise an interruption with no effect.
    delivery_began: bool,
    left: Option<Token>,
    state: State,
}

#[derive(Debug)]
enum State {
    Preparing { kind: JobKind, spec: Box<[u8]>, deadline: Duration },
    Cancelling { reason: Reason },
    Running { workspace: Option<Token>, started: bool, ending: Option<Ending>, gone: bool, delivering: bool },
    Closed,
}

#[derive(Debug)]
enum Ending {
    Finished { result: Box<[u8]> },
    Failed { failure: JobFailure, detail: Box<[u8]> },
    Refused { why: Box<[u8]> },
}

impl Job {
    pub(crate) fn phase(&self) -> Phase {
        match &self.state {
            State::Preparing { .. } => Phase::Preparing,
            State::Cancelling { .. } | State::Running { ending: Some(_), .. } => Phase::Ending,
            State::Running { started: false, ending: None, .. } => Phase::Starting,
            State::Running { started: true, ending: None, .. } => Phase::Active,
            State::Closed => Phase::Answered,
        }
    }
}

pub(crate) fn owns(domain: &Hub, owner: Token) -> bool {
    match domain.hosted.get(Id::from_token(owner)) {
        Some(entry) => entry.job.is_some(),
        None => false,
    }
}

pub(crate) fn assign(domain: &mut Hub, env: &Env<Limits>, job: JobAssignment, out: &mut Queue<Request>) {
    if domain.names.contains_key(&job.run) || domain.answers.contains_key(&(job.run, job.attempt)) {
        refuse(job.run, job.attempt, JobRefusal::Busy, out);
        return;
    }
    let mut bound = None;
    for declared in &domain.job_kinds {
        if declared.kind == job.kind {
            bound = Some(declared);
            break;
        }
    }
    let mut invalid_grants = job.grants.len() > skein_lib::length::usize_of(env.limits.accounts);
    if !invalid_grants {
        for grant in &job.grants {
            if grant.form != crate::Form::Application {
                invalid_grants = true;
            }
        }
    }
    let invalid = match bound {
        None => Some(JobInvalid::Kind),
        Some(bound) if crate::assignment::len(&job.spec) > u64::from(bound.spec) => Some(JobInvalid::Spec),
        Some(bound) if job.deadline > bound.deadline => Some(JobInvalid::Deadline),
        Some(_) if invalid_grants => Some(JobInvalid::Grants),
        Some(_) => None,
    };
    if let Some(invalid) = invalid {
        refuse(job.run, job.attempt, JobRefusal::Invalid(invalid), out);
        return;
    }
    if domain.shut || domain.hosted.len() >= env.limits.slots {
        refuse(job.run, job.attempt, JobRefusal::Busy, out);
        return;
    }
    let result_bound = bound.expect("validated kind").result;
    let JobAssignment { run, attempt, kind, spec, deadline, workspace, grants: _ } = job;
    let job = Job {
        result_bound,
        stopped: false,
        delivery_began: false,
        left: None,
        state: State::Preparing { kind, spec, deadline },
    };
    let id = domain.hosted.insert(Hosted::job(run, attempt, job)).expect("checked slot capacity");
    domain.names.insert(run, id).expect("one name per slot");
    match workspace {
        Some(workspace) => out.push(Request::Prepare { owner: id.token(), workspace }),
        None => run_prepared(domain, id, None, out),
    }
}

fn refuse(run: Token, attempt: Token, refusal: JobRefusal, out: &mut Queue<Request>) {
    out.push(Request::Up(LinkUp::JobAnswer { run, attempt, answer: JobEnding::Refused(refusal) }));
}

pub(crate) fn prepared(domain: &mut Hub, owner: Token, workspace: Token, out: &mut Queue<Request>) {
    run_prepared(domain, Id::from_token(owner), Some(workspace), out);
}

fn run_prepared(domain: &mut Hub, id: Id<Hosted>, workspace: Option<Token>, out: &mut Queue<Request>) {
    let job = domain.hosted.get_mut(id).expect("pending prepare").job.as_mut().expect("job");
    job.state = match mem::replace(&mut job.state, State::Closed) {
        State::Preparing { kind, spec, deadline } => {
            out.push(Request::Job(ToJob::Run { owner: id.token(), kind, spec, deadline, workspace }));
            State::Running { workspace, started: false, ending: None, gone: false, delivering: false }
        }
        State::Cancelling { reason } => {
            State::Running { workspace, started: false, ending: Some(cancelled(reason)), gone: true, delivering: false }
        }
        State::Running { .. } | State::Closed => unreachable!("a prepare settles once"),
    };
    conclude(domain, id, out);
}

pub(crate) fn unprepared(
    domain: &mut Hub,
    env: &Env<Limits>,
    owner: Token,
    failure: Preparation,
    detail: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let id = Id::from_token(owner);
    let job = domain.hosted.get_mut(id).expect("pending prepare").job.as_mut().expect("job");
    let ending = match mem::replace(&mut job.state, State::Closed) {
        State::Preparing { .. } => {
            Ending::Failed { failure: JobFailure::Unprepared(failure), detail: tail(detail, &env.limits) }
        }
        State::Cancelling { reason } => cancelled(reason),
        State::Running { .. } | State::Closed => unreachable!("a prepare settles once"),
    };
    job.state = State::Running { workspace: None, started: false, ending: Some(ending), gone: true, delivering: false };
    conclude(domain, id, out);
}

pub(crate) fn reported(domain: &mut Hub, env: &Env<Limits>, event: FromJob, out: &mut Queue<Request>) {
    let owner = match &event {
        FromJob::Started { owner }
        | FromJob::StartRefused { owner, .. }
        | FromJob::Ended { owner, .. }
        | FromJob::Gone { owner, .. } => *owner,
    };
    let id = Id::from_token(owner);
    let Some(entry) = domain.hosted.get_mut(id) else { return };
    let Some(job) = &mut entry.job else { return };
    match &mut job.state {
        State::Running { workspace, started, ending, gone, delivering } => {
            match event {
                FromJob::Started { .. } => *started = true,
                FromJob::StartRefused { why, .. } => {
                    if ending.is_none() {
                        *ending = Some(Ending::Refused { why: tail(why, &env.limits) });
                    }
                    *gone = true;
                }
                FromJob::Gone { detail, .. } => {
                    *gone = true;
                    if ending.is_none() {
                        *ending =
                            Some(Ending::Failed { failure: JobFailure::Unstarted, detail: tail(detail, &env.limits) });
                    }
                }
                FromJob::Ended { end, .. } => {
                    // Cancellation owns the answer, including a late successful end.
                    if ending.is_none() {
                        *ending = Some(match end {
                            JobEnd::Finished { result, delivery } => {
                                if crate::assignment::len(&result) > u64::from(job.result_bound) {
                                    failed(JobFailure::Result)
                                } else {
                                    if let Some(workspace) = *workspace
                                        && let Some(delivery) = delivery
                                    {
                                        let call = crate::call::Call {
                                            hosted: id,
                                            state: crate::call::State::JobDelivering,
                                            name: None,
                                        };
                                        let call_id = domain.calls.insert(call).expect("one delivery per job slot");
                                        *delivering = true;
                                        job.delivery_began = true;
                                        out.push(Request::DeliverJob { owner: call_id.token(), workspace, delivery });
                                    }
                                    Ending::Finished { result }
                                }
                            }
                            JobEnd::Failed { detail } => {
                                Ending::Failed { failure: JobFailure::Failed, detail: tail(detail, &env.limits) }
                            }
                            JobEnd::Deadline => failed(JobFailure::Deadline),
                        });
                    }
                }
            }
        }
        State::Preparing { .. } | State::Cancelling { .. } | State::Closed => return,
    }
    conclude(domain, id, out);
}

fn failed(failure: JobFailure) -> Ending {
    Ending::Failed { failure, detail: Box::new([]) }
}
fn cancelled(reason: Reason) -> Ending {
    failed(JobFailure::Cancelled(reason))
}

pub(crate) fn stop(domain: &mut Hub, id: Id<Hosted>, reason: Reason, out: &mut Queue<Request>) {
    let job = domain.hosted.get_mut(id).expect("named job").job.as_mut().expect("job");
    job.state = match mem::replace(&mut job.state, State::Closed) {
        State::Preparing { .. } => {
            out.push(Request::Abort { owner: id.token() });
            State::Cancelling { reason }
        }
        State::Running { workspace, started, ending, gone, delivering } => {
            let was_cancelled = match &ending {
                Some(Ending::Failed { failure, .. }) => match failure {
                    JobFailure::Cancelled(_) => true,
                    JobFailure::Unprepared(_)
                    | JobFailure::Failed
                    | JobFailure::Deadline
                    | JobFailure::Result
                    | JobFailure::Unstarted
                    | JobFailure::Undelivered(_) => false,
                },
                Some(Ending::Finished { .. } | Ending::Refused { .. }) | None => false,
            };
            if !gone && !job.stopped {
                job.stopped = true;
                out.push(Request::Job(ToJob::Stop { owner: id.token() }));
            }
            // Shutdown may interrupt only work whose delivery has not begun
            // (domain/hosts/hub.md, 9). A begun delivery owns its answer.
            let keep_ending = was_cancelled || reason == Reason::Shutdown && job.delivery_began;
            State::Running {
                workspace,
                started,
                ending: if keep_ending { ending } else { Some(cancelled(reason)) },
                gone,
                delivering,
            }
        }
        state @ (State::Cancelling { .. } | State::Closed) => state,
    };
    conclude(domain, id, out);
}

pub(crate) fn delivery_owner(domain: &Hub, owner: Token) -> bool {
    match domain.calls.get(Id::from_token(owner)) {
        Some(call) => call.state == crate::call::State::JobDelivering,
        None => false,
    }
}

pub(crate) fn delivered(domain: &mut Hub, owner: Token, delivery: Delivery, out: &mut Queue<Request>) {
    let call_id = Id::from_token(owner);
    let call = domain.calls.get_mut(call_id).expect("pending job delivery");
    let id = call.hosted;
    call.state = crate::call::State::Closed;
    domain.calls.retire(call_id);
    let job = domain.hosted.get_mut(id).expect("delivery holds job").job.as_mut().expect("job");
    if delivery.changed {
        job.left = Some(delivery.left);
    }
    match &mut job.state {
        State::Running { delivering, ending, .. } => {
            *delivering = false;
            let finished = match ending {
                Some(Ending::Finished { .. }) => true,
                Some(Ending::Failed { .. } | Ending::Refused { .. }) | None => false,
            };
            if finished {
                match delivery.outcome {
                    DeliveryOutcome::Delivered | DeliveryOutcome::Nothing => {}
                    outcome @ (DeliveryOutcome::Stale | DeliveryOutcome::Refused | DeliveryOutcome::Failed) => {
                        *ending = Some(failed(JobFailure::Undelivered(outcome)));
                    }
                }
            }
        }
        State::Preparing { .. } | State::Cancelling { .. } | State::Closed => unreachable!("pending delivery"),
    }
    conclude(domain, id, out);
}

fn conclude(domain: &mut Hub, id: Id<Hosted>, out: &mut Queue<Request>) {
    let entry = domain.hosted.get_mut(id).expect("job holds slot");
    let job = entry.job.as_mut().expect("job");
    let settled = match &job.state {
        State::Running { gone, delivering, ending, .. } => *gone && !*delivering && ending.is_some(),
        State::Preparing { .. } | State::Cancelling { .. } | State::Closed => false,
    };
    if !settled {
        return;
    }
    let (workspace, ending) = match mem::replace(&mut job.state, State::Closed) {
        State::Running { workspace, ending, .. } => (workspace, ending),
        State::Preparing { .. } | State::Cancelling { .. } | State::Closed => unreachable!("settled job"),
    };
    if let Some(workspace) = workspace {
        out.push(Request::Release { workspace });
    }
    let work = Work { left: job.left, saved: None };
    let answer = match ending.expect("settled ending") {
        Ending::Finished { result } => JobEnding::Finished { result, work },
        Ending::Failed { failure, detail } => JobEnding::Failed { failure, detail, work },
        Ending::Refused { why } => JobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why })),
    };
    let refused = match &answer {
        JobEnding::Refused(_) => true,
        JobEnding::Finished { .. } | JobEnding::Failed { .. } => false,
    };
    if !refused {
        entry.answer = Some(crate::answers::Kept::Job(answer.clone()));
        domain.answers.insert((entry.run, entry.attempt), id).expect("one answer per slot");
    }
    entry.asks.clear();
    domain.names.remove(&entry.run);
    domain.ready.remove(&id);
    out.push(Request::Up(LinkUp::JobAnswer { run: entry.run, attempt: entry.attempt, answer }));
    if refused {
        domain.hosted.retire(id);
    }
}

fn tail(detail: Box<[u8]>, limits: &Limits) -> Box<[u8]> {
    let keep = skein_lib::length::usize_of(limits.detail_bytes);
    match detail.len().checked_sub(keep) {
        Some(cut) if cut > 0 => skein_lib::bytes::copy_of(detail.get(cut..).expect("bounded tail")),
        Some(_) | None => detail,
    }
}
