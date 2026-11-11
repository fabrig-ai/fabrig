//! One exclusive snapshot for checked delivery and unchecked saving (smith/docs/design/domain/run.md,
//! sections 8 and 9). State keeps opaque fields, durable name, caller deadline
//! and its purpose: finishing Change, delivery or save; a running check keeps its given span and retry marker.
//! It never knows receipt encoding,
//! provider ids, callback identity or host delivery policy. Main's write batch
//! waits while jobs settle, through checks and submission; children have already returned.
//! Before submission withdrawal aborts checks. Afterwards no cancellation is
//! emitted: the bounded host terminal settles even during shutdown.
//! A stopped check returns its retained withdrawal cause to the run for its terminal fact.

use crate::boundary::{Exit, Place, Ran, Request, Returned};
use crate::call::{self, Call, Withdrawal};
use crate::conventions;
use crate::delivery::{CallName, Delivery, DeliveryFailure, DeliveryReason};
use crate::limits::Limits;
use crate::outcome::Change;
use crate::run::Run;
use crate::workspace::{self, Directory};
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

pub(crate) fn landing(change: Change, name: CallName, deadline: Time, purpose: Purpose) -> Landing {
    Landing { change, name, deadline, purpose, stage: Stage::Closed }
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
            if env.now >= landing.deadline {
                back(call, Returned::TimedOut, Settled::Cancelled, out)
            } else {
                begin(landing, id, run, env, out);
                Settled::Going
            }
        }
        Stage::Checking { .. } | Stage::Aborting { .. } | Stage::Delivering | Stage::Closed => {
            unreachable!("job settlement precedes checks and submission")
        }
    }
}

pub(crate) fn begin(landing: &mut Landing, id: Id<Call>, run: &Run, env: &Env<Limits>, out: &mut Queue<Request>) {
    landing.stage = next(landing, id, run, 0, env, out);
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
            if ran.exit == Exit::TimedOut
                && !retry
                && may_finish
                && let Some(until) = crate::run::check_retry_deadline(run)
            {
                let deadline = landing.deadline.min(until).min(env.now.saturating_add(given.saturating_add(given)));
                if deadline.saturating_since(env.now) > given {
                    landing.stage = start_check(id, run, check, deadline, true, env, out);
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
                let check = copy_of(conventions::checks(&run.charter));
                back(owner, Returned::ChecksFailed { repository, check, given, ran }, Settled::Refused, out)
            } else if env.now >= landing.deadline {
                back(owner, Returned::TimedOut, Settled::Cancelled, out)
            } else if !may_finish {
                back(owner, Returned::Cancelled, Settled::Cancelled, out)
            } else {
                landing.stage = next(landing, id, run, check.checked_add(1).expect("bounded mounted checks"), env, out);
                Settled::Going
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

fn next(landing: &Landing, id: Id<Call>, run: &Run, check: u32, env: &Env<Limits>, out: &mut Queue<Request>) -> Stage {
    run.lease.assert_workspace(id.token());
    let check_index = if landing.purpose == Purpose::Save { None } else { run.found.checks.get(check) };
    let Some(_) = check_index else {
        let deadline = landing.deadline.min(env.now.saturating_add(env.limits.delivery_timeout));
        assert!(env.now < deadline, "admission and completed checks refuse expired submission");
        if landing.purpose == Purpose::Save {
            out.push(Request::Save { host_run: run.host_name, owner: id.token(), name: landing.name, deadline });
            return Stage::Delivering;
        }
        out.push(Request::Deliver {
            host_run: run.host_name,
            owner: id.token(),
            name: landing.name,
            deadline,
            change: landing.change.clone(),
        });
        return Stage::Delivering;
    };
    let deadline = landing.deadline.min(run.deadline).min(env.now.saturating_add(checks_bound(run, env)));
    start_check(id, run, check, deadline, false, env, out)
}

fn checks_bound(run: &Run, env: &Env<Limits>) -> Duration {
    run.charter.check_timeout.unwrap_or(env.limits.check_timeout)
}

fn start_check(
    id: Id<Call>,
    run: &Run,
    check: u32,
    deadline: Time,
    retry: bool,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> Stage {
    run.lease.assert_workspace(id.token());
    let root = repository(run, check).root;
    let program = Place { root, path: copy_of(conventions::checks(&run.charter)) };
    out.push(Request::Check { owner: id.token(), program, deadline, tail: env.limits.check_tail });
    out.push(Request::Checking { host_run: run.host_name, deadline });
    Stage::Checking { check, given: deadline.saturating_since(env.now), retry }
}

fn back(owner: Token, result: Returned, settled: Settled, out: &mut Queue<Request>) -> Settled {
    out.push(Request::Return { spent: 0, call: owner, result });
    settled
}

fn repository(run: &Run, check: u32) -> &Directory {
    let index = run.found.checks.get(check).expect("only discovered checks run");
    repository_at(run, *index)
}

fn repository_at(run: &Run, index: u32) -> &Directory {
    workspace::directories(run.workspace.as_ref())
        .get(usize::try_from(index).expect("bounded mount"))
        .expect("checks belong to admitted mounts")
}
