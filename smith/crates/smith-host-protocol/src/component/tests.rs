//! Grant-form checks and crossing call terminals (protocol/channel.md, sections 3–4).

use super::{Component, Phase};
use crate::{Error, Values, grant_form};
use alloc::boxed::Box;
use skein_channel::{Control, Direction, Lower, LowerEvent, StreamMode, Term, control_frame, frame_writer};
use skein_lib::{Duration, List, Queue, Token, Writer, stream};
use smith_host_domain::{Form, Scheme, SourceName};
use smith_protocol_channel::GrantValue;

fn mismatched_values() -> [GrantValue; 2] {
    [GrantValue::Key(Box::from(b"key".as_slice())), GrantValue::Program(Box::from(b"claude-code".as_slice()))]
}

fn channel(phase: Phase) -> (Component, Queue<crate::OpenEvent>, Queue<Lower>) {
    let limits = crate::tests::process_limits();
    let mut component = Component::new(&limits, StreamMode::Two, crate::tests::endpoints()).expect("channel limits");
    component.phase = phase;
    (component, Queue::with_capacity(limits.channel.output_frames), Queue::with_capacity(limits.channel.kinds))
}

// The wire version and frame header size come from protocol/channel.md, section 2,
// and skein/docs/design/channel.md, section 3.
const VERSION: u16 = 5;
const HEADER_BYTES: usize = 8;
const CALL: Token = Token::new(1);
const REUSED_CALL: Token = Token::new(2);
const REPLY: Token = Token::new(11);
const CANCEL: Token = Token::new(12);

fn admitted_channel() -> (Component, Queue<crate::OpenEvent>, Queue<Lower>) {
    let (mut component, mut above, mut below) = channel(Phase::Opening);
    let limits = crate::tests::process_limits();
    let mut entries = List::with_capacity(limits.channel.kinds);
    for kind in &component.schema.version(VERSION).expect("channel version").kinds {
        if kind.direction == Direction::FromInitiator {
            entries.push(Term { kind: kind.kind, largest: kind.largest }).expect("receiving terms");
        }
    }
    let accept =
        control_frame(&Control::Accept { version: VERSION, features: 0 }, &limits.channel).expect("accept frame");
    let terms = control_frame(&Control::Terms { entries }, &limits.channel).expect("terms frame");
    let mut incoming = Writer::new(accept.bytes().len() + terms.bytes().len());
    incoming.put(accept.bytes()).expect("accept bytes");
    incoming.put(terms.bytes()).expect("terms bytes");
    let incoming = incoming.finish();
    let mut offset = 0;
    component.open(&mut above, &mut below);
    while let Some(request) = below.pop() {
        match request {
            Lower::Read(stream::Down::Demand { read: stream::Read::Fill(count), .. }) => {
                let end = offset + usize::try_from(count).expect("bounded demand");
                component.from_below(
                    LowerEvent::Read(stream::Up::Bytes(Box::from(&incoming[offset..end]))),
                    &mut above,
                    &mut below,
                );
                offset = end;
            }
            Lower::Write(stream::OutputDown::Room { right, .. }) => component.from_below(
                LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
                &mut above,
                &mut below,
            ),
            Lower::Write(stream::OutputDown::Send { .. }) => component.fire(&mut above, &mut below),
            other => panic!("unexpected opening operation: {other:?}"),
        }
    }
    component.phase = Phase::Admitted;
    while above.pop().is_some() {}
    (component, above, below)
}

fn name(completion: u32) -> smith_channel::CallName {
    smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: 1, completion, position: 1 },
    )
    .expect("call name")
}

fn call_body(completion: u32) -> Box<[u8]> {
    let limits = smith_channel::CEILINGS;
    let ask = smith_channel::HostAsk::new(
        &limits,
        smith_channel::HostAskParts { tool: Box::from(*b"lookup"), input: Box::from(*b"{}") },
    )
    .expect("host ask");
    let record = smith_channel::Call::new(
        &limits,
        smith_channel::CallParts {
            name: name(completion),
            effect: smith_channel::Effect::Read,
            deadline: Duration::from_secs(30),
            ask: smith_channel::Ask::Host(ask),
        },
    )
    .expect("call");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("call size"));
    record.encode(&mut writer).expect("call body");
    writer.finish()
}

