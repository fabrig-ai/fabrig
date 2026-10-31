//! Hub-link transitions (jig/docs/design/domain/hosts.md, 6.1–6.8).
use super::*;
use skein_lib::Duration;
const CHANNEL: Token = Token::new(0);
const RUN: Token = Token::new(2);
const ATTEMPT: Token = Token::new(3);
fn link() -> Link {
    Link::new(Limits { assignments: 2, payloads: 4, turns: 2 }, Rng::new(7))
}
fn turn() -> host::Turn {
    host::Turn {
        turn: 1,
        spent: 9,
        read: Some(Token::new(7)),
        refused: Box::new([]),
        opens_window: true,
        body: Box::from(&b"opaque turn"[..]),
    }
}
fn answer() -> host::LinkUp {
    host::LinkUp::Answer {
        run: RUN,
        attempt: ATTEMPT,
        answer: host::RunAnswer {
            turns: 1,
            spent: 12,
            read: Some(Token::new(7)),
            refused: Box::new([]),
            ending: host::Ending::Ended {
                outcome: Box::from(&b"opaque result"[..]),
                work: host::Work { left: None, saved: None },
            },
        },
    }
}
fn run() -> Box<core::RunCharter> {
    Box::new(core::RunCharter {
        revision: 1,
        ended_by: jig_domain_tasks::EndedBy::Result,
        sources: Box::new([Some(1)]),
        accounts: Box::new([]),
        placement: jig_domain_fleet::PlacementConstraint {
            channel: Token::new(1),
            kind: jig_domain_fleet::HostKind::Worker,
        },
        budget: 37,
        contract: tasks::Contract::Report { words: 128 },
        authority: tasks::Authority {
            tools: tasks::Tools(1023),
            models: tasks::Models::Named(1),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 100, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        policy: core::RunPolicy {
            instructions: b"Answer the person".as_slice().into(),
            waiting: Duration::from_secs(30),
            resume: true,
            turns: 8,
            time: Duration::from_secs(60),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(10),
            model: core::Model {
                dialect: 1,
                endpoint: 1,
                program: None,
                name: b"small".as_slice().into(),
                window: 1024,
                output: 128,
                input_price: 1,
                cached_price: 1,
                cache_write_price: None,
                output_price: 1,
                price_unit: 1_000_000,
                efforts: Box::new([]),
                effort: None,
            },
            alternatives: Box::new([]),
            inspect: true,
            modify: false,
            shell: false,
            agents: false,
            call_timeout: Duration::from_secs(10),
        },
    })
}
fn assignment() -> Assignment {
    Assignment {
        live: core::LiveAsk::NONE,
        brief: Box::new([]),
        conversation: 1,
        task: RUN.raw(),
        attempt: ATTEMPT.raw(),
        charter: 1,
        run: run(),
        messages: Box::new([]),
        transcript: Box::new([b"prior turn".as_slice().into()]),
        answered: Box::new([]),
        grants: Box::new([jig_domain_accounts::Grant {
            form: jig_domain_accounts::Form::Credential(jig_domain_accounts::Scheme::Bearer),
            account: 1,
            generation: 2,
            valid: Duration::from_secs(30),
        }]),
    }
}

#[test]
fn an_assignment_is_relayed_and_answered() {
    let mut link = link();
    let mut out = Queue::uncapped();
    link.keep_assignment(assignment());
    step(&mut link, Time::ZERO, Event::Assign { channel: CHANNEL, run: RUN, attempt: ATTEMPT }, &mut out);
    let Request::Assigned { channel, assignment } = out.pop().expect("assignment") else { panic!("typed assignment") };
    assert_eq!(channel, CHANNEL);
    assert_eq!((assignment.task, assignment.attempt, assignment.conversation), (RUN.raw(), ATTEMPT.raw(), 1));
    assert!(link.assignment(RUN.raw()).is_none());
    step(&mut link, Time::ZERO, Event::Up { channel: CHANNEL, value: answer() }, &mut out);
    let Request::Core(core::Event::Fleet(fleet::Event::Answer { channel, run, attempt, answer, payload })) =
        out.pop().expect("answer")
    else {
        panic!("fleet answer")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(answer, fleet::Answer::Ended);
    assert!(link.payload(payload).is_some());
    assert!(out.is_empty());
}
#[test]
fn a_busy_turn_is_resent_at_its_deadline() {
    let mut link = link();
    let mut out = Queue::uncapped();
    let now = Time::from_nanos(100);
    step(&mut link, now, Event::TurnBusy { run: RUN, attempt: ATTEMPT, turn: 1, retained: true }, &mut out);
    let due = next_deadline(&link).expect("retry deadline");
    assert!(due >= now.saturating_add(Duration::from_nanos(50_000_000)));
    assert!(due <= now.saturating_add(Duration::from_nanos(100_000_000)));
    fire(&mut link, Time::from_nanos(due.as_nanos().checked_sub(1).expect("positive deadline")), &mut out);
    assert!(out.is_empty());
    fire(&mut link, due, &mut out);
    let Request::TurnCopy { run, attempt, turn: number } = out.pop().expect("copy retained turn") else {
        panic!("hub hand-out")
    };
    assert_eq!((run, attempt, number), (RUN, ATTEMPT, 1));
    step(
        &mut link,
        due,
        Event::Up { channel: CHANNEL, value: host::LinkUp::Turn { run, attempt, turn: turn() } },
        &mut out,
    );
    let Request::Core(core::Event::Fleet(fleet::Event::Turn { turn, body, .. })) = out.pop().expect("resent turn")
    else {
        panic!("fleet turn")
    };
    assert_eq!(turn, 1);
    let Some(Payload::Turn { transcript, cumulative, .. }) = link.payload(body) else {
        panic!("retained turn payload")
    };
    assert_eq!(transcript.as_ref(), b"opaque turn");
    assert_eq!(*cumulative, 9);
    assert_eq!(next_deadline(&link), None);
    assert!(out.is_empty());
}
#[test]
fn an_acknowledgement_cancels_the_busy_turn_retry() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::TurnBusy { run: RUN, attempt: ATTEMPT, turn: 1, retained: true }, &mut out);
    let due = next_deadline(&link).expect("retry deadline");
    step(&mut link, Time::ZERO, Event::TurnKept { channel: CHANNEL, run: RUN, attempt: ATTEMPT, turn: 1 }, &mut out);
    let Request::Down { channel, value: host::LinkDown::TurnKept { run, attempt, turn } } =
        out.pop().expect("acknowledgement")
    else {
        panic!("hub acknowledgement")
    };
    assert_eq!((channel, run, attempt, turn), (CHANNEL, RUN, ATTEMPT, 1));
    fire(&mut link, due, &mut out);
    assert!(out.is_empty());
    assert_eq!(next_deadline(&link), None);
}
#[test]
fn a_payload_is_handed_out_and_its_decoded_answer_is_routed() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::Up { channel: CHANNEL, value: answer() }, &mut out);
    let Request::Core(core::Event::Fleet(fleet::Event::Answer { payload, .. })) = out.pop().expect("answer") else {
        panic!("fleet answer")
    };
    step(&mut link, Time::ZERO, Event::AnswerPayload { run: RUN, attempt: ATTEMPT, payload }, &mut out);
    let Request::ReadResult { payload: handed, outcome } = out.pop().expect("protocol hand-out") else {
        panic!("opaque result")
    };
    assert_eq!(handed, payload);
    assert_eq!(outcome.as_ref(), b"opaque result");
    assert_eq!(link.payload_count(), 1);
    step(&mut link, Time::ZERO, Event::ResultRead { payload: handed, end: tasks::End::Parked }, &mut out);
    let Request::Core(core::Event::AnswerPayload { run, attempt, payload: returned, cumulative, end, read, .. }) =
        out.pop().expect("decoded answer")
    else {
        panic!("core payload")
    };
    assert_eq!((run, attempt, returned), (RUN, ATTEMPT, payload));
    assert_eq!((cumulative, end, read), (12, tasks::End::Parked, Some(7)));
    step(&mut link, Time::ZERO, Event::DropPayload { payload }, &mut out);
    assert_eq!(link.payload_count(), 0);
    step(&mut link, Time::ZERO, Event::ResultRead { payload, end: tasks::End::Parked }, &mut out);
    assert!(out.is_empty(), "a stale hand-out cannot answer another run");
}
#[test]
fn a_relay_answer_keeps_its_opaque_name_and_consumes_its_payload() {
    let mut link = link();
    let mut out = Queue::uncapped();
    let name: Box<[u8]> = Box::from(&b"opaque call"[..]);
    step(
        &mut link,
        Time::ZERO,
        Event::Up {
            channel: CHANNEL,
            value: host::LinkUp::Call {
                run: RUN,
                attempt: ATTEMPT,
                call: name.clone(),
                tool: Box::from(&b"read"[..]),
                writes: false,
                input: Box::from(&b"input"[..]),
                deadline: Duration::from_secs(1),
            },
        },
        &mut out,
    );
    let Request::Core(core::Event::Fleet(fleet::Event::Relay { channel, run, attempt, call })) =
        out.pop().expect("relay")
    else {
        panic!("fleet relay")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(call.name, name);
    let expected = core::SettledCall {
        serial: 4,
        tool: Box::from(&b"read"[..]),
        answer: core::SettledAnswer::Host { error: false, body: Box::from(&b"answer"[..]) },
    };
    let payload = link.retain(Payload::SettledCall(expected.clone()));
    step(
        &mut link,
        Time::ZERO,
        Event::Relayed { channel: CHANNEL, run: RUN, attempt: ATTEMPT, call: name.clone(), answer: payload },
        &mut out,
    );
    let Request::Relayed { channel, run, attempt, call, answer } = out.pop().expect("relay answer") else {
        panic!("protocol relay answer")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(call, name);
    assert_eq!(answer, Some(expected));
    assert!(link.payload(payload).is_none());
    assert!(link.host_calls.is_empty());
    assert!(out.is_empty());
}

#[test]
fn payloads_and_root_continuations_share_one_token_sequence() {
    let mut link = link();
    let effect = link.fresh();
    let body = link.retain(Payload::Busy);
    let judge = link.fresh();
    assert_eq!((effect.raw(), body.raw(), judge.raw()), (1, 2, 3));
    assert!(link.take_payload(effect).is_none());
    assert!(link.take_payload(judge).is_none());
    assert!(link.take_payload(body).is_some());
}
