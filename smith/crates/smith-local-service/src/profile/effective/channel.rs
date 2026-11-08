//! The spawned adapter's own derived codec bounds, apart from the agent half.
//! Contract: protocol/limits.md, sections 3.6 and 9.

use alloc::boxed::Box;
use smith_agent_service::profile::{Layer, Limit, Value};

use super::entry;

pub(super) fn transcript(limits: &smith_transcript::v7::Limits) -> Box<[Limit]> {
    let mut entries = skein_lib::List::with_capacity(35);
    for group in [transcript_provider(limits), transcript_tools(limits)] {
        for (name, value) in group {
            entries.push(entry(Layer::Host, name, value)).expect("every transcript bound counted");
        }
    }
    entries.into_boxed()
}

pub(super) fn bodies(limits: &smith_channel::Limits) -> Box<[Limit]> {
    let smith_channel::Limits {
        directory_name,
        directory_path,
        directory_conflicts,
        directory_conflicts_item,
        workspace_directories,
        credential_value_bytes,
        program_value_name,
        account_refused_code,
        account_refused_reason,
        completion_fault_code,
        completion_fault_request_id,
        completion_fault_field,
        refused_fault_code,
        program_fault_name,
        program_fault_detail,
        receipt_text,
        delivered_receipts,
        marker_path,
        delivery_refusal_explanation,
        delivery_failure_diagnostic,
        host_reply_text,
        answered_call_tool,
        message_label,
        message_text,
        start_charter,
        start_transcript,
        start_answered,
        start_grants,
        start_messages,
        host_ask_tool,
        host_ask_input,
        field_name,
        field_text,
        deliver_ask_fields,
        turn_body,
        accepted_result,
        live_call_id,
        live_call_tool,
        live_bytes,
        list_row_name: _,
        list_row_shown_some: _,
        list_row_efforts: _,
        list_row_efforts_item: _,
        list_rows_rows: _,
    } = *limits;
    Box::new([
        body_bytes("process.channel.bodies.directory_name", directory_name),
        body_bytes("process.channel.bodies.directory_path", directory_path),
        body_count("process.channel.bodies.directory_conflicts", directory_conflicts),
        body_bytes("process.channel.bodies.directory_conflicts_item", directory_conflicts_item),
        body_count("process.channel.bodies.workspace_directories", workspace_directories),
        body_bytes("process.channel.bodies.credential_value_bytes", credential_value_bytes),
        body_bytes("process.channel.bodies.program_value_name", program_value_name),
        body_bytes("process.channel.bodies.account_refused_code", account_refused_code),
        body_bytes("process.channel.bodies.account_refused_reason", account_refused_reason),
        body_bytes("process.channel.bodies.completion_fault_code", completion_fault_code),
        body_bytes("process.channel.bodies.completion_fault_request_id", completion_fault_request_id),
        body_bytes("process.channel.bodies.completion_fault_field", completion_fault_field),
        body_bytes("process.channel.bodies.refused_fault_code", refused_fault_code),
        body_bytes("process.channel.bodies.program_fault_name", program_fault_name),
        body_bytes("process.channel.bodies.program_fault_detail", program_fault_detail),
        body_bytes("process.channel.bodies.receipt_text", receipt_text),
        body_count("process.channel.bodies.delivered_receipts", delivered_receipts),
        body_bytes("process.channel.bodies.marker_path", marker_path),
        body_bytes("process.channel.bodies.delivery_refusal_explanation", delivery_refusal_explanation),
        body_bytes("process.channel.bodies.delivery_failure_diagnostic", delivery_failure_diagnostic),
        body_bytes("process.channel.bodies.host_reply_text", host_reply_text),
        body_bytes("process.channel.bodies.answered_call_tool", answered_call_tool),
        body_bytes("process.channel.bodies.message_label", message_label),
        body_bytes("process.channel.bodies.message_text", message_text),
        body_bytes("process.channel.bodies.start_charter", start_charter),
        body_bytes("process.channel.bodies.start_transcript", start_transcript),
        body_count("process.channel.bodies.start_answered", start_answered),
        body_count("process.channel.bodies.start_grants", start_grants),
        body_count("process.channel.bodies.start_messages", start_messages),
        body_bytes("process.channel.bodies.host_ask_tool", host_ask_tool),
        body_bytes("process.channel.bodies.host_ask_input", host_ask_input),
        body_bytes("process.channel.bodies.field_name", field_name),
        body_bytes("process.channel.bodies.field_text", field_text),
        body_count("process.channel.bodies.deliver_ask_fields", deliver_ask_fields),
        body_bytes("process.channel.bodies.turn_body", turn_body),
        body_bytes("process.channel.bodies.accepted_result", accepted_result),
        body_bytes("process.channel.bodies.live_call_id", live_call_id),
        body_bytes("process.channel.bodies.live_call_tool", live_call_tool),
        body_bytes("process.channel.bodies.live_bytes", live_bytes),
    ])
}

