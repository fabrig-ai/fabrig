//! One exclusive snapshot for checked delivery and unchecked saving (smith/docs/design/domain/run-delivery.md, section 1, and smith/docs/design/domain/run.md, section 9
//! ). State keeps opaque fields, durable name, caller deadline
//! and its purpose: finishing Change, delivery or save; a running check keeps its given span and retry marker.
//! It never knows receipt encoding,
//! provider ids, callback identity or host delivery policy. Main's write batch
//! waits while jobs settle, through checks and submission; children have already returned.
//! Before submission withdrawal aborts checks. Afterwards no cancellation is
//! emitted: the bounded host terminal settles even during shutdown.
//! A stopped check returns its retained withdrawal cause to the run for its terminal fact.

use crate::boundary::{Exit, Hook, HookProblem, Place, Ran, Request, Returned};
use crate::call::{self, Call, Withdrawal};
use crate::conventions;
use crate::delivery::{CallName, Delivery, DeliveryFailure, DeliveryReason};
use crate::limits::Limits;
use crate::outcome::Change;
use crate::prepare;
use crate::run::Run;
use crate::workspace::{self, Directory, HookTime, HookTimes};
use alloc::boxed::Box;
use core::mem;
use skein_lib::bytes::copy_of;
use skein_lib::{Duration, Env, Id, Queue, Time, Token};

#[derive(Debug)]
pub(crate) struct Landing {
    change: Change,
    name: CallName,
    deadline: Time,
    purpose: Purpose,
    stage: Stage,
    recovered: Option<Box<Delivery>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Finish,
    Deliver,
    Save,
}

