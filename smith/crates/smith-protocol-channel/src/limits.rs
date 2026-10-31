//! Agent-side opening limits (protocol/channel.md, sections 2 and 7).

/// Codec, framing and queue allowances fixed before the channel opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub bodies: smith_channel::Limits,
    /// Aggregate incoming Start bytes, including its whole saved window.
    pub start_bytes: u32,
    /// This agent's bounded charter decoder allowances.
    pub charter: smith_charter::v4::Limits,
    /// This agent's bounded saved-turn decoder allowances.
    pub transcript: smith_transcript::v6::Limits,
    /// Saved turns admitted on resume, from one window's message capacity.
    pub resume_turns: u32,
    /// Hits admitted within one search outcome on resume.
    pub search_hits: u32,
    pub channel: skein_channel::Limits,
    /// Maximum configured endpoint names kept by this component.
    pub endpoints: u32,
    /// Maximum live host tool and delivery calls awaiting a channel answer.
    pub calls: u32,
    /// Maximum turns retained while durable host acknowledgement is pending.
    pub turns: u32,
    /// Authoritative frame slots kept free by best-effort senders.
    pub fact_reserve_frames: u32,
    /// Authoritative bytes kept free by best-effort senders.
    pub fact_reserve_bytes: u32,
    /// Credential generations retained below the domain; budget two per account.
    pub grants: u32,
}

pub use smith_protocol_translate::Error;

/// Room a caller keeps at the host-channel edge for one entrance.
/// At most one owner event and six lower operations fit; past this room the
/// entrance waits. `send_live` and `offer_fact` count the dropped piece's bytes
/// or fact instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeRoom {
    pub to_domain: u32,
    pub below: u32,
}

/// One entrance's room at the host-channel edge.
pub(crate) const EDGE_ROOM: EdgeRoom = EdgeRoom { to_domain: 1, below: 6 };

/// Room a caller keeps at the host-channel edge for one entrance.
#[must_use]
pub const fn edge_room(_limits: &Limits) -> EdgeRoom {
    EDGE_ROOM
}

/// Whether codec edges and responder capacities fit (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    let Ok(schema) = schema(limits) else {
        return false;
    };
    skein_channel::valid(&schema, skein_channel::Role::Responder, &limits.channel)
        && start_cells(&limits.bodies).is_some()
        && charter_cells(&limits.charter).is_some()
        && turn_cells(&limits.transcript).is_some()
}

/// Build this agent's directional schema, using its aggregate Start entrance.
/// Codec field bounds remain independently checked by their decoders.
pub fn schema(limits: &Limits) -> Result<skein_channel::Schema, smith_channel::Overflow> {
    schema_for_start(&limits.bodies, limits.start_bytes)
}

/// Build either channel role with the deployment's joint Start body bound.
/// Codec field ceilings remain independent of this aggregate frame allowance.
pub fn schema_for_start(
    bodies: &smith_channel::Limits,
    start_bytes: u32,
) -> Result<skein_channel::Schema, smith_channel::Overflow> {
    let mut schema = smith_channel::schema(bodies)?;
    for version_index in 0..schema.versions.len() {
        let version = schema.versions.get_mut(version_index).expect("indexed version");
        for kind_index in 0..version.kinds.len() {
            let kind = version.kinds.get_mut(kind_index).expect("indexed kind");
            if kind.kind == smith_channel::kind::START {
                if start_bytes == 0 || start_bytes > kind.largest {
                    return Err(smith_channel::Overflow::Frame);
                }
                kind.largest = start_bytes;
            }
        }
    }
    Ok(schema)
}

fn start_cells(limits: &smith_channel::Limits) -> Option<u64> {
    let cells = smith_channel::Limits {
        directory_name: 0,
        directory_path: 0,
        directory_conflicts_item: 0,
        credential_value_bytes: 0,
        program_value_name: 0,
        withdrawn_value_id: 0,
        withdrawn_value_bearer: 0,
        account_refused_code: 0,
        account_refused_reason: 0,
        completion_fault_code: 0,
        completion_fault_request_id: 0,
        completion_fault_field: 0,
        refused_fault_code: 0,
        program_fault_name: 0,
        program_fault_detail: 0,
        receipt_text: 0,
        marker_path: 0,
        delivery_refusal_explanation: 0,
        delivery_failure_diagnostic: 0,
        host_reply_text: 0,
        answered_call_tool: 0,
        message_label: 0,
        message_text: 0,
        start_charter: 0,
        start_transcript: 0,
        host_ask_tool: 0,
        host_ask_input: 0,
        field_name: 0,
        field_text: 0,
        turn_body: 0,
        accepted_result: 0,
        ..*limits
    };
    smith_channel::Start::decoded_bound(&cells)
}

fn charter_cells(limits: &smith_charter::v4::Limits) -> Option<u64> {
    let cells = smith_charter::v4::Limits {
        section_title: 0,
        section_body: 0,
        host_tool_name: 0,
        host_tool_description: 0,
        host_tool_input: 0,
        field_rule_name: 0,
        item_kind_kind: 0,
        verdict_rule_label: 0,
        conventions_guide: 0,
        conventions_checks: 0,
        llm_endpoint: 0,
        llm_model: 0,
        charter_instructions: 0,
        field_name: 0,
        field_text: 0,
        item_kind: 0,
        run_result_label_some: 0,
        run_result_text: 0,
        ..*limits
    };
    smith_charter::Charter::decoded_bound(&cells)
}

fn turn_cells(limits: &smith_transcript::v6::Limits) -> Option<u64> {
    let cells = smith_transcript::v6::Limits {
        replay_bytes: 0,
        text_text: 0,
        opaque_bytes: 0,
        call_id: 0,
        call_name: 0,
        call_name_name: 0,
        call_input: 0,
        entry_name: 0,
        hit_path: 0,
        hit_text: 0,
        read_content: 0,
        command_end_head: 0,
        command_end_last: 0,
        command_running_head: 0,
        command_running_last: 0,
        job_read_head: 0,
        job_read_last: 0,
        working_path_path: 0,
        other_path_directory: 0,
        other_path_path: 0,
        outside_path_path: 0,
        field_problem_field: 0,
        said_text: 0,
        tool_result_id: 0,
        turn_endpoint: 0,
        program_name: 0,
        ..*limits
    };
    smith_transcript::Turn::decoded_bound(&cells)
}
