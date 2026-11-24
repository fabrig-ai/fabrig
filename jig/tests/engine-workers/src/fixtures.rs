//! Prepared constants and per-reader copies (protocol/channel/testing.md, 1).
use jig_domain_link::{Form, Grant, Scheme};
use jig_protocol_engine_workers::{Limits, Worker};
use jig_protocol_worker_wire::Record;
use skein_lib::sha256::digest;
use skein_lib::{Duration, Secret, Writer};
pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const WORKER_C: &[u8] = b"worker-c";
pub const SECRET_C: &[u8] = b"secret-c";
pub const SECRET_A: &[u8] = b"secret-a";
pub const SECRET_B: &[u8] = b"secret-b";
pub const APPLICATION_VERSION: u32 = 3;
pub const OTHER_VERSION: u32 = 4;

#[must_use]
pub fn application() -> skein_lib::List<jig_protocol_worker_wire::Spoken> {
    use jig_protocol_worker_wire::Spoken;
    let text = "family test-worker-wire 3\nrecord Test { value: u32 note: u8 = 0 }\n";
    let layout = skein_codec::schema::parse(text.as_bytes()).expect("test wire").fingerprints().family().digest();
    skein_lib::List::from_boxed(Box::new([Spoken { name: "test-worker-wire", version: 3, layout, text }]))
}

#[must_use]
pub fn statement(version: u32) -> skein_lib::List<jig_protocol_worker_wire::Wire> {
    use jig_protocol_worker_wire as wire;
    let mut wires = skein_lib::List::with_capacity(wire::CEILINGS.credential_wires);
    for spoken in prepared_kept_wires().iter().chain(&application()) {
        let (layout, text) = if spoken.name == "test-worker-wire" && version != APPLICATION_VERSION {
            ([0; 32], b"not a schema".as_slice())
        } else {
            (spoken.layout, spoken.text.as_bytes())
        };
        wires.push(wire::Wire { name: spoken.name.as_bytes().into(), layout, text: text.into() }).expect("wire slot");
    }
    wires
}
pub const SIZES: jig_protocol_worker_wire::sizes::Sizes = jig_protocol_worker_wire::sizes::Sizes {
    application_ask: 64,
    application_answer: 64,
    program_sources: 2,
    piece: 16,
    charter: 16,
    transcript: 16,
    turn: 16,
    message: 16,
    sender: 16,
    call_name: 16,
    tool_name: 16,
    call_input: 16,
    answer: 16,
    result: 16,
    detail: 16,
    grant: 16,
    name: 16,
    secret: 16,

    items: 16,
    left: 16,
    saved: 16,
    evidence: 16,
    resource: 16,
    slots: 1,
    calls: 1,
    turns: 1,
    inbox: 1,
    answered: 1,
    models: 1,
    workstreams: 1,
    accounts: 1,
    credentials: 1,
    facts: 390,
};

pub const LIMITS: Limits = Limits {
    sizes: SIZES,
    names: 3,
    workers: 2,
    opening: 1,
    chunk: 64,
    room: u32::MAX,
    handshake: Duration::from_secs(1),
    hello: Duration::from_secs(1),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
};
#[must_use]
pub fn workers() -> Box<[Worker]> {
    Box::new([
        Worker { number: 1, name: Box::from(WORKER_A), places: 4, digest: Some(digest(SECRET_A)), withdrawn: false },
        Worker { number: 2, name: Box::from(WORKER_B), places: 4, digest: Some(digest(SECRET_B)), withdrawn: false },
        Worker { number: 3, name: Box::from(WORKER_C), places: 4, digest: Some(digest(SECRET_C)), withdrawn: false },
    ])
}

#[must_use]
pub fn credential(name: &[u8], secret: &[u8], version: u32) -> Box<[u8]> {
    credential_with_output(name, secret, version, LIMITS.room)
}
fn credential_with_output(name: &[u8], secret: &[u8], version: u32, output: u32) -> Box<[u8]> {
    let value = jig_protocol_worker_wire::Credential {
        opening: jig_protocol_worker_wire::Opening::Link,
        name: Box::from(name),
        secret: Secret::new(Box::from(secret)),
        wires: statement(version),
        output,
    };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"),
    ));
    value.write(&jig_protocol_worker_wire::CEILINGS, &mut writer).expect("measured credential");
    writer.finish()
}

