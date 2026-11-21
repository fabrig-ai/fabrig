use crate::v20::Record;
use crate::{Bounds, Credential, EndpointBound, Hello, Hosting, Phase, v20};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Secret, Writer};

pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";

pub const LIMITS: v20::Limits = v20::Limits {
    ask_bytes: 4,
    ask_bytes_bytes: 4,
    agreed_wire_name: 32,
    agreed_wires: 16,
    model_failure_endpoint: v20::CEILINGS.model_failure_endpoint,
    model_failure_model: v20::CEILINGS.model_failure_model,
    model_failure_refusal_some: v20::CEILINGS.model_failure_refusal_some,
    mismatch_wire: 32,
    list_row_name: v20::CEILINGS.list_row_name,
    list_row_shown_some: v20::CEILINGS.list_row_shown_some,
    list_row_efforts: v20::CEILINGS.list_row_efforts,
    list_row_efforts_item: v20::CEILINGS.list_row_efforts_item,
    listed_rows_rows: v20::CEILINGS.listed_rows_rows,
    input_call_id: 256,
    input_call_tool: 64,
    piece_bytes: 32,
    program_source_name: 32,
    sources_sources: 2,
    check_name: 32,
    sign_out_name: 32,
    source_failed_detail: 16,
    login_otherwise_method: 32,
    login_name: 32,
    login_email_some: 255,
    logins_logins: 2,
    signed_out_name: 32,
    wire_name: v20::CEILINGS.wire_name,
    wire_layouts: v20::CEILINGS.wire_layouts,
    credential_wires: v20::CEILINGS.credential_wires,
    credential_name: 16,
    credential_secret: 32,
    bounds_endpoints: 2,
    bounds_program_endpoints: v20::CEILINGS.bounds_program_endpoints,
    hello_jobs: 2,
    job_spec: 32,
    job_grants: 2,
    job_invalid_start_why: 16,
    job_finished_result: 32,
    job_failed_detail: 16,
    hello_workstreams: 2,
    hello_hosting: 2,
    turn_refused: 2,
    turn_body: 32,
    call_name: 16,
    call_tool: 16,
    call_input: 32,
    work_left_some: 16,
    work_saved_some: 16,
    ended_outcome: 32,
    resource_resource_some: 16,
    failed_detail: 16,
    invalid_start_why: 16,
    answer_refused: 2,
    account_refused_code: 16,
    account_refused_reason: 16,
    message_sender: 16,
    message_words: 32,
    workspace_items: 32,
    host_answer_body: 32,
    delivery_answer_evidence: 32,
    answered_call_name: 16,
    answered_call_tool: 16,
    credential_value_bytes: 32,
    application_value_secret: 32,
    program_value_name: 32,
    assignment_charter: 32,
    assignment_transcript: 32,
    assignment_answered: 2,
    assignment_messages: 2,
    assignment_grants: 2,
    inbound_sender: 16,
    inbound_words: 32,
    relay_text_text: 32,
    relayed_name: 16,
    ..v20::CEILINGS
};

pub fn encode(hello: &Hello) -> Box<[u8]> {
    let mut writer =
        Writer::new(skein_lib::length::usize_of(hello.measure(&crate::CEILINGS).expect("record within limits")));
    hello.write(&crate::CEILINGS, &mut writer).expect("measured room");
    writer.finish()
}

pub fn credential(name: &[u8], secret: &[u8]) -> Credential {
    Credential {
        name: name.into(),
        secret: Secret::new(secret.into()),
        wires: List::with_capacity(0),
        output: 1_048_576,
        opening: crate::Opening::Link,
    }
}

pub fn model() -> EndpointBound {
    EndpointBound { endpoint: 1, window: 8192, output: 1024 }
}

pub fn hello_fullest() -> Hello {
    let mut models = List::with_capacity(2);
    models.push(model()).expect("first model slot");
    models.push(model()).expect("second model slot");
    let mut program_endpoints = List::with_capacity(v20::CEILINGS.bounds_program_endpoints);
    for endpoint in 0..v20::CEILINGS.bounds_program_endpoints {
        program_endpoints.push(endpoint).unwrap();
    }
    let bounds = Bounds {
        program_endpoints,
        endpoints: models,
        turns: 40,
        spend: 100,
        time: Duration::from_secs(300),
        waiting: Duration::from_secs(60),
        checks: Duration::from_secs(20),
        message_bytes: 32,
        inbox: 2,
        unacknowledged: 2,
        reply_bytes: Some(4096),
    };
    let mut workstreams = List::with_capacity(2);
    workstreams.push(7).expect("first workstream slot");
    workstreams.push(8).expect("second workstream slot");
    let mut hosting = List::with_capacity(2);
    for phase in [Phase::Active, Phase::Answered] {
        hosting.push(Hosting { task: 3, attempt: 4, phase }).expect("two hosting slots");
    }
    Hello {
        fresh: false,
        slots: 2,
        stop_bound: Duration::from_secs(95),
        workstreams,
        hosting,
        bounds,
        jobs: List::with_capacity(0),
    }
}

