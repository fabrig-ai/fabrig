//! Requester question keeps and task-batched exits (domain/tasks/lifecycle.md, 8 and domain/tasks/changes.md).
use jig_domain_tasks::{self as tasks, Cancel, *};
use jig_tasks_world::{LIMITS, PERSON, PERSON_PARTY, Reply, World, task};
use skein_lib::{Queue, ReplyTo, Token, Wall};

const REQUESTER: u64 = 1;
const ASKER: u64 = 2;
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
    message_words(world, source, target, number, kind, b"proceed?")
}

fn message_words(world: &mut World, source: Party, target: u64, number: u64, kind: MessageKind, words: &[u8]) -> Reply {
    response(world, |reply_to| Event::Message {
        reply_to,
        project: 1,
        operation: tasks::Message {
            task: target,
            word: Word { number, from: source, kind, words: words.into(), at: Wall::EPOCH, hits: 1, eligible: false },
        },
    })
}

fn pair() -> World {
    pair_with_asks(false)
}

fn pair_with_asks(asks: bool) -> World {
    pair_with_limits(asks, &LIMITS)
}

fn pair_with_limits(asks: bool, limits: &Limits) -> World {
    let mut world = World::new(880, *limits);
    let mut parent = task(REQUESTER, &[]);
    parent.numbers.budget = 1_000;
    parent.authority.budget.spend = 1_000;
    world.make(PERSON_PARTY, vec![parent]);
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
    assert_question_parked(&world);
}

fn assert_question_parked(world: &World) {
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
fn a_question_keep_parks_a_finished_failure() {
    let mut world = asked();
    world.terminal(ASKER, failure());
    assert_question_parked(&world);
    message(&mut world, Party::Task(REQUESTER), ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
    assert!(world.record(ASKER).questions.is_empty());
    assert!(world.activations.contains(&ASKER));
    world.claim(ASKER, ANSWER);
    world.terminal(ASKER, failure());
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Failed { .. }));
}

fn failure() -> End {
    End::Finished {
        result: TaskResult::Failure { reason: b"cannot proceed".as_slice().into() },
        cancel_delegates: false,
    }
}

#[test]
fn a_question_keep_lets_a_failing_run_count() {
    let mut world = asked();
    world.terminal(ASKER, End::Failed(Class::Run));
    assert_eq!(world.record(ASKER).tries.of(Class::Run), 1);
    assert!(matches!(world.record(ASKER).phase, Phase::Active(Active::BackingOff { .. })));
}

#[test]
fn a_question_keep_permits_cancellation() {
    let mut world = parked();
    response(&mut world, |reply_to| Event::Cancel {
        reply_to,
        by: PERSON_PARTY,
        operation: Cancel { task: ASKER, reason: Box::new([]), grace: None, message: None },
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
        operation: tasks::Move { task: ASKER, reason: Box::new([]) },
        reply_to,
        person: PERSON,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
    });
    world.restart();
    world
}