#[must_use]
pub fn limits() -> Limits {
    LIMITS
}
#[must_use]
pub fn channel_limits() -> skein_channel::Limits {
    *prepared_channel_limits()
}
fn prepared_channel_limits() -> &'static skein_channel::Limits {
    static LIMITS: std::sync::OnceLock<skein_channel::Limits> = std::sync::OnceLock::new();
    LIMITS.get_or_init(|| {
        let codec = jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec");
        let cap = jig_protocol_worker_wire::sizes::worker_cap(&SIZES).expect("small cap").cap;
        skein_channel::Limits {
            chunk: 64,
            credential: u32::try_from(
                jig_protocol_worker_wire::Credential::encoded_bound(&codec).expect("small credential"),
            )
            .expect("fits"),
            skip: 64,
            output_bytes: cap.bytes,
            output_frames: cap.frames,
            kinds: u32::try_from(jig_protocol_worker_wire::KINDS.len()).expect("kinds fit"),
        }
    })
}
#[must_use]
pub fn schema() -> skein_channel::Schema {
    prepared_schema().clone()
}
/// Reuse the fixture's table only when its size inputs match; changed limits build their own.
#[must_use]
pub fn schema_for_sizes(sizes: &jig_protocol_worker_wire::sizes::Sizes) -> skein_channel::Schema {
    if sizes == &SIZES {
        schema()
    } else {
        let codec = jig_protocol_worker_wire::sizes::limits(sizes).expect("world codec");
        jig_protocol_worker_wire::schema(&codec).expect("world schema")
    }
}
fn prepared_schema() -> &'static skein_channel::Schema {
    &prepared_channel().0
}
#[must_use]
pub fn kept_wires() -> skein_lib::List<jig_protocol_worker_wire::Spoken> {
    prepared_kept_wires().clone()
}
fn prepared_kept_wires() -> &'static skein_lib::List<jig_protocol_worker_wire::Spoken> {
    &prepared_channel().1
}
fn prepared_channel() -> &'static (skein_channel::Schema, skein_lib::List<jig_protocol_worker_wire::Spoken>) {
    static CHANNEL: std::sync::OnceLock<(skein_channel::Schema, skein_lib::List<jig_protocol_worker_wire::Spoken>)> =
        std::sync::OnceLock::new();
    CHANNEL.get_or_init(|| jig_protocol_worker_wire::schema_and_spoken(&codec()).expect("small schema and kept wires"))
}
#[must_use]
pub fn peer(
    schema: skein_channel::Schema,
    channel_version: u16,
    name: &[u8],
    secret: &[u8],
    version: u32,
) -> skein_fake_channel::ScriptedPeer {
    peer_with_output(schema, channel_version, name, secret, version, LIMITS.room)
}
#[must_use]
pub fn peer_with_output(
    schema: skein_channel::Schema,
    channel_version: u16,
    name: &[u8],
    secret: &[u8],
    version: u32,
    output: u32,
) -> skein_fake_channel::ScriptedPeer {
    skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Initiator,
        channel_limits(),
        channel_version,
        credential_with_output(name, secret, version, output),
    )
    .expect("scripted worker")
}
#[must_use]
pub fn worker(name: &[u8], secret: &[u8], version: u32) -> skein_fake_channel::ScriptedPeer {
    peer(schema(), jig_protocol_worker_wire::VERSION, name, secret, version)
}

#[must_use]
pub fn hello_record() -> jig_protocol_worker_wire::Hello {
    use jig_protocol_worker_wire as wire;
    use skein_lib::List;
    let mut models = List::with_capacity(1);
    models.push(wire::EndpointBound { endpoint: 1, window: 8192, output: 1024 }).expect("one model");
    let bounds = wire::Bounds {
        program_endpoints: List::with_capacity(0),
        endpoints: models,
        turns: 8,
        spend: 100,
        time: Duration::from_secs(60),
        waiting: Duration::from_secs(10),
        checks: Duration::from_secs(2),
        message_bytes: 16,
        inbox: 1,
        unacknowledged: 1,
        reply_bytes: None,
    };
    let mut hosting = List::with_capacity(1);
    hosting.push(wire::Hosting { task: 7, attempt: 2, phase: wire::Phase::Waiting }).expect("one hosted run");
    let mut workstreams = List::with_capacity(1);
    workstreams.push(9).expect("one workstream");
    wire::Hello {
        fresh: false,
        slots: 1,
        stop_bound: Duration::from_secs(20),
        workstreams,
        hosting,
        bounds,
        jobs: List::with_capacity(0),
    }
}

#[must_use]
pub fn frame(kind: u16, bytes: &[u8]) -> skein_channel::Frame {
    let mut writer = skein_channel::frame_writer(kind, u32::try_from(bytes.len()).expect("small body")).expect("frame");
    writer.put(bytes).expect("measured frame");
    writer.finish().expect("complete frame")
}
#[must_use]
pub fn hello_frame() -> skein_channel::Frame {
    let hello = hello_record();
    let mut writer = Writer::new(skein_lib::length::usize_of(
        hello.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"),
    ));
    hello.write(&jig_protocol_worker_wire::CEILINGS, &mut writer).expect("measured hello");
    frame(0x0100, &writer.finish())
}
#[must_use]
pub fn reporting_worker(name: &[u8], secret: &[u8]) -> skein_fake_channel::ScriptedPeer {
    let mut peer = worker(name, secret, APPLICATION_VERSION);
    peer.play(hello_frame());
    peer
}

