//! Requester question keeps and task-batched exits (domain/tasks.md, 5.7 and 7).
use jig_domain_tasks::{self as tasks, *};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{Queue, ReplyTo, Token, Wall};

const REQUESTER: u64 = 1;
const ASKER: u64 = 2;
const PERSON: u64 = 9;
const QUESTION: u64 = 1;
const ANSWER: u64 = 2;
const LATER_QUESTION: u64 = 3;
const ATTEMPT: u64 = 1;
const PERSON_TASK: u64 = 3;

fn response(world: &mut World, make: impl FnOnce(ReplyTo) -> Event) -> Reply {
    let to = world.to();
    let number = to.into_token().raw();
    world.send(make(ReplyTo::new(Token::new(number))));
    world.replies[&number].clone()
}

fn message(world: &mut World, source: Party, target: u64, number: u64, kind: MessageKind) -> Reply {
    response(world, |reply_to| Event::Message {
        reply_to,
        project: 1,
        task: target,
        word: Word {
            number,
            from: source,
            kind,
            words: b"proceed?".as_slice().into(),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        },
    })
}

fn pair() -> World {
    pair_with_asks(false)
}

fn pair_with_asks(asks: bool) -> World {
    let mut world = World::new(880, LIMITS);
    let mut parent = task(REQUESTER, &[]);
    parent.numbers.budget = 1_000;
    parent.authority.budget.spend = 1_000;
    world.make(Party::Person(PERSON), vec![parent]);
    let mut child = task(ASKER, &[]);
    child.spec.asks = asks;
    child.funder = Funder::Task(REQUESTER);
    child.wake.answers = false;
    world.make(Party::Task(REQUESTER), vec![child]);
    world.claim(ASKER, ATTEMPT);
    world
}

fn asked() -> World {
    let mut world = pair();
    message(&mut world, Party::Task(ASKER), REQUESTER, QUESTION, MessageKind::Question);
    world
}

fn report() -> End {
    End::Finished { result: TaskResult::Report { words: b"ready".as_slice().into() }, cancel_delegates: false }
}

fn parked() -> World {
    let mut world = asked();
    world.terminal(ASKER, report());
    world
}

#[test]
fn a_task_with_its_own_question_unanswered_parks_on_finish() {
    let world = parked();
    assert_eq!(world.record(ASKER).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(ASKER).tries, Tries::NONE);
    assert!(world.records.values().any(|row| matches!(row, Stored::Activity(ActivityLine {
        task: ASKER, line: Line::Kept { attempt: ATTEMPT, pending, question: Some(QUESTION) }, ..
    }) if pending.is_empty())));
    assert!(!world.activations.contains(&ASKER));
}

#[test]
fn the_answer_wakes_a_task_kept_by_its_question() {
    let mut world = parked();
    world.restart();
    message(&mut world, Party::Task(REQUESTER), ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
    assert!(world.record(ASKER).questions.is_empty());
    assert!(world.activations.contains(&ASKER));
    assert_eq!(world.contexts[&ASKER].inbox[0].from, Party::Task(REQUESTER));
    world.claim(ASKER, ANSWER);
    world.finish(ASKER);
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Done(_)));
}

#[test]
fn a_question_keep_permits_failure() {
    let mut world = asked();
    world.terminal(
        ASKER,
        End::Finished {
            result: TaskResult::Failure { reason: b"cannot proceed".as_slice().into() },
            cancel_delegates: false,
        },
    );
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Failed { .. }));
}

#[test]
fn a_question_keep_permits_cancellation() {
    let mut world = parked();
    response(&mut world, |reply_to| Event::Control {
        reply_to,
        by: Party::Person(PERSON),
        task: ASKER,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Cancelled { .. }));
}

#[test]
fn a_question_through_a_person_task_keeps_nothing() {
    let mut world = pair();
    let mut person = task(PERSON_TASK, &[]);
    person.executor = Executor::Person(PersonAddress::Person(PERSON));
    person.funder = Funder::Task(ASKER);
    person.numbers.budget = 20;
    person.authority.budget.spend = 20;
    world.make(Party::Task(ASKER), vec![person]);
    message(&mut world, Party::Task(ASKER), PERSON_TASK, QUESTION, MessageKind::Words);
    assert!(world.record(ASKER).questions.is_empty());
    // Detachment is a separate task rule; finishing with delegate cancellation
    // demonstrates that words introduced no question keep.
    world.terminal(
        ASKER,
        End::Finished { result: TaskResult::Report { words: b"ready".as_slice().into() }, cancel_delegates: true },
    );
    assert!(matches!(world.record(ASKER).phase, Phase::Closing(_)));
}