#[test]
fn a_moved_asker_carries_its_open_question_to_its_new_requester() {
    let mut world = moved();
    let row = world.record(ASKER);
    assert_eq!(row.requester, PERSON_PARTY);
    assert_eq!(row.phase, Phase::Active(Active::Idle));
    let credit = row.questions[0].clone();
    assert_eq!(credit.number, QUESTION);
    assert_eq!(credit.answerer, PERSON_PARTY);
    assert_eq!(credit.asked, PERSON_PARTY);
    assert_eq!(row.question_lines[0].line, credit.line);
    assert!(matches!(&world.records[&Key::Activity { task: ASKER, position: credit.line }],
        Stored::Activity(ActivityLine { line: Line::Words { message: QUESTION, words, .. }, .. })
            if words.as_ref() == b"proceed?"));
    assert!(world.record(REQUESTER).question_lines.is_empty());
    message(&mut world, PERSON_PARTY, ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
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
    message(&mut world, PERSON_PARTY, ASKER, ANSWER, MessageKind::Answer { question: QUESTION });
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
    world.make(PERSON_PARTY, vec![parent]);
    let children = [ASKER, SECOND_ASKER, FIRST_PEER, SECOND_PEER].map(|number| {
        let mut child = task(number, &[]);
        child.funder = Funder::Task(REQUESTER);
        child
    });
    world.make(Party::Task(REQUESTER), children.into());
    let mut serial = 0;
    for asker in [ASKER, SECOND_ASKER] {
        world.claim_task(asker);
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
                operation: tasks::Move { task: asker, reason: Box::new([]) },
                reply_to: ReplyTo::new(Token::new(1)),
                person: PERSON,
                period: 0,
                pool_budget: 10_000,
                period_budget: 100_000,
            },
        );
        let row = domain.task(asker).expect("moved asker");
        assert_eq!(row.questions.len(), skein_lib::length::usize_of(env.limits.inbox_messages));
        assert_eq!(
            row.questions.iter().map(|credit| (credit.number, credit.line)).collect::<Vec<_>>(),
            original.iter().map(|credit| (credit.number, credit.line)).collect::<Vec<_>>()
        );
        assert!(row.questions.iter().all(|credit| credit.asked == PERSON_PARTY));
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
        Event::Cancel {
            reply_to: ReplyTo::new(Token::new(1)),
            by: Party::Task(REQUESTER),
            operation: Cancel { task: FIRST_PEER, reason: Box::new([]), grace: None, message: None },
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
        assert_eq!(questions.len(), skein_lib::length::usize_of(env.limits.inbox_messages) / ASKER_COUNT);
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
    person_asked_with(&LIMITS)
}

fn person_asked_with(limits: &Limits) -> World {
    let mut world = pair_with_limits(false, limits);
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
    assert_eq!(world.record(ASKER).questions[0].answerer, PERSON_PARTY);
    world
}

fn person_reply(world: &mut World, person: u64) -> Reply {
    person_reply_with(world, person, b"proceed")
}

fn person_reply_with(world: &mut World, person: u64, words: &[u8]) -> Reply {
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
            words: words.into(),
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
    let notice = row
        .inbox
        .iter()
        .find(|word| word.kind == MessageKind::QuestionAnswered { question: QUESTION })
        .expect("answer notice");
    assert_eq!(notice.from, Party::Task(ASKER));
    assert_eq!(notice.words.as_ref(), b"proceed");
    assert!(!row.inbox.iter().any(|word| matches!(word.kind, MessageKind::Answer { .. })));
}

#[test]
fn answer_notices_keep_the_bound_and_mark_utf8_cuts() {
    const BOUND: usize = 256;
    const ASCII_PREFIX: usize = 240;
    const UTF8_PREFIX: usize = 120;
    let cases = [
        ("a".repeat(BOUND - 1), "a".repeat(BOUND - 1)),
        ("a".repeat(BOUND), "a".repeat(BOUND)),
        ("a".repeat(BOUND + 1), format!("{}\n[17 bytes cut]\n", "a".repeat(ASCII_PREFIX))),
        ("é".repeat(129), format!("{}\n[18 bytes cut]\n", "é".repeat(UTF8_PREFIX))),
    ];
    for (answer, expected) in cases {
        let mut world = person_asked_with(&Limits { message_bytes: 512, inbox_bytes: 1024, ..LIMITS });
        person_reply_with(&mut world, PERSON, answer.as_bytes());
        let notice = world
            .record(REQUESTER)
            .inbox
            .iter()
            .find(|word| word.kind == MessageKind::QuestionAnswered { question: QUESTION })
            .expect("answer notice");
        assert_eq!(notice.words.as_ref(), expected.as_bytes());
        assert!(world.record(ASKER).inbox.iter().any(|word| word.words.as_ref() == answer.as_bytes()));
        world.restart();
    }
}

#[test]
fn admitting_a_question_keeps_the_answer_notice_bytes() {
    const NEEDED: u32 = LIMITS.result_bytes + 8 + 256;
    for (inbox_bytes, accepted) in [(NEEDED - 1, false), (NEEDED, true)] {
        let mut world = pair_with_limits(false, &Limits { inbox_bytes, ..LIMITS });
        let reply = message(&mut world, Party::Task(ASKER), REQUESTER, QUESTION, MessageKind::Question);
        assert_eq!(matches!(reply, Reply::Done), accepted);
        assert_eq!(world.record(ASKER).questions.len(), usize::from(accepted));
        world.restart();
    }
}

#[test]
fn empty_answer_notices_still_restore() {
    let mut world = person_asked();
    person_reply(&mut world, PERSON);
    for record in world.records.values_mut() {
        if let Stored::Live(row) = record {
            for word in &mut row.inbox {
                if word.kind == (MessageKind::QuestionAnswered { question: QUESTION }) {
                    word.words = Box::new([]);
                }
            }
        }
    }
    world.restart();
    assert!(
        world
            .record(REQUESTER)
            .inbox
            .iter()
            .any(|word| word.kind == MessageKind::QuestionAnswered { question: QUESTION } && word.words.is_empty())
    );
}

#[test]
fn a_byte_full_inbox_keeps_the_whole_answer_notice_credit() {
    const BOUND: u32 = 256;
    const WORD_BYTES: u32 = 8;
    let limits =
        Limits { message_bytes: BOUND, inbox_bytes: LIMITS.result_bytes + WORD_BYTES + BOUND + WORD_BYTES, ..LIMITS };
    let mut world = person_asked_with(&limits);
    assert!(matches!(message(&mut world, PERSON_PARTY, REQUESTER, 2, MessageKind::Words), Reply::Done));
    assert!(matches!(message(&mut world, PERSON_PARTY, REQUESTER, 3, MessageKind::Words), Reply::Refused(_)));
    world.restart();
    person_reply_with(&mut world, PERSON, &vec![b'a'; BOUND as usize]);
    let row = world.record(REQUESTER);
    assert_eq!(
        row.inbox.iter().map(|word| word.words.len()).sum::<usize>() + LIMITS.result_bytes as usize,
        limits.inbox_bytes as usize
    );
    assert_eq!(row.inbox.last().expect("answer notice").words.len(), BOUND as usize);
    world.restart();
}

#[test]
fn the_current_person_answers_after_the_asked_task_moves() {
    const NEXT_PERSON: u64 = PERSON + 1;
    let mut world = person_asked();
    response(&mut world, |reply_to| Event::Move {
        operation: tasks::Move { task: REQUESTER, reason: Box::new([]) },
        reply_to,
        person: NEXT_PERSON,
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
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
        message(&mut world, PERSON_PARTY, REQUESTER, number, MessageKind::Words);
    }
    let before = world.record(REQUESTER).inbox.len();
    message(&mut world, PERSON_PARTY, REQUESTER, 7, MessageKind::Words);
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
    assert!(!row.inbox.iter().any(|word| word.kind == MessageKind::NothingAwaited));
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
            operation: tasks::Amend { task: ASKER, stop_run: false, amendment: Box::new(amendment), message: ANSWER },
            reply_to,
            by: Party::Task(REQUESTER),
            places: Box::new([]),
            funder: None,
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
fn asks_requires_room_for_the_fixed_answer_notice() {
    const BOUND: u32 = 256;
    for (inbox_bytes, accepted) in [(BOUND - 1, false), (BOUND, true)] {
        let limits = Limits { inbox_bytes, ..LIMITS };
        let mut world = World::new(882, limits);
        let mut new = task(REQUESTER, &[]);
        new.spec.asks = true;
        let reply =
            response(&mut world, |reply_to| Event::Make { reply_to, creator: PERSON_PARTY, batch: Box::new([new]) });
        assert_eq!(matches!(reply, Reply::Made(_)), accepted);
        assert_eq!(world.records.contains_key(&Key::Live(REQUESTER)), accepted);
        world.restart();
    }
}

fn read_and_park_requester(world: &mut World, attempt: u64) {
    world.claim(REQUESTER, attempt);
    read_inbox(world, REQUESTER, attempt);
    world.terminal(REQUESTER, End::Parked);
}

#[test]
fn a_read_question_left_open_wakes_the_requester_without_a_notice() {
    let mut world = asked();
    let last = world.record(REQUESTER).last_message;
    read_and_park_requester(&mut world, ATTEMPT);
    assert_eq!(world.record(REQUESTER).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(REQUESTER).last_message, last);
    assert!(world.record(REQUESTER).inbox.is_empty());
    assert_eq!(world.record(REQUESTER).tries.run, 1);
    assert!(world.contexts[&REQUESTER].reasons.contains(Reasons::QUESTION));
}

#[test]
fn repeated_parks_with_an_open_question_hold_and_release_counts_again() {
    let mut world = asked();
    let attempts = u64::from(world.env.limits.retries.run.retries) + 1;
    for attempt in 1..=attempts {
        read_and_park_requester(&mut world, attempt);
        assert_eq!(u64::from(world.record(REQUESTER).tries.run), attempt);
        if attempt < attempts {
            assert_eq!(world.record(REQUESTER).phase, Phase::Active(Active::Due));
            world.restart();
        }
    }
    let row = world.record(REQUESTER);
    let Phase::Held { why: Hold::Failures(Class::Run), source: Some(source), .. } = &row.phase else {
        panic!("question exhaustion holds, never fails: {:?}", row.phase)
    };
    assert_eq!(
        source.as_ref(),
        &HoldSource::Preparation { words: b"Task 2's question 1 is still open: proceed?".as_slice().into() }
    );
    assert!(matches!(world.record(ASKER).phase, Phase::Active(_)));
    assert_eq!(
        response(&mut world, |reply_to| Event::Release {
            reply_to,
            by: PERSON_PARTY,
            operation: Release { task: REQUESTER, again: None, message: None },
        }),
        Reply::Done
    );
    read_and_park_requester(&mut world, attempts + 1);
    assert_eq!(world.record(REQUESTER).tries.run, 1);
    assert_eq!(world.record(REQUESTER).phase, Phase::Active(Active::Due));
}

#[test]
fn a_question_hold_keeps_only_the_first_line_at_a_utf8_edge() {
    let prefix = "Task 2's question 1 is still open: ";
    let long = format!("{}\nnot the first line", "é".repeat(100));
    let expected = format!("{prefix}{}", "é".repeat((128 - prefix.len()) / "é".len()));
    for (words, expected) in
        [(long.as_str(), expected.as_str()), ("short\nlater", "Task 2's question 1 is still open: short")]
    {
        let limits = Limits {
            message_bytes: 256,
            inbox_bytes: 1024,
            retries: Retries { run: Retry { retries: 0, ..LIMITS.retries.run }, ..LIMITS.retries },
            ..LIMITS
        };
        let mut world = pair_with_limits(false, &limits);
        assert_eq!(
            message_words(&mut world, Party::Task(ASKER), REQUESTER, QUESTION, MessageKind::Question, words.as_bytes()),
            Reply::Done
        );
        read_and_park_requester(&mut world, ATTEMPT);
        let Phase::Held { source: Some(source), .. } = &world.record(REQUESTER).phase else {
            panic!("held with question words")
        };
        assert_eq!(source.as_ref(), &HoldSource::Preparation { words: expected.as_bytes().into() });
        assert!(expected.len() <= 128);
    }
}

#[derive(Clone, Copy, Debug)]
enum AwaitingAnswer {
    Kept,
    Proposal,
    Person,
    Timer,
    Watch,
    Quiet,
    Conversation,
    RequesterEnded,
    Resuming,
    Standing,
}

fn keep_requester(world: &mut World) {
    const DEPENDENCY: u64 = 4;
    assert_eq!(world.make(PERSON_PARTY, vec![task(DEPENDENCY, &[])]), Reply::Made(vec![DEPENDENCY]));
    let amendment = Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: Some(Box::new([DEPENDENCY])),
        authority: None,
        llm: None,
        reason: Box::new([]),
    };
    assert_eq!(
        response(world, |reply_to| Event::Amend {
            reply_to,
            by: PERSON_PARTY,
            operation: tasks::Amend {
                task: REQUESTER,
                stop_run: false,
                amendment: Box::new(amendment),
                message: ANSWER,
            },
            places: Box::new([]),
            funder: None,
        }),
        Reply::Done
    );
}

#[test]
fn an_open_question_does_not_poll_while_something_can_bring_the_answer() {
    for exemption in [
        AwaitingAnswer::Kept,
        AwaitingAnswer::Proposal,
        AwaitingAnswer::Person,
        AwaitingAnswer::Timer,
        AwaitingAnswer::Watch,
        AwaitingAnswer::Quiet,
        AwaitingAnswer::Conversation,
        AwaitingAnswer::RequesterEnded,
        AwaitingAnswer::Resuming,
        AwaitingAnswer::Standing,
    ] {
        let mut world = World::new(883, LIMITS);
        let mut parent = task(REQUESTER, &[]);
        parent.numbers.budget = 1_000;
        parent.authority.budget.spend = 1_000;
        match exemption {
            AwaitingAnswer::Quiet => parent.wake.questions = false,
            AwaitingAnswer::Conversation => parent.flags.conversation = true,
            AwaitingAnswer::RequesterEnded => {
                parent.flags.conversation = true;
                parent.flags.ended_by = EndedBy::Requester;
            }
            AwaitingAnswer::Standing => parent.flags.ended_by = EndedBy::Standing,
            _ => {}
        }
        if matches!(exemption, AwaitingAnswer::Resuming) {
            let mut charters = jig_tasks_world::charters();
            charters[0].resumes = true;
            world.send(Event::Charters { newest: 1, charters, efforts: Box::new(jig_tasks_world::EFFORTS) });
        }
        assert_eq!(world.make(PERSON_PARTY, vec![parent]), Reply::Made(vec![REQUESTER]));
        let mut child = task(ASKER, &[]);
        child.funder = Funder::Task(REQUESTER);
        assert_eq!(world.make(Party::Task(REQUESTER), vec![child]), Reply::Made(vec![ASKER]));
        world.claim(ASKER, ATTEMPT);
        message(&mut world, Party::Task(ASKER), REQUESTER, QUESTION, MessageKind::Question);
        world.claim_budget(REQUESTER, ATTEMPT, 1);
        read_inbox(&mut world, REQUESTER, ATTEMPT);
        match exemption {
            AwaitingAnswer::Kept => keep_requester(&mut world),
            AwaitingAnswer::Proposal => {
                let proposal = Proposal {
                    number: 100,
                    proposer: REQUESTER,
                    project: 1,
                    action: ProposalAction::Batch(Box::new([])),
                    reason: Box::new([1]),
                    as_holder: false,
                    state: ProposalState::Pending { holder: ProposalHolder::Person(PERSON), since: world.env.wall },
                };
                assert_eq!(response(&mut world, |reply_to| Event::Propose { reply_to, proposal }), Reply::Done);
            }
            AwaitingAnswer::Person => {
                let mut person = task(PERSON_TASK, &[]);
                person.executor = Executor::Person(PersonAddress::Person(PERSON));
                person.numbers.budget = 20;
                person.authority.budget.spend = 20;
                person.funder = Funder::Task(REQUESTER);
                assert_eq!(world.make(Party::Task(REQUESTER), vec![person]), Reply::Made(vec![PERSON_TASK]));
            }
            AwaitingAnswer::Timer | AwaitingAnswer::Watch => {
                let kind = if matches!(exemption, AwaitingAnswer::Timer) {
                    SubscriptionKind::Timer { at: Wall::from_nanos(1_000_000), period: None }
                } else {
                    SubscriptionKind::Task { target: ASKER, held: true, result: true }
                };
                assert_eq!(
                    response(&mut world, |reply_to| Event::Subscribe {
                        reply_to,
                        task: REQUESTER,
                        subscription: Subscription { number: 1, kind },
                    }),
                    Reply::Done
                );
            }
            _ => {}
        }
        world.terminal(REQUESTER, End::Parked);
        assert_eq!(world.record(REQUESTER).phase, Phase::Active(Active::Idle), "{exemption:?}");
        assert_eq!(world.record(REQUESTER).tries, Tries::NONE, "{exemption:?}");
    }
}

const NOTHING_AWAITED: &[u8] =
    b"Nothing more will come: your delegates have ended and no question waits. Finish with your result.";

#[test]
fn a_result_ended_task_parking_after_its_delegate_ends_is_due_with_a_notice() {
    let mut world = pair();
    world.finish(ASKER);
    world.settle(ASKER);
    world.claim(REQUESTER, ATTEMPT);
    let read = world.record(REQUESTER).last_message;
    let outstanding = world.record(REQUESTER).inbox.iter().map(|word| word.number).collect();
    response(&mut world, |reply_to| Event::Turn {
        reply_to,
        task: REQUESTER,
        attempt: ATTEMPT,
        turn: 1,
        read: Some(read),
        refused: Box::new([]),
        cumulative: 0,
        offered: Some(read),
        outstanding,
    });
    world.terminal(REQUESTER, End::Parked);
    let row = world.record(REQUESTER);
    assert_eq!(row.phase, Phase::Active(Active::Due));
    assert_eq!(row.tries.run, 1);
    assert!(row.inbox.iter().any(|word| word.words.as_ref() == NOTHING_AWAITED));
}

fn read_inbox(world: &mut World, task: u64, attempt: u64) {
    let read = world.record(task).last_message;
    if read > 0 {
        let outstanding = world.record(task).inbox.iter().map(|word| word.number).collect();
        response(world, |reply_to| Event::Turn {
            reply_to,
            task,
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

#[test]
fn parking_with_nothing_awaited_exhausts_run_tries() {
    let mut world = pair();
    let attempts = u64::from(world.env.limits.retries.run.retries) + 1;
    for attempt in 1..=attempts {
        if attempt > ATTEMPT {
            world.claim(ASKER, attempt);
        }
        read_inbox(&mut world, ASKER, attempt);
        world.terminal(ASKER, End::Parked);
        if attempt < attempts {
            assert_eq!(u64::from(world.record(ASKER).tries.run), attempt);
            world.restart();
        }
    }
    assert_eq!(u64::from(world.record(ASKER).tries.run), attempts);
    assert!(matches!(world.record(ASKER).phase, Phase::Closing(Closing { ending: Ending::Failed { .. }, .. })));
    world.settle(ASKER);
    assert!(matches!(world.results[&ASKER], Ending::Failed { .. }));
}

#[test]
fn a_live_delegate_is_something_to_await() {
    let mut world = pair();
    world.claim(REQUESTER, ATTEMPT);
    world.terminal(REQUESTER, End::Parked);
    assert_eq!(world.record(REQUESTER).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(REQUESTER).tries, Tries::NONE);
}

#[test]
fn only_timers_and_task_watches_count_as_pending_arrivals() {
    for kind in [
        SubscriptionKind::Timer { at: Wall::from_nanos(1_000_000), period: None },
        SubscriptionKind::Task { target: REQUESTER, held: true, result: true },
        SubscriptionKind::Topic { connector: 1, topic: 1 },
    ] {
        let mut world = pair();
        let subscription = Subscription { number: 1, kind };
        let reply = response(&mut world, |reply_to| match kind {
            SubscriptionKind::Topic { .. } => Event::SubscribeTopic { reply_to, task: ASKER, subscription },
            SubscriptionKind::Task { .. } | SubscriptionKind::Timer { .. } => {
                Event::Subscribe { reply_to, task: ASKER, subscription }
            }
        });
        assert_eq!(reply, Reply::Done);
        world.terminal(ASKER, End::Parked);
        let topic = matches!(kind, SubscriptionKind::Topic { .. });
        assert_eq!(world.record(ASKER).phase, Phase::Active(if topic { Active::Due } else { Active::Idle }));
        assert_eq!(world.record(ASKER).tries.run, u32::from(topic));
    }
}

const GOAL: u64 = 4;
const GOAL_CHILD: u64 = 23;
const GOAL_PEER: u64 = 25;
const GOAL_PERSON: u64 = 35;

fn subscribing_goal(limits: &Limits) -> World {
    let mut world = World::new(471, *limits);
    let mut goal = task(GOAL, &[]);
    goal.tracked = Some(1);
    goal.numbers.budget = 1_000;
    goal.authority.budget.spend = 1_000;
    assert_eq!(world.make(PERSON_PARTY, vec![goal]), Reply::Made(vec![GOAL]));
    let delegates = [GOAL_CHILD, GOAL_PEER, GOAL_PERSON].map(|number| {
        let mut delegate = task(number, &[]);
        delegate.funder = Funder::Task(GOAL);
        if number == GOAL_PERSON {
            delegate.executor = Executor::Person(PersonAddress::Person(PERSON));
        }
        delegate
    });
    assert_eq!(world.make(Party::Task(GOAL), delegates.into()), Reply::Made(vec![23, 25, 35]));
    world.claim(GOAL, ATTEMPT);
    world
}

fn task_interest(number: u64, target: u64) -> Subscription {
    Subscription { number, kind: SubscriptionKind::Task { target, held: true, result: true } }
}

fn timer_interest(number: u64) -> Subscription {
    Subscription {
        number,
        kind: SubscriptionKind::Timer {
            at: Wall::from_nanos(1_000_000),
            period: Some(skein_lib::Duration::from_millis(1)),
        },
    }
}

fn subscribe_goal(world: &mut World, subscription: Subscription) -> Reply {
    response(world, |reply_to| Event::Subscribe { reply_to, task: GOAL, subscription })
}

fn subscription_full(world: &mut World, subscription: Subscription, cause: CapacityCause) {
    let before = world.record(GOAL).subscriptions.clone();
    assert_eq!(
        subscribe_goal(world, subscription),
        Reply::Refused(Problem::capacity(
            Some(GOAL),
            Refusal::Busy,
            Capacity {
                project: Some(1),
                by: Party::Task(GOAL),
                action: CapacityAction::Subscription { number: subscription.number },
                cause,
            },
        ))
    );
    assert_eq!(world.record(GOAL).subscriptions, before);
    assert_eq!(world.record(GOAL).phase, Phase::Active(Active::Running { attempt: ATTEMPT }));
}

fn replace_interest(world: &mut World, obsolete: u64, subscription: Subscription) {
    assert_eq!(
        response(world, |reply_to| Event::Unsubscribe { reply_to, task: GOAL, subscription: obsolete }),
        Reply::Done
    );
    assert_eq!(subscribe_goal(world, subscription), Reply::Done);
    assert!(world.record(GOAL).subscriptions.contains(&subscription));
}

#[test]
fn an_active_goals_subscription_limit_names_task_and_timer_refusals() {
    let mut world = subscribing_goal(&Limits { subscriptions: 4, inbox_messages: 12, ..LIMITS });
    for subscription in
        [task_interest(1, GOAL_CHILD), task_interest(2, GOAL_PEER), timer_interest(3), timer_interest(4)]
    {
        assert_eq!(subscribe_goal(&mut world, subscription), Reply::Done);
    }
    let cause = CapacityCause::Subscriptions { limit: 4 };
    subscription_full(&mut world, task_interest(5, GOAL_PERSON), cause);
    subscription_full(&mut world, timer_interest(6), cause);
    replace_interest(&mut world, 1, task_interest(5, GOAL_PERSON));
    replace_interest(&mut world, 3, timer_interest(6));
    assert_eq!(world.record(GOAL).subscriptions.len(), 4);
}

#[test]
fn an_active_goals_subscription_inbox_reservations_outlive_a_read() {
    // The goal keeps core news, three delegates' results and a person-stall slot.
    // Two standing interests then fill either the message or byte bound alone.
    for limits in
        [Limits { inbox_messages: 7, run_inbox: 7, ..LIMITS }, Limits { message_bytes: 16, inbox_bytes: 176, ..LIMITS }]
    {
        let mut world = subscribing_goal(&limits);
        assert_eq!(subscribe_goal(&mut world, task_interest(1, GOAL_CHILD)), Reply::Done);
        assert_eq!(subscribe_goal(&mut world, timer_interest(2)), Reply::Done);
        let cause = CapacityCause::Inbox { task: GOAL, messages: limits.inbox_messages, bytes: limits.inbox_bytes };
        for subscription in [task_interest(3, GOAL_PEER), task_interest(4, GOAL_PERSON), timer_interest(5)] {
            subscription_full(&mut world, subscription, cause);
        }
        world.send(Event::Notice {
            task: GOAL,
            word: Word {
                number: 1,
                from: Party::Task(GOAL),
                kind: MessageKind::Timer { subscription: 2 },
                words: b"tick".as_slice().into(),
                at: Wall::EPOCH,
                hits: 1,
                eligible: false,
            },
        });
        assert_eq!(world.record(GOAL).inbox.len(), 1);
        read_inbox(&mut world, GOAL, ATTEMPT);
        assert!(world.record(GOAL).inbox.is_empty());
        subscription_full(&mut world, task_interest(4, GOAL_PERSON), cause);
        subscription_full(&mut world, timer_interest(5), cause);
        replace_interest(&mut world, 1, task_interest(4, GOAL_PERSON));
        replace_interest(&mut world, 2, timer_interest(5));
        assert_eq!(world.record(GOAL).subscriptions.len(), 2);
    }
}
