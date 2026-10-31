//! Whole-set checks precede replay (domain/engine.md, 6; tasks.md, 6.7).
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "entry-point fixtures are ordinary Rust (programming-model.md, 10.2)"
)]
use super::revisions::{Harness, LIMITS, task};
use crate::{
    Active, Closing, Domain, Ending, Event, Key, Name, Phase, Refusal, Request, Stage, Status, Stored, Stub, Writer,
    WriterSlot,
};
use skein_lib::Token;

mod reports;

const DUE: u64 = 1;
const RUNNING: u64 = 2;
const STOPPING: u64 = 3;
const EFFECTS: u64 = 4;
const RELEASING: u64 = 5;
const UNNAMED: u64 = 99;
const ATTEMPT: u64 = 7;

fn cold(source: &Harness) -> Harness {
    let mut cold = Harness::new();
    cold.domain = Domain::new(&LIMITS, 91, source.domain.newest(), super::charters(), Box::new([0]));
    cold.stored.clear();
    for (key, hold) in &source.domain.hold_kinds {
        cold.send(Event::Kinds {
            connector: key.connector,
            kinds: Box::new([crate::Kind { connector: key.connector, kind: key.kind, hold: *hold }]),
        });
    }
    for row in source.stored.values().filter(|row| row.key().live()) {
        cold.send(Event::Restore { record: row.clone() });
    }
    cold
}

fn one_task() -> Harness {
    let mut source = Harness::new();
    source.make(vec![task(DUE, &[])]);
    source
}

const PARENT: u64 = 1;
const DELEGATE: u64 = 2;
const PERSON: u64 = 9; // The existing Harness root creator, not the delegate's requester.
const WORD: u64 = 1;

fn delegate_with_person_word(kind: crate::MessageKind) -> Harness {
    let mut source = Harness::new();
    source.make(vec![task(PARENT, &[])]);
    source.send(Event::Make {
        reply_to: super::revisions::to(),
        creator: crate::Party::Task(PARENT),
        batch: Box::new([task(DELEGATE, &[])]),
    });
    let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DELEGATE)) else { panic!("delegate fixture") };
    row.last_message = WORD;
    row.inbox = Box::new([crate::Word {
        number: WORD,
        from: crate::Party::Person(PERSON),
        kind,
        words: b"whole words".as_slice().into(),
        at: source.env.wall,
        hits: 1,
        eligible: true,
    }]);
    source
}

#[test]
fn a_delegates_words_from_a_person_restore() {
    let source = delegate_with_person_word(crate::MessageKind::Words);
    let mut cold = cold(&source);
    let out = cold.send(Event::Check);
    assert!(cold.domain.checked(), "{out:?}");
    assert_eq!(cold.domain.task(DELEGATE).expect("restored delegate").inbox[0].from, crate::Party::Person(PERSON));
}

fn refusal(out: &[Request], why: Refusal) {
    let [Request::RestoreRefused { problem }] = out else { panic!("one restore refusal: {out:?}") };
    assert_eq!(problem.why, why);
}

fn lifecycle(request: &Request) -> bool {
    match request {
        Request::Activate { .. }
        | Request::Adopt { .. }
        | Request::Stop { .. }
        | Request::Close { .. }
        | Request::Release { .. } => true,
        _ => false,
    }
}

fn replay_fixture() -> Harness {
    let mut source = Harness::new();
    source.make(vec![
        task(DUE, &[]),
        task(RUNNING, &[]),
        task(STOPPING, &[]),
        task(EFFECTS, &[]),
        task(RELEASING, &[]),
    ]);
    for (number, phase) in [
        (RUNNING, Phase::Active(Active::Running { attempt: ATTEMPT })),
        (
            STOPPING,
            Phase::Closing(Closing {
                stage: Stage::Run { attempt: ATTEMPT },
                ending: Ending::Failed { reason: Box::new([]) },
            }),
        ),
        (EFFECTS, Phase::Closing(Closing { stage: Stage::Effects, ending: Ending::Failed { reason: Box::new([]) } })),
        (
            RELEASING,
            Phase::Closing(Closing { stage: Stage::Releases, ending: Ending::Failed { reason: Box::new([]) } }),
        ),
    ] {
        let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(number)) else { panic!("live fixture") };
        row.phase = phase;
        row.attempt = ATTEMPT;
        if number == RUNNING {
            row.run_since = Some(source.env.wall);
        }
    }
    cold(&source)
}