fn moved() -> World {
    let mut world = parked();
    response(&mut world, |reply_to| Event::Move {
        reply_to,
        task: ASKER,
        person: PERSON,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
        reason: Box::new([]),
    });
    world.restart();
    world
}

#[test]
fn a_moved_asker_carries_its_open_question_to_its_new_requester() {
    let mut world = moved();
    let row = world.record(ASKER);
    assert_eq!(row.requester, Party::Person(PERSON));
    assert_eq!(row.phase, Phase::Active(Active::Idle));
    let credit = row.questions[0].clone();
    assert_eq!(credit.number, QUESTION);
    assert_eq!(credit.answerer, Party::Person(PERSON));
    assert_eq!(credit.asked, Party::Person(PERSON));
    assert_eq!(row.question_lines[0].line, credit.line);
    assert!(matches!(&world.records[&Key::Activity { task: ASKER, position: credit.line }],
        Stored::Activity(ActivityLine { line: Line::Words { message: QUESTION, words, .. }, .. })
            if words.as_ref() == b"proceed?"));
    assert!(world.record(REQUESTER).question_lines.is_empty());
    message(&mut world, Party::Person(PERSON), ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
    assert!(world.record(ASKER).questions.is_empty());
    assert!(world.activations.contains(&ASKER));
}

#[test]
fn a_question_ending_unanswered_lifts_the_keep_with_a_notice() {
    let mut world = parked();
    response(&mut world, |reply_to| Event::CancelQuestion { reply_to, task: ASKER, question: QUESTION });
    assert!(world.record(ASKER).questions.is_empty());
    assert!(world.activations.contains(&ASKER));
    assert!(
        world.record(ASKER).inbox.iter().any(|word| word.kind == MessageKind::QuestionEnded { question: QUESTION })
    );
    assert!(world.record(REQUESTER).question_lines.is_empty());
}

#[test]
fn an_asked_task_ending_unanswered_notifies_each_asker() {
    // Move the asker under its own person so its old requester may end,
    // then ask that old requester through the retained bilateral reference.
    let mut world = moved();
    message(&mut world, Party::Person(PERSON), ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
    message(&mut world, Party::Task(ASKER), REQUESTER, LATER_QUESTION, MessageKind::Question);
    world.claim(ASKER, ANSWER);
    world.terminal(ASKER, End::Parked);
    world.claim(REQUESTER, LATER_QUESTION);
    world.finish(REQUESTER);
    world.settle(REQUESTER);
    assert!(world.record(ASKER).questions.is_empty());
    assert!(
        world
            .record(ASKER)
            .inbox
            .iter()
            .any(|word| word.kind == MessageKind::QuestionEnded { question: LATER_QUESTION })
    );
    assert!(world.activations.contains(&ASKER));
}

/// A single tasks step's publications, before the parent numbers continuations.
fn step(domain: &mut Domain, env: &skein_lib::Env<Limits>, event: Event) -> Vec<Request> {
    let mut out = Queue::uncapped();
    tasks::step(domain, env, event, &mut out);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}

const SECOND_ASKER: u64 = 3;
const FIRST_PEER: u64 = 4;
const SECOND_PEER: u64 = 5;
const ASKER_COUNT: usize = 2;

fn fanout() -> (Domain, skein_lib::Env<Limits>) {
    let limits = Limits { message_bytes: 8, ..LIMITS };
    let mut world = World::new(881, limits);
    let mut parent = task(REQUESTER, &[]);
    parent.numbers.budget = 10_000;
    parent.authority.budget.spend = 10_000;
    world.make(Party::Person(PERSON), vec![parent]);
    let children = [ASKER, SECOND_ASKER, FIRST_PEER, SECOND_PEER].map(|number| {
        let mut child = task(number, &[]);
        child.funder = Funder::Task(REQUESTER);
        child
    });
    world.make(Party::Task(REQUESTER), children.into());
    let mut serial = 0;
    for asker in [ASKER, SECOND_ASKER] {
        world.claim(asker, asker);
        for peer in [FIRST_PEER, SECOND_PEER] {
            response(&mut world, |reply_to| Event::Introduce { reply_to, by: REQUESTER, left: asker, right: peer });
            // Each peer receives half of each asker's full credit list.
            for _ in 0..limits.inbox_messages / u32::try_from(ASKER_COUNT).expect("asker count") {
                serial += 1;
                message(&mut world, Party::Task(asker), peer, serial, MessageKind::Question);
            }
        }
        world.terminal(asker, End::Parked);
    }
    let mut domain = jig_tasks_world::domain(&limits, 881);
    for record in world.records.values().filter(|row| row.key().live()) {
        step(&mut domain, &world.env, Event::Restore { record: record.clone() });
    }
    step(&mut domain, &world.env, Event::Check);
    step(&mut domain, &world.env, Event::Restored);
    (domain, world.env)
}

fn publications(out: &[Request], task: u64) -> usize {
    out.iter()
        .filter(|request| {
            matches!(request,
        Request::Save { record: Stored::Live(row) } if row.number == task)
        })
        .count()
}

#[test]
fn moving_full_credit_lists_publishes_each_affected_task_once() {
    let (mut domain, env) = fanout();
    for asker in [ASKER, SECOND_ASKER] {
        let original = domain.task(asker).expect("asker").questions.clone();
        let out = step(
            &mut domain,
            &env,
            Event::Move {
                reply_to: ReplyTo::new(Token::new(1)),
                task: asker,
                person: PERSON,
                period: 0,
                pool_budget: 10_000,
                period_budget: 100_000,
                reason: Box::new([]),
            },
        );
        let row = domain.task(asker).expect("moved asker");
        assert_eq!(row.questions.len(), usize::try_from(env.limits.inbox_messages).expect("credit bound"));
        assert_eq!(
            row.questions.iter().map(|credit| (credit.number, credit.line)).collect::<Vec<_>>(),
            original.iter().map(|credit| (credit.number, credit.line)).collect::<Vec<_>>()
        );
        assert!(row.questions.iter().all(|credit| credit.asked == Party::Person(PERSON)));
        for affected in [REQUESTER, asker, FIRST_PEER, SECOND_PEER] {
            assert_eq!(publications(&out, affected), 1, "task {affected}");
        }
        assert!(!out.iter().any(|request| matches!(
            request,
            Request::Save {
                record: Stored::Activity(ActivityLine { line: Line::Words { kind: MessageKind::Question, .. }, .. })
            }
        )));
    }
}

#[test]
fn ending_full_credit_lists_emits_one_batch_per_asker_and_one_publication() {
    let (mut domain, env) = fanout();
    step(
        &mut domain,
        &env,
        Event::Control {
            reply_to: ReplyTo::new(Token::new(1)),
            by: Party::Task(REQUESTER),
            task: FIRST_PEER,
            control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
        },
    );
    step(&mut domain, &env, Event::EffectsSettled { task: FIRST_PEER });
    let out = step(&mut domain, &env, Event::Settled { task: FIRST_PEER });
    let batches = out
        .iter()
        .filter_map(|request| {
            let Request::QuestionEndedDue { task, questions } = request else { return None };
            Some((*task, questions.clone()))
        })
        .collect::<Vec<_>>();
    assert_eq!(batches.len(), ASKER_COUNT);
    for (task, questions) in batches {
        assert_eq!(questions.len(), usize::try_from(env.limits.inbox_messages).expect("credit bound") / ASKER_COUNT);
        assert_eq!(publications(&out, task), 1);
        let notices = questions
            .iter()
            .enumerate()
            .map(|(at, question)| QuestionNotice {
                question: *question,
                message: 100 + u64::try_from(at).expect("bounded notice index"),
            })
            .collect();
        let out = step(&mut domain, &env, Event::QuestionEndedNotice { task, notices });
        assert_eq!(publications(&out, task), 1);
        let row = domain.task(task).expect("asker");
        assert_eq!(
            row.inbox.iter().filter(|word| matches!(word.kind, MessageKind::QuestionEnded { .. })).count(),
            questions.len()
        );
    }
}

fn person_asked() -> World {
    let mut world = pair();
    response(&mut world, |reply_to| Event::PersonQuestion {
        reply_to,
        project: 1,
        task: REQUESTER,
        person: PERSON,
        word: Word {
            number: QUESTION,
            from: Party::Task(ASKER),
            kind: MessageKind::Question,
            words: b"proceed?".as_slice().into(),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        },
    });
    assert_eq!(world.record(ASKER).questions[0].answerer, Party::Person(PERSON));
    world
}

fn person_reply(world: &mut World, person: u64) -> Reply {
    let number = world.record(REQUESTER).last_message.max(world.record(ASKER).last_message) + 1;
    response(world, |reply_to| Event::PersonQuestionAnswer {
        reply_to,
        project: 1,
        asked: REQUESTER,
        person,
        question: QUESTION,
        notice: number + 1,
        word: Word {
            number,
            from: Party::Person(person),
            kind: MessageKind::Answer { question: QUESTION },
            words: b"proceed".as_slice().into(),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        },
    })
}

#[test]
fn the_persons_answer_reaches_the_asking_task() {
    let mut world = person_asked();
    world.terminal(ASKER, report());
    world.restart();
    assert!(!world.record(ASKER).wake.answers);
    person_reply(&mut world, PERSON);
    assert!(world.record(ASKER).questions.is_empty());
    assert!(world.activations.contains(&ASKER));
    assert_eq!(world.contexts[&ASKER].inbox[0].from, Party::Task(REQUESTER));
    assert_eq!(world.record(REQUESTER).question_lines[0].answer.expect("first answer retained").task, ASKER);
    world.claim(ASKER, ANSWER);
    world.finish(ASKER);
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Done(_)));
}