fn withdraw_body(completion: u32) -> Box<[u8]> {
    let record =
        smith_channel::Withdraw::new(&smith_channel::CEILINGS, smith_channel::WithdrawParts { name: name(completion) })
            .expect("withdraw");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("withdraw size"));
    record.encode(&mut writer).expect("withdraw body");
    writer.finish()
}

fn receive(
    component: &mut Component,
    kind: u16,
    body: Box<[u8]>,
    above: &mut Queue<crate::OpenEvent>,
    below: &mut Queue<Lower>,
) {
    if !component.reading {
        component.read(above, below);
    }
    let mut frame = frame_writer(kind, u32::try_from(body.len()).expect("body size")).expect("frame");
    frame.put(&body).expect("frame body");
    let frame = frame.finish().expect("complete frame");
    let mut remaining = frame.bytes();
    while !remaining.is_empty() {
        let Some(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(count), .. })) = below.pop() else {
            panic!("the channel demands the next record's bytes");
        };
        let (bytes, rest) = remaining.split_at(usize::try_from(count).expect("bounded demand"));
        component.from_below(LowerEvent::Read(stream::Up::Bytes(Box::from(bytes))), above, below);
        remaining = rest;
    }
}

fn called_channel() -> (Component, Queue<crate::OpenEvent>, Queue<Lower>) {
    let (mut component, mut above, mut below) = admitted_channel();
    receive(&mut component, 0x0107, call_body(1), &mut above, &mut below);
    while above.pop().is_some() {}
    (component, above, below)
}

fn answer(component: &mut Component, above: &mut Queue<crate::OpenEvent>, below: &mut Queue<Lower>) {
    component.send_reply(CALL, smith_host_domain::Reply::Unavailable, REPLY, above, below).expect("reply");
    // Finish the answer's write before checking for any later refusal output.
    while let Some(request) = below.pop() {
        match request {
            Lower::Write(stream::OutputDown::Room { right, .. }) => component.from_below(
                LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
                above,
                below,
            ),
            Lower::Write(stream::OutputDown::Send { .. }) => component.fire(above, below),
            other => panic!("unexpected reply operation: {other:?}"),
        }
    }
    while above.pop().is_some() {}
}

fn crossed_channel() -> (Component, Queue<crate::OpenEvent>, Queue<Lower>) {
    let (mut component, mut above, mut below) = called_channel();
    answer(&mut component, &mut above, &mut below);
    receive(&mut component, 0x0108, withdraw_body(1), &mut above, &mut below);
    (component, above, below)
}

#[test]
fn a_withdraw_crossing_the_answer_settles_the_call() {
    let (mut component, mut above, mut below) = crossed_channel();
    assert!(above.is_empty(), "a settled withdrawal is not forwarded");
    assert!(component.reading, "a settled withdrawal leaves the read owed");
    assert!(below.iter().all(|request| matches!(request, Lower::Read(_))), "no refusal output");
    receive(&mut component, 0x0107, call_body(2), &mut above, &mut below);
    assert!(matches!(above.pop(), Some(crate::OpenEvent::Call { call: REUSED_CALL, .. })));
    assert!(above.is_empty());
}

#[test]
fn a_name_withdrawn_after_its_answer_can_be_called_again() {
    let (mut component, mut above, mut below) = crossed_channel();
    receive(&mut component, 0x0107, call_body(1), &mut above, &mut below);
    assert!(matches!(above.pop(), Some(crate::OpenEvent::Call { call: REUSED_CALL, .. })));
    receive(&mut component, 0x0108, withdraw_body(1), &mut above, &mut below);
    assert_eq!(above.pop(), Some(crate::OpenEvent::Withdraw { call: REUSED_CALL }));
    assert!(above.is_empty(), "one withdrawal event for the new call");
    assert!(below.iter().all(|request| matches!(request, Lower::Read(_))), "no refusal output");
}

