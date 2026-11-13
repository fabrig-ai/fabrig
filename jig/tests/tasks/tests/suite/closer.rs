//! Closer ending authority is retained and checked at making (jig/docs/design/domain/tasks/lifecycle.md, 11).
use jig_domain_tasks::{self as tasks, EndedBy, Executor, Party, Problem, Refusal};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn member() -> tasks::New {
    let mut member = task(1, &[]);
    member.flags.ended_by = EndedBy::Closer(tasks::Closer { connector: 1, code: 6 });
    member.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Procedure(6)]);
    member
}
#[test]
fn a_task_made_with_a_closer_keeps_it_through_restart() {
    let mut world = World::new(861, LIMITS);
    let member = member();
    let flags = member.flags;
    assert_eq!(world.make(Party::Person(1), vec![member]), Reply::Made(vec![1]));
    assert_eq!(world.record(1).flags, flags);
    assert_eq!(world.record(1).closer, None);
    let record = world.record(1).clone();
    world.restart();
    assert_eq!(world.record(1), &record);
}
#[test]
fn a_closer_on_a_conversation_is_refused() {
    let mut member = member();
    member.flags.conversation = true;
    refused(member);
}
#[test]
fn a_closer_on_a_task_that_is_no_agents_is_refused() {
    for executor in [Executor::Person(tasks::PersonAddress::Person(1)), Executor::Procedure { connector: 1, code: 6 }] {
        let mut member = member();
        member.executor = executor;
        refused(member);
    }
}
fn refused(member: tasks::New) {
    let mut world = World::new(861, LIMITS);
    let before = world.records.clone();
    assert_eq!(world.make(Party::Person(1), vec![member]), Reply::Refused(Problem::new(Some(1), Refusal::Contract)));
    assert_eq!(world.records, before);
}

#[test]
fn a_closer_its_tasks_delegation_does_not_name_is_refused() {
    use jig_core_kit::{
        fixtures,
        world::{Setup, World as CoreWorld},
    };
    use jig_domain_people as people;
    let mut world = CoreWorld::new(Setup { seed: 861, ..Setup::default() });
    let people::Ask::Create { project, mut batch } = fixtures::chat(&world, 1, b"close".as_slice().into()) else {
        panic!("create fixture");
    };
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Closer(people::Closer { connector: 1, code: 6 });
    batch[0].authority.delegation.kinds = Box::new([]);
    fixtures::ask(&mut world, 91, people::Ask::Create { project, batch });
    fixtures::refusal(
        &world,
        people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::TaskKind {
                executor: Some(people::TaskExecutor::Procedure { connector: 1, code: 6 }),
            },
        },
    );
}

#[test]
fn a_delegates_closer_must_be_named_by_its_own_delegation() {
    use jig_core_kit::{
        fixtures,
        world::{Setup, World as CoreWorld},
    };
    let mut world = CoreWorld::new(Setup { seed: 861, ..Setup::default() });
    let mut member = fixtures::member(&world, Executor::Agent { charter: 1 }, b"close");
    member.flags.ended_by = EndedBy::Closer(tasks::Closer { connector: 1, code: 6 });
    let key = world.call_key(1);
    let limits = world.env().limits.core.tasks;
    let findings = Box::new([jig_domain_authority::Finding::Executor { source: jig_domain_authority::Source::Task }]);
    assert_eq!(
        world.domain.core_mut().delegate_preflight(&limits, key, &[member.clone()], skein_lib::Wall::EPOCH),
        Err(Box::new(jig_domain_core::CallPart::DelegationDenied {
            answer: jig_domain_authority::Answer::Refuse,
            findings: findings.clone(),
        })),
    );
    assert_eq!(
        world.domain.core_mut().procedure_batch_admit(&limits, key.task, &[member], skein_lib::Wall::EPOCH),
        Err(jig_domain_core::StepRefusal::Denied { answer: jig_domain_authority::Answer::Refuse, findings }),
    );
}

const TASK: u64 = 1;
const WORK: u64 = 2;

fn working() -> World {
    working_for(false)
}