#[test]
fn the_asked_task_hears_its_person_answered() {
    let mut world = person_asked();
    person_reply(&mut world, PERSON);
    let row = world.record(REQUESTER);
    assert!(row.inbox.iter().any(|word| word.kind == MessageKind::QuestionAnswered { question: QUESTION }));
    assert!(!row.inbox.iter().any(|word| matches!(word.kind, MessageKind::Answer { .. })));
}

#[test]
fn the_current_person_answers_after_the_asked_task_moves() {
    const NEXT_PERSON: u64 = PERSON + 1;
    let mut world = person_asked();
    response(&mut world, |reply_to| Event::Move {
        reply_to,
        task: REQUESTER,
        person: NEXT_PERSON,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
        reason: Box::new([]),
    });
    assert_eq!(world.record(ASKER).questions[0].answerer, Party::Person(NEXT_PERSON));
    person_reply(&mut world, PERSON);
    assert_eq!(world.record(ASKER).questions.len(), 1);
    person_reply(&mut world, NEXT_PERSON);
    assert!(world.record(ASKER).questions.is_empty());
}

#[test]
fn a_full_inbox_keeps_room_for_its_person_answer_notice() {
    let mut world = person_asked();
    // One delegate, the unread question and its notice credit use three slots.
    for number in 2..=6 {
        message(&mut world, Party::Person(PERSON), REQUESTER, number, MessageKind::Words);
    }
    let before = world.record(REQUESTER).inbox.len();
    message(&mut world, Party::Person(PERSON), REQUESTER, 7, MessageKind::Words);
    assert_eq!(world.record(REQUESTER).inbox.len(), before, "ordinary words cannot consume the notice credit");
    person_reply(&mut world, PERSON);
    assert_eq!(world.record(REQUESTER).inbox.len(), before + 1);
    assert!(world.record(ASKER).questions.is_empty());
    let once = world.record(REQUESTER).clone();
    person_reply(&mut world, PERSON);
    assert_eq!(world.record(REQUESTER), &once, "a repeated answer cannot make a second notice");
}

