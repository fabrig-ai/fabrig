//! Job preparation and claims, without agent state
//! (jig/docs/design/domain/engine.md, 7.5).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

use crate::{Core, Event, Limits, Now, Request, routing::Work};

pub(crate) fn activate(core: &mut Core, work: &mut Queue<Work>, context: Box<tasks::RunContext>) -> Queue<Request> {
    let task = context.task;
    let Some(attempt) = crate::fresh(&mut core.counters, crate::Family::Run) else {
        work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects })));
        return Queue::uncapped();
    };
    assert!(core.contexts.insert(task, context.clone()).is_ok(), "bounded activation context");
    work.push(Work::Event(Event::Tasks(tasks::Event::Prepare {
        reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Acknowledge)),
        task,
    })));
    assert!(core.workspace_flight(task).is_none(), "one workspace flight per task");
    core.fly(crate::routing::flight::Flight::Workspace { task, attempt });
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::WorkspaceRequest { task, attempt, context })));
    out
}

/// Jobs wait for readiness findings, but deadlines hold even when their check
/// answers Wait. Hard findings use the run's resource-hold precedence.
pub(crate) fn admission(
    core: &Core,
    context: &tasks::RunContext,
    wall: skein_lib::Wall,
    writes: Box<[authority::Write]>,
) -> Result<(), crate::RunAdmission> {
    let mut current = List::with_capacity(core.authority.limits().writes);
    for mut write in writes {
        write.effect = crate::effects::resources::effect(core, context.project, write.effect);
        if current.push(write).is_err() {
            return Err(crate::RunAdmission::Hold(tasks::Hold::Effects));
        }
    }
    let mut findings = Queue::uncapped();
    let answer = authority::check_job(
        &core.authority,
        &authority::JobAsk {
            project: context.project,
            authority: crate::translate::tasks_authority(&context.authority),
            wall,
            credentials: core.run_credentials_check(context.task, core.run_now),
            writes: current.into_boxed(),
        },
        &mut findings,
    );
    if answer == authority::Answer::Allow {
        return Ok(());
    }
    let mut deadline = false;
    for finding in &findings {
        if *finding == authority::Finding::Deadline {
            deadline = true;
        }
    }
    if answer == authority::Answer::Wait && !deadline {
        return Err(crate::RunAdmission::Wait(crate::RunWait::Credential));
    }
    Err(crate::runs::state::denied(&mut findings, crate::RunWait::Credential))
}

pub(crate) fn prepared(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    context: Box<tasks::RunContext>,
    attempt: u64,
    writes: Box<[authority::Write]>,
) -> Queue<Request> {
    let task = context.task;
    let mut names = List::with_capacity(env.limits.tasks.holdings);
    for write in &writes {
        let name = tasks::Name { connector: write.effect.connector, path: write.effect.name.segments.clone() };
        if !names.as_slice().contains(&name) && names.push(name).is_err() {
            return hold(work, task, tasks::Hold::Effects);
        }
        for resource in &write.effect.additional {
            let name = tasks::Name { connector: write.effect.connector, path: resource.name.segments.clone() };
            if !names.as_slice().contains(&name) && names.push(name).is_err() {
                return hold(work, task, tasks::Hold::Effects);
            }
        }
    }
    match admission(core, &context, env.wall, writes) {
        Ok(()) => {}
        Err(crate::RunAdmission::Wait(_)) => return super::preparation::wait_for_account(core, work, context),
        Err(crate::RunAdmission::Hold(why)) => return hold(work, task, why),
        Err(crate::RunAdmission::Allow(_)) => unreachable!("job admission has no run allowance"),
    }
    // Verify the current generation is grantable before the claim changes tasks.
    for number in core.run_credentials(task) {
        if core.accounts.credential_grant(*number, env.now).is_none() {
            return super::preparation::wait_for_account(core, work, context);
        }
    }
    if !core.proofs.contains_key(&task) && core.proofs.len() == core.proofs.capacity() {
        work.push(Work::Event(Event::Tasks(tasks::Event::PreparationFailed { task })));
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        return out;
    }
    let mut proof = super::preparation::start_proof(core, task, attempt, attempt, &[]);
    proof.credentials = core.workspace_credentials.remove(&task).expect("prepared job credentials");
    if let Some(previous) = core.proofs.insert(task, proof).expect("claim proof room checked") {
        assert!(core.replaced.insert(task, previous) == Ok(None), "one pending replacement per task");
    }
    drop(core.contexts.remove(&task));
    assert!(core.claiming.insert(task, attempt) == Ok(None), "one pending claim per task");
    work.push(Work::Event(Event::Tasks(tasks::Event::Claim {
        reply_to: ReplyTo::new(Token::new(task)),
        task,
        attempt,
        budget: 0,
        account: None,
        resumed: false,
        writes: names.into_boxed(),
    })));
    Queue::uncapped()
}

fn hold(work: &mut Queue<Work>, task: u64, why: tasks::Hold) -> Queue<Request> {
    work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why })));
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
    out
}

/// The connector handoff carries the start of a claimed job.
pub(crate) fn claimed(core: &Core, task: u64, attempt: u64, out: &mut Queue<Request>) {
    let row = core.tasks.task(task).expect("claimed job task");
    let kind = match row.executor {
        tasks::Executor::Job { kind } => jig_domain_link::JobKind(kind),
        tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => {
            unreachable!("job claim handoff")
        }
    };
    let spec = row.spec.job.as_ref().expect("job spec admitted with executor");
    let result = match row.contract {
        tasks::Contract::Job { bytes } => bytes,
        tasks::Contract::Report { .. }
        | tasks::Contract::Verdict { .. }
        | tasks::Contract::Change { .. }
        | tasks::Contract::Approval { .. } => unreachable!("job result contract"),
    };
    let mut grants = List::with_capacity(core.accounts.credential_limit());
    for number in core.run_credentials(task) {
        let grant = core.accounts.credential_grant(*number, core.run_now).expect("job claim checked credentials");
        if !grants.as_slice().contains(&grant) {
            grants.push(grant).expect("bounded job credentials");
        }
    }
    let grants = grants.into_boxed();
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: crate::Ask::JobClaimed {
                task,
                attempt,
                kind,
                spec: spec.bytes.clone(),
                deadline: spec.deadline,
                result,
                grants: grants.clone(),
            },
        });
    }
}
