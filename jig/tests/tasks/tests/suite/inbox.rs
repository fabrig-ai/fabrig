use jig_domain_tasks::{
    Accepted, Active, End, Event, MessageKind, NewsClass, Party, Phase, Refusal, Subscription, SubscriptionKind,
    WakeRule, Word,
};
use jig_tasks_world::{LIMITS, PERSON_PARTY, Reply, World, task};
use skein_lib::{ReplyTo, Token, Wall};

fn say(world: &mut World, number: u64) {
    let reply_to = world.to();
    world.send(Event::Message {
        reply_to,
        project: 1,
        operation: jig_domain_tasks::Message {
            task: 1,
            word: Word {
                number,
                from: PERSON_PARTY,
                kind: jig_domain_tasks::MessageKind::Words,
                words: Box::from([u8::try_from(number).expect("small message")]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        },
    });
}

#[test]
fn a_read_fence_takes_only_offered_words() {
    let mut world = World::new(81, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.claim_task(1);
    say(&mut world, 1);
    say(&mut world, 2);
    let before = world.record(1).clone();
    let call = 1001;
    world.send(Event::Turn {
        refused: Box::new([]),
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        attempt: 1,
        turn: 1,
        read: Some(2),
        offered: Some(1),
        outstanding: Box::new([1]),
        cumulative: 1,
    });
    assert!(matches!(&world.replies[&call], Reply::Refused(problem) if problem.why == Refusal::Read));
    assert_eq!(world.record(1), &before);
    let call = 1002;
    world.send(Event::Turn {
        refused: Box::new([]),
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        attempt: 1,
        turn: 1,
        read: Some(1),
        offered: Some(1),
        outstanding: Box::new([1]),
        cumulative: 1,
    });
    assert_eq!(world.replies[&call], Reply::Turn(Accepted::New));
    assert_eq!(world.record(1).inbox.len(), 1);
    assert_eq!(world.record(1).inbox[0].number, 2);
}

#[test]
fn a_parked_chat_wakes_on_its_persons_words() {
    let mut world = World::new(82, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.claim_task(1);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    say(&mut world, 1);
    assert!(world.activations.contains(&1));
    assert_eq!(world.contexts[&1].inbox[0].number, 1);
}

fn notification_goal() -> World {
    let mut world = World::new(473, LIMITS);
    let mut goal = task(1, &[]);
    goal.tracked = Some(1);
    goal.wake.notices = WakeRule::Immediate;
    goal.wake.results = jig_domain_tasks::ResultsWake::Each;
    assert_eq!(world.make(PERSON_PARTY, vec![goal]), Reply::Made(vec![1]));
    assert_eq!(world.make(Party::Task(1), vec![task(2, &[]), task(3, &[])]), Reply::Made(vec![2, 3]));
    for target in [2, 3] {
        let reply_to = world.to();
        let call = reply_to.into_token().raw();
        world.send(Event::Subscribe {
            reply_to: ReplyTo::new(Token::new(call)),
            task: 1,
            subscription: Subscription {
                number: target,
                kind: SubscriptionKind::Task { target, held: true, result: true },
            },
        });
        assert_eq!(world.replies[&call], Reply::Done);
    }
    let reply_to = world.to();
    world.send(Event::Subscribe {
        reply_to,
        task: 1,
        subscription: Subscription {
            number: 10,
            kind: SubscriptionKind::Timer { at: Wall::from_nanos(10_000_000_000), period: None },
        },
    });
    world.claim_task(1);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    world
}

fn consume_goal_wake(world: &mut World, attempt: u64, cause: u64) {
    world.claim(1, attempt);
    assert!(world.records.values().any(|record| matches!(record,
    jig_domain_tasks::Stored::Activity(line) if line.task == 1
        && line.line == jig_domain_tasks::Line::RunBegan {
            attempt, why: jig_domain_tasks::Why::Message { message: cause }, resumed: false,
        })));
    goal_turn(world, attempt, 1, None);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert!(world.record(1).inbox.is_empty());
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    assert!(!world.activations.contains(&1));
}

fn goal_turn(world: &mut World, attempt: u64, turn: u32, entry: Option<u64>) {
    let outstanding: Box<[_]> = world.record(1).inbox.iter().map(|word| word.number).chain(entry).collect();
    let read = outstanding.iter().copied().max();
    assert!(read.is_some());
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::Turn {
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        attempt,
        turn,
        read,
        refused: Box::new([]),
        offered: read,
        outstanding,
        cumulative: 0,
    });
    assert_eq!(world.replies[&call], Reply::Turn(Accepted::New));
}

#[test]
fn a_handled_wake_does_not_reactivate_a_parked_goal() {
    let mut world = notification_goal();
    for target in [2, 3] {
        world.send(Event::Hold { task: target, why: jig_domain_tasks::Hold::Budget });
    }
    let cause = world.record(1).inbox[0].number;
    assert!(world.record(1).inbox.iter().all(|word| matches!(
        word.kind,
        MessageKind::Notice { state: jig_domain_tasks::NoticeState::Held { .. }, .. }
    )));
    consume_goal_wake(&mut world, 4, cause);
    let reply_to = world.to();
    world.send(Event::Release {
        reply_to,
        by: Party::Task(1),
        operation: jig_domain_tasks::Release { task: 2, again: None, message: None },
    });
    let reply_to = world.to();
    world.send(Event::Cancel {
        reply_to,
        by: Party::Task(1),
        operation: jig_domain_tasks::Cancel { task: 3, reason: Box::new([]), grace: None, message: None },
    });
    world.settle(3);
    let next = world.record(1).inbox[0].number;
    assert!(next > cause);
    assert!(
        world
            .record(1)
            .inbox
            .iter()
            .any(|word| word.kind == MessageKind::Result(jig_domain_tasks::ResultKind::Cancelled))
    );
    consume_goal_wake(&mut world, 5, next);
    for _ in 0..3 {
        world.elapse(skein_lib::Duration::from_secs(1));
        world.restart();
        assert!(world.record(1).inbox.is_empty());
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
        assert!(!world.activations.contains(&1));
    }
    world.finish_task(2);
    let result = world.record(1).inbox[0].number;
    assert!(result > next);
    assert!(
        world.record(1).inbox.iter().any(|word| word.kind == MessageKind::Result(jig_domain_tasks::ResultKind::Report))
    );
    consume_goal_wake(&mut world, 6, result);
}

fn goal_with_read_escalation() -> World {
    let mut world = notification_goal();
    world.send(Event::Hold { task: 2, why: jig_domain_tasks::Hold::Budget });
    let notice = world.record(1).inbox[0].number;
    consume_goal_wake(&mut world, 4, notice);
    world.send(Event::RoutedEscalation {
        task: 2,
        revision: world.record(2).escalation.revision(),
        holder: jig_domain_tasks::EscalationHolder::Task(1),
        entry: 23,
    });
    world.claim(1, 5);
    goal_turn(&mut world, 5, 1, Some(23));
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).due_reason, jig_domain_tasks::Why::Message { message: 23 });
    world.claim(1, 6);
    goal_turn(&mut world, 6, 1, Some(23));
    world
}

#[test]
fn a_retired_escalation_does_not_reactivate_its_holder() {
    for cancel in [false, true] {
        let mut world = goal_with_read_escalation();
        let reply_to = world.to();
        let call = reply_to.into_token().raw();
        let reply_to = ReplyTo::new(Token::new(call));
        if cancel {
            world.send(Event::Cancel {
                reply_to,
                by: Party::Task(1),
                operation: jig_domain_tasks::Cancel { task: 2, reason: Box::new([]), grace: None, message: None },
            });
        } else {
            world.send(Event::Release {
                reply_to,
                by: Party::Task(1),
                operation: jig_domain_tasks::Release { task: 2, again: None, message: None },
            });
        }
        assert_eq!(world.replies[&call], Reply::Done);
        assert!(matches!(world.record(2).escalation, jig_domain_tasks::Escalation::Unheld { .. }));
        if cancel {
            world.settle(2);
        } else {
            world.finish_task(2);
        }
        goal_turn(&mut world, 6, 2, None);
        assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
        world.restart();
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
        assert!(!world.activations.contains(&1));
        say(&mut world, 24);
        assert!(world.contexts[&1].inbox.iter().all(|word| word.number != 23));
        consume_goal_wake(&mut world, 7, 24);
    }
}

#[test]
fn a_deadline_wake_does_not_reuse_a_handled_message_reason() {
    let mut world = World::new(475, LIMITS);
    let mut goal = task(1, &[]);
    goal.tracked = Some(1);
    goal.authority.budget.deadline = Some(Wall::from_nanos(100));
    assert_eq!(world.make(PERSON_PARTY, vec![goal]), Reply::Made(vec![1]));
    let reply_to = world.to();
    world.send(Event::Subscribe {
        reply_to,
        task: 1,
        subscription: Subscription {
            number: 10,
            kind: SubscriptionKind::Timer { at: Wall::from_nanos(1_000), period: None },
        },
    });
    world.claim(1, 1);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    say(&mut world, 23);
    consume_goal_wake(&mut world, 2, 23);
    world.elapse(skein_lib::Duration::from_nanos(101));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert!(world.record(1).inbox.is_empty());
    // The core's deadline admission holds before preparation or claim
    // (jig/docs/design/domain/authority.md, 7 and 8.3).
    world.send(Event::Hold { task: 1, why: jig_domain_tasks::Hold::Deadline });
    assert!(matches!(world.record(1).phase, Phase::Held { why: jig_domain_tasks::Hold::Deadline, .. }));
    assert_eq!(world.record(1).attempt, 2);
    assert_eq!(run_reason(&world, 3), None);
    world.restart();
    assert!(matches!(world.record(1).phase, Phase::Held { why: jig_domain_tasks::Hold::Deadline, .. }));
    assert!(!world.activations.contains(&1));
    assert_eq!(run_reason(&world, 3), None);
    extend_goal_deadline(&mut world);
    let reply = jig_tasks_world::response(&mut world, |reply_to| Event::Release {
        reply_to,
        by: PERSON_PARTY,
        operation: jig_domain_tasks::Release { task: 1, again: None, message: None },
    });
    assert_eq!(reply, Reply::Done);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    world.claim(1, 3);
    assert_eq!(run_reason(&world, 3), Some(jig_domain_tasks::Why::Released));
}

fn run_reason(world: &World, attempt: u64) -> Option<jig_domain_tasks::Why> {
    world.records.values().find_map(|record| match record {
        jig_domain_tasks::Stored::Activity(line) if line.task == 1 => match line.line {
            jig_domain_tasks::Line::RunBegan { attempt: number, why, .. } if number == attempt => Some(why),
            _ => None,
        },
        _ => None,
    })
}

fn extend_goal_deadline(world: &mut World) {
    let mut authority = world.record(1).authority.clone();
    authority.budget.deadline = Some(Wall::from_nanos(500));
    let funder = world.record(1).funder;
    let reply = jig_tasks_world::response(world, |reply_to| Event::Amend {
        reply_to,
        by: PERSON_PARTY,
        funder: Some(funder),
        places: Box::new([]),
        operation: jig_domain_tasks::Amend {
            task: 1,
            message: 24,
            stop_run: false,
            amendment: Box::new(jig_domain_tasks::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                release: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: Some(authority),
                llm: None,
                reason: b"more time".as_slice().into(),
            }),
        },
    });
    assert_eq!(reply, Reply::Done);
    assert!(matches!(world.record(1).phase, Phase::Held { why: jig_domain_tasks::Hold::Deadline, .. }));
}

