//! Load bounded live rows, assert required links, then rebuild derived state
//! (jig/docs/design/domain/engine.md, 6; jig/docs/design/domain/tasks.md, 12).
//! Historical rows stay in root storage. A fit refusal remains sticky.
use crate::domain::{Domain, Startup, Task, activate, publish, record, snapshot, task_mut};
use crate::{
    Active, Closing, Executor, Limits, Party, Phase, Problem, Refusal, Request, Stage, Stored, TaskRecord, Was,
};
use skein_lib::{Env, Queue};

pub(crate) mod admission;
pub(crate) mod failure;
pub(crate) mod fit;

pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>, stored: Stored, out: &mut Queue<Request>) {
    match domain.startup {
        Startup::Failed => return,
        Startup::Checked | Startup::Ready | Startup::Restoring => {}
    }
    assert!(domain.startup == Startup::Restoring, "restore row after load completed");
    let key = stored.key();
    if let Err(cause) = load(domain, &env.limits, stored) {
        failed(domain, crate::RestoreFailure { key, cause }, out);
    }
}

fn load(domain: &mut Domain, limits: &Limits, stored: Stored) -> Result<(), crate::RestoreCause> {
    let key = stored.key();
    match stored {
        Stored::PlacedHold(hold) => {
            assert!(!domain.placed.contains_key(&hold.number), "restore {key:?}: duplicate hold index");
            fit::placed(&hold, limits)?;
            fit::room(crate::RestoreBound::PlacedHolds, domain.placed.len(), domain.placed.capacity())?;
            domain.placed.insert(hold.number, *hold).expect("restored hold room");
        }
        Stored::PersonProposal(row) => {
            assert!(!domain.person_proposals.contains_key(&row.number), "restore {key:?}: duplicate proposal index");
            fit::person_proposal(&row, limits)?;
            fit::room(
                crate::RestoreBound::PersonProposals,
                domain.person_proposals.len(),
                domain.person_proposals.capacity(),
            )?;
            let inserted = domain.person_proposals.insert(row.number, *row);
            assert!(inserted == Ok(None), "restored person proposal admitted");
        }
        Stored::Live(task) => {
            let number = task.number;
            assert!(!domain.names.contains_key(&number), "restore {key:?}: duplicate live index");
            fit::room(crate::RestoreBound::Tasks, domain.tasks.len(), domain.tasks.capacity())?;
            fit::task(limits, &task)?;
            fit::settings(domain, &task)?;
            let observed_hold = match task.phase {
                Phase::Held { why, .. } => Some(why),
                Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
            };
            match task.executor {
                Executor::Agent { .. } => {
                    domain.live_agents = domain.live_agents.checked_add(1).expect("bounded restored agents");
                }
                Executor::Person(_) | Executor::Procedure { .. } => {}
            }
            crate::settings::hold(domain, &task);
            let id = domain
                .tasks
                .insert(Task {
                    published: Some(task.clone()),
                    procedure_offered: None,
                    observed_phase: Some(task.phase.clone()),
                    record: *task,
                    alarm: None,
                    grace_alarm: None,
                    observed_hold,
                })
                .expect("restored task admitted");
            let indexed = domain.names.insert(number, id);
            assert!(indexed == Ok(None), "restored name admitted");
            crate::waits::refresh(domain, number);
        }
        Stored::Stub(stub) => {
            assert!(!domain.stubs.contains_key(&stub.task), "restore {key:?}: duplicate stub index");
            fit::room(crate::RestoreBound::Stubs, domain.stubs.len(), domain.stubs.capacity())?;
            domain.stubs.insert(stub.task, stub).expect("bounded restored stub");
        }
        Stored::Writer(slot) => crate::writers::restore(domain, limits, slot)?,
        Stored::Pool(row) => crate::holds::restore_pool(domain, limits, row)?,
        Stored::Ledger(record) => crate::funders::restore_funding(domain, record)?,
        Stored::Update(_)
        | Stored::EndedByParty(_)
        | Stored::EndedByProject(_)
        | Stored::EndedDelegate(_)
        | Stored::Delegate(_)
        | Stored::Result(_)
        | Stored::DecidedProposal(_)
        | Stored::ClosedLedger(_)
        | Stored::Ended(_)
        | Stored::Milestone(_)
        | Stored::History(_)
        | Stored::Activity(_) => {
            assert!(key.live(), "restore {key:?}: historical row in live load");
        }
    }
    Ok(())
}

fn failed(domain: &mut Domain, failure: crate::RestoreFailure, out: &mut Queue<Request>) {
    let task = match failure.key {
        crate::Key::Live(number) | crate::Key::Stub(number) => Some(number),
        crate::Key::Ledger(_)
        | crate::Key::Writer(_)
        | crate::Key::Pool(_)
        | crate::Key::PersonProposal(_)
        | crate::Key::PlacedHold(_) => None,
        crate::Key::Ended(_)
        | crate::Key::ClosedLedger(_)
        | crate::Key::DecidedProposal(_)
        | crate::Key::History { .. }
        | crate::Key::Milestone { .. }
        | crate::Key::Activity { .. }
        | crate::Key::Update { .. }
        | crate::Key::Result { .. }
        | crate::Key::EndedByParty { .. }
        | crate::Key::EndedByProject { .. }
        | crate::Key::EndedDelegate { .. }
        | crate::Key::Delegate { .. } => {
            assert!(failure.key.live(), "fit failures belong to live rows");
            None
        }
    };
    if domain.restore_failure.is_none() {
        domain.restore_failure = Some(failure);
    }
    domain.startup = Startup::Failed;
    out.push(Request::RestoreRefused {
        problem: Problem { task, why: Refusal::Restore, blocked_by: None, capacity: None },
    });
}