const OWED: &[u8] = b"You owe your requester a question: ask your plan with message, as a question, and wait.";

#[test]
fn a_task_owing_a_question_parks_on_finish_and_wakes_with_a_notice() {
    let mut world = pair_with_asks(true);
    world.terminal(ASKER, report());
    let row = world.record(ASKER);
    assert_eq!(row.phase, Phase::Active(Active::Due));
    assert_eq!(row.tries.run, 1);
    assert!(row.inbox.iter().any(|word| word.kind == MessageKind::QuestionOwed && word.words.as_ref() == OWED));
    assert!(world.records.values().any(|row| matches!(
        row,
        Stored::Activity(ActivityLine {
            task: ASKER,
            line: Line::RunEnded { attempt: ATTEMPT, end: EndKind::Parked, .. },
            ..
        })
    )));
    world.restart();
    assert_eq!(world.record(ASKER).tries.run, 1);
    assert!(world.record(ASKER).inbox.iter().any(|word| word.words.as_ref() == OWED));
    world.claim(ASKER, ATTEMPT + 1);
}

#[test]
fn once_asked_the_owing_task_is_kept_by_its_question() {
    let mut world = pair_with_asks(true);
    message(&mut world, Party::Task(ASKER), REQUESTER, QUESTION, MessageKind::Question);
    world.terminal(ASKER, report());
    world.restart();
    assert!(world.record(ASKER).asked_requester);
    assert_eq!(world.record(ASKER).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(ASKER).questions[0].number, QUESTION);
    assert!(!world.record(ASKER).inbox.iter().any(|word| word.kind == MessageKind::QuestionOwed));
    message(&mut world, Party::Task(REQUESTER), ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
    world.restart();
    assert!(world.record(ASKER).asked_requester);
    assert!(world.record(ASKER).questions.is_empty());
    world.claim(ASKER, ATTEMPT + 1);
    world.finish(ASKER);
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Done(_)));
}