fn working_for(requester: bool) -> World {
    let mut world = World::new(862, LIMITS);
    if requester {
        let mut root = task(100, &[]);
        root.authority.budget.spend = 2_000;
        root.numbers.budget = 2_000;
        world.make(jig_tasks_world::PERSON_PARTY, vec![root]);
    }
    let mut parent = member();
    parent.authority.budget.spend = 1_000;
    parent.numbers.budget = 1_000;
    parent.wake.results = tasks::ResultsWake::LastOrFailure;
    let creator = if requester {
        parent.funder = tasks::Funder::Task(100);
        Party::Task(100)
    } else {
        jig_tasks_world::PERSON_PARTY
    };
    assert_eq!(world.make(creator, vec![parent]), Reply::Made(vec![TASK]));
    let mut child = task(WORK, &[]);
    child.funder = tasks::Funder::Task(TASK);
    assert_eq!(world.make(Party::Task(TASK), vec![child]), Reply::Made(vec![WORK]));
    world.claim(TASK, 1);
    world
}

fn read_inbox(world: &mut World) {
    let read = world.record(TASK).last_message;
    let attempt = world.record(TASK).attempt;
    if read != 0 {
        let outstanding = world.record(TASK).inbox.iter().map(|word| word.number).collect();
        jig_tasks_world::response(world, |reply_to| tasks::Event::Turn {
            reply_to,
            task: TASK,
            attempt,
            turn: 1,
            read: Some(read),
            offered: Some(read),
            refused: Box::new([]),
            outstanding,
            cumulative: 0,
        });
    }
}

fn parked_closer() -> World {
    park_after_work(working())
}

fn park_after_work(mut world: World) -> World {
    world.claim(WORK, 1);
    world.finish(WORK);
    world.settle(WORK);
    read_inbox(&mut world);
    world.terminal(TASK, tasks::End::Parked);
    world
}

fn live_closer(world: &World) -> u64 {
    world.record(TASK).closer.expect("the idle task has its own closer")
}

fn result_step(world: &mut World, closer: u64, words: &[u8]) -> Reply {
    jig_tasks_world::step(
        world,
        closer,
        tasks::ProcedureDecision::new(tasks::ProcedureEnd::Result(tasks::TaskResult::Report { words: words.into() })),
    )
}

#[test]
fn a_closer_is_made_where_the_notice_would_be() {
    let world = parked_closer();
    let parent = world.record(TASK);
    let closer = live_closer(&world);
    let row = world.record(closer);
    assert_eq!(parent.phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(parent.tries, tasks::Tries::NONE);
    assert_eq!(parent.delegates.as_ref(), [closer]);
    assert_eq!(row.executor, Executor::Procedure { connector: 1, code: 6 });
    assert_eq!(row.requester, Party::Task(TASK));
    assert_eq!(row.spec.words.as_ref(), tasks::CLOSER_WORDS);
    assert!(row.spec.parameters.is_empty() && row.spec.inputs.is_empty());
    assert_eq!(row.contract, parent.contract);
    let mut expected = parent.authority.clone();
    expected.delegation.depth -= 1;
    expected.budget.spend = parent.numbers.budget - parent.numbers.spent - parent.numbers.spent_below;
    assert_eq!(row.authority, expected);
    assert_eq!(row.funder, tasks::Funder::Task(TASK));
    assert_eq!(row.numbers.budget, expected.budget.spend);
    assert_eq!(parent.attempt, 1);
    assert!(!parent.inbox.iter().any(|word| word.kind == tasks::MessageKind::NothingAwaited));
    assert_eq!(parent.last_message, 1, "only the work's result was numbered");
}

#[test]
fn an_idle_tasks_last_result_makes_its_closer_with_no_run() {
    let mut world = working();
    world.terminal(TASK, tasks::End::Parked);
    world.claim(WORK, 1);
    world.finish(WORK);
    world.settle(WORK);
    let parent = world.record(TASK);
    assert_eq!(parent.phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(parent.attempt, 1);
    assert!(parent.closer.is_some());
    assert!(parent.inbox.iter().any(
        |word| word.from == Party::Task(WORK) && word.kind == tasks::MessageKind::Result(tasks::ResultKind::Report)
    ));
}

#[test]
fn a_failure_still_wakes_a_closer_ended_task() {
    let mut world = working();
    world.terminal(TASK, tasks::End::Parked);
    world.claim(WORK, 1);
    world.terminal(
        WORK,
        tasks::End::Finished {
            result: tasks::TaskResult::Failure { reason: b"failed".as_slice().into() },
            cancel_delegates: false,
        },
    );
    world.settle(WORK);
    assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(world.record(TASK).closer, None);
    world.claim(TASK, 2);
    read_inbox(&mut world);
    world.terminal(TASK, tasks::End::Parked);
    assert!(world.record(TASK).closer.is_some());
}

#[test]
fn a_task_with_no_closer_keeps_the_notice() {
    let mut world = World::new(863, LIMITS);
    world.make(jig_tasks_world::PERSON_PARTY, vec![task(TASK, &[])]);
    world.claim(TASK, 1);
    world.terminal(TASK, tasks::End::Parked);
    let row = world.record(TASK);
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(row.tries.run, 1);
    assert_eq!(row.closer, None);
    assert!(row.inbox.iter().any(|word| word.kind == tasks::MessageKind::NothingAwaited));
}

#[test]
fn a_closers_success_ends_its_task_with_its_result() {
    let mut world = park_after_work(working_for(true));
    let closer = live_closer(&world);
    assert_eq!(result_step(&mut world, closer, b"closed"), Reply::Done);
    world.settle(closer);
    assert!(matches!(world.record(TASK).phase, tasks::Phase::Closing(_)));
    assert_eq!(world.record(TASK).closer, None);
    assert_eq!(world.record(TASK).attempt, 1);
    world.settle(TASK);
    assert_eq!(
        world.results[&TASK],
        tasks::Ending::Done(tasks::TaskResult::Report { words: b"closed".as_slice().into() })
    );
    let inbox = &world.record(100).inbox;
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].from, Party::Task(TASK));
    assert_eq!(inbox[0].kind, tasks::MessageKind::Result(tasks::ResultKind::Report));
    assert_eq!(inbox[0].words.as_ref(), b"closed");
}