#[test]
fn a_cancelled_run_can_withdraw_after_the_host_answers() {
    use smith_host_domain::{Down, Input, Reply, Up, parent};

    let (mut component, mut above, mut below) = called_channel();
    let mut world = smith_host_world::World::new(0, smith_host_world::limits());
    world.live();
    world.up(Up::Call {
        call: CALL,
        name: super::CallKey::from_wire(&name(1)).domain(),
        deadline: skein_lib::Time::ZERO.saturating_add(Duration::from_secs(30)),
        ask: smith_host_domain::Ask::Host {
            tool: Box::from(*b"lookup"),
            effect: smith_host_domain::Effect::Read,
            body: Box::from(*b"{}"),
        },
    });
    world.event(Input::Parent(parent::Event::Stop { agent: world.agent() }));
    world.event(Input::Parent(parent::Event::Answer { agent: world.agent(), call: CALL, reply: Reply::Unavailable }));
    world.sent();
    component.send_cancel(CANCEL, &mut above, &mut below).expect("cancel");
    answer(&mut component, &mut above, &mut below);
    receive(&mut component, 0x0108, withdraw_body(1), &mut above, &mut below);
    assert!(matches!(
        world.seen.down.as_slice(),
        [Down::Start { .. }, Down::Cancel, Down::Answer { call: CALL, reply: Reply::Unavailable }]
    ));
    assert!(above.is_empty(), "cancel and answer may precede the run's withdrawal");
    assert!(below.iter().all(|request| matches!(request, Lower::Read(_))), "no refusal output");
}

#[test]
fn a_withdraw_that_does_not_decode_is_still_refused() {
    let (mut component, mut above, mut below) = admitted_channel();
    receive(&mut component, 0x0108, Box::default(), &mut above, &mut below);
    assert!(above.is_empty(), "malformed withdrawal never reaches the domain");
    let Some(Lower::Write(stream::OutputDown::Room { right, .. })) = below.pop() else {
        panic!("refusal requests output room");
    };
    component.from_below(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut above,
        &mut below,
    );
    let Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) = below.pop() else {
        panic!("refusal frame");
    };
    assert_eq!(skein_channel::parse_header(&bytes[..HEADER_BYTES]).expect("header").kind, 3);
    assert_eq!(u16::from_be_bytes([bytes[HEADER_BYTES], bytes[HEADER_BYTES + 1]]), super::RULES);
}

#[test]
fn each_value_lends_its_own_form() {
    for (value, form) in [
        (GrantValue::Bearer(Box::from(b"bearer".as_slice())), Form::Credential(Scheme::Bearer)),
        (GrantValue::Key(Box::from(b"key".as_slice())), Form::Credential(Scheme::Key)),
        (
            GrantValue::Program(Box::from(b"claude-code".as_slice())),
            Form::Program(SourceName::new(b"claude-code").expect("channel name")),
        ),
        (
            GrantValue::Withdrawn {
                account_id: Box::from(b"account".as_slice()),
                bearer: Box::from(b"token".as_slice()),
            },
            Form::Withdrawn,
        ),
    ] {
        assert_eq!(grant_form(&value), Ok(form));
    }
}

#[test]
fn a_program_value_without_a_source_name_has_no_form() {
    for name in [b"".as_slice(), b"A".as_slice(), b"abcdefghijklmnopqrstuvwxyz012345-".as_slice()] {
        assert_eq!(grant_form(&GrantValue::Program(Box::from(name))), Err(Error::Grants));
    }
}

#[test]
fn a_start_whose_value_disagrees_with_its_grant_is_refused() {
    let (mut component, mut to_service, mut below) = channel(Phase::Opened);
    for value in mismatched_values() {
        let mut start = smith_host_world::start();
        start.grants[0].form = Form::Credential(Scheme::Bearer);
        let result = component.send_start(
            start,
            smith_host_domain::Window { turns: 1, bytes: 4096 },
            Values { paths: Box::default(), credentials: Box::from([value]) },
            Token::new(1),
            &mut to_service,
            &mut below,
        );
        assert_eq!(result, Err(Error::Grants));
        assert!(to_service.is_empty());
        assert!(below.is_empty());
        assert_eq!(component.phase, Phase::Opened);
    }
}

#[test]
fn a_refresh_whose_value_disagrees_with_its_grant_is_refused() {
    let (mut component, mut to_service, mut below) = channel(Phase::Admitted);
    for value in mismatched_values() {
        let grant = smith_host_domain::Grant {
            account: 1,
            generation: 2,
            valid: Duration::from_secs(60),
            form: Form::Credential(Scheme::Bearer),
        };
        assert_eq!(component.send_grant(grant, value, Token::new(1), &mut to_service, &mut below), Err(Error::Grants));
        assert!(to_service.is_empty());
        assert!(below.is_empty());
    }
}
