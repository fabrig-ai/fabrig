//! Channel bounds from the owners' numbers (protocol/channel/README.md, 8 and 10).
use crate::{Kind, v13};

/// Bounds supplied by the domains and the application's protocol layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sizes {
    /// Whole program sources per worker, independent of run slots.
    pub program_sources: u32,
    /// Bytes in one live piece (protocol/channel/README.md, 5.4 and 10).
    pub piece: u32,
    pub charter: u32,
    /// The whole resumed window, including each turn's four-byte length.
    pub transcript: u32,
    pub turn: u32,
    pub message: u32,
    pub sender: u32,
    pub call_name: u32,
    pub tool_name: u32,
    pub call_input: u32,
    pub answer: u32,
    pub result: u32,
    pub detail: u32,
    pub grant: u32,
    pub name: u32,
    pub secret: u32,
    pub items: u32,
    pub left: u32,
    pub saved: u32,
    pub evidence: u32,
    pub resource: u32,
    pub slots: u32,
    pub calls: u32,
    pub turns: u32,
    pub inbox: u32,
    pub answered: u32,
    pub models: u32,
    pub workstreams: u32,
    pub accounts: u32,
    pub credentials: u32,
    /// Bytes reserved for facts, including their frame headers.
    pub facts: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cap {
    pub bytes: u32,
    pub frames: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WorkerCap {
    pub cap: Cap,
    pub facts: u32,
}

/// Map field and count bounds without inspecting any payload or sum.
#[must_use]
pub fn limits(sizes: &Sizes) -> Option<v13::Limits> {
    let limits = v13::Limits {
        agreed_wire_name: v13::CEILINGS.agreed_wire_name,
        agreed_wires: v13::CEILINGS.agreed_wires,
        model_failure_endpoint: v13::CEILINGS.model_failure_endpoint,
        model_failure_model: v13::CEILINGS.model_failure_model,
        mismatch_wire: v13::CEILINGS.mismatch_wire,
        list_row_name: v13::CEILINGS.list_row_name,
        list_row_shown_some: v13::CEILINGS.list_row_shown_some,
        list_row_efforts: v13::CEILINGS.list_row_efforts,
        list_row_efforts_item: v13::CEILINGS.list_row_efforts_item,
        listed_rows_rows: v13::CEILINGS.listed_rows_rows,
        input_call_id: v13::CEILINGS.input_call_id,
        input_call_tool: v13::CEILINGS.input_call_tool,
        piece_bytes: sizes.piece,
        program_source_name: 32,
        sources_sources: sizes.program_sources,
        check_name: 32,
        sign_out_name: 32,
        source_failed_detail: sizes.detail,
        login_otherwise_method: 32,
        login_name: 32,
        login_email_some: 255,
        logins_logins: sizes.program_sources,
        signed_out_name: 32,
        wire_name: v13::CEILINGS.wire_name,
        wire_layouts: v13::CEILINGS.wire_layouts,
        credential_wires: v13::CEILINGS.credential_wires,
        credential_name: sizes.name,
        credential_secret: sizes.secret,
        bounds_program_endpoints: v13::CEILINGS.bounds_program_endpoints,
        bounds_endpoints: sizes.models,
        hello_jobs: sizes.models,
        job_spec: sizes.charter,
        job_grants: sizes.accounts.checked_add(sizes.credentials)?,
        job_invalid_start_why: sizes.detail,
        job_finished_result: sizes.result,
        job_failed_detail: sizes.detail,
        hello_workstreams: sizes.workstreams,
        hello_hosting: sizes.slots,
        turn_refused: sizes.inbox,
        turn_body: sizes.turn,
        call_name: sizes.call_name,
        call_tool: sizes.tool_name,
        call_input: sizes.call_input,
        work_left_some: sizes.left,
        work_saved_some: sizes.saved,
        ended_outcome: sizes.result,
        resource_resource_some: sizes.resource,
        failed_detail: sizes.detail,
        invalid_start_why: sizes.detail,
        // A provider's code and reason are smith's text, within the detail's bound.
        account_refused_code: sizes.detail.min(v13::CEILINGS.account_refused_code),
        account_refused_reason: sizes.detail.min(v13::CEILINGS.account_refused_reason),
        answer_refused: sizes.inbox,
        message_sender: sizes.sender,
        message_words: sizes.message,
        workspace_items: sizes.items,
        host_answer_body: sizes.answer,
        delivery_answer_evidence: sizes.evidence,
        answered_call_name: sizes.call_name,
        answered_call_tool: sizes.tool_name,
        credential_value_bytes: sizes.grant,
        application_value_secret: sizes.grant,
        program_value_name: v13::CEILINGS.program_source_name,
        assignment_charter: sizes.charter,
        assignment_transcript: sizes.transcript,
        assignment_answered: sizes.answered,
        assignment_messages: sizes.inbox,
        assignment_grants: sizes.accounts.checked_add(sizes.credentials)?,
        inbound_sender: sizes.sender,
        inbound_words: sizes.message,
        relay_text_text: sizes.answer,
        relayed_name: sizes.call_name,
    };
    v13::check_limits(&limits).ok()?;
    Some(limits)
}

#[must_use]
pub fn largest_body(sizes: &Sizes, kind: Kind) -> Option<u32> {
    body(&limits(sizes)?, kind)
}

/// The smallest resumed start's frame, its header included (jig/docs/design/protocol/channel/README.md, 10).
#[must_use]
pub fn smallest_resumed(calls: u32, name: u32, tool: u32, answer: u32) -> Option<u32> {
    let call = 14_u64
        .checked_add(u64::from(name))?
        .checked_add(u64::from(tool))?
        .checked_add(u64::from(crate::fit::cut_length(answer)))?;
    let bytes = 8_u64.checked_add(57)?.checked_add(u64::from(calls).checked_mul(call)?)?;
    u32::try_from(bytes).ok()
}

pub(crate) fn body(limits: &v13::Limits, kind: Kind) -> Option<u32> {
    let bytes = match kind {
        Kind::Wires => v13::Agreed::encoded_bound(limits),
        Kind::Draining => v13::Draining::encoded_bound(limits),
        Kind::Hello => v13::Hello::encoded_bound(limits),
        Kind::Turn => v13::Turn::encoded_bound(limits),
        Kind::Call => v13::Call::encoded_bound(limits),
        Kind::Answer => v13::Answer::encoded_bound(limits),
        Kind::Fact => v13::Fact::encoded_bound(limits),
        Kind::Unauthorized => v13::Unauthorized::encoded_bound(limits),
        Kind::Limited => v13::Limited::encoded_bound(limits),
        Kind::AccountRefused => v13::AccountRefused::encoded_bound(limits),
        Kind::Assignment => v13::Assignment::encoded_bound(limits),
        Kind::Message => v13::Inbound::encoded_bound(limits),
        Kind::Relayed => v13::Relayed::encoded_bound(limits),
        Kind::TurnKept => v13::TurnKept::encoded_bound(limits),
        Kind::TurnBusy => v13::TurnBusy::encoded_bound(limits),
        Kind::Acknowledge => v13::Acknowledge::encoded_bound(limits),
        Kind::Cancel => v13::Cancel::encoded_bound(limits),
        Kind::Sources => v13::Sources::encoded_bound(limits),
        Kind::ReadList => v13::ReadList::encoded_bound(limits),
        Kind::ListRead => v13::ListRead::encoded_bound(limits),
        Kind::Check => v13::Check::encoded_bound(limits),
        Kind::SignOut => v13::SignOut::encoded_bound(limits),
        Kind::Logins => v13::Logins::encoded_bound(limits),
        Kind::SignedOut => v13::SignedOut::encoded_bound(limits),
        Kind::Job => v13::Job::encoded_bound(limits),
        Kind::JobAnswer => v13::JobAnswer::encoded_bound(limits),
        Kind::Grant => v13::Granted::encoded_bound(limits),
    }?;
    u32::try_from(bytes).ok()
}

fn frame(limits: &v13::Limits, kind: Kind) -> Option<u32> {
    body(limits, kind)?.checked_add(8)
}

#[must_use]
pub fn engine_cap(sizes: &Sizes) -> Option<Cap> {
    let limits = limits(sizes)?;
    let grants = sizes.accounts.checked_add(sizes.credentials)?;
    let mut bytes = frame(&limits, Kind::Assignment)?;
    bytes = bytes.checked_add(sizes.inbox.checked_mul(frame(&limits, Kind::Message)?)?)?;
    bytes = bytes.checked_add(sizes.calls.checked_mul(frame(&limits, Kind::Relayed)?)?)?;
    bytes = bytes.checked_add(sizes.turns.checked_mul(frame(&limits, Kind::TurnKept)?)?)?;
    bytes = bytes.checked_add(frame(&limits, Kind::Acknowledge)?)?;
    bytes = bytes.checked_add(frame(&limits, Kind::Cancel)?)?;
    bytes = bytes.checked_add(grants.checked_mul(frame(&limits, Kind::Grant)?)?)?;
    let frames = 3_u32.checked_add(sizes.inbox)?.checked_add(sizes.calls)?.checked_add(sizes.turns)?;
    let frames = frames.checked_add(grants)?.checked_mul(sizes.slots)?;
    let sources = frame(&limits, Kind::Sources)?;
    // One host-wide list request fits alongside run traffic and source commands.
    let commands =
        sizes.program_sources.checked_mul(frame(&limits, Kind::Check)?.checked_add(frame(&limits, Kind::SignOut)?)?)?;
    Some(Cap {
        bytes: bytes
            .checked_mul(sizes.slots)?
            .checked_add(sources)?
            .checked_add(commands)?
            .checked_add(frame(&limits, Kind::ReadList)?)?
            .checked_add(frame(&limits, Kind::Wires)?)?,
        frames: frames.checked_add(3)?.checked_add(sizes.program_sources.checked_mul(2)?)?,
    })
}

#[must_use]
pub fn worker_cap(sizes: &Sizes) -> Option<WorkerCap> {
    let limits = limits(sizes)?;
    let mut run = sizes.turns.checked_mul(frame(&limits, Kind::Turn)?)?;
    run = run.checked_add(frame(&limits, Kind::Answer)?)?;
    run = run.checked_add(sizes.calls.checked_mul(frame(&limits, Kind::Call)?)?)?;
    let notices = frame(&limits, Kind::Unauthorized)?
        .checked_add(frame(&limits, Kind::Limited)?)?
        .checked_add(frame(&limits, Kind::AccountRefused)?)?;
    let mut bytes = frame(&limits, Kind::Hello)?.checked_add(sizes.slots.checked_mul(run)?)?;
    bytes = bytes.checked_add(sizes.accounts.checked_mul(notices)?)?;
    bytes = bytes.checked_add(frame(&limits, Kind::Draining)?)?;
    // LongDone is the shortest fact: task + attempt + value tag + header.
    let fact_frames = sizes.facts / 25;
    let run_frames = sizes.turns.checked_add(1)?.checked_add(sizes.calls)?;
    let mut frames = 2_u32.checked_add(sizes.slots.checked_mul(run_frames)?)?;
    frames = frames.checked_add(sizes.accounts.checked_mul(3)?)?.checked_add(fact_frames)?;
    bytes = bytes
        .checked_add(frame(&limits, Kind::Logins)?)?
        .checked_add(sizes.program_sources.checked_mul(frame(&limits, Kind::SignedOut)?)?)?;
    // A list answer is host-wide and best effort, independent of run slots.
    frames = frames.checked_add(2)?.checked_add(sizes.program_sources)?;
    Some(WorkerCap {
        cap: Cap { bytes: bytes.checked_add(frame(&limits, Kind::ListRead)?)?.checked_add(sizes.facts)?, frames },
        facts: sizes.facts,
    })
}
