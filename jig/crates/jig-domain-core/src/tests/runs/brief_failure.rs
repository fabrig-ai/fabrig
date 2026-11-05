//! Brief failure classification at bounded core routes (domain/engine.md, 9.1).
use crate::tests::*;
use jig_domain_brief as brief;

fn preparing() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.begin_transcript(1, 0, None);
    (core, limits)
}

fn logged(out: &[Request], expected: brief::GatherFailure) {
    let mut count = 0_u32;
    for request in out {
        if let Request::Now(now) = request
            && let Now::BriefFailed { task, failure } = now.as_ref()
        {
            assert_eq!((*task, *failure), (1, expected));
            count = count.checked_add(1).unwrap();
        }
    }
    assert_eq!(count, 1);
}

#[test]
fn a_required_child_section_keeps_its_failure_and_releases_preparation() {
    let (mut core, limits) = preparing();
    let section = Token::new(19);
    watch_outputs(
        &mut core,
        &limits,
        Event::Brief(brief::GatherEvent::Plan {
            brief: Token::new(1),
            budget: 128,
            deadline: Time::from_nanos(100),
            sections: Box::new([brief::Planned::Connector {
                name: b"context",
                connector: 1,
                kind: 1,
                token: section,
                size: 0,
                limit: 128,
                priority: 0,
                required: true,
            }]),
        }),
    );
    let failure = brief::GatherFailure {
        fate: brief::GatherFate::Transient,
        section: b"context",
        words: b"not gathered by its deadline",
    };
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::Brief(brief::GatherEvent::Missing { brief: Token::new(1), section, why: failure }),
    );
    logged(&out, failure);
    assert_eq!(core.tasks.task(1).unwrap().tries.transient, 1);
    assert_eq!(
        core.tasks.task(1).unwrap().retry_notice,
        Some(tasks::RetryNotice::Failed {
            class: tasks::Class::Transient,
            count: 1,
            allowance: limits.tasks.retries.transient.retries,
            reason: Some(failure.reason()),
        })
    );
    assert!(core.contexts.is_empty());
    assert!(core.transcripts.is_empty());
}

#[test]
fn an_exhausted_run_number_counts_as_permanent_and_tells_the_root() {
    let (mut core, limits) = preparing();
    core.counters.deployment.runs = u64::MAX;
    let out = watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
    let failure = brief::GatherFailure {
        fate: brief::GatherFate::Permanent,
        section: b"run",
        words: b"no attempt number is left",
    };
    logged(&out, failure);
    assert_eq!(
        core.tasks.task(1).unwrap().retry_notice,
        Some(tasks::RetryNotice::Failed {
            class: tasks::Class::Permanent,
            count: 1,
            allowance: limits.tasks.retries.permanent.retries,
            reason: Some(failure.reason()),
        })
    );
    assert_eq!(core.tasks.task(1).unwrap().attempt, 0);
    assert!(core.workspace_flight(1).is_none());
    assert!(core.contexts.is_empty());
}

#[test]
fn a_full_result_read_table_pauses_with_its_reason() {
    use crate::routing::{
        flight::Flight,
        run::results::{ReadFor, ResultRead},
    };
    let (mut core, limits) = preparing();
    for task in 10..10 + core.flight_tasks_capacity {
        core.fly(Flight::ResultRead(ResultRead {
            why: ReadFor::Preparation { task: u64::from(task), last_message: 0 },
            ids: Box::new([12]),
            at: 0,
            results: List::with_capacity(1),
        }));
    }
    let mut work = Queue::uncapped();
    let out = crate::routing::run::results::begin(&mut core, &mut work, 1, Box::new([12]));
    assert!(out.is_empty());
    let Some(Work::PreparationFailed { task, failure }) = work.pop() else { panic!("no read flight failure") };
    assert_eq!(task, 1);
    assert_eq!(
        failure,
        brief::GatherFailure {
            fate: brief::GatherFate::Capacity,
            section: b"results",
            words: b"no read flight is free",
        }
    );
    let out = crate::routing::run::preparation::route_preparation_failed(&mut core, &mut work, task, failure);
    let Some(Work::Event(Event::Tasks(event))) = work.pop() else { panic!("tasks failure route") };
    send_task(&mut core, &limits, event);
    let mut out = out;
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    logged(requests.as_slice(), failure);
    assert_eq!(core.tasks.task(1).unwrap().tries, tasks::Tries::NONE);
    assert_eq!(
        core.tasks.task(1).unwrap().retry_notice,
        Some(tasks::RetryNotice::Refused { count: 1, reason: Some(failure.reason()) })
    );
}

#[test]
fn an_unread_core_section_counts_as_permanent() {
    let (mut core, mut limits) = preparing();
    limits.brief_parts = 0;
    let mut work = Queue::uncapped();
    let out = crate::routing::run::preparation::plan_brief(
        &core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        &mut work,
        1,
        List::with_capacity(0),
        0,
    );
    assert!(out.is_empty());
    let Some(Work::PreparationFailed { task, failure }) = work.pop() else { panic!("unread core section") };
    assert_eq!(task, 1);
    assert_eq!(
        failure,
        brief::GatherFailure {
            fate: brief::GatherFate::Permanent,
            section: b"task",
            words: b"could not be read within its bounds",
        }
    );
    let _out = crate::routing::run::preparation::route_preparation_failed(&mut core, &mut work, task, failure);
    let Some(Work::Event(Event::Tasks(event))) = work.pop() else { panic!("tasks failure route") };
    send_task(&mut core, &limits, event);
    assert_eq!(core.tasks.task(1).unwrap().tries.permanent, 1);
}

#[test]
fn a_full_proof_table_pauses_before_claiming_an_agent() {
    let (mut core, limits) = preparing();
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    for task in 10..10 + core.proofs.capacity() {
        let task = u64::from(task);
        core.proofs.insert(task, RunProof::claimed(task, 1, 1, None, fleet::HostKind::Worker)).unwrap();
    }
    watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
    let attempt = core.workspace_flight(1).unwrap().1;
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    let failure =
        brief::GatherFailure { fate: brief::GatherFate::Capacity, section: b"run", words: b"no room for its proof" };
    logged(&out, failure);
    assert_eq!(
        core.tasks.task(1).unwrap().retry_notice,
        Some(tasks::RetryNotice::Refused { count: 1, reason: Some(failure.reason()) })
    );
    assert_eq!(core.tasks.task(1).unwrap().tries, tasks::Tries::NONE);
    assert_eq!(core.tasks.task(1).unwrap().attempt, 0);
    assert!(!core.proofs.contains_key(&1));
}
