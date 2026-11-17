//! The checkout's dependency question through the real hub, beside its other asks.
use super::*;

const DEPENDENCY: [u8; 32] = [7; 32];
const WORK: Commit = Commit::new([8; 32]);

fn prepared() -> Harness {
    let mut items = fixtures::items(&[b"app"]);
    items.repositories[0].start =
        wire::Start::Merged { branch: Box::from(b"goal".as_slice()), heads: Box::new([DEPENDENCY]) };
    items.repositories[0].access = wire::Access::Writable {
        push: Box::from(b"topic".as_slice()),
        landing: Some(Box::from(b"goal".as_slice())),
        expected: None,
    };
    let mut h = Harness::new();
    h.env.limits.checkout.conflicts = 1;
    h.env.limits.checkout.path_bytes = 32;
    prepared_with(h, items)
}
fn delivering(h: &mut Harness) -> Token {
    let rows = h.step(Event::Process(Box::new(agent::process::Event::Received {
        owner: h.agent.unwrap(),
        message: agent::Up::Call {
            call: Token::new(1),
            name: agent::CallName { activation: fixtures::ATTEMPT.raw(), completion: 1, position: 0 },
            deadline: h.env.now.saturating_add(Duration::from_secs(30)),
            ask: agent::Ask::Deliver { fields: fixtures::fields(&[(b"title", b"checked"), (b"body", b"")]) },
        },
    })));
    let mut pending = Queue::uncapped();
    for row in rows {
        pending.push(row);
    }
    let mut hold = None;
    for _ in 0..8_u32 {
        let Some(row) = pending.pop() else { return hold.expect("delivery snapshot") };
        if let Request::Encode { owner, value: crate::Encodable::CallName(name) } = row {
            for row in h.step(Event::Encoded {
                owner,
                encoded: crate::Encoded::CallName(Box::new(name.completion.to_be_bytes())),
            }) {
                pending.push(row);
            }
        } else if let Request::Io { owner, op: Op::Commit { snapshot: Some(_), .. }, .. } = row {
            assert!(hold.replace(owner).is_none(), "one delivery snapshot");
        } else if let Request::Process(process) = row {
            let agent::process::Request::Read { .. } = *process else { panic!("delivery read: {process:?}") };
        } else {
            panic!("delivery snapshot: {row:?}");
        }
    }
    panic!("delivery settles")
}
fn unreachable(h: &mut Harness, hold: Token) -> Box<[Request]> {
    h.step(Event::Done {
        owner: hold,
        done: Done::ParentUnreachable { head: WORK, merging: None, parent: Commit::new(DEPENDENCY) },
    })
}
fn asking(h: &mut Harness) -> (Token, Token, u64) {
    let hold = delivering(h);
    let rows = unreachable(h, hold);
    let [Request::RedsAsk { owner, check, ask: wire::Ask::Landed { head } }] = rows.as_ref() else {
        panic!("exactly one landed ask, no answer yet: {rows:?}")
    };
    assert_eq!(*head, DEPENDENCY);
    assert_eq!(Some(*owner), h.owner);
    (hold, *owner, *check)
}
fn sent(h: &mut Harness, owner: Token, cookie: u64) -> u64 {
    let rows = h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"land".as_slice()) });
    let [Request::Link(host::LinkUp::Ask { run, attempt, number, bytes })] = rows.as_ref() else {
        panic!("exactly one hub ask: {rows:?}")
    };
    assert_eq!((*run, *attempt), (fixtures::RUN, fixtures::ATTEMPT));
    assert_eq!(bytes.as_ref(), b"land");
    *number
}
fn reply(h: &mut Harness, number: u64, answer: host::AskAnswer) -> Box<[Request]> {
    h.step(Event::Link(host::LinkDown::Asked { run: fixtures::RUN, attempt: fixtures::ATTEMPT, number, answer }))
}
fn decode(h: &mut Harness, number: u64) -> (Token, u64) {
    let rows = reply(h, number, host::AskAnswer::Bytes(Box::from(b"answer".as_slice())));
    let mut decoded = None;
    for row in rows {
        if let Request::RedsAnswer { owner, check, bytes } = row {
            assert_eq!(bytes.as_ref(), b"answer");
            assert!(decoded.replace((owner, check)).is_none(), "one answer to decode");
        } else if let Request::Link(host::LinkUp::Activity { .. }) = row {
        } else {
            panic!("one answer to decode: {row:?}");
        }
    }
    decoded.expect("one answer to decode")
}
fn caught_up(rows: &[Request], hold: Token) {
    let [Request::Io { owner, op: Op::Commit { parent, merging, snapshot: None, .. }, .. }] = rows else {
        panic!("landed starts catch-up exactly once: {rows:?}")
    };
    assert_eq!((*owner, *parent, *merging), (hold, WORK, None));
}
fn refused(rows: &[Request]) {
    let [Request::Process(process)] = rows else { panic!("one refused delivery, no ask or io: {rows:?}") };
    let agent::process::Request::Send {
        message: agent::Down::Answer { call, reply: agent::Reply::Delivery(agent::Delivery::Failed(failure)) },
        ..
    } = process.as_ref()
    else {
        panic!("delivery refusal: {process:?}")
    };
    assert_eq!(*call, Token::new(1));
    assert_eq!(failure.reason, agent::DeliveryReason::RefusedByTarget);
}
fn expired(h: &mut Harness) -> Box<[Request]> {
    assert!(h.domain.is_due(h.env.now), "the dependency ask arms the root's timer");
    crate::fire(&mut h.domain, &h.env, &mut h.out);
    let mut rows = List::uncapped();
    while let Some(row) = h.out.pop() {
        rows.push(row).unwrap();
    }
    rows.into_boxed()
}
#[test]
fn a_landed_answer_catches_up_the_requesting_hold_once() {
    let mut h = prepared();
    let (hold, owner, cookie) = asking(&mut h);
    let number = sent(&mut h, owner, cookie);
    assert_eq!(decode(&mut h, number), (owner, cookie));
    let rows = h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) });
    caught_up(&rows, hold);
    assert!(reply(&mut h, number, host::AskAnswer::Refused).is_empty());
    assert!(h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }).is_empty());
}
#[test]
fn only_landed_is_true_and_every_other_answer_refuses_once() {
    for answer in
        [Some(wire::Answer::Unlanded), None, Some(wire::Answer::Kept), Some(wire::Answer::Reds { reds: Box::new([]) })]
    {
        let mut h = prepared();
        let (_, owner, cookie) = asking(&mut h);
        let number = sent(&mut h, owner, cookie);
        assert_eq!(decode(&mut h, number), (owner, cookie));
        refused(&h.step(Event::RedsAnswer { owner, check: cookie, answer }));
        assert!(h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }).is_empty());
    }
    let mut h = prepared();
    let (_, owner, cookie) = asking(&mut h);
    let number = sent(&mut h, owner, cookie);
    refused(&reply(&mut h, number, host::AskAnswer::Refused));
    assert!(reply(&mut h, number, host::AskAnswer::Bytes(Box::new([]))).is_empty());
}
#[test]
fn the_dependency_deadline_bounds_encoding_the_answer_and_decoding() {
    for phase in ["encoding", "answer", "decoding"] {
        let mut h = prepared();
        let (_, owner, cookie) = asking(&mut h);
        let number = if phase == "encoding" { None } else { Some(sent(&mut h, owner, cookie)) };
        if phase == "decoding" {
            assert_eq!(decode(&mut h, number.unwrap()), (owner, cookie));
        }
        h.env.now = h.env.now.saturating_add(Duration::from_millis(u64::from(h.env.limits.checkout.reds_read_ms) + 1));
        // Progress after the bound cannot win even before the loop fires the alarm.
        if phase == "encoding" {
            assert!(h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"land".as_slice()) }).is_empty());
        } else if phase == "answer" {
            assert!(reply(&mut h, number.unwrap(), host::AskAnswer::Bytes(Box::new([]))).is_empty());
        } else {
            assert!(h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }).is_empty());
        }
        refused(&expired(&mut h));
        assert!(h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::new([]) }).is_empty());
        if let Some(number) = number {
            assert!(reply(&mut h, number, host::AskAnswer::Bytes(Box::new([]))).is_empty());
        }
        assert!(h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }).is_empty());
        assert!(!crate::reds::due(&h.domain, h.env.now));
    }
}
#[test]
fn a_client_without_a_hosted_run_is_refused_without_an_ask() {
    let mut h = prepared();
    let hold = delivering(&mut h);
    let owner = h.owner.unwrap();
    let (id, workspace) = (&h.domain.workspaces).into_iter().next().unwrap();
    assert_eq!(workspace.run, owner);
    h.domain.workspaces.get_mut(id).unwrap().run = Token::new(999);
    refused(&unreachable(&mut h, hold));
}
#[test]
fn reds_and_reports_keep_their_answers_beside_a_landed_ask() {
    let mut h = prepared();
    let (reds_number, reds_hold) = start(&mut h);
    h.step(Event::Done { owner: reds_hold, done: Done::Succeeded });
    let (hold, owner, cookie) = asking(&mut h);
    let number = sent(&mut h, owner, cookie);
    let rows = answer(&mut h, reds_number, Some(b"test a::t red"));
    assert!(rows.is_empty(), "reds wait behind the active push");
    ended(&mut h);
    assert!(h.out.is_empty(), "report reads also wait behind the active push");
    assert_eq!(decode(&mut h, number), (owner, cookie));
    caught_up(&h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }), hold);

    let mut h = prepared();
    let hold = checking(&mut h);
    reading(&mut h);
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("passed")) } });
    let (owner, report_cookie, _) = results(rows);
    let report_number = super::sent(&mut h, owner, report_cookie);
    let (hold, owner, cookie) = asking(&mut h);
    let number = sent(&mut h, owner, cookie);
    assert!(kept(&mut h, report_number).is_empty());
    assert_eq!(decode(&mut h, number), (owner, cookie));
    caught_up(&h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Landed) }), hold);
}
#[test]
fn a_full_hub_refuses_the_landed_ask_without_disturbing_its_other_asks() {
    let mut h = prepared();
    let (reds_number, hold) = start(&mut h);
    h.step(Event::Done { owner: hold, done: Done::Succeeded });
    reading(&mut h);
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("passed")) } });
    let (owner, cookie, _) = results(rows);
    let report_number = super::sent(&mut h, owner, cookie);
    let (_, owner, cookie) = asking(&mut h);
    refused(&h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"land".as_slice()) }));
    assert!(kept(&mut h, report_number).is_empty());
    assert!(answer(&mut h, reds_number, None).is_empty());
}
