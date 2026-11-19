//! One append-only wire history, including a marker for an unrelated run.
use crate::processes::Wire;
use jig_protocol_worker_wire as w;
use skein_lib::{Duration, List, Time, Writer};

pub(crate) fn frames() -> Wire {
    let limits = &w::CEILINGS;
    let turn = w::Turn::new(
        limits,
        w::TurnParts {
            task: 7,
            attempt: 2,
            number: 3,
            spent: 29,
            read: Some(9),
            refused: List::with_capacity(0),
            opens_window: false,
            body: b"turn".as_slice().into(),
        },
    )
    .expect("turn");
    let work = w::Work::new(limits, w::WorkParts { left: None, saved: None }).expect("work");
    let ended = w::Ended::new(limits, w::EndedParts { outcome: b"answer".as_slice().into(), work }).expect("ending");
    let answer = w::Answer::new(
        limits,
        w::AnswerParts {
            task: 7,
            attempt: 2,
            turns: 3,
            spent: 31,
            read: Some(9),
            refused: List::with_capacity(0),
            ending: w::Ending::Ended(ended),
        },
    )
    .expect("answer");
    let mut parts = answer.clone().into_parts();
    parts.task = crate::worker_orders::probe::OTHER_RUN;
    let other = w::Answer::new(limits, parts).expect("unrelated answer");
    let live = w::LiveAsk::new(limits, w::LiveAskParts { text: false, reasoning: false, input: false }).expect("live");
    let program =
        w::ProgramValue::new(limits, w::ProgramValueParts { name: b"alice".as_slice().into() }).expect("program");
    let grant = w::Grant::new(
        limits,
        w::GrantParts {
            number: 1,
            generation: 4,
            valid: Duration::from_secs(10),
            value: w::GrantValue::Program(program),
        },
    )
    .expect("grant");
    let message = w::Message::new(
        limits,
        w::MessageParts { name: 9, sender: Box::default(), words: b"Hello".as_slice().into(), finish_requested: false },
    )
    .expect("message");
    let assignment = w::Assignment::new(
        limits,
        w::AssignmentParts {
            task: 7,
            attempt: 2,
            identity: [0; 16],
            workspace: None,
            save: false,
            live,
            charter: Box::default(),
            transcript: Box::default(),
            answered: List::with_capacity(0),
            messages: List::from_boxed(Box::new([message])),
            grants: List::from_boxed(Box::new([grant])),
        },
    )
    .expect("assignment");
    let limited = w::Limited::new(
        limits,
        w::LimitedParts { task: 7, attempt: 2, account: 1, reset: Some(Duration::from_secs(5)) },
    )
    .expect("limited");
    let unauthorized =
        w::Unauthorized::new(limits, w::UnauthorizedParts { task: 7, attempt: 2, account: 1, generation: 4 })
            .expect("unauthorized");
    let mut history = Wire::default();
    macro_rules! push {
        ($kind:ident, $value:ident) => {{
            let mut bytes = Writer::new(skein_lib::length::usize_of($value.measure()));
            $value.encode(&mut bytes).expect("encoded frame");
            history.frames.push((Time::ZERO, w::Kind::$kind.number(), bytes.finish()));
        }};
    }
    push!(Assignment, assignment);
    push!(Turn, turn);
    push!(Answer, other);
    push!(Limited, limited);
    push!(Unauthorized, unauthorized);
    push!(Answer, answer);
    history
}
