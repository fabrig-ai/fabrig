//! Gatherings wait for their requester's word, never a step (tasks/gatherings.md, 2–5).
use jig_domain_tasks::{self as t, Event, Executor, Funder, Party, Phase, TaskResult};
use jig_tasks_world::{LIMITS, PERSON, PERSON_PARTY, Reply, World, response, task};
use skein_lib::Wall;

fn gathering() -> t::New {
    let mut new = task(1, &[]);
    new.executor = Executor::Procedure { connector: u16::MAX, code: 2 };
    new.flags.ended_by = t::EndedBy::Requester;
    new.contract = t::Contract::Report { words: 64 };
    new
}
fn world() -> World {
    let mut world = World::new(930, t::Limits { result_bytes: 64, ..LIMITS });
    assert_eq!(world.make(PERSON_PARTY, vec![gathering()]), Reply::Made(vec![1]));
    world
}
fn child(number: u64) -> t::New {
    let mut new = task(number, &[]);
    new.funder = Funder::Task(1);
    new.authority.budget.spend = 1;
    new.numbers.budget = 1;
    new
}
fn conclude(world: &mut World, by: u64) {
    assert_eq!(response(world, |reply_to| Event::Conclude { reply_to, by, task: 1 }), Reply::Done);
}
fn finish(world: &mut World, number: u64, result: TaskResult) {
    world.claim_task(number);
    world.terminal(number, t::End::Finished { result, cancel_delegates: false });
    world.settle(number);
}
fn report(world: &mut World, words: &[u8]) {
    world.settle(1);
    assert_eq!(world.results[&1], t::Ending::Done(TaskResult::Report { words: words.into() }));
}
fn cancel(world: &mut World, task: u64) {
    assert_eq!(
        response(world, |reply_to| Event::Cancel {
            reply_to,
            by: PERSON_PARTY,
            operation: t::Cancel { task, reason: Box::new([]), grace: None, message: None },
        }),
        Reply::Done
    );
}

#[test]
fn a_gathering_is_active_but_never_due_for_a_step() {
    let mut world = world();
    assert!(matches!(world.record(1).phase, Phase::Active(_)));
    assert_eq!(world.procedure_due(1), None);
    assert_eq!(
        response(&mut world, |reply_to| Event::Message {
            reply_to,
            project: 1,
            operation: t::Message {
                task: 1,
                word: t::Word {
                    number: 100,
                    from: PERSON_PARTY,
                    kind: t::MessageKind::Words,
                    words: b"wait".as_slice().into(),
                    at: Wall::EPOCH,
                    hits: 1,
                    eligible: true
                }
            },
        }),
        Reply::Done
    );
    assert_eq!(world.procedure_due(1), None);
    assert_eq!(world.make(Party::Task(1), vec![child(2)]), Reply::Made(vec![2]));
    finish(&mut world, 2, TaskResult::Report { words: Box::new([1]) });
    assert_eq!(world.procedure_due(1), None);
    assert_eq!(world.record(1).inbox.len(), 1, "party words stay, delegate results do not");
}

#[test]
fn a_gathering_refuses_departures_from_its_shape_without_making_anything() {
    for case in 0..9 {
        let mut world = World::new(931, t::Limits { result_bytes: 64, ..LIMITS });
        let mut new = gathering();
        let mut creator = PERSON_PARTY;
        match case {
            0 => new.flags.ended_by = t::EndedBy::Result,
            1 => {
                new.contract = t::Contract::Verdict {
                    choices: Box::new([t::Verdict { code: 1, words: 8, followups: 0, fields: Box::new([]) }]),
                }
            }
            2 => new.flags.conversation = true,
            3 => new.spec.asks = true,
            4 => {
                assert_eq!(world.make(PERSON_PARTY, vec![task(2, &[])]), Reply::Made(vec![2]));
                new.dependencies = Box::new([2]);
            }
            5 => {
                assert_eq!(world.make(PERSON_PARTY, vec![task(2, &[])]), Reply::Made(vec![2]));
                creator = Party::Task(2);
                new.funder = Funder::Task(2);
                new.numbers.budget = 1;
                new.authority.budget.spend = 1;
            }
            6 => new.executor = Executor::Procedure { connector: 1, code: 1 },
            7 => creator = Party::Person(0),
            8 => new.spec.parameters = Box::new([t::Parameter::Number { name: 1, value: 1 }]),
            _ => unreachable!(),
        }
        assert!(matches!(world.make(creator, vec![new]), Reply::Refused(_)), "case {case}");
        assert!(!world.records.contains_key(&t::Key::Live(1)), "case {case}");
    }
}