#[test]
fn a_closers_result_past_its_contract_is_refused_at_its_step() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    let before = world.records.clone();
    assert_eq!(
        result_step(&mut world, closer, &[b'x'; 33]),
        Reply::Refused(Problem::new(Some(closer), Refusal::Contract))
    );
    assert_eq!(world.records, before);
    assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(world.record(TASK).closer, Some(closer));
}

#[test]
fn a_live_closer_survives_restart() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    let row = world.record(closer).clone();
    world.restart();
    assert_eq!(world.record(TASK).closer, Some(closer));
    assert_eq!(world.record(closer), &row);
    assert_eq!(result_step(&mut world, closer, b"restored"), Reply::Done);
    world.settle(closer);
    world.settle(TASK);
    assert_eq!(
        world.results[&TASK],
        tasks::Ending::Done(tasks::TaskResult::Report { words: b"restored".as_slice().into() })
    );
}

#[test]
fn a_closer_that_fails_spends_a_run_try_and_is_made_again() {
    let mut world = parked_closer();
    let mut previous = 0;
    for failure in 1..=world.env.limits.retries.run.retries + 1 {
        let closer = live_closer(&world);
        assert!(closer > previous);
        previous = closer;
        assert_eq!(
            jig_tasks_world::step(
                &mut world,
                closer,
                tasks::ProcedureDecision::new(tasks::ProcedureEnd::Result(tasks::TaskResult::Failure {
                    reason: b"retry".as_slice().into(),
                }))
            ),
            Reply::Done
        );
        world.settle(closer);
        assert_eq!(world.record(TASK).closer, None);
        assert_eq!(world.record(TASK).tries.run, failure);
        if failure > world.env.limits.retries.run.retries {
            assert!(matches!(
                world.record(TASK).phase,
                tasks::Phase::Closing(tasks::Closing { ending: tasks::Ending::Failed { .. }, .. })
            ));
            world.settle(TASK);
            assert!(matches!(world.results[&TASK], tasks::Ending::Failed { .. }));
            break;
        }
        assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Due));
        world.claim(TASK, u64::from(failure) + 1);
        read_inbox(&mut world);
        world.terminal(TASK, tasks::End::Parked);
        assert_eq!(world.record(TASK).tries.run, failure);
    }
}

fn route_hold(world: &mut World, task: u64) {
    world.send(tasks::Event::Hold { task, why: tasks::Hold::Procedure });
    let revision = world.record(task).escalation.revision();
    world.send(tasks::Event::RoutedEscalation {
        task,
        revision,
        holder: tasks::EscalationHolder::Task(TASK),
        entry: 100,
    });
}

fn release_hold(world: &mut World, task: u64) {
    assert_eq!(
        jig_tasks_world::response(world, |reply_to| tasks::Event::Release {
            reply_to,
            by: jig_tasks_world::PERSON_PARTY,
            operation: tasks::Release { task, again: None, message: None },
        }),
        Reply::Done
    );
}

