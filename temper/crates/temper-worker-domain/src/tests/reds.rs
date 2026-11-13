//! Check notifications, the real hub's ask and serialized checkout operations.
use super::{fixtures, harness::Harness};
use crate::{Event, Request, agent, host, wire};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Queue, Token};
use temper_worker_domain_checkout::git::{Commit, Done, Op, Reds};
pub(crate) fn prepared() -> Harness {
    let mut h = Harness::new();
    h.env.limits.host.application_ask_bytes = 4;
    h.env.limits.host.application_answer_bytes = 64;
    let mut items = fixtures::items(&[b"app", b"reference"]);
    items.repositories[0].access = wire::Access::Writable {
        push: Box::from(b"topic".as_slice()),
        landing: Some(Box::from(b"goal".as_slice())),
        expected: None,
    };
    let rows = h.step(Event::Assign { assignment: fixtures::assignment(true, Box::new([])), items: Ok(items) });
    let mut pending = Queue::uncapped();
    for row in rows {
        pending.push(row);
    }
    for _ in 0..64_u32 {
        let row = pending.pop().expect("prepare continues");
        if let Request::Io { owner, op, .. } = row {
            let done = if let Op::Fetch { .. } = op {
                Done::Fetched { commit: Commit::new([1; 32]) }
            } else {
                Done::Succeeded
            };
            for row in h.step(Event::Done { owner, done }) {
                pending.push(row);
            }
        } else if let Request::Decode { agent, run, start } = row {
            h.owner = Some(agent);
            h.run(Event::Decoded { agent, start: Ok(Box::new(fixtures::start(run, &start))), endpoints: Box::new([]) });
            h.say(agent::Up::Admitted);
            return h;
        }
    }
    panic!("prepared agent starts");
}
pub(crate) fn notify(h: &mut Harness, operation: agent::Operation) -> Box<[Request]> {
    let owner = h.owner.unwrap();
    crate::agents::up(
        &mut h.domain,
        &h.env,
        agent::parent::Request::Long { client: owner, operation, span: Duration::from_secs(1) },
        &mut h.out,
    );
    crate::route::hand_off(&mut h.domain, &h.env, &mut h.out);
    let mut rows = List::uncapped();
    while let Some(row) = h.out.pop() {
        rows.push(row).expect("test rows");
    }
    rows.into_boxed()
}
fn start(h: &mut Harness) -> (u64, Token) {
    let rows = notify(h, agent::Operation::Checks);
    let mut ask = None;
    let mut hold = None;
    for row in rows {
        if let Request::RedsAsk { owner, check, ask: wire::Ask::Reds } = &row {
            assert!(ask.is_none());
            ask = Some((*owner, *check));
        }
        if let Request::Io { owner, op: Op::Reds { at, action: Reds::Clear }, .. } = row {
            assert_eq!(at.repository.as_ref(), b"app");
            hold = Some(owner);
        }
    }
    let (owner, check) = ask.expect("one fresh typed ask");
    let rows = h.step(Event::RedsAsk { owner, check, bytes: Box::from(b"reds".as_slice()) });
    let mut number = None;
    for row in rows {
        if let Request::Link(host::LinkUp::Ask { number: given, run, attempt, .. }) = row {
            assert_eq!((run, attempt), (fixtures::RUN, fixtures::ATTEMPT));
            assert!(number.is_none());
            number = Some(given);
        }
    }
    (number.expect("hub ask"), hold.expect("check clears writable repository"))
}
pub(crate) fn ended(h: &mut Harness) {
    crate::agents::up(&mut h.domain, &h.env, agent::parent::Request::LongDone { client: h.owner.unwrap() }, &mut h.out);
    crate::route::hand_off(&mut h.domain, &h.env, &mut h.out);
}
fn stale(h: &mut Harness, number: u64) {
    let rows = h.step(Event::Link(host::LinkDown::Asked {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        number,
        answer: host::AskAnswer::Bytes(Box::from(b"stale".as_slice())),
    }));
    for row in rows {
        if let Request::RedsAnswer { .. } = row {
            panic!("stale answer must not decode");
        }
    }
}
fn answer(h: &mut Harness, number: u64, key: Option<&[u8]>) -> Box<[Request]> {
    let reply = match key {
        Some(_) => host::AskAnswer::Bytes(Box::from(b"answer".as_slice())),
        None => host::AskAnswer::Refused,
    };
    let rows = h.step(Event::Link(host::LinkDown::Asked {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        number,
        answer: reply,
    }));
    let mut decoded = None;
    for row in rows {
        if let Request::RedsAnswer { owner, check, .. } = row {
            decoded = Some((owner, check));
        }
    }
    match decoded {
        Some((owner, check)) => {
            let answer = match key {
                Some(key) => Some(wire::Answer::Reds {
                    reds: Box::new([wire::Red { key: Box::from(key), head: Box::new([1; 20]) }]),
                }),
                None => None,
            };
            h.step(Event::RedsAnswer { owner, check, answer })
        }
        None => Box::new([]),
    }
}
#[test]
fn each_check_asks_afresh_writes_its_own_answer_and_clears_at_long_done() {
    let mut h = prepared();
    for key in [b"test a::t first".as_slice(), b"test b::t second"] {
        let (number, hold) = start(&mut h);
        assert!(answer(&mut h, number, Some(key)).is_empty());
        let rows = h.step(Event::Done { owner: hold, done: Done::Succeeded });
        let [Request::Io { op: Op::Reds { at, action: Reds::Write { reds } }, .. }] = rows.as_ref() else {
            panic!("current reds: {rows:?}")
        };
        assert_eq!(at.repository.as_ref(), b"app");
        assert_eq!(reds.len(), 1);
        assert_eq!(reds[0].key.as_ref(), key);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
        ended(&mut h);
        let Request::Io { owner, op: Op::Reds { action: Reds::Clear, .. }, .. } = h.out.pop().expect("end clears")
        else {
            panic!("end remove")
        };
        assert_eq!(owner, hold);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
    }
}
#[test]
fn refused_malformed_timed_out_and_superseded_answers_write_nothing() {
    for case in 0..5_u32 {
        let mut h = prepared();
        let (number, hold) = start(&mut h);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
        match case {
            0 => {
                assert!(answer(&mut h, number, None).is_empty());
            }
            1 => {
                let rows = h.step(Event::Link(host::LinkDown::Asked {
                    run: fixtures::RUN,
                    attempt: fixtures::ATTEMPT,
                    number,
                    answer: host::AskAnswer::Bytes(Box::new([])),
                }));
                let Request::RedsAnswer { owner, check, .. } = rows.into_iter().next().expect("decode") else {
                    panic!("decode")
                };
                assert!(h.step(Event::RedsAnswer { owner, check, answer: None }).is_empty());
            }
            2 => {
                h.env.now = h.env.now.saturating_add(Duration::from_millis(5001));
                crate::fire(&mut h.domain, &h.env, &mut h.out);
                stale(&mut h, number);
            }
            3 => {
                ended(&mut h);
                while h.out.pop().is_some() {}
                stale(&mut h, number);
            }
            4 => {
                let (next, _) = start(&mut h);
                assert_ne!(number, next);
                stale(&mut h, number);
            }
            _ => unreachable!(),
        }
        assert!(h.out.is_empty());
    }
}
#[test]
fn other_long_operations_do_not_start_a_reds_read() {
    let mut h = prepared();
    for row in notify(&mut h, agent::Operation::Command) {
        if let Request::RedsAsk { .. } | Request::Io { op: Op::Reds { .. }, .. } = row {
            panic!("only checks read reds");
        }
    }
}
