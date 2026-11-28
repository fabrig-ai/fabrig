//! Version-fourteen order and resume boundaries, through both channel halves.
use super::*;
use skein_lib::Duration;
use smith_domain_session::record;

fn inject(peer: &mut Peer, body: smith_channel::Body) {
    let frame = smith_channel::frame(body).expect("scripted frame");
    peer.incoming.extend(frame.bytes().iter().copied());
}

fn rules(why: Closed) -> bool {
    matches!(why, Closed::RefusedHere(reason) if reason == smith_channel::reason::RULES)
}

#[test]
fn an_unasked_part_is_a_rule_fault() {
    let mut world = World::admitted();
    let body =
        smith_protocol_translate::encode_turn(&window_turn_fixture(), &smith_transcript::CEILINGS, &test_endpoints())
            .expect("bounded fixture");
    inject(
        &mut world.host,
        smith_channel::Body::Part(smith_channel::Part {
            number: 1,
            turn: 1,
            thread: 0,
            call: 0,
            kind: smith_channel::PartKind::Answer,
            body,
        }),
    );
    world.settle();
    assert!(world.observations().iter().any(|event| matches!(event, Observation::HostEnded(why) if rules(*why))));
}

#[test]
fn keeping_an_untold_part_is_a_rule_fault() {
    let mut world = World::admitted();
    inject(&mut world.agent, smith_channel::Body::PartKept(smith_channel::PartKept { number: 1 }));
    world.settle();
    assert!(world.observations().iter().any(|event| matches!(event, Observation::AgentEnded(why) if rules(*why))));
}

#[test]
fn grace_without_asking_for_parts_is_a_rule_fault() {
    let mut world = World::admitted();
    inject(&mut world.agent, smith_channel::Body::Drain(smith_channel::Drain { grace: Some(Duration::from_secs(3)) }));
    world.settle();
    assert!(world.observations().iter().any(|event| matches!(event, Observation::AgentEnded(why) if rules(*why))));
}

fn start(parts: Box<[u8]>, transcript: Box<[u8]>) -> smith_channel::Start {
    smith_channel::Start {
        identity: None,
        activation: 7,
        charter: smallest_charter(),
        workspace: None,
        transcript,
        tell_parts: true,
        parts,
        answered: skein_lib::List::with_capacity(0),
        grants: skein_lib::List::with_capacity(0),
        window: smith_channel::Window { turns: 2, bytes: 1_000_000 },
        messages: skein_lib::List::with_capacity(0),
        live: None,
    }
}

fn resume(source: smith_channel::Start, turns: u32, bytes: u32) -> Box<smith_protocol_translate::DecodedStart> {
    let mut limits = smith_protocol_channel_fixtures::small();
    limits.resume_turns = turns;
    limits.endpoints = 1;
    limits.turns = 2;
    limits.bodies.start_transcript = bytes;
    limits.start_bytes = u32::try_from(smith_channel::Start::encoded_bound(&limits.bodies).expect("bounded fixture"))
        .expect("bounded fixture");
    let mut world = World::with_agent_limits(limits, StreamMode::Two);
    world.settle();
    inject(&mut world.agent, smith_channel::Body::Start(source));
    world.settle();
    world.take_agent_start().expect("parts cannot refuse the start")
}

#[test]
fn malformed_resumed_parts_are_dropped_whole() {
    let source = start(Box::new([0, 0, 0, 1, 255]), Box::default());
    let decoded = resume(source, 2, 4096);
    assert!(decoded.tell_parts);
    assert!(decoded.parts.is_empty());
}

#[test]
fn resumed_parts_share_the_transcripts_exact_count_and_byte_limit() {
    let body =
        smith_protocol_translate::encode_turn(&window_turn_fixture(), &smith_transcript::CEILINGS, &test_endpoints())
            .expect("bounded fixture");
    let kept = smith_channel::KeptPart {
        activation: 6,
        turn: 1,
        thread: 0,
        call: 0,
        kind: smith_channel::PartKind::Answer,
        body: body.clone(),
    };
    let piece =
        smith_protocol_translate::transcript_piece(&[kept.encode(&smith_channel::CEILINGS).expect("bounded fixture")])
            .expect("bounded fixture");
    let transcript = smith_protocol_translate::transcript_piece(&[body]).expect("bounded fixture");
    let bytes = u32::try_from(transcript.len() + piece.len()).expect("bounded fixture");
    let decoded = resume(start(piece.clone(), transcript.clone()), 2, bytes);
    assert!(decoded.tell_parts);
    assert_eq!(decoded.parts.len(), 1);
    assert_eq!(decoded.parts[0].activation, 6);
    assert_eq!(decoded.parts[0].part.body, window_turn_fixture());
    assert!(resume(start(piece.clone(), transcript.clone()), 1, bytes).parts.is_empty());
    assert!(resume(start(piece, transcript), 2, bytes - 1).parts.is_empty());
}

