//! Incremental observations equal a fresh reading of the append-only evidence.
//! Contract: jig/docs/design/protocol/channel/testing.md, 1.
use jig_channel_world::{fixtures, restarts::State};
use jig_protocol_worker_wire::{self as wire, Record};
use skein_fake_llm_domain::api;
use skein_fake_peers::llm::Observation;
use skein_lib::{List, Time, Token};

fn fresh_peer_bounds(sizes: &wire::sizes::Sizes) -> (skein_channel::Schema, skein_channel::Limits) {
    let codec = wire::sizes::limits(sizes).expect("fixture codec");
    let credential = u32::try_from(wire::Credential::encoded_bound(&codec).expect("credential bound"))
        .expect("credential bound fits");
    let cap = wire::sizes::worker_cap(sizes).expect("fixture cap").cap;
    (
        wire::schema(&codec).expect("fixture schema"),
        skein_channel::Limits {
            chunk: 16_384,
            credential,
            skip: 16_384,
            output_bytes: cap.bytes,
            output_frames: cap.frames,
            kinds: u32::try_from(wire::KINDS.len()).expect("kinds fit"),
        },
    )
}

#[test]
fn prepared_peer_bounds_follow_sizes_and_keep_copies_independent() {
    let sizes = fixtures::store_limits().workers.sizes;
    let expected = fresh_peer_bounds(&sizes);
    assert_eq!(fixtures::peer_bounds_for(&sizes), expected);
    let (mut schema, _) = fixtures::peer_bounds_for(&sizes);
    schema.magic = *b"else";
    assert_ne!(schema, fixtures::peer_bounds_for(&sizes).0);
    assert_eq!(fixtures::peer_bounds_for(&sizes), expected);
    let changed = wire::sizes::Sizes { turn: sizes.turn + 1, ..sizes };
    let changed_expected = fresh_peer_bounds(&changed);
    assert_ne!(changed_expected, expected);
    assert!(fixtures::peer_bounds_for(&changed) == changed_expected, "changed sizes build fresh bounds");
}

#[test]
fn prepared_store_codec_and_grace_equal_the_admitted_bounds() {
    let limits = fixtures::store_limits();
    assert!(
        fixtures::store_codec() == jig_protocol_store::codec::limits(&limits.root.core).expect("codec"),
        "prepared store codec equals the admitted bounds"
    );
    assert_eq!(fixtures::store_grace(), limits.root.core.fleet.grace);
}

#[test]
fn prepared_agent_keeps_independent_copies_and_custom_limits() {
    let limits = smith_agent_process_world::limits();
    let mut copy = fixtures::agent();
    copy.bounds.turns += 1;
    let prepared = fixtures::agent();
    assert_ne!(copy.bounds, prepared.bounds);
    let fresh = fixtures::agent_with_limits(&limits);
    assert!(
        (prepared.endpoints, prepared.limits, prepared.host_limits, prepared.bounds)
            == (fresh.endpoints, fresh.limits, fresh.host_limits, fresh.bounds),
        "prepared agent equals a fresh build"
    );
    let mut changed = limits;
    changed.channel.start_bytes -= 1;
    let custom = fixtures::agent_with_limits(&changed);
    assert_eq!(custom.limits.start_bytes, changed.channel.start_bytes);
    assert_ne!(custom.limits.start_bytes, fixtures::agent().limits.start_bytes);
}

fn assignment(body: &[u8]) -> (Time, u16, Box<[u8]>) {
    let parts = wire::parts::encode(
        &[wire::KeptPart { attempt: 1, turn: 2, thread: 0, call: 3, kind: wire::PartKind::Answer, body: body.into() }],
        &wire::CEILINGS,
    )
    .expect("fixture parts");
    let value = wire::Assignment {
        task: 1,
        attempt: 2,
        identity: [0; 16],
        workspace: None,
        save: false,
        live: wire::LiveAsk { text: false, reasoning: false, input: false },
        charter: Box::default(),
        transcript: Box::default(),
        answered: List::with_capacity(0),
        messages: List::with_capacity(0),
        grants: List::with_capacity(0),
        parts,
    };
    (Time::ZERO, wire::Kind::Assignment.number(), value.encode(&wire::CEILINGS).expect("fixture assignment"))
}

#[test]
fn incremental_assignments_keep_the_second_and_notice_new_frames() {
    let mut frames = vec![assignment(b"first")];
    let mut incremental = State::default();
    incremental.observe_assignments(&frames).expect("first reading");
    incremental.observe_assignments(&frames).expect("unchanged reading");
    assert!(incremental.evidence.resumed.is_empty(), "one assignment is not a resume");

    frames.push(assignment(b"second"));
    incremental.observe_assignments(&frames).expect("new assignment");
    frames.push(assignment(b"third"));
    incremental.observe_assignments(&frames).expect("later assignment");
    let mut fresh = State::default();
    fresh.observe_assignments(&frames).expect("fresh reading");
    assert_eq!(incremental.evidence.resumed, fresh.evidence.resumed);
    assert_eq!(incremental.evidence.resumed[0].body.as_ref(), b"second");

    frames.push((Time::ZERO, 3, b"refusal".as_slice().into()));
    assert_eq!(incremental.observe_assignments(&frames), fresh.observe_assignments(&frames));
    assert!(incremental.observe_assignments(&frames).is_err(), "a later refusal is still checked");
}

fn query(output: &[u8]) -> Observation {
    Observation::Query {
        connection: Token::new(1),
        at: Time::ZERO,
        query: api::Query {
            credential: None,
            caching: api::Caching::Unscoped,
            model: Box::default(),
            system: Box::default(),
            tools: Box::default(),
            choice: api::ToolChoice::Auto,
            messages: Box::new([api::Message {
                role: api::Role::User,
                parts: Box::new([api::Part::ToolOutput {
                    id: b"call".as_slice().into(),
                    output: output.into(),
                    is_error: false,
                }]),
            }]),
            max_tokens: 1,
        },
    }
}

#[test]
fn incremental_queries_keep_outputs_and_notice_new_requests() {
    let mut observations = vec![Observation::Accepted { connection: Token::new(1) }, query(b"first")];
    let mut incremental = State::default();
    incremental.observe_queries(&observations).expect("first reading");
    incremental.observe_queries(&observations).expect("unchanged reading");
    assert_eq!(incremental.evidence.queries, 1);

    observations.push(query(b"interrupted by a worker restart after command output before the cut\n"));
    observations.push(query(b"Delivered"));
    observations.push(query(b"Delivered"));
    incremental.observe_queries(&observations).expect("new requests");
    let mut fresh = State::default();
    fresh.observe_queries(&observations).expect("fresh reading");
    assert_eq!(incremental.evidence.queries, fresh.evidence.queries);
    assert_eq!(incremental.evidence.queries, 4);
    assert_eq!(incremental.evidence.outputs, fresh.evidence.outputs);
    assert_eq!(incremental.evidence.outputs.len(), 3, "repeated tool outputs are kept once");
    assert!(incremental.evidence.interrupted_output);
    assert!(incremental.evidence.delivered);
}