fn live_link(domain: &Domain, key: crate::Key, target: u64) {
    assert!(record(domain, target).is_some(), "restore {key:?}: missing task {target}");
}

fn retained_link(domain: &Domain, key: crate::Key, target: u64) {
    assert!(
        record(domain, target).is_some() || domain.stubs.contains_key(&target),
        "restore {key:?}: missing task or stub {target}"
    );
}

fn links(domain: &Domain, task: &TaskRecord) {
    let key = crate::Key::Live(task.number);
    live_link(domain, key, task.root);
    match task.executor {
        Executor::Agent { charter } => {
            assert!(
                domain.charter_models(charter, task.settings.task).is_some(),
                "restore {key:?}: missing charter {charter} at revision {}",
                task.settings.task
            );
            if let Some(revision) = task.settings.run {
                assert!(
                    domain.charter_models(charter, revision).is_some(),
                    "restore {key:?}: missing run charter {charter} at revision {revision}"
                );
            }
        }
        Executor::Procedure { .. } | Executor::Person(_) => {}
    }
    match task.requester {
        Party::Task(parent) => live_link(domain, key, parent),
        Party::Person(_) | Party::Deployment { .. } => {}
    }
    for target in &task.delegates {
        live_link(domain, key, *target);
    }
    for target in &task.waiting_on {
        live_link(domain, key, *target);
    }
    for target in &task.references {
        retained_link(domain, key, *target);
    }
    // Completed dependencies and historical inputs are store-read identities;
    // only waiting_on consumes a live dependency here (domain/tasks.md, 5.7).

    if let Some(proposal) = &task.proposal {
        match proposal.state {
            crate::ProposalState::Pending { holder, .. } => match holder {
                crate::ProposalHolder::Task(holder) => live_link(domain, key, holder),
                crate::ProposalHolder::Person(_) | crate::ProposalHolder::Policy { .. } => {}
            },
            crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Rejected { .. }
            | crate::ProposalState::Withdrawn => {}
        }
        proposal_links(domain, key, &proposal.action);
    }
    match task.escalation {
        crate::Escalation::Waiting { holder, .. } => match holder {
            crate::EscalationHolder::Task(holder) => live_link(domain, key, holder),
            crate::EscalationHolder::Person(_) | crate::EscalationHolder::Role { .. } => {}
        },
        crate::Escalation::Unheld { .. } | crate::Escalation::Routing { .. } | crate::Escalation::Rejected { .. } => {}
    }
}

fn batch_links(domain: &Domain, key: crate::Key, batch: &[crate::New]) {
    for member in batch {
        for targets in [&*member.spec.inputs, &*member.dependencies] {
            for target in targets {
                let mut proposed = false;
                for other in batch {
                    proposed |= other.number == *target;
                }
                if !proposed {
                    retained_link(domain, key, *target);
                }
            }
        }
    }
}

fn proposal_links(domain: &Domain, key: crate::Key, action: &crate::ProposalAction) {
    match action {
        crate::ProposalAction::Batch(batch) => batch_links(domain, key, batch),
        crate::ProposalAction::Amend { task, .. }
        | crate::ProposalAction::Widen { task, .. }
        | crate::ProposalAction::Release { task }
        | crate::ProposalAction::ReleasePlaced { task, .. } => live_link(domain, key, *task),
        crate::ProposalAction::Settings { .. }
        | crate::ProposalAction::Effect { .. }
        | crate::ProposalAction::ProcedureEffect { .. } => {}
    }
}

pub(crate) fn check(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.startup {
        Startup::Checked | Startup::Ready | Startup::Failed => return,
        Startup::Restoring => {}
    }
    for (number, _) in &domain.names {
        links(domain, record(domain, *number).expect("name indexes task"));
    }
    for (number, row) in &domain.person_proposals {
        let key = crate::Key::PersonProposal(*number);
        match &row.action {
            crate::PersonAction::Batch(batch) => batch_links(domain, key, batch),
            crate::PersonAction::Amend { task, .. } => live_link(domain, key, *task),
            crate::PersonAction::Settings { .. } => {}
        }
    }
    crate::placed::link(domain);
    crate::writers::link(domain);
    crate::funders::link(domain);
    if let Err(failure) = crate::placed::fits(domain, &env.limits) {
        return failed(domain, failure, out);
    }
    if let Err(failure) = crate::funders::restore_fits(domain) {
        return failed(domain, failure, out);
    }
    for (number, _) in &domain.names {
        let task = record(domain, *number).expect("name indexes task");
        let checked = linked_fit(domain, &env.limits, task);
        if let Err(cause) = checked {
            return failed(domain, crate::RestoreFailure { key: crate::Key::Live(*number), cause }, out);
        }
    }
    crate::waits::restore(domain, env, out);
    domain.startup = Startup::Checked;
}