#[test]
fn a_told_part_its_acknowledgement_and_grace_cross_both_halves() {
    let mut world = World::new(smith_channel::CEILINGS, smith_channel::CEILINGS, StreamMode::Two);
    world.settle();
    world.send_domain_start(
        smith_host_domain::Start {
            tell_parts: true,
            parts: Box::default(),
            live: None,
            identity: None,
            messages: Box::default(),
            logical_run: Token::new(1),
            activation: 1,
            workspace: None,
            charter: typed_charter(&smallest_charter()),
            transcript: None,
            answered: Box::default(),
            directories: Box::default(),
            grants: Box::default(),
        },
        smith_host_domain::Window { turns: 2, bytes: 1_000_000 },
        host::Values { paths: Box::default(), credentials: Box::default() },
    );
    world.settle();
    world.agent_admits();
    world.settle();
    let part = record::Part { thread: 0, call: 0, kind: record::PartKind::Answer, body: window_turn_fixture() };
    let Half::Agent(agent) = &mut world.agent.half else { unreachable!() };
    agent
        .send_part(1, 1, &part, Token::new(40), &mut world.agent.agent_events, &mut world.agent.below)
        .expect("bounded fixture");
    world.settle();
    assert!(world.observations().contains(&Observation::HostPart { number: 1, turn: 1 }));
    let Half::Host(host) = &mut world.host.half else { unreachable!() };
    host.send_part_kept(1, Token::new(41), &mut world.host.host_events, &mut world.host.below)
        .expect("bounded fixture");
    let grace = Duration::from_secs(3);
    host.send_drain(Some(grace), Token::new(42), &mut world.host.host_events, &mut world.host.below)
        .expect("bounded fixture");
    world.settle();
    assert!(world.observations().contains(&Observation::AgentPartKept(1)));
    assert!(world.observations().contains(&Observation::AgentDrainGrace(grace)));
    let part = record::Part { thread: 0, call: 0, kind: record::PartKind::Result, body: first_result() };
    let Half::Agent(agent) = &mut world.agent.half else { unreachable!() };
    agent
        .send_part(2, 1, &part, Token::new(43), &mut world.agent.agent_events, &mut world.agent.below)
        .expect("first turn result part");
    world.settle();
    assert!(world.observations().contains(&Observation::HostPart { number: 2, turn: 1 }));
}

fn first_result() -> record::Turn {
    use smith_domain_session::llm;
    let mut result = window_turn_fixture();
    result.kind = record::Kind::Ordinary;
    result.sequence = 1;
    result.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"finish".as_slice().into(),
            result: llm::Returned::Text { text: b"accepted".as_slice().into(), error: false, replay: None },
        }]),
    }]);
    result
}

#[test]
fn a_host_start_preserves_kept_parts_and_the_results_path() {
    let mut world = World::new(smith_channel::CEILINGS, smith_channel::CEILINGS, StreamMode::Two);
    world.settle();
    let body = window_turn_fixture();
    let kept = smith_host_domain::channel::KeptPart {
        activation: 6,
        turn: 3,
        thread: 2,
        call: 4,
        kind: record::PartKind::Window,
        body: smith_host_domain::TurnValue::new(body.clone(), u64::MAX).expect("bounded turn"),
    };
    let result = first_result();
    let kept_result = smith_host_domain::channel::KeptPart {
        activation: 6,
        turn: 3,
        thread: 0,
        call: 0,
        kind: record::PartKind::Result,
        body: smith_host_domain::TurnValue::new(result.clone(), u64::MAX).expect("bounded result"),
    };
    world.send_domain_start(
        smith_host_domain::Start {
            tell_parts: true,
            parts: Box::new([kept, kept_result]),
            live: None,
            identity: None,
            messages: Box::default(),
            logical_run: Token::new(1),
            activation: 7,
            workspace: Some(Token::new(2)),
            charter: typed_charter(&smallest_charter()),
            transcript: None,
            answered: Box::default(),
            grants: Box::default(),
            directories: Box::new([smith_host_domain::Directory {
                root: Token::new(2),
                name: b"repo".as_slice().into(),
                host: b"/repo".as_slice().into(),
                writable: true,
                git: false,
                conflicts: Box::default(),
                results: Some(b"/results".as_slice().into()),
                hooks: None,
            }]),
        },
        smith_host_domain::Window { turns: 2, bytes: 1_000_000 },
        host::Values { paths: Box::new([b"/repo".as_slice().into()]), credentials: Box::default() },
    );
    world.settle();
    let decoded = world.take_agent_start().expect("start decoded");
    assert_eq!(
        decoded.parts,
        Box::new([
            record::Resumed {
                activation: 6,
                turn: 3,
                part: record::Part { thread: 2, call: 4, kind: record::PartKind::Window, body },
            },
            record::Resumed {
                activation: 6,
                turn: 3,
                part: record::Part { thread: 0, call: 0, kind: record::PartKind::Result, body: result },
            },
        ]) as Box<[record::Resumed]>
    );
    assert_eq!(decoded.mounts.as_ref().expect("workspace")[0].results.as_deref(), Some(b"/results".as_slice()));
}