#[test]
fn a_held_closer_escalates_to_its_task_once() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    route_hold(&mut world, closer);
    assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(world.record(TASK).closer, Some(closer));
    let wakes = world.activations(TASK);
    let escalation = world.record(closer).escalation.clone();
    world.send(tasks::Event::Hold { task: closer, why: tasks::Hold::Procedure });
    assert_eq!(world.record(closer).escalation, escalation);
    assert_eq!(world.activations(TASK), wakes);
    assert_eq!(
        world.contexts[&TASK]
            .inbox
            .iter()
            .filter(|word| matches!(word.kind, tasks::MessageKind::Escalation { task, .. } if task == closer))
            .count(),
        1
    );
    let mut child = task(closer + 1, &[]);
    child.funder = tasks::Funder::Task(closer);
    assert_eq!(world.make(Party::Task(closer), vec![child]), Reply::Made(vec![closer + 1]));
    route_hold(&mut world, closer + 1);
    assert!(matches!(
        world.record(closer + 1).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(TASK), .. }
    ));
    assert_eq!(world.activations(TASK), wakes);
}

#[test]
fn a_task_woken_by_its_held_closer_runs() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    let before = world.record(TASK).clone();
    assert_eq!(world.record(closer).numbers.budget, 1_000);
    let mut child = task(closer + 1, &[]);
    child.funder = tasks::Funder::Task(closer);
    assert_eq!(world.make(Party::Task(closer), vec![child]), Reply::Made(vec![closer + 1]));
    route_hold(&mut world, closer);
    world.claim_budget(TASK, 2, 100);
    let floor = world.record(closer).numbers;
    assert_eq!(floor.budget, floor.spent + floor.spent_below + floor.reserved);
    assert_eq!(floor.budget, 100);
    let parent = world.record(TASK);
    assert_eq!(parent.numbers.budget, before.numbers.budget);
    assert_eq!(parent.funder, before.funder);
    assert_eq!(parent.numbers.spent, before.numbers.spent);
    assert_eq!(parent.numbers.reserved, 200);
    assert_eq!(
        parent.numbers.budget - parent.numbers.spent - parent.numbers.spent_below - parent.numbers.reserved,
        800
    );
}

#[test]
fn an_idle_task_gives_what_it_has_left_to_its_closer() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    route_hold(&mut world, closer);
    world.claim_budget(TASK, 2, 100);
    assert_eq!(world.record(closer).numbers.budget, 0);
    let work = closer + 1;
    let mut child = task(work, &[]);
    child.funder = tasks::Funder::Task(TASK);
    assert_eq!(world.make(Party::Task(TASK), vec![child]), Reply::Made(vec![work]));
    read_inbox(&mut world);
    assert_eq!(world.terminal(TASK, tasks::End::Parked), Reply::Acknowledged(tasks::Accepted::New));
    let parent = world.record(TASK).numbers;
    assert_eq!(parent.budget - parent.spent - parent.spent_below - parent.reserved, 0);
    assert_eq!(world.record(closer).numbers.budget, 900);
    assert_eq!(
        parent.spent + parent.spent_below + world.record(closer).numbers.budget + world.record(work).numbers.budget,
        parent.budget
    );
    world.finish_task(work);
    let parent = world.record(TASK).numbers;
    assert_eq!(parent.budget - parent.spent - parent.spent_below - parent.reserved, 0);
    assert_eq!(world.record(closer).numbers.budget, 1_000);
    assert_eq!(parent.spent + parent.spent_below + world.record(closer).numbers.budget, parent.budget);
}

#[test]
fn a_delegate_made_while_the_closer_lives_waits_the_closer() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    route_hold(&mut world, closer);
    resize(&mut world, closer, 500, 10);
    world.claim_budget(TASK, 2, 100);
    let new_work = closer + 1;
    let mut child = task(new_work, &[]);
    child.funder = tasks::Funder::Task(TASK);
    let reply_to = world.to();
    world.stage(tasks::Event::Make { reply_to, creator: Party::Task(TASK), batch: Box::new([child]) });
    let staged = world.staged_requests();
    assert!(staged.iter().any(|request| matches!(request, tasks::Request::Save { record: tasks::Stored::Live(row) }
        if row.number == closer && row.waiting_on.as_ref() == [new_work])));
    assert!(staged.iter().any(|request| matches!(request, tasks::Request::Save { record: tasks::Stored::Live(row) }
        if row.number == new_work)));
    world.durable();
    world.deliver();
    assert_eq!(world.record(closer).dependencies.as_ref(), [new_work]);
    release_hold(&mut world, closer);
    assert_eq!(
        result_step(&mut world, closer, b"too soon"),
        Reply::Refused(Problem::naming(Some(closer), Refusal::Kept, &[new_work]))
    );
    world.finish_task(new_work);
    world.terminal(TASK, tasks::End::Parked);
    assert_eq!(result_step(&mut world, closer, b"all done"), Reply::Done);
    world.settle(closer);
    world.settle(TASK);
    assert_eq!(
        world.results[&TASK],
        tasks::Ending::Done(tasks::TaskResult::Report { words: b"all done".as_slice().into() })
    );
}