#[test]
fn a_persons_words_left_unread_wake_the_task_as_its_run_ends() {
    let mut world = refusal_world(94);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert!(world.activations.contains(&1));
    assert_eq!(world.contexts[&1].inbox.iter().map(|word| word.number).collect::<Vec<_>>(), [1, 2]);
}

#[test]
fn a_runs_refusals_end_with_its_attempt() {
    for end in [End::Parked, End::Failed(jig_domain_tasks::Class::Transient), End::Refused] {
        let mut world = refused_world(95);
        assert_eq!(world.record(1).refused.as_ref(), [1]);
        assert_eq!(world.terminal(1, end), Reply::Acknowledged(Accepted::New));
        assert!(world.record(1).refused.is_empty());
        assert_eq!(world.record(1).inbox[0].number, 1);
        world.restart();
        assert!(world.record(1).refused.is_empty());
    }
}

#[test]
fn unread_words_do_not_wake_a_task_its_run_closes() {
    let mut world = refusal_world(96);
    world.finish(1);
    world.settle(1);
    assert!(!world.activations.contains(&1));
    assert!(world.records.contains_key(&jig_domain_tasks::Key::Ended(1)));
}

fn task_words(world: &mut World, source: u64, target: u64, number: u64, kind: MessageKind) -> Reply {
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    world.send(Event::Message {
        reply_to: ReplyTo::new(Token::new(key)),
        project: 1,
        operation: jig_domain_tasks::Message {
            task: target,
            word: Word {
                number,
                from: Party::Task(source),
                kind,
                words: Box::new([1]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        },
    });
    world.replies[&key].clone()
}

#[test]
fn a_question_keeps_room_for_its_answer_while_words_fill_the_inbox() {
    let mut limits = LIMITS;
    limits.inbox_messages = 2;
    limits.run_inbox = 2;
    let mut world = World::new(83, limits);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.make(Party::Task(1), vec![task(2, &[])]);
    world.make(Party::Task(2), vec![task(3, &[])]);
    assert!(matches!(task_words(&mut world, 3, 2, 1, MessageKind::Question), Reply::Done));
    assert_eq!(world.record(3).questions.len(), 1);
    assert!(matches!(task_words(&mut world, 2, 3, 2, MessageKind::Words), Reply::Done));
    assert!(
        matches!(task_words(&mut world, 2, 3, 3, MessageKind::Words), Reply::Refused(problem) if problem.why == Refusal::Busy)
    );
    assert!(matches!(task_words(&mut world, 2, 3, 4, MessageKind::Answer { question: 1 }), Reply::Done));
    assert!(world.record(3).questions.is_empty());
    assert_eq!(world.record(3).inbox.len(), 2);
    world.restart();
}

#[test]
fn a_person_answers_one_numbered_question_in_their_active_task() {
    let mut world = World::new(830, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.make(Party::Task(1), vec![task(2, &[])]);
    world.claim_task(1);
    assert!(matches!(task_words(&mut world, 2, 1, 1, MessageKind::Question), Reply::Done));
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    world.send(Event::Message {
        reply_to: ReplyTo::new(Token::new(key)),
        project: 1,
        operation: jig_domain_tasks::Message {
            task: 1,
            word: Word {
                number: 2,
                from: PERSON_PARTY,
                kind: MessageKind::Answer { question: 1 },
                words: Box::from(&b"yes"[..]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        },
    });
    assert_eq!(world.replies[&key], Reply::Done);
    assert_eq!(world.record(1).inbox.len(), 2);
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    world.send(Event::Message {
        reply_to: ReplyTo::new(Token::new(key)),
        project: 1,
        operation: jig_domain_tasks::Message {
            task: 1,
            word: Word {
                number: 3,
                from: PERSON_PARTY,
                kind: MessageKind::Answer { question: 1 },
                words: Box::from(&b"twice"[..]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        },
    });
    assert!(matches!(&world.replies[&key], Reply::Refused(problem) if problem.why == Refusal::Reference));
    world.restart();
}

#[test]
fn a_coordinator_is_woken_once_by_a_burst() {
    let mut world = World::new(84, LIMITS);
    let mut coordinator = task(1, &[]);
    coordinator.wake.words = WakeRule::Batch { count: 3, age: skein_lib::Duration::from_nanos(20) };
    world.make(PERSON_PARTY, vec![coordinator]);
    world.make(Party::Task(1), vec![task(2, &[])]);
    world.claim_task(1);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    for number in 1..3 {
        assert!(matches!(task_words(&mut world, 2, 1, number, MessageKind::Words), Reply::Done));
        assert!(!world.activations.contains(&1));
    }
    world.restart();
    assert!(matches!(task_words(&mut world, 2, 1, 3, MessageKind::Words), Reply::Done));
    assert_eq!(world.activations.iter().filter(|number| **number == 1).count(), 1);
    assert_eq!(world.record(1).inbox.len(), 3);
}

#[test]
fn an_inbox_filling_merges_news_refuses_words_and_still_takes_a_result() {
    let mut limits = LIMITS;
    limits.inbox_messages = 3;
    limits.run_inbox = 3;
    let mut world = World::new(85, limits);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.make(Party::Task(1), vec![task(2, &[])]);
    let call = 1000;
    let reply_to = ReplyTo::new(Token::new(call));
    world.send(Event::SubscribeTopic {
        reply_to,
        task: 1,
        subscription: Subscription { number: 10, kind: SubscriptionKind::Topic { connector: 0, topic: 2 } },
    });
    assert_eq!(world.replies[&call], Reply::Done);
    assert!(matches!(task_words(&mut world, 2, 1, 1, MessageKind::Words), Reply::Done));
    for number in [2, 3] {
        world.send(Event::Notice {
            task: 1,
            word: Word {
                number,
                from: Party::Task(2),
                kind: MessageKind::News { subscription: 10, class: NewsClass::Kept },
                words: Box::new([]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        });
    }
    assert_eq!(world.record(1).inbox.len(), 2);
    assert_eq!(world.record(1).inbox[1].hits, 2);
    assert!(
        matches!(task_words(&mut world, 2, 1, 4, MessageKind::Words), Reply::Refused(problem) if problem.why == Refusal::Busy)
    );
    world.finish_task(2);
    assert_eq!(world.record(1).inbox.len(), 3);
    assert!(world.record(1).inbox.iter().any(|word| matches!(word.kind, MessageKind::Result(_))));
    world.restart();
}

#[test]
fn a_watch_woken_once_by_a_burst_of_news() {
    let mut world = World::new(86, LIMITS);
    let mut watch = task(1, &[]);
    watch.flags =
        jig_domain_tasks::Flags { conversation: true, ended_by: jig_domain_tasks::EndedBy::Requester, ..watch.flags };
    watch.wake.news = WakeRule::Batch { count: 2, age: skein_lib::Duration::from_nanos(20) };
    world.make(PERSON_PARTY, vec![watch]);
    world.claim_task(1);
    assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::SubscribeTopic {
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        subscription: Subscription { number: 10, kind: SubscriptionKind::Topic { connector: 0, topic: 10 } },
    });
    assert_eq!(world.replies[&call], Reply::Done);
    for (number, class) in [(1, NewsClass::Wakes), (2, NewsClass::Kept), (3, NewsClass::Kept)] {
        world.send(Event::Notice {
            task: 1,
            word: Word {
                number,
                from: Party::Task(1),
                kind: MessageKind::News { subscription: 10, class },
                words: Box::new([]),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        });
        if number == 1 {
            assert!(!world.activations.contains(&1));
            world.restart();
        }
    }
    assert_eq!(world.record(1).inbox.len(), 1);
    assert_eq!(world.record(1).inbox[0].hits, 3);
    assert_eq!(world.record(1).inbox[0].kind, MessageKind::News { subscription: 10, class: NewsClass::Wakes });
    assert!(world.activations.contains(&1));
    assert_eq!(world.activations.len(), 1);
    world.send(Event::Notice {
        task: 1,
        word: Word {
            number: 4,
            from: Party::Task(1),
            kind: MessageKind::News { subscription: 10, class: NewsClass::Dropped },
            words: Box::new([]),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        },
    });
    assert_eq!(world.record(1).last_message, 3);
}

const NEWS_BATCH_COUNT: u32 = 3;

fn procedure_watch(seed: u64, count: u32, age: u64) -> World {
    let mut world = World::new(seed, LIMITS);
    let mut watch = task(1, &[]);
    watch.executor = jig_domain_tasks::Executor::Procedure { connector: 0, code: 1 };
    watch.wake.news = WakeRule::Batch { count, age: skein_lib::Duration::from_nanos(age) };
    assert_eq!(world.make(PERSON_PARTY, vec![watch]), Reply::Made(vec![1]));
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::SubscribeTopic {
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        subscription: Subscription { number: 10, kind: SubscriptionKind::Topic { connector: 0, topic: 10 } },
    });
    assert_eq!(world.replies[&call], Reply::Done);
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::Procedure {
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        step: 1,
        read: None,
        decision: jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait),
    });
    assert_eq!(world.replies[&call], Reply::Done);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    world
}

fn procedure_news(world: &mut World, number: u64) {
    world.send(Event::Notice {
        task: 1,
        word: Word {
            number,
            from: Party::Task(1),
            kind: MessageKind::News { subscription: 10, class: NewsClass::Wakes },
            words: Box::new([]),
            at: world.env.wall,
            hits: 1,
            eligible: false,
        },
    });
}

#[test]
fn a_procedure_woken_once_by_a_burst_of_news_by_count() {
    let mut world = procedure_watch(861, NEWS_BATCH_COUNT, 20);
    for number in 1..3 {
        procedure_news(&mut world, number);
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    }
    world.restart();
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    procedure_news(&mut world, 3);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).attempt, 1);
    procedure_news(&mut world, 4);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).attempt, 1);
    assert_eq!(world.record(1).inbox.len(), 1);
    // The fourth hint arrived after the offered storm and begins its own batch.
    assert_eq!(world.record(1).inbox[0].hits, 1);
    assert!(!world.record(1).inbox[0].eligible);
}

#[test]
fn a_procedure_woken_once_by_a_burst_of_news_by_age() {
    let mut world = procedure_watch(862, NEWS_BATCH_COUNT, 20);
    procedure_news(&mut world, 1);
    world.elapse(skein_lib::Duration::from_nanos(10));
    procedure_news(&mut world, 2);
    world.restart();
    world.elapse(skein_lib::Duration::from_nanos(9));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    world.elapse(skein_lib::Duration::from_nanos(1));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).attempt, 1);
    assert_eq!(world.record(1).inbox.len(), 1);
    assert_eq!(world.record(1).inbox[0].hits, 2);
    assert!(world.record(1).inbox[0].eligible);
    world.elapse(skein_lib::Duration::from_nanos(20));
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).attempt, 1);
}