fn linked_fit(domain: &Domain, limits: &Limits, task: &TaskRecord) -> Result<(), crate::RestoreCause> {
    let mut tasks = 0_u64;
    let mut goals = 0_u64;
    for (other, _) in &domain.names {
        let other = record(domain, *other).expect("name indexes task");
        if other.project == task.project {
            tasks = tasks.checked_add(1).ok_or(crate::RestoreCause::Accounting)?;
            goals = goals.checked_add(u64::from(other.tracked.is_some())).ok_or(crate::RestoreCause::Accounting)?;
        }
    }
    fit::bound(crate::RestoreBound::ProjectTasks, tasks, u64::from(limits.project_tasks))?;
    fit::bound(crate::RestoreBound::ProjectGoals, goals, u64::from(limits.project_goals))?;
    fit::linked_inbox(domain, limits, task)?;
    fit::choice(domain, task)
}

pub(crate) fn restored(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.startup {
        Startup::Ready | Startup::Failed => return,
        Startup::Restoring | Startup::Checked => {}
    }
    assert!(domain.startup == Startup::Checked, "restored completion before link");
    domain.startup = Startup::Ready;
    crate::holds::restore(domain, env);
    crate::subscriptions::restore(domain, env);
    crate::wake::restore(domain, env);
    let numbers = snapshot(domain, env.limits.tasks);
    for number in numbers.into_boxed() {
        crate::grace::schedule(domain, env, number);
        let phase = record(domain, number).expect("restored name exists").phase.clone();
        match phase {
            Phase::Active(Active::Due | Active::Preparing) => {
                let row = &mut task_mut(domain, number).expect("restored name exists").record;
                let next = crate::run::ends::for_record(row).amended(row.executor, Phase::Active(Active::Due));
                let changed = row.phase != next;
                row.phase = next;
                let due = row.phase == Phase::Active(Active::Due);
                if changed {
                    publish(domain, env, number, out);
                }
                if due {
                    activate(domain, number, out);
                }
            }
            Phase::Active(Active::Claimed { attempt } | Active::Running { attempt }) => {
                out.push(Request::Adopt {
                    task: number,
                    attempt,
                    kept: record(domain, number).expect("restored live task").turn,
                });
            }
            Phase::Active(Active::Idle | Active::BackingOff { .. }) => publish(domain, env, number, out),
            Phase::Closing(closing) => match closing.stage {
                Stage::Run { attempt } => {
                    out.push(Request::Adopt {
                        task: number,
                        attempt,
                        kept: record(domain, number).expect("restored live task").turn,
                    });
                    out.push(Request::Stop { task: number, attempt });
                }
                Stage::Effects => out.push(Request::Close { task: number, ending: closing.ending }),
                Stage::Releases => out.push(Request::Release { task: number, ending: closing.ending }),
                Stage::Delegates | Stage::Settled => {}
            },
            Phase::Held { was, .. } => match was {
                Was::Active(Active::Claimed { attempt } | Active::Running { attempt })
                | Was::Closing(Closing { stage: Stage::Run { attempt }, .. }) => {
                    out.push(Request::Adopt {
                        task: number,
                        attempt,
                        kept: record(domain, number).expect("restored live task").turn,
                    });
                    out.push(Request::Stop { task: number, attempt });
                }
                Was::Closing(Closing { stage: Stage::Effects, ending }) => {
                    out.push(Request::Close { task: number, ending });
                }
                Was::Waiting
                | Was::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. })
                | Was::Closing(Closing { stage: Stage::Delegates | Stage::Releases | Stage::Settled, .. }) => {}
            },
            Phase::Waiting | Phase::Ended(_) => {}
        }
    }
}

/// Bounded durable amendment payload shared by both proposal origins.
pub(crate) fn valid_amendment(limits: &Limits, amendment: &crate::Amendment) -> bool {
    (match amendment.addressed {
        Some(change) => crate::control::valid_address(change.address),
        None => true,
    }) && match &amendment.instructions {
        Some(instructions) => instructions.len() <= usize::try_from(limits.message_bytes).expect("u32 fits usize"),
        None => true,
    } && amendment.reason.len() <= usize::try_from(limits.message_bytes).expect("u32 fits usize")
        && match &amendment.spec {
            Some(spec) => crate::batch::valid_spec(limits, spec) && spec.inputs.is_empty(),
            None => true,
        }
        && match &amendment.authority {
            Some(authority) => crate::batch::valid_authority(limits, authority),
            None => true,
        }
        && match &amendment.dependencies {
            Some(dependencies) => dependencies.len() <= usize::try_from(limits.dependencies).expect("u32 fits usize"),
            None => true,
        }
        && match &amendment.wake {
            Some(wake) => crate::wake::valid(wake),
            None => true,
        }
}