fn body_bytes(field: &'static str, value: u32) -> Limit {
    entry(Layer::Host, field, Value::Bytes(u64::from(value)))
}

fn body_count(field: &'static str, value: u32) -> Limit {
    entry(Layer::Host, field, Value::Count(u64::from(value)))
}

fn transcript_provider(limits: &smith_transcript::v7::Limits) -> Box<[(&'static str, Value)]> {
    Box::new([
        ("process.channel.transcript.program_name", Value::Bytes(u64::from(limits.program_name))),
        ("process.channel.transcript.replay_bytes", Value::Bytes(u64::from(limits.replay_bytes))),
        ("process.channel.transcript.text_text", Value::Bytes(u64::from(limits.text_text))),
        ("process.channel.transcript.opaque_bytes", Value::Bytes(u64::from(limits.opaque_bytes))),
        ("process.channel.transcript.call_id", Value::Bytes(u64::from(limits.call_id))),
        ("process.channel.transcript.call_name", Value::Bytes(u64::from(limits.call_name))),
        ("process.channel.transcript.call_name_name", Value::Bytes(u64::from(limits.call_name_name))),
        ("process.channel.transcript.call_input", Value::Bytes(u64::from(limits.call_input))),
        ("process.channel.transcript.said_text", Value::Bytes(u64::from(limits.said_text))),
        ("process.channel.transcript.turn_endpoint", Value::Bytes(u64::from(limits.turn_endpoint))),
        ("process.channel.transcript.turn_model", Value::Bytes(u64::from(limits.turn_model))),
        ("process.channel.transcript.turn_effort_some", Value::Bytes(u64::from(limits.turn_effort_some))),
        ("process.channel.transcript.turn_system_some", Value::Bytes(u64::from(limits.turn_system_some))),
    ])
}

fn transcript_tools(limits: &smith_transcript::v7::Limits) -> Box<[(&'static str, Value)]> {
    Box::new([
        ("process.channel.transcript.allowed_calls_names", Value::Count(u64::from(limits.allowed_calls_names))),
        ("process.channel.transcript.entry_name", Value::Bytes(u64::from(limits.entry_name))),
        ("process.channel.transcript.hit_path", Value::Bytes(u64::from(limits.hit_path))),
        ("process.channel.transcript.hit_text", Value::Bytes(u64::from(limits.hit_text))),
        ("process.channel.transcript.read_content", Value::Bytes(u64::from(limits.read_content))),
        ("process.channel.transcript.listed_entries", Value::Count(u64::from(limits.listed_entries))),
        ("process.channel.transcript.found_hits", Value::Bytes(u64::from(limits.found_hits))),
        ("process.channel.transcript.command_end_head", Value::Bytes(u64::from(limits.command_end_head))),
        ("process.channel.transcript.command_end_last", Value::Bytes(u64::from(limits.command_end_last))),
        ("process.channel.transcript.command_running_head", Value::Bytes(u64::from(limits.command_running_head))),
        ("process.channel.transcript.command_running_last", Value::Bytes(u64::from(limits.command_running_last))),
        ("process.channel.transcript.job_read_head", Value::Bytes(u64::from(limits.job_read_head))),
        ("process.channel.transcript.job_read_last", Value::Bytes(u64::from(limits.job_read_last))),
        ("process.channel.transcript.ambiguous_lines", Value::Count(u64::from(limits.ambiguous_lines))),
        ("process.channel.transcript.working_path_path", Value::Bytes(u64::from(limits.working_path_path))),
        ("process.channel.transcript.other_path_directory", Value::Bytes(u64::from(limits.other_path_directory))),
        ("process.channel.transcript.other_path_path", Value::Bytes(u64::from(limits.other_path_path))),
        ("process.channel.transcript.outside_path_path", Value::Bytes(u64::from(limits.outside_path_path))),
        ("process.channel.transcript.field_problem_field", Value::Bytes(u64::from(limits.field_problem_field))),
        ("process.channel.transcript.tool_result_id", Value::Bytes(u64::from(limits.tool_result_id))),
        ("process.channel.transcript.message_blocks", Value::Count(u64::from(limits.message_blocks))),
        ("process.channel.transcript.turn_messages", Value::Count(u64::from(limits.turn_messages))),
    ])
}