#[test]
fn checked_tasks_replay_each_lifecycle_request_exactly_once() {
    #[derive(Debug, PartialEq, Eq)]
    enum Replay {
        Activate(u64),
        Adopt(u64),
        Stop(u64),
        Close(u64),
        Release(u64),
    }
    let mut cold = replay_fixture();
    assert!(cold.send(Event::Check).is_empty());
    assert!(cold.domain.checked());
    assert!(!cold.domain.ready());
    assert!(cold.send(Event::Check).is_empty(), "a duplicate check cannot replay work");
    let lifecycle: Vec<_> = cold
        .send(Event::Restored)
        .into_iter()
        .filter(lifecycle)
        .map(|request| match request {
            Request::Activate { context } => Replay::Activate(context.task),
            Request::Adopt { task, attempt: ATTEMPT, kept: 0 } => Replay::Adopt(task),
            Request::Stop { task, attempt: ATTEMPT } => Replay::Stop(task),
            Request::Close { task, ending: Ending::Failed { .. } } => Replay::Close(task),
            Request::Release { task, ending: Ending::Failed { .. } } => Replay::Release(task),
            other => panic!("unexpected replay {other:?}"),
        })
        .collect();
    assert_eq!(
        lifecycle,
        [
            Replay::Activate(DUE),
            Replay::Adopt(RUNNING),
            Replay::Adopt(STOPPING),
            Replay::Stop(STOPPING),
            Replay::Close(EFFECTS),
            Replay::Release(RELEASING)
        ]
    );
    assert!(cold.domain.ready());
    assert!(cold.send(Event::Restored).is_empty(), "duplicate completion cannot replay work");
}

#[test]
fn a_delegates_question_or_answer_does_not_repeat_sender_admission() {
    for kind in [crate::MessageKind::Question, crate::MessageKind::Answer { question: WORD }] {
        let source = delegate_with_person_word(kind);
        loaded_delegate(&source);
    }
}

#[test]
fn a_delegates_words_do_not_repeat_person_admission() {
    let mut source = delegate_with_person_word(crate::MessageKind::Words);
    let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DELEGATE)) else { panic!("delegate fixture") };
    row.inbox[0].from = crate::Party::Person(0);
    loaded_delegate(&source);
}

fn loaded_delegate(source: &Harness) {
    let mut restored = cold(source);
    assert!(restored.send(Event::Check).is_empty());
    assert!(restored.domain.checked());
    assert_eq!(
        restored.domain.task(DELEGATE).expect("linked delegate"),
        match &source.stored[&Key::Live(DELEGATE)] {
            Stored::Live(row) => row.as_ref(),
            _ => panic!("delegate row"),
        }
    );
}

#[test]
#[should_panic(expected = "restore Live(1): missing task 99")]
fn a_broken_task_link_panics_at_link() {
    let mut source = one_task();
    let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("live fixture") };
    row.root = UNNAMED;
    cold(&source).send(Event::Check);
}

#[test]
fn a_stub_is_retained_without_rejudging_reciprocal_references() {
    let mut cold = cold(&one_task());
    cold.send(Event::Restore {
        record: Stored::Stub(Stub { task: UNNAMED, phase: Status::Done, result: Token::new(UNNAMED) }),
    });
    assert!(cold.send(Event::Check).is_empty());
    assert!(cold.domain.checked());
}

#[test]
#[should_panic(expected = "restore Writer(1): missing hold owner")]
fn a_writer_without_a_holder_panics_at_link() {
    let mut cold = cold(&one_task());
    cold.send(Event::Restore {
        record: Stored::Writer(WriterSlot {
            number: 1,
            resource: Name { connector: 1, path: Box::new([b"writer".as_slice().into()]) },
            writer: Writer::Effect { entry: 1 },
            lost: false,
        }),
    });
    cold.send(Event::Check);
}

mod detail;