#[test]
fn a_procedures_words_step_it_while_news_waits_for_its_batch() {
    let mut world = procedure_watch(863, NEWS_BATCH_COUNT, 20);
    procedure_news(&mut world, 1);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    say(&mut world, 2);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(1).attempt, 1);
    assert_eq!(world.record(1).inbox.len(), 2);
    assert!(!world.record(1).inbox[0].eligible);
}

fn procedure_step(
    world: &mut World,
    step: u64,
    read: Option<u64>,
    decision: jig_domain_tasks::ProcedureDecision,
) -> Reply {
    let to = world.to();
    let key = to.into_token().raw();
    world.send(Event::Procedure { reply_to: ReplyTo::new(Token::new(key)), task: 1, step, read, decision });
    world.replies[&key].clone()
}

#[test]
fn a_procedures_step_takes_its_storm_and_a_later_result_steps_it_on_the_result_alone() {
    let mut world = procedure_watch(864, NEWS_BATCH_COUNT, 20);
    world.make(Party::Task(1), vec![task(2, &[])]);
    for number in 1..=3 {
        procedure_news(&mut world, number);
    }
    let offered = world.contexts[&1].inbox.last().expect("offered storm").number;
    assert_eq!(offered, 3);
    assert_eq!(
        procedure_step(
            &mut world,
            2,
            Some(offered),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert!(world.record(1).inbox.is_empty());
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    world.restart();
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    world.finish_task(2);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    let inbox = &world.contexts[&1].inbox;
    assert_eq!(inbox.len(), 1);
    assert!(matches!(inbox[0].kind, MessageKind::Result(_)));
    let read = Some(inbox[0].number);
    assert_eq!(
        procedure_step(
            &mut world,
            3,
            read,
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert!(world.record(1).inbox.is_empty());
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
}

#[test]
fn a_result_arriving_after_a_procedures_offer_survives_its_step() {
    let mut world = procedure_watch(865, NEWS_BATCH_COUNT, 20);
    world.make(Party::Task(1), vec![task(2, &[])]);
    for number in 1..=3 {
        procedure_news(&mut world, number);
    }
    world.finish_task(2);
    assert_eq!(world.record(1).inbox.len(), 2);
    assert_eq!(
        procedure_step(
            &mut world,
            2,
            Some(3),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.contexts[&1].previous_attempt, 2);
    assert_eq!(world.contexts[&1].inbox.len(), 1);
    assert!(matches!(world.contexts[&1].inbox[0].kind, MessageKind::Result(_)));
    let read = Some(world.contexts[&1].inbox[0].number);
    assert_eq!(
        procedure_step(
            &mut world,
            3,
            read,
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert!(world.record(1).inbox.is_empty());
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
}

#[test]
fn a_refused_procedure_step_takes_nothing_and_restart_offers_the_unread_storm_again() {
    let mut world = procedure_watch(866, NEWS_BATCH_COUNT, 20);
    for number in 1..=3 {
        procedure_news(&mut world, number);
    }
    let before = world.record(1).clone();
    assert!(matches!(
        procedure_step(&mut world, 2, Some(4), jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)),
        Reply::Refused(problem) if problem.why == Refusal::Read
    ));
    assert_eq!(world.record(1), &before);
    let invalid = task(1, &[]);
    assert!(matches!(
        procedure_step(
            &mut world,
            2,
            Some(3),
            jig_domain_tasks::ProcedureDecision {
                batch: Box::new([invalid]),
                ..jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
            }
        ),
        Reply::Refused(_)
    ));
    assert_eq!(world.record(1), &before);
    world.restart();
    assert_eq!(world.record(1), &before);
    assert_eq!(world.contexts[&1].inbox.last().expect("unread storm").number, 3);
    assert_eq!(
        procedure_step(
            &mut world,
            2,
            Some(3),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert!(world.record(1).inbox.is_empty());
}

#[test]
fn news_arriving_after_a_procedures_offer_waits_for_its_own_batch() {
    let mut world = procedure_watch(867, NEWS_BATCH_COUNT, 20);
    for number in 1..=4 {
        procedure_news(&mut world, number);
    }
    assert_eq!(
        procedure_step(
            &mut world,
            2,
            Some(3),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(1).inbox[0].number, 4);
    assert_eq!(world.record(1).inbox[0].hits, 1);
    procedure_news(&mut world, 5);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    procedure_news(&mut world, 6);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(world.contexts[&1].inbox[0].hits, 3);
    assert_eq!(
        procedure_step(
            &mut world,
            3,
            Some(6),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    assert!(world.record(1).inbox.is_empty());
}

#[test]
fn the_referee_rejects_a_procedure_keeping_its_prefix_or_taking_later_words() {
    struct Case {
        name: &'static str,
        after_step: bool,
        retain_prefix: bool,
        expected: &'static str,
    }
    let mut world = procedure_watch(868, NEWS_BATCH_COUNT, 20);
    for number in 1..=3 {
        procedure_news(&mut world, number);
    }
    say(&mut world, 4);
    let before = world.records.clone();
    assert_eq!(
        procedure_step(
            &mut world,
            2,
            Some(3),
            jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Wait)
        ),
        Reply::Done
    );
    let cases = [
        Case {
            name: "consumed prefix kept",
            after_step: true,
            retain_prefix: true,
            expected: "procedure step kept its consumed inbox prefix",
        },
        Case {
            name: "unread suffix lost",
            after_step: true,
            retain_prefix: false,
            expected: "committed message disappeared without a turn, procedure step or merge",
        },
        Case {
            name: "inbox lost before step",
            after_step: false,
            retain_prefix: false,
            expected: "committed message disappeared without a turn, procedure step or merge",
        },
    ];
    for case in cases {
        let mut rows = if case.after_step { world.records.clone() } else { before.clone() };
        let jig_domain_tasks::Stored::Live(task) =
            rows.get_mut(&jig_domain_tasks::Key::Live(1)).expect("live procedure")
        else {
            unreachable!()
        };
        task.inbox = if case.retain_prefix {
            let jig_domain_tasks::Stored::Live(old) = &before[&jig_domain_tasks::Key::Live(1)] else { unreachable!() };
            old.inbox.clone()
        } else {
            Box::new([])
        };
        let mut referee = jig_tasks_world::accounting_referee::Accounting::default();
        referee.reset(&before);
        referee.procedure(1, 2, Some(3));
        assert_eq!(referee.committed(&rows, &[]), Err(case.expected), "{}", case.name);
    }
}

#[test]
fn a_connector_topic_subscription_ends_with_its_task() {
    let mut world = World::new(87, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    let reply_to = world.to();
    world.send(Event::SubscribeTopic {
        reply_to,
        task: 1,
        subscription: Subscription { number: 10, kind: SubscriptionKind::Topic { connector: 0, topic: 10 } },
    });
    world.finish_task(1);
    assert_eq!(world.ended_topics, [(1, 10, 0)]);
    world.restart();
    assert_eq!(world.ended_topics, [(1, 10, 0)]);
}

#[test]
fn only_introduced_peers_can_message_each_other() {
    let mut world = World::new(88, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.make(Party::Task(1), vec![task(2, &[]), task(3, &[]), task(4, &[])]);
    assert!(matches!(task_words(&mut world, 2, 3, 1, MessageKind::Words), Reply::Refused(_)));
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::Introduce { reply_to: ReplyTo::new(Token::new(call)), by: 1, left: 2, right: 3 });
    assert_eq!(world.replies[&call], Reply::Done);
    assert_eq!(task_words(&mut world, 2, 3, 2, MessageKind::Words), Reply::Done);
    assert!(matches!(task_words(&mut world, 2, 4, 3, MessageKind::Words), Reply::Refused(_)));
    world.restart();
}

fn refusal_world(seed: u64) -> World {
    let mut world = World::new(seed, LIMITS);
    world.make(PERSON_PARTY, vec![task(1, &[])]);
    world.claim_task(1);
    say(&mut world, 1);
    say(&mut world, 2);
    world
}

fn refusal_snapshot(world: &World) -> jig_domain_tasks::TaskRecord {
    world.record(1).clone()
}

fn refusal_turn(world: &mut World, turn: u32, read: u64, offered: u64, refused: &[u64]) -> Reply {
    let row = world.record(1);
    let outstanding = row
        .inbox
        .iter()
        .filter(|word| word.number <= offered && !row.refused.contains(&word.number))
        .map(|word| word.number)
        .collect();
    let reply_to = world.to();
    let call = reply_to.into_token().raw();
    world.send(Event::Turn {
        reply_to: ReplyTo::new(Token::new(call)),
        task: 1,
        attempt: 1,
        turn,
        read: Some(read),
        refused: refused.into(),
        offered: Some(offered),
        outstanding,
        cumulative: u64::from(turn),
    });
    world.replies[&call].clone()
}

fn refused_world(seed: u64) -> World {
    let mut world = refusal_world(seed);
    assert_eq!(refusal_turn(&mut world, 1, 2, 2, &[1]), Reply::Turn(Accepted::New));
    world
}

fn advance_fence(world: &mut World) {
    say(world, 3);
    assert_eq!(refusal_turn(world, 2, 3, 3, &[]), Reply::Turn(Accepted::New));
}

fn inbox_numbers(world: &World) -> Vec<u64> {
    world.record(1).inbox.iter().map(|word| word.number).collect()
}

#[test]
fn a_refused_word_survives_a_later_fence() {
    let mut world = refused_world(91);
    advance_fence(&mut world);
    assert_eq!(inbox_numbers(&world), [1]);
    assert_eq!(world.record(1).refused.as_ref(), [1]);
}

#[test]
fn a_refusal_naming_an_unoffered_word_is_refused_as_read() {
    let mut world = refusal_world(92);
    let before = refusal_snapshot(&world);
    assert!(matches!(refusal_turn(&mut world, 1, 1, 1, &[2]),
        Reply::Refused(problem) if problem.why == Refusal::Read));
    assert_eq!(world.record(1), &before);
}

#[test]
fn a_refusal_naming_a_word_outside_the_inbox_is_refused_as_read() {
    let mut world = refusal_world(93);
    let before = refusal_snapshot(&world);
    assert!(matches!(refusal_turn(&mut world, 1, 2, 3, &[3]),
        Reply::Refused(problem) if problem.why == Refusal::Read));
    assert_eq!(world.record(1), &before);
}

#[test]
fn a_refused_word_survives_a_restart_and_a_later_fence() {
    let mut world = refused_world(94);
    world.restart();
    advance_fence(&mut world);
    assert_eq!(inbox_numbers(&world), [1]);
    assert_eq!(world.record(1).refused.as_ref(), [1]);
}