#[must_use]
pub fn turn_record() -> jig_protocol_worker_wire::Turn {
    use jig_protocol_worker_wire as wire;
    let refused = refused_messages();
    wire::Turn {
        task: 7,
        attempt: 2,
        number: 3,
        spent: 29,
        read: Some(9),
        refused,
        opens_window: true,
        body: Box::from(b"opaque-turn".as_slice()),
    }
}
#[must_use]
pub fn answer_record() -> jig_protocol_worker_wire::Answer {
    use jig_protocol_worker_wire as wire;
    let work = wire::Work { left: Some(Box::from(b"left".as_slice())), saved: Some(Box::from(b"saved".as_slice())) };
    let ended = wire::Ended { outcome: Box::from(b"opaque-result".as_slice()), work };
    let refused = refused_messages();
    wire::Answer {
        task: 7,
        attempt: 2,
        turns: 3,
        spent: 31,
        read: Some(9),
        refused,
        ending: wire::Ending::Ended(ended),
    }
}
#[must_use]
pub fn call_record() -> jig_protocol_worker_wire::Call {
    use jig_protocol_worker_wire as wire;
    wire::Call {
        task: 7,
        attempt: 2,
        name: Box::from([0, 2, 7]),
        tool: Box::from(b"read".as_slice()),
        writes: false,
        input: Box::from(b"input".as_slice()),
        deadline: Duration::from_secs(4),
        contract: 1,
    }
}
#[must_use]
pub fn fact_record() -> jig_protocol_worker_wire::Fact {
    use jig_protocol_worker_wire as wire;
    let count = wire::Count { kind: wire::CountKind::LlmStarted, elapsed: Duration::from_secs(2), value: 3 };
    wire::Fact { task: 7, attempt: 2, value: wire::FactValue::Count(count) }
}
#[must_use]
pub fn unauthorized_record() -> jig_protocol_worker_wire::Unauthorized {
    use jig_protocol_worker_wire as wire;
    wire::Unauthorized { task: 7, attempt: 2, account: 1, generation: 4 }
}
#[must_use]
pub fn limited_record() -> jig_protocol_worker_wire::Limited {
    use jig_protocol_worker_wire as wire;
    wire::Limited { task: 7, attempt: 2, account: 1, reset: Some(Duration::from_secs(5)) }
}
#[must_use]
pub fn account_refused_record() -> jig_protocol_worker_wire::AccountRefused {
    use jig_protocol_worker_wire as wire;
    wire::AccountRefused {
        task: 7,
        attempt: 2,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: b"suspended".as_slice().into(),
    }
}