#[test]
fn an_owing_task_that_never_asks_ends_by_its_tries() {
    let mut world = pair_with_asks(true);
    let attempts = u64::from(world.env.limits.retries.run.retries) + 1;
    for attempt in 1..=attempts {
        if attempt > ATTEMPT {
            world.claim(ASKER, attempt);
        }
        world.terminal(ASKER, report());
    }
    assert_eq!(u64::from(world.record(ASKER).tries.run), attempts);
    assert!(matches!(world.record(ASKER).phase, Phase::Closing(Closing { ending: Ending::Failed { .. }, .. })));
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Failed { .. }));
}

#[test]
fn asks_cannot_be_amended() {
    for (old, replacement) in [(false, true), (true, false)] {
        let mut world = pair_with_asks(old);
        let mut spec = world.record(ASKER).spec.clone();
        spec.asks = replacement;
        let amendment = Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: Some(spec),
            wake: None,
            dependencies: None,
            authority: None,
            llm: None,
            reason: Box::new([]),
        };
        let reply = response(&mut world, |reply_to| Event::Amend {
            reply_to,
            by: Party::Task(REQUESTER),
            task: ASKER,
            places: Box::new([]),
            funder: None,
            stop_run: false,
            amendment,
            message: ANSWER,
        });
        assert!(matches!(reply, Reply::Refused(problem) if problem.why == Refusal::Spec));
        assert_eq!(world.record(ASKER).spec.asks, old);
    }
}

#[test]
fn a_task_without_asks_finishes_as_before() {
    let mut world = pair();
    world.finish(ASKER);
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Done(_)));
}

#[test]
fn unread_words_cannot_prepare_an_owing_task_before_its_reminder() {
    let mut world = pair_with_asks(true);
    message(&mut world, Party::Task(REQUESTER), ASKER, QUESTION, MessageKind::Words);
    world.terminal(ASKER, report());
    let context = &world.contexts[&ASKER];
    assert!(context.inbox.iter().any(|word| word.kind == MessageKind::QuestionOwed && word.words.as_ref() == OWED));
    assert!(!world.record(ASKER).asked_requester);
}

#[test]
fn an_inbox_too_small_for_the_fixed_reminder_refuses_asks_at_creation() {
    let limits = Limits { inbox_bytes: LIMITS.message_bytes, ..LIMITS };
    let mut world = World::new(882, limits);
    let mut new = task(REQUESTER, &[]);
    new.spec.asks = true;
    let reply = response(&mut world, |reply_to| Event::Make {
        reply_to,
        creator: Party::Person(PERSON),
        batch: Box::new([new]),
    });
    assert!(matches!(reply, Reply::Refused(problem) if problem.why == Refusal::Spec));
    assert!(!world.records.contains_key(&Key::Live(REQUESTER)));
}
