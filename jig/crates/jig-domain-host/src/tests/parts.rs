//! Parts share the turns' retention, order and room (domain/hosts/hub.md, 4).
use super::*;
use crate::{FromAgent, ToAgent};
use jig_domain_link::{Part, PartKind};

const RUN: Token = Token::new(1);
const ATTEMPT: Token = Token::new(1001);

pub(super) fn part(number: u32, turn: u32, bytes: usize) -> Part {
    Part { number, turn, thread: 2, call: 3, kind: PartKind::Result, body: skein_lib::bytes::zeroed(bytes) }
}

pub(super) fn live(h: &mut Harness, tell_parts: bool) -> (Token, Token, Token) {
    let Event::Down(LinkDown::Assign { assignment: mut value }) = fixtures::assign(assignment(1)) else {
        unreachable!("fixture assignment");
    };
    value.tell_parts = tell_parts;
    let kept = jig_domain_link::KeptPart {
        attempt: 900,
        turn: 1,
        thread: 2,
        call: 3,
        kind: PartKind::Result,
        body: b"kept".as_slice().into(),
    };
    value.parts = Box::new([kept.clone()]);
    let requests = h.step(Event::Down(LinkDown::Assign { assignment: value }));
    let [Request::Prepare { owner, .. }] = &*requests else { panic!("{requests:?}") };
    let owner = *owner;
    let workspace = Token::new(201);
    let requests = h.step(Event::Prepared { owner, workspace });
    let [Request::Agent(ToAgent::Start { start, .. })] = &*requests else { panic!("{requests:?}") };
    assert_eq!(start.tell_parts, tell_parts);
    assert_eq!(&*start.parts, &[kept]);
    let agent = Token::new(301);
    assert!(h.started(owner, agent).is_empty());
    (owner, workspace, agent)
}

fn tell(h: &mut Harness, owner: Token, part: Part) -> Box<[Request]> {
    h.step(Event::Agent(FromAgent::Part { owner, part }))
}
fn turn(h: &mut Harness, owner: Token, number: u32) -> Box<[Request]> {
    h.step(Event::Agent(FromAgent::Turn {
        owner,
        turn: crate::Turn {
            turn: number,
            spent: 0,
            read: None,
            opens_window: false,
            refused: Box::new([]),
            body: Box::new([8; 64]),
        },
    }))
}
fn kept(h: &mut Harness, number: u32) -> Box<[Request]> {
    h.step(Event::Down(LinkDown::PartKept { run: RUN, attempt: ATTEMPT, part: number }))
}

#[test]
fn parts_and_turns_share_room_and_replay_in_telling_order() {
    let mut h = Harness::new();
    let (owner, _, agent) = live(&mut h, true);
    tell(&mut h, owner, part(1, 1, 64));
    turn(&mut h, owner, 1);
    assert!(tell(&mut h, owner, part(2, 2, 64)).is_empty());
    assert!(turn(&mut h, owner, 2).is_empty());
    assert_eq!(h.domain.retained_turns(), 2);
    for _ in 0_u32..2 {
        h.step(jobs::opened());
        assert_eq!(h.domain.turn_at(0), Some(LinkUp::Part { run: RUN, attempt: ATTEMPT, part: part(1, 1, 64) }));
        let Some(LinkUp::Turn { turn, .. }) = h.domain.turn_at(1) else { panic!("second record is the turn") };
        assert_eq!(turn.turn, 1);
        assert_eq!(h.domain.turn_at(2), None);
    }
    assert!(h.step(Event::Down(LinkDown::PartKept { run: RUN, attempt: Token::new(999), part: 1 })).is_empty());
    assert_eq!(
        &*kept(&mut h, 1),
        &[
            Request::Up(LinkUp::Part { run: RUN, attempt: ATTEMPT, part: part(2, 2, 64) }),
            Request::Agent(ToAgent::AcknowledgePart { agent, number: 2 }),
        ]
    );
    let requests = h.step(Event::Down(LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 1 }));
    let [Request::Up(LinkUp::Turn { turn, .. }), Request::Agent(ToAgent::Acknowledge { turn: acknowledged, .. })] =
        &*requests
    else {
        panic!("{requests:?}")
    };
    assert_eq!((turn.turn, *acknowledged), (2, 2));
    assert_eq!(h.domain.turn_at(0), Some(LinkUp::Part { run: RUN, attempt: ATTEMPT, part: part(2, 2, 64) }));
    kept(&mut h, 2);
    assert!(h.domain.part(RUN, ATTEMPT, 2).is_none());
    assert!(h.domain.has_turn(RUN, ATTEMPT, 2));
}

#[test]
fn part_bytes_fill_the_shared_room_before_the_record_count() {
    let mut h = Harness::new();
    h.env.limits.turn_queue_bytes = 96;
    let (owner, _, agent) = live(&mut h, true);
    tell(&mut h, owner, part(1, 1, 33));
    assert!(turn(&mut h, owner, 1).is_empty(), "33 held bytes leave less than one 64-byte record");
    let requests = kept(&mut h, 1);
    let [Request::Up(LinkUp::Turn { .. }), Request::Agent(ToAgent::Acknowledge { agent: got, turn })] = &*requests
    else {
        panic!("{requests:?}")
    };
    assert_eq!((*got, *turn), (agent, 1));
}

#[test]
fn stopping_takes_staged_parts_past_the_room_before_the_answer() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = live(&mut h, true);
    tell(&mut h, owner, part(1, 1, 64));
    tell(&mut h, owner, part(2, 1, 64));
    assert!(tell(&mut h, owner, part(3, 1, 64)).is_empty());
    let requests = h.step(Event::Down(LinkDown::Cancel { run: RUN, attempt: ATTEMPT }));
    assert_eq!(
        &*requests,
        &[
            Request::Agent(ToAgent::Cancel { agent }),
            Request::Up(LinkUp::Part { run: RUN, attempt: ATTEMPT, part: part(3, 1, 64) }),
            Request::Agent(ToAgent::AcknowledgePart { agent, number: 3 }),
        ]
    );
    let requests = tell(&mut h, owner, part(4, 1, 64));
    assert_eq!(
        &*requests,
        &[
            Request::Up(LinkUp::Part { run: RUN, attempt: ATTEMPT, part: part(4, 1, 64) }),
            Request::Agent(ToAgent::AcknowledgePart { agent, number: 4 }),
        ]
    );
    h.step(Event::Agent(FromAgent::Gone { owner, detail: Box::new([]) }));
    let requests = h.step(Event::Saved { owner, at: None });
    let [Request::Release { workspace: got }, Request::Up(LinkUp::Answer { .. })] = &*requests else {
        panic!("{requests:?}")
    };
    assert_eq!(*got, workspace);
    h.step(Event::Down(LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    assert_eq!(h.domain.retained_turns(), 4);
    kept(&mut h, 4);
    assert_eq!(h.domain.retained_turns(), 0, "part kept is cumulative");
}
