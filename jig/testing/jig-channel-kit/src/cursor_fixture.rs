//! One append-only wire history, including a marker for an unrelated run.
use crate::processes::Wire;
use jig_protocol_worker_wire::{self as w, Record};
use skein_lib::{Duration, List, Time};

pub(crate) fn frames() -> Wire {
    let limits = &w::CEILINGS;
    let turn = w::Turn {
        task: 7,
        attempt: 2,
        number: 3,
        spent: 29,
        read: Some(9),
        refused: List::with_capacity(0),
        opens_window: false,
        body: b"turn".as_slice().into(),
    };
    let work = w::Work { left: None, saved: None };
    let ended = w::Ended { outcome: b"answer".as_slice().into(), work };
    let answer = w::Answer {
        task: 7,
        attempt: 2,
        turns: 3,
        spent: 31,
        read: Some(9),
        refused: List::with_capacity(0),
        ending: w::Ending::Ended(ended),
    };
    let mut other = answer.clone();
    other.task = crate::worker_orders::probe::OTHER_RUN;
    let live = w::LiveAsk { text: false, reasoning: false, input: false };
    let program = w::ProgramValue { name: b"alice".as_slice().into() };
    let grant =
        w::Grant { number: 1, generation: 4, valid: Duration::from_secs(10), value: w::GrantValue::Program(program) };
    let message =
        w::Message { name: 9, sender: Box::default(), words: b"Hello".as_slice().into(), finish_requested: false };
    let assignment = w::Assignment {
        parts: Box::new([]),
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
    };
    let limited = w::Limited { task: 7, attempt: 2, account: 1, reset: Some(Duration::from_secs(5)) };
    let unauthorized = w::Unauthorized { task: 7, attempt: 2, account: 1, generation: 4 };
    let mut history = Wire::default();
    macro_rules! push {
        ($kind:ident, $value:ident) => {{
            let bytes = $value.encode(limits).expect("encoded frame");
            history.frames.push((Time::ZERO, w::Kind::$kind.number(), bytes));
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
