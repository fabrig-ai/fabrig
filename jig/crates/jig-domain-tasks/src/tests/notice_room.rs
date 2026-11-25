//! Procedure question and notice capacity (jig/docs/design/domain/tasks/messages.md, 2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "step tests script admissions and inspect their outcomes in ordinary Rust"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Cancel, Event, Executor, Funder, Message, MessageKind, Party, ProcedureDecision, ProcedureEnd, ProcedureMessage,
    Refusal, Request, Word,
};

const ROOT: u64 = 1;
const ASKER: u64 = 2;
const ASKED: u64 = 3;
const PERSON: u64 = 9;
const QUESTION: u64 = 10;
const ANSWER: u64 = 11;
const SLOTS: u32 = 4;
const ROOMY_BYTES: u32 = 1024;
const NOTICE: usize = crate::ANSWER_NOTICE_BYTES;

fn delegate(h: &mut Harness, parent: u64, mut child: crate::New) {
    child.funder = Funder::Task(parent);
    let out = h.send(Event::Make {
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(parent),
        reads_project: false,
        batch: Box::new([child]),
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn setup() -> Harness {
    let mut h = Harness::new();
    let mut root = task(ROOT, &[]);
    root.numbers.budget = 1000;
    root.authority.budget.spend = 1000;
    h.make(vec![root]);
    let mut procedure = task(ASKER, &[]);
    procedure.executor = Executor::Procedure { connector: 1, code: 1 };
    procedure.numbers.budget = 500;
    procedure.authority.budget.spend = 500;
    delegate(&mut h, ROOT, procedure);
    delegate(&mut h, ASKER, task(ASKED, &[]));
    h.env.limits.inbox_messages = SLOTS;
    h.env.limits.inbox_bytes = ROOMY_BYTES;
    h.env.limits.run_inbox = SLOTS;
    h
}

fn message(task: u64, number: u64, from: Party, kind: MessageKind, words: &[u8]) -> Message {
    Message {
        task,
        word: Word { number, from, kind, words: words.into(), at: skein_lib::Wall::EPOCH, hits: 1, eligible: true },
    }
}

fn fill(h: &mut Harness, target: u64, number: u64) -> Vec<Request> {
    h.send(Event::AuthorizedMessage {
        reply_to: to(),
        project: 1,
        operation: message(target, number, Party::Person(PERSON), MessageKind::Words, b"w"),
    })
}

fn step(h: &mut Harness, messages: Box<[ProcedureMessage]>) -> Vec<Request> {
    let read = h.domain.procedure_offered(ASKER);
    let attempt = h.domain.task(ASKER).expect("fixture task or record").attempt;
    h.send(Event::Procedure {
        reply_to: to(),
        task: ASKER,
        step: attempt.checked_add(1).expect("next fixture step"),
        read,
        reads_project: false,
        decision: ProcedureDecision { messages, ..ProcedureDecision::new(ProcedureEnd::Wait) },
    })
}

fn ask(h: &mut Harness, target: u64) -> Vec<Request> {
    step(
        h,
        Box::new([ProcedureMessage::ordinary(message(
            target,
            QUESTION,
            Party::Task(ASKER),
            MessageKind::Question,
            b"?",
        ))]),
    )
}

fn answer(h: &mut Harness) {
    let out = h.send(Event::Message {
        reply_to: to(),
        project: 1,
        operation: message(ASKER, ANSWER, Party::Task(ASKED), MessageKind::Answer { question: QUESTION }, b"yes"),
    });
    sent(&out, true);
}

fn stepped(out: &[Request], refusal: Option<Refusal>) {
    assert!(
        out.iter().any(|r| match (&refusal, r) {
            (None, Request::Done { .. }) => true,
            (Some(why), Request::ProcedureRefused { problem, .. }) => &problem.why == why,
            _ => false,
        }),
        "{out:?}"
    );
}

fn sent(out: &[Request], admitted: bool) {
    assert!(
        out.iter().any(|r| match r {
            Request::Sent { .. } => admitted,
            Request::Refused { problem, .. } => !admitted && problem.why == Refusal::Busy && problem.capacity.is_some(),
            _ => false,
        }),
        "{out:?}"
    );
}

#[test]
fn asking_a_procedure_question_prices_one_more_slot_and_256_bytes() {
    let notice_bytes = u32::try_from(NOTICE).expect("fixed notice size");
    let question_and_notice = notice_bytes.checked_add(1).expect("notice and one byte question");
    for (held, bytes, admitted) in [
        (SLOTS - 2, ROOMY_BYTES, true),
        (SLOTS - 1, ROOMY_BYTES, false),
        (0, question_and_notice, true),
        (0, notice_bytes, false),
    ] {
        let mut h = setup();
        h.env.limits.inbox_bytes = bytes;
        for number in 1..=held {
            sent(&fill(&mut h, ASKED, u64::from(number)), true);
        }
        stepped(&ask(&mut h, ASKED), if admitted { None } else { Some(Refusal::Busy) });
    }

    // The top-level requester also reserves a delegate result and both notices.
    for (question_bytes, admitted) in [(1, true), (0, false)] {
        let mut h = setup();
        let notices = notice_bytes.checked_mul(2).expect("two notices");
        h.env.limits.inbox_bytes = h
            .env
            .limits
            .result_bytes
            .checked_add(notices)
            .expect("result and two notices")
            .checked_add(question_bytes)
            .expect("question bytes");
        stepped(&ask(&mut h, ROOT), if admitted { None } else { Some(Refusal::Busy) });
    }
}

#[test]
fn a_taken_answer_can_spend_its_notice_in_a_full_inbox_after_restart() {
    let mut h = setup();
    stepped(&ask(&mut h, ASKED), None);
    answer(&mut h);
    h.restart();
    for number in [12, 13] {
        sent(&fill(&mut h, ASKED, number), true);
    }
    sent(&fill(&mut h, ASKED, 14), false);
    let notice = message(ASKED, 15, Party::Task(ASKER), MessageKind::Words, &[b'n'; NOTICE]);
    stepped(&step(&mut h, Box::new([ProcedureMessage { message: notice.clone(), notice: Some(QUESTION) }])), None);
    let inbox = &h.domain.task(ASKED).expect("fixture task or record").inbox;
    assert_eq!(inbox.len(), skein_lib::length::usize_of(SLOTS));
    assert_eq!(inbox.last(), Some(&notice.word));
    h.restart();
    assert_eq!(h.domain.task(ASKED).expect("fixture task or record").inbox.last(), Some(&notice.word));
}

#[test]
fn taking_an_answer_without_a_notice_gives_its_room_back() {
    let mut h = setup();
    stepped(&ask(&mut h, ASKED), None);
    answer(&mut h);
    for number in [12, 13] {
        sent(&fill(&mut h, ASKED, number), true);
    }
    sent(&fill(&mut h, ASKED, 14), false);
    stepped(&step(&mut h, Box::new([])), None);
    sent(&fill(&mut h, ASKED, 14), true);
    sent(&fill(&mut h, ASKED, 15), false);
}

#[test]
fn ending_an_unanswered_question_gives_its_room_back() {
    let mut h = setup();
    // Enough room for the existing delegate result, question, person notice
    // and procedure notice, but not another word. Use a retained requester.
    stepped(&ask(&mut h, ROOT), None);
    sent(&fill(&mut h, ROOT, 12), false);
    for (task, by) in [(ASKED, Party::Task(ASKER)), (ASKER, Party::Task(ROOT))] {
        h.send(Event::Cancel {
            reply_to: to(),
            by,
            operation: Cancel { task, reason: Box::new([]), grace: None, message: None },
        });
        h.send(Event::EffectsSettled { task });
        h.send(Event::Settled { task });
        assert!(h.domain.task(task).is_none());
    }
    // With the delegate result room gone as well, two additional words fit
    // even if the notice credit leaked. The third needs the notice's room.
    for number in [12, 13, 14] {
        sent(&fill(&mut h, ROOT, number), true);
    }
    sent(&fill(&mut h, ROOT, 15), false);
}

#[test]
fn a_notice_for_a_question_saved_without_reserved_room_can_be_refused() {
    let mut h = setup();
    stepped(&ask(&mut h, ASKED), None);
    let crate::Stored::Live(saved) = h.stored.get_mut(&crate::Key::Live(ASKED)).expect("fixture task or record") else {
        panic!("asked task's saved row");
    };
    // The registered layout supplies an empty list for a row written before
    // notice reservations existed. Restore that row with its question.
    saved.notice_credits = Box::new([]);
    h.restart();
    answer(&mut h);
    for number in [12, 13] {
        sent(&fill(&mut h, ASKED, number), true);
    }
    let ordinary = message(ASKED, 15, Party::Task(ASKER), MessageKind::Words, b"unreserved notice");
    let notice = ProcedureMessage { message: ordinary.clone(), notice: Some(QUESTION) };
    // Adding a word before the unreserved notice fills its slot in the
    // same projection. No part of this attempt may commit.
    let last = ProcedureMessage::ordinary(message(ASKED, 14, Party::Task(ASKER), MessageKind::Words, b"full"));
    stepped(&step(&mut h, Box::new([last, notice.clone()])), Some(Refusal::Busy));
    assert_eq!(h.domain.task(ASKED).expect("fixture task or record").inbox.len(), 3);
    stepped(&step(&mut h, Box::new([notice])), None);
    assert_eq!(h.domain.task(ASKED).expect("fixture task or record").inbox.last(), Some(&ordinary.word));
}