#[test]
fn a_refused_closer_holds_the_task() {
    let mut world = World::new(864, LIMITS);
    world.make(jig_tasks_world::PERSON_PARTY, vec![member()]);
    world.claim(TASK, 1);
    assert_eq!(
        jig_tasks_world::response(&mut world, |reply_to| tasks::Event::Turn {
            reply_to,
            task: TASK,
            attempt: 1,
            turn: 1,
            read: None,
            offered: None,
            refused: Box::new([]),
            outstanding: Box::new([]),
            cumulative: 100,
        }),
        Reply::Turn(tasks::Accepted::New)
    );
    world.terminal(TASK, tasks::End::Parked);
    assert_eq!(
        world.record(TASK).phase,
        tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Budget, source: None }
    );
    assert_eq!(world.record(TASK).closer, None);
    assert_eq!(world.record(TASK).last_message, 0);
    resize(&mut world, TASK, 200, 1);
    release_hold(&mut world, TASK);
    assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Due));
    world.claim(TASK, 2);
    read_inbox(&mut world);
    world.terminal(TASK, tasks::End::Parked);
    assert!(world.record(TASK).closer.is_some());
}

fn resize(world: &mut World, task: u64, spend: u64, message: u64) {
    let mut authority = world.record(task).authority.clone();
    authority.budget.spend = spend;
    assert_eq!(
        jig_tasks_world::response(world, |reply_to| tasks::Event::Amend {
            reply_to,
            by: jig_tasks_world::PERSON_PARTY,
            funder: None,
            places: Box::new([]),
            operation: tasks::Amend {
                task,
                message,
                stop_run: false,
                amendment: Box::new(tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    llm: None,
                    reason: Box::new([]),
                })
            },
        }),
        Reply::Done
    );
}

#[test]
fn a_batch_waiting_on_its_requesters_closer_is_a_circle() {
    let mut world = parked_closer();
    let closer = live_closer(&world);
    let before = world.records.clone();
    let reply = world.make(Party::Task(TASK), vec![task(closer + 1, &[closer])]);
    assert!(matches!(reply, Reply::Refused(Problem { why: Refusal::Circle { .. }, .. })), "{reply:?}");
    assert_eq!(world.records, before);
}

#[test]
fn an_idle_tasks_last_watch_ending_makes_its_closer() {
    let mut world = World::new(865, LIMITS);
    let mut parent = member();
    parent.wake.notices = tasks::WakeRule::Never;
    let mut root = task(100, &[]);
    root.authority.budget.spend = 1_000;
    root.numbers.budget = 1_000;
    world.make(jig_tasks_world::PERSON_PARTY, vec![root]);
    let mut work = task(WORK, &[]);
    parent.funder = tasks::Funder::Task(100);
    work.funder = tasks::Funder::Task(100);
    world.make(Party::Task(100), vec![parent, work]);
    assert_eq!(
        jig_tasks_world::response(&mut world, |reply_to| tasks::Event::Introduce {
            reply_to,
            by: 100,
            left: TASK,
            right: WORK,
        }),
        Reply::Done
    );
    assert_eq!(
        jig_tasks_world::response(&mut world, |reply_to| tasks::Event::Subscribe {
            reply_to,
            task: TASK,
            subscription: tasks::Subscription {
                number: 1,
                kind: tasks::SubscriptionKind::Task { target: WORK, held: false, result: true },
            },
        }),
        Reply::Done
    );
    world.claim(TASK, 1);
    world.terminal(TASK, tasks::End::Parked);
    assert_eq!(world.record(TASK).closer, None);
    world.finish_task(WORK);
    assert_eq!(world.record(TASK).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert!(world.record(TASK).closer.is_some());
    assert_eq!(world.record(TASK).attempt, 1);
}