#[test]
fn concluding_an_empty_gathering_ends_it_with_the_exact_report() {
    let mut world = world();
    conclude(&mut world, PERSON);
    report(&mut world, b"Concluded: 0 done, 0 failed, 0 cancelled.");
}

#[test]
fn concluded_gatherings_keep_the_first_word_refuse_batches_and_end_with_the_last_delegate_after_restart() {
    let mut world = world();
    assert_eq!(world.make(Party::Task(1), vec![child(2), child(3)]), Reply::Made(vec![2, 3]));
    conclude(&mut world, PERSON);
    let first = world.record(1).concluded.expect("concluding word");
    world.env.wall = Wall::from_nanos(200);
    conclude(&mut world, PERSON + 1);
    assert_eq!(world.record(1).concluded, Some(first));
    assert!(matches!(world.make(Party::Task(1), vec![child(4)]), Reply::Refused(problem)
        if problem.why == t::Refusal::Concluded { at: first.at }));
    finish(&mut world, 2, TaskResult::Report { words: Box::new([1]) });
    assert!(matches!(world.record(1).phase, Phase::Active(_)));
    world.restart();
    assert_eq!(world.record(1).concluded, Some(first));
    finish(&mut world, 3, TaskResult::Failure { reason: b"failed".as_slice().into() });
    report(&mut world, b"Concluded: 1 done, 1 failed, 0 cancelled.");
}

#[test]
fn cancelling_a_gathering_closes_its_tree_concluded_or_not() {
    for concluded in [false, true] {
        let mut world = world();
        assert_eq!(world.make(Party::Task(1), vec![child(2)]), Reply::Made(vec![2]));
        if concluded {
            conclude(&mut world, PERSON);
        }
        cancel(&mut world, 1);
        world.settle(2);
        world.settle(1);
        assert!(matches!(world.results[&1], t::Ending::Cancelled { .. }));
        assert!(matches!(world.results[&2], t::Ending::Cancelled { .. }));
    }
}

#[test]
fn a_placed_hold_keeps_a_concluded_gathering_until_lift_or_release() {
    for release in [false, true] {
        let mut world = world();
        world.make(Party::Task(1), vec![child(2)]);
        conclude(&mut world, PERSON);
        assert_eq!(
            response(&mut world, |reply_to| Event::Place {
                reply_to,
                hold: t::PlacedHold {
                    number: 100,
                    project: 1,
                    owner: t::Owner::Party { person: PERSON },
                    mode: t::Mode::Stop,
                    scope: t::Scope::Subtree { task: 1 },
                    reason: Box::from(b"Parked".as_slice()),
                    placed_at: 0,
                    revision: 1,
                    exempt: Box::new([]),
                    lifted: false,
                }
            }),
            Reply::Done
        );
        cancel(&mut world, 2);
        world.settle(2);
        assert!(matches!(world.record(1).phase, Phase::Held { .. }));
        assert!(!world.closing.contains(&1));
        assert_eq!(
            response(&mut world, |reply_to| if release {
                Event::Release {
                    reply_to,
                    by: PERSON_PARTY,
                    operation: t::Release { task: 1, again: None, message: None },
                }
            } else {
                Event::Lift { reply_to, by: PERSON_PARTY, hold: 100, revision: 1, with_tasks: true }
            }),
            if release { Reply::Done } else { Reply::Lifted { released: 1 } }
        );
        report(&mut world, b"Concluded: 0 done, 0 failed, 1 cancelled.");
    }
}

#[test]
fn a_gatherings_lifetime_delegates_are_not_bounded_by_its_inbox() {
    let mut world = world();
    world.env.limits.inbox_messages = 2;
    for number in 2..7 {
        assert_eq!(world.make(Party::Task(1), vec![child(number)]), Reply::Made(vec![number]));
        finish(&mut world, number, TaskResult::Report { words: Box::new([1]) });
    }
    conclude(&mut world, PERSON);
    report(&mut world, b"Concluded: 5 done, 0 failed, 0 cancelled.");
}

#[test]
fn an_amendment_cannot_add_a_dependency_to_a_gathering() {
    let mut world = world();
    world.make(PERSON_PARTY, vec![task(2, &[])]);
    let before = world.records.clone();
    assert!(matches!(
        response(&mut world, |reply_to| Event::Amend {
            reply_to,
            by: PERSON_PARTY,
            places: Box::new([]),
            operation: t::Amend {
                funder: None,
                task: 1,
                message: 100,
                stop_run: false,
                amendment: Box::new(t::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: None,
                    wake: None,
                    dependencies: Some(Box::new([2])),
                    authority: None,
                    llm: None,
                    reason: Box::new([])
                })
            },
        }),
        Reply::Refused(_)
    ));
    assert_eq!(world.records, before);
}