#[derive(Debug)]
enum Stage {
    Cancelling { why: Option<Withdrawal> },
    Checking { check: u32, given: Duration, retry: bool },
    Aborting { why: Withdrawal },
    Delivering,
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Settled {
    Going,
    Finished(Change),
    MidDelivered,
    Refused,
    Stale,
    Cancelled,
}

pub(crate) fn landing(
    change: Change,
    name: CallName,
    deadline: Time,
    purpose: Purpose,
    recovered: Option<Box<Delivery>>,
) -> Landing {
    Landing { change, name, deadline, purpose, stage: Stage::Closed, recovered }
}

pub(crate) fn cancel_jobs(landing: &mut Landing, peer: Token, out: &mut Queue<Request>) {
    landing.stage = Stage::Cancelling { why: None };
    out.push(Request::CancelJobs { peer });
}

pub(crate) fn jobs_cancelled(
    landing: &mut Landing,
    id: Id<Call>,
    call: Token,
    run: &Run,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> Settled {
    match mem::replace(&mut landing.stage, Stage::Closed) {
        Stage::Cancelling { why: Some(why) } => back(call, call::stopped(why), Settled::Cancelled, out),
        Stage::Cancelling { why: None } => {
            if let Some(delivery) = landing.recovered.take() {
                landing.stage = Stage::Delivering;
                delivered(landing, call, *delivery, run, out)
            } else if env.now >= landing.deadline {
                back(call, Returned::TimedOut, Settled::Cancelled, out)
            } else {
                next(landing, id, call, run, 0, env, out)
            }
        }
        Stage::Checking { .. } | Stage::Aborting { .. } | Stage::Delivering | Stage::Closed => {
            unreachable!("job settlement precedes checks and submission")
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "a cell handler takes the fields it touches")]
pub(crate) fn checked(
    landing: &mut Landing,
    id: Id<Call>,
    owner: Token,
    run: &Run,
    may_finish: bool,
    ran: Ran,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> Settled {
    let stage = mem::replace(&mut landing.stage, Stage::Closed);
    match stage {
        Stage::Checking { check, given, retry } => {
            let hooks = hook_times(run, check);
            let time = moment_time(hooks, landing.purpose);
            if ran.exit == Exit::TimedOut
                && !retry
                && may_finish
                && let Some(until) = crate::run::check_retry_deadline(run)
            {
                let cap = landing.deadline.min(until);
                let mut bound = given.saturating_add(given).min(cap.saturating_since(env.now));
                if let Some(hooks) = hooks
                    && time.is_some()
                {
                    bound = bound.min(hooks.ceiling);
                }
                if bound > given {
                    landing.stage = start_check(id, run, landing.purpose, check, bound, cap, true, env, out);
                    return Settled::Going;
                }
            }
            let passed = match ran.exit {
                Exit::Code { code: 0 } => true,
                Exit::Code { .. } | Exit::Signalled { .. } | Exit::TimedOut | Exit::Unstarted | Exit::Cancelled => {
                    false
                }
            };
            let required = match &run.charter.outcome.change {
                Some(spec) => spec.checks_must_pass,
                None => true,
            };
            if !passed && required {
                let repository = copy_of(&repository(run, check).name);
                let check = check_path(run, check, landing.purpose);
                let ceiling_cut = match time {
                    Some(time) if time.asked > time.time => Some(time.asked),
                    Some(_) | None => None,
                };
                let result = match ran.exit {
                    Exit::Code { .. } if !ran.said_not_judged => {
                        Returned::ChecksFailed { repository, check, given, ceiling_cut, ran }
                    }
                    Exit::Code { .. } | Exit::Signalled { .. } | Exit::TimedOut | Exit::Unstarted | Exit::Cancelled => {
                        Returned::ChecksNotJudged { repository, check, given, ceiling_cut, ran }
                    }
                };
                back(owner, result, Settled::Refused, out)
            } else if env.now >= landing.deadline {
                back(owner, Returned::TimedOut, Settled::Cancelled, out)
            } else if !may_finish {
                back(owner, Returned::Cancelled, Settled::Cancelled, out)
            } else {
                next(landing, id, owner, run, check.checked_add(1).expect("bounded mounted checks"), env, out)
            }
        }
        Stage::Aborting { why } => back(owner, call::stopped(why), Settled::Cancelled, out),
        Stage::Cancelling { .. } | Stage::Delivering | Stage::Closed => unreachable!("checks end only while they run"),
    }
}

pub(crate) fn aborted(landing: &mut Landing, owner: Token, out: &mut Queue<Request>) -> Exit {
    let stage = mem::replace(&mut landing.stage, Stage::Closed);
    match stage {
        Stage::Aborting { why } => {
            back(owner, call::stopped(why), Settled::Cancelled, out);
            match why {
                Withdrawal::Withdrawn => Exit::Cancelled,
                Withdrawal::Expired => Exit::TimedOut,
            }
        }
        Stage::Cancelling { .. } | Stage::Checking { .. } | Stage::Delivering | Stage::Closed => {
            unreachable!("terminal follows an abort")
        }
    }
}

pub(crate) fn delivered(
    landing: &mut Landing,
    owner: Token,
    delivery: Delivery,
    run: &Run,
    out: &mut Queue<Request>,
) -> Settled {
    let stage = mem::replace(&mut landing.stage, Stage::Closed);
    match stage {
        Stage::Delivering => {}
        Stage::Cancelling { .. } | Stage::Checking { .. } | Stage::Aborting { .. } | Stage::Closed => {
            unreachable!("terminal follows submission")
        }
    }
    match delivery {
        Delivery::Delivered(receipts) => {
            for receipt in receipts.receipts() {
                if !writable(run, receipt.directory()) {
                    return malformed(run, owner, out);
                }
            }
            let settled = if landing.purpose == Purpose::Finish {
                Settled::Finished(landing.change.clone())
            } else {
                Settled::MidDelivered
            };
            back(owner, Returned::Delivered(receipts), settled, out)
        }
        Delivery::Nothing => back(owner, Returned::Nothing, Settled::Refused, out),
        Delivery::Refused(refusal) => {
            if let Some(marker) = refusal.marker()
                && !writable(run, marker.directory())
            {
                return malformed(run, owner, out);
            }
            back(owner, Returned::DeliveryRefused(refusal), Settled::Refused, out)
        }
        Delivery::Failed(failure) => {
            if !writable(run, failure.directory) {
                return malformed(run, owner, out);
            }
            back(owner, Returned::DeliveryFailed { failure }, Settled::Refused, out)
        }
        Delivery::Stale => back(owner, Returned::Stale, Settled::Stale, out),
    }
}

fn malformed(run: &Run, owner: Token, out: &mut Queue<Request>) -> Settled {
    let mut directory = 0;
    for (position, mounted) in workspace::directories(run.workspace.as_ref()).iter().enumerate() {
        if mounted.writable {
            directory = u32::try_from(position).expect("bounded directory ordinal fits");
            break;
        }
    }
    back(
        owner,
        Returned::DeliveryFailed { failure: DeliveryFailure::new(directory, DeliveryReason::Broken) },
        Settled::Refused,
        out,
    )
}

fn writable(run: &Run, directory: u32) -> bool {
    let position = usize::try_from(directory).expect("bounded directory ordinal fits");
    match workspace::directories(run.workspace.as_ref()).get(position) {
        Some(repository) => repository.writable,
        None => false,
    }
}

pub(crate) fn withdraw(landing: &mut Landing, id: Id<Call>, why: Withdrawal, out: &mut Queue<Request>) {
    let stage = mem::replace(&mut landing.stage, Stage::Closed);
    landing.stage = match stage {
        Stage::Cancelling { why: first } => Stage::Cancelling { why: first.or(Some(why)) },
        Stage::Checking { .. } => {
            out.push(Request::Abort { owner: id.token() });
            Stage::Aborting { why }
        }
        // Submission owes its terminal. Caller-only withdrawal or expiry
        // grants no authority to decide the run's independent shutdown outcome.
        Stage::Delivering => Stage::Delivering,
        Stage::Aborting { why: first } => {
            assert!(why == Withdrawal::Withdrawn, "withdraw cancels the call alarm");
            Stage::Aborting { why: first }
        }
        Stage::Closed => unreachable!("only a live call is withdrawn"),
    };
}

fn next(
    landing: &mut Landing,
    id: Id<Call>,
    call: Token,
    run: &Run,
    first: u32,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> Settled {
    run.lease.assert_workspace(id.token());
    if landing.purpose != Purpose::Save {
        for check in first..run.found.checks.len() {
            let found = run.found.checks.get(check).expect("bounded mounted checks");
            let hook = match &found.kind {
                prepare::Kind::Legacy => Hook::Executable,
                prepare::Kind::Hooks(hooks) => match landing.purpose {
                    Purpose::Finish => hooks.finish,
                    Purpose::Deliver => hooks.deliver,
                    Purpose::Save => unreachable!("save runs no checks"),
                },
            };
            let times = hook_times(run, check);
            if hook != Hook::Missing
                && let Some(times) = times
                && !times.fault.is_empty()
            {
                return broken(run, check, landing.purpose, call, &times.fault, out);
            }
            match hook {
                Hook::Missing => {}
                Hook::Broken(problem) => return broken(run, check, landing.purpose, call, hook_problem(problem), out),
                Hook::Executable => {
                    let cap = landing.deadline.min(run.deadline);
                    let bound = match moment_time(times, landing.purpose) {
                        Some(time) => time.time,
                        None => checks_bound(run, env),
                    };
                    let bound = bound.min(cap.saturating_since(env.now));
                    landing.stage = start_check(id, run, landing.purpose, check, bound, cap, false, env, out);
                    return Settled::Going;
                }
            }
        }
    }
    let deadline = landing.deadline.min(env.now.saturating_add(env.limits.delivery_timeout));
    assert!(env.now < deadline, "admission and completed checks refuse expired submission");
    if landing.purpose == Purpose::Save {
        out.push(Request::Save { host_run: run.host_name, owner: id.token(), name: landing.name, deadline });
    } else {
        out.push(Request::Deliver {
            host_run: run.host_name,
            owner: id.token(),
            name: landing.name,
            deadline,
            change: landing.change.clone(),
        });
    }
    landing.stage = Stage::Delivering;
    Settled::Going
}

fn broken(run: &Run, check: u32, purpose: Purpose, call: Token, output: &[u8], out: &mut Queue<Request>) -> Settled {
    let ran = Ran { exit: Exit::Unstarted, said_not_judged: false, output: copy_of(output), cut: 0 };
    let repository = copy_of(&repository(run, check).name);
    let check = check_path(run, check, purpose);
    back(
        call,
        Returned::ChecksFailed { repository, check, given: Duration::ZERO, ceiling_cut: None, ran },
        Settled::Refused,
        out,
    )
}

fn hook_problem(problem: HookProblem) -> &'static [u8] {
    match problem {
        HookProblem::Unreadable => b"the hook could not be read",
        HookProblem::NotExecutable => b"the hook is not executable",
        HookProblem::NotFile => b"the hook is not a regular file",
        HookProblem::BadLink => b"the hook does not link directly to an executable file beside it",
    }
}

fn hook_times(run: &Run, check: u32) -> Option<&HookTimes> {
    let found = run.found.checks.get(check).expect("only discovered checks run");
    match &found.kind {
        prepare::Kind::Legacy => None,
        prepare::Kind::Hooks(_) => repository(run, check).hooks.as_ref(),
    }
}

fn moment_time(hooks: Option<&HookTimes>, purpose: Purpose) -> Option<HookTime> {
    let hooks = hooks?;
    match purpose {
        Purpose::Finish => hooks.finish,
        Purpose::Deliver => hooks.deliver,
        Purpose::Save => unreachable!("save runs no checks"),
    }
}

fn check_path(run: &Run, check: u32, purpose: Purpose) -> Box<[u8]> {
    let found = run.found.checks.get(check).expect("only discovered checks run");
    match &found.kind {
        prepare::Kind::Legacy => copy_of(conventions::checks(&run.charter)),
        prepare::Kind::Hooks(_) => {
            let directory = conventions::hooks(&run.charter).expect("hooks directory selected");
            let name: &[u8] = match purpose {
                Purpose::Finish => b"finish",
                Purpose::Deliver => b"deliver",
                Purpose::Save => unreachable!("save runs no checks"),
            };
            skein_lib::bytes::concat(&[directory, b"/", name])
        }
    }
}

fn checks_bound(run: &Run, env: &Env<Limits>) -> Duration {
    run.charter.check_timeout.unwrap_or(env.limits.check_timeout)
}

#[expect(clippy::too_many_arguments, reason = "the check's held bound and absolute cap travel separately")]
fn start_check(
    id: Id<Call>,
    run: &Run,
    purpose: Purpose,
    check: u32,
    bound: Duration,
    cap: Time,
    retry: bool,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> Stage {
    run.lease.assert_workspace(id.token());
    let directory = repository(run, check);
    let root = directory.root;
    let results = directory.results.clone();
    let program = Place { root, path: check_path(run, check, purpose) };
    let deadline = env.now.saturating_add(bound).min(cap);
    out.push(Request::Check { owner: id.token(), program, results, bound, cap, tail: env.limits.check_tail });
    out.push(Request::Checking { host_run: run.host_name, deadline });
    Stage::Checking { check, given: bound, retry }
}

fn back(owner: Token, result: Returned, settled: Settled, out: &mut Queue<Request>) -> Settled {
    out.push(Request::Return { spent: 0, call: owner, result });
    settled
}

fn repository(run: &Run, check: u32) -> &Directory {
    let index = run.found.checks.get(check).expect("only discovered checks run");
    repository_at(run, index.directory)
}

fn repository_at(run: &Run, index: u32) -> &Directory {
    workspace::directories(run.workspace.as_ref())
        .get(usize::try_from(index).expect("bounded mount"))
        .expect("checks belong to admitted mounts")
}