pub fn hello_smallest() -> Hello {
    // Five empty counts, the fixed fields, absent reply bound's tag and freshness.
    Hello::decode(&LIMITS, &mut skein_lib::Reader::new(&[0; 82])).expect("smallest hello")
}

pub const MESSAGE_FIRST: u64 = 41;
pub const MESSAGE_SECOND: u64 = 42;

pub fn refusal(name: u64, reason: crate::MessageRefusal) -> crate::Refused {
    crate::Refused { name, reason }
}

fn refusals() -> List<crate::Refused> {
    let mut refused = List::with_capacity(2);
    refused.push(refusal(MESSAGE_FIRST, crate::MessageRefusal::TooLarge)).expect("first refusal slot");
    refused.push(refusal(MESSAGE_SECOND, crate::MessageRefusal::Ending)).expect("second refusal slot");
    refused
}

pub fn turn_with_refusals() -> crate::Turn {
    crate::Turn {
        task: 3,
        attempt: 4,
        number: 7,
        spent: 100,
        read: Some(MESSAGE_FIRST),
        opens_window: true,
        refused: refusals(),
        body: b"opaque-turn".as_slice().into(),
    }
}

fn work() -> crate::Work {
    crate::Work { left: Some(b"left-work".as_slice().into()), saved: Some(b"saved-work".as_slice().into()) }
}

fn answer(ending: crate::Ending) -> crate::Answer {
    crate::Answer { task: 3, attempt: 4, turns: 7, spent: 100, read: Some(MESSAGE_FIRST), refused: refusals(), ending }
}

pub fn answer_ended() -> crate::Answer {
    let ended = crate::Ended { outcome: b"opaque-result".as_slice().into(), work: work() };
    answer(crate::Ending::Ended(ended))
}

pub fn answer_failed() -> crate::Answer {
    let run = crate::RunFailureValue { value: crate::RunFailure::Model };
    let failed = crate::Failed {
        model: None,
        failure: crate::Failure::Run(run),
        detail: b"bounded-detail".as_slice().into(),
        work: work(),
    };
    answer(crate::Ending::Failed(failed))
}

fn fact(value: crate::FactValue) -> crate::Fact {
    crate::Fact { task: 3, attempt: 4, value }
}

pub fn fact_count() -> crate::Fact {
    let count = crate::Count { kind: crate::CountKind::LlmRetried, elapsed: Duration::from_secs(2), value: 3 };
    fact(crate::FactValue::Count(count))
}

pub fn fact_long() -> crate::Fact {
    let long = crate::Long { operation: crate::Operation::Checks, span: Duration::from_secs(6) };
    fact(crate::FactValue::Long(long))
}

pub const IDENTITY_FIRST: [u8; 16] = [7; 16];
pub const TOKEN_ONE: &[u8] = b"token-one";
pub const KEY_ONE: &[u8] = b"key-one";
pub const CREDENTIAL_ONE: &[u8] = b"credential-one";

pub fn credential_value(scheme: crate::Scheme, bytes: &[u8]) -> crate::GrantValue {
    let value = crate::CredentialValue { scheme, bytes: Secret::new(bytes.into()) };
    crate::GrantValue::Credential(value)
}

pub fn application_value() -> crate::GrantValue {
    let value = crate::ApplicationValue { secret: Secret::new(CREDENTIAL_ONE.into()) };
    crate::GrantValue::Application(value)
}

pub fn grant(value: crate::GrantValue, number: u32, generation: u64) -> crate::Grant {
    crate::Grant { number, generation, valid: Duration::from_secs(60), value }
}

pub fn granted_key(key: &[u8]) -> crate::Granted {
    let value = crate::CredentialValue { scheme: crate::Scheme::Key, bytes: Secret::new(key.into()) };
    let grant = crate::Grant {
        number: 5,
        generation: 9,
        valid: Duration::from_secs(60),
        value: crate::GrantValue::Credential(value),
    };
    crate::Granted { task: 3, attempt: 4, grant }
}

fn start_message(name: u64) -> crate::Message {
    crate::Message {
        name,
        sender: b"person".as_slice().into(),
        words: b"message".as_slice().into(),
        finish_requested: name == MESSAGE_SECOND,
    }
}

fn answered_call() -> crate::AnsweredCall {
    let answer = crate::HostAnswer { error: true, body: b"host-answer".as_slice().into() };
    crate::AnsweredCall {
        name: b"call-one".as_slice().into(),
        tool: b"tool".as_slice().into(),
        answer: crate::SettledAnswer::Host(answer),
    }
}