fn codec() -> jig_protocol_worker_wire::v22::Limits {
    static CODEC: std::sync::OnceLock<jig_protocol_worker_wire::v22::Limits> = std::sync::OnceLock::new();
    *CODEC.get_or_init(|| jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec"))
}
fn refused_messages() -> skein_lib::List<jig_protocol_worker_wire::Refused> {
    use jig_protocol_worker_wire as wire;
    let mut refused = skein_lib::List::with_capacity(1);
    refused.push(wire::Refused { name: 8, reason: wire::MessageRefusal::Full }).expect("one refusal");
    refused
}

fn encoded<R: Record<Limits = jig_protocol_worker_wire::v22::Limits>>(kind: u16, record: &R) -> skein_channel::Frame {
    frame(kind, &record.encode(&codec()).expect("record within fixture limits"))
}
#[must_use]
pub fn up_frames() -> Vec<skein_channel::Frame> {
    let call = call_record();
    let answer = answer_record();
    let fact = fact_record();
    let unauthorized = unauthorized_record();
    let limited = limited_record();
    let refused = account_refused_record();
    vec![
        turn_frame(),
        encoded(0x0102, &call),
        encoded(0x0103, &answer),
        encoded(0x0104, &fact),
        encoded(0x0105, &unauthorized),
        encoded(0x0106, &limited),
        encoded(0x0107, &refused),
    ]
}

#[must_use]
pub fn turn_frame() -> skein_channel::Frame {
    let turn = turn_record();
    encoded(0x0101, &turn)
}

#[must_use]
pub fn answer_frame() -> skein_channel::Frame {
    let answer = answer_record();
    encoded(0x0103, &answer)
}

pub const TOKEN_ONE: &[u8] = b"TOKEN_ONE";
pub const TOKEN_TWO: &[u8] = b"TOKEN_TWO";
pub const IDENTITY: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
#[must_use]
pub fn grant(form: Form) -> Grant {
    Grant { form, account: 1, generation: 4, valid: Duration::from_secs(30) }
}
#[must_use]
pub fn values() -> Box<[Option<jig_protocol_engine_workers::GrantValue>]> {
    Box::new([
        Some(credential_value(jig_protocol_engine_workers::Scheme::Bearer)),
        Some(jig_protocol_engine_workers::GrantValue::Application { secret: Secret::new(Box::from(TOKEN_TWO)) }),
    ])
}
#[must_use]
pub fn credential_value(scheme: jig_protocol_engine_workers::Scheme) -> jig_protocol_engine_workers::GrantValue {
    jig_protocol_engine_workers::GrantValue::Credential { scheme, token: Secret::new(Box::from(TOKEN_ONE)) }
}
#[must_use]
pub fn parts() -> Box<[jig_protocol_worker_wire::Part]> {
    Box::new([jig_protocol_worker_wire::Part {
        token: skein_lib::Token::new(500),
        bytes: Box::from(b"items".as_slice()),
    }])
}
#[must_use]
pub fn assignment() -> jig_domain_host::Assignment {
    use jig_domain_host as host;
    use skein_lib::Token;
    host::Assignment {
        parts: Box::new([]),
        tell_parts: false,
        assignment: host::RunAssignment {
            run: Token::new(7),
            attempt: Token::new(2),
            identity: IDENTITY,
            workspace: Some(host::Workspace { workstream: 9, items: Token::new(500) }),
            save: true,
            live: jig_domain_host::LiveAsk::NONE,
            charter: Box::from(b"charter".as_slice()),
            grants: Box::new([grant(Form::Credential(Scheme::Bearer)), grant(Form::Application)]),
        },
        turns: Box::new([Box::from([0, 127]), Box::from([255, 1, 2])]),
        answered: Box::new([host::AnsweredCall {
            name: Box::from([0, 2, 7]),
            tool: Box::from(b"read".as_slice()),
            answer: host::SettledAnswer::Host { error: true, body: Box::from(b"answer".as_slice()) },
        }]),
        messages: Box::new([host::StartMessage {
            name: Token::new(8),
            sender: Box::from(b"sender".as_slice()),
            words: Box::from(b"message".as_slice()),
            finish_requested: false,
        }]),
    }
}
#[must_use]
pub fn assignment_down(channel: skein_lib::Token) -> jig_protocol_engine_workers::Event {
    jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::Assign { assignment: assignment() },
        parts: parts(),
        values: values(),
    }
}

/// A worker at a derived layout with one optional responder kind omitted.
#[must_use]
pub fn worker_without(kind: jig_protocol_worker_wire::Kind) -> skein_fake_channel::ScriptedPeer {
    use jig_protocol_worker_wire::{self as wire, Kind};
    let phrase = match kind {
        Kind::Restarting => ", restarting: Restarting",
        Kind::Pause => ", pause: Pause",
        Kind::Figures => ", figures: Figures",
        Kind::PartKept => ", part_kept: PartKept",
        Kind::PartBusy => ", part_busy: PartBusy",
        Kind::ReadUsage => ", read_usage: ReadUsage",
        _ => panic!("an optional responder kind"),
    };
    assert_eq!(wire::SCHEMA.matches(phrase).count(), 1);
    let text = wire::SCHEMA.replace(phrase, "");
    let mut wires = statement(APPLICATION_VERSION);
    let changed = wires.get_mut(0).expect("jig wire");
    changed.layout =
        skein_codec::schema::parse(text.as_bytes()).expect("derived wire").fingerprints().family().digest();
    changed.text = text.into_bytes().into_boxed_slice();
    let mut schema = schema();
    let version = schema.versions.get_mut(0).expect("worker version");
    let omit = kind.number();
    version.kinds = skein_lib::List::from_boxed(
        version
            .kinds
            .iter()
            .filter_map(|rule| {
                if rule.kind == omit {
                    return None;
                }
                Some(skein_channel::Kind { kind: if rule.kind > omit { rule.kind - 1 } else { rule.kind }, ..*rule })
            })
            .collect(),
    );
    let codec = wire::sizes::limits(&SIZES).expect("codec");
    let value = wire::Credential {
        opening: wire::Opening::Link,
        name: WORKER_A.into(),
        secret: Secret::new(SECRET_A.into()),
        wires,
        output: LIMITS.room,
    };
    skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Initiator,
        channel_limits(),
        wire::VERSION,
        value.encode(&codec).expect("credential"),
    )
    .expect("worker at derived layout")
}