pub fn assignment_resumed() -> crate::Assignment {
    let mut answered = List::with_capacity(2);
    answered.push(answered_call()).expect("answered call slot");
    let mut messages = List::with_capacity(2);
    for name in [MESSAGE_FIRST, MESSAGE_SECOND] {
        messages.push(start_message(name)).expect("two start message slots");
    }
    let mut grants = List::with_capacity(2);
    for value in [credential_value(crate::Scheme::Bearer, TOKEN_ONE), application_value()] {
        grants.push(grant(value, 5, 9)).expect("two grant slots");
    }
    let workspace = crate::Workspace { workstream: 7, items: b"workspace-items".as_slice().into() };
    crate::Assignment {
        task: 3,
        attempt: 4,
        identity: IDENTITY_FIRST,
        workspace: Some(workspace),
        save: true,
        live: crate::LiveAsk { text: false, reasoning: false, input: false },
        charter: b"opaque-charter".as_slice().into(),
        transcript: b"\0\0\0\x08turn-one\0\0\0\x08turn-two".as_slice().into(),
        answered,
        messages,
        grants,
    }
}

pub const SIZES_ONE_SLOT: crate::sizes::Sizes = crate::sizes::Sizes {
    application_ask: 16,
    application_answer: 16,
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

pub const SIZES_LARGE: crate::sizes::Sizes = crate::sizes::Sizes {
    application_ask: 16,
    application_answer: 16,
    program_sources: 2,
    piece: 4096,
    charter: 67_108_864,
    transcript: 134_217_728,
    turn: 33_554_432,
    message: 1_048_576,
    sender: 1024,
    call_name: 64,
    tool_name: 64,
    call_input: 1_048_576,
    answer: 1_048_576,
    result: 67_108_864,
    detail: 4096,
    grant: 8192,
    name: 256,
    secret: 1024,

    items: 1_048_576,
    left: 65_536,
    saved: 65_536,
    evidence: 1_048_576,
    resource: 1024,
    slots: 65_536,
    calls: 1024,
    turns: 1024,
    inbox: 1024,
    answered: 1024,
    models: 64,
    workstreams: 65_536,
    accounts: 32,
    credentials: 32,
    facts: 4470,
};

fn one_slot_limits() -> v20::Limits {
    crate::sizes::limits(&SIZES_ONE_SLOT).expect("small size bounds")
}

fn full_bytes() -> Box<[u8]> {
    b"xxxxxxxxxxxxxxxx".as_slice().into()
}

pub fn fullest_turn() -> crate::Turn {
    let mut refused = List::with_capacity(1);
    refused.push(refusal(MESSAGE_FIRST, crate::MessageRefusal::Ending)).expect("one refusal slot");
    crate::Turn {
        task: 3,
        attempt: 4,
        number: 7,
        spent: 100,
        read: Some(MESSAGE_FIRST),
        opens_window: true,
        refused,
        body: full_bytes(),
    }
}

fn fullest_call() -> crate::AnsweredCall {
    let answer = crate::HostAnswer { error: true, body: full_bytes() };
    crate::AnsweredCall { name: full_bytes(), tool: full_bytes(), answer: crate::SettledAnswer::Host(answer) }
}

fn fullest_grants(limits: &v20::Limits) -> List<crate::Grant> {
    let mut grants = List::with_capacity(2);
    for number in [5, 6] {
        let mut name = skein_lib::bytes::zeroed(skein_lib::length::usize_of(limits.program_value_name));
        name.fill(b'x');
        let value = crate::ProgramValue { name };
        let grant = crate::Grant {
            number,
            generation: 9,
            valid: Duration::from_secs(60),
            value: crate::GrantValue::Program(value),
        };
        grants.push(grant).expect("two grant slots");
    }
    grants
}

pub fn fullest_assignment() -> crate::Assignment {
    let limits = one_slot_limits();
    let mut answered = List::with_capacity(1);
    answered.push(fullest_call()).expect("one answered call slot");
    let mut messages = List::with_capacity(1);
    messages
        .push(crate::Message {
            name: MESSAGE_FIRST,
            sender: full_bytes(),
            words: full_bytes(),
            finish_requested: false,
        })
        .expect("one message slot");
    let workspace = crate::Workspace { workstream: 7, items: full_bytes() };
    crate::Assignment {
        task: 3,
        attempt: 4,
        identity: IDENTITY_FIRST,
        workspace: Some(workspace),
        save: true,
        live: crate::LiveAsk { text: false, reasoning: false, input: false },
        charter: full_bytes(),
        transcript: full_bytes(),
        answered,
        messages,
        grants: fullest_grants(&limits),
    }
}

pub fn repeated(byte: u8, length: u32) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(length).unwrap());
    for _ in 0..length {
        writer.put(&[byte]).unwrap();
    }
    writer.finish()
}
