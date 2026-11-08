//! Named projections of admitted layer limits, without retained state.
//! Contract: smith/docs/design/protocol/limits.md, section 9.

use super::list::Inventory;
use super::{Layer, Value};

pub(super) fn transcript(inventory: &mut Inventory, limits: &smith_transcript::v7::Limits) -> Option<()> {
    inventory.layer = Layer::Transcript;
    transcript_bytes(inventory, limits)?;
    transcript_counts(inventory, limits)
}

pub(super) fn bodies(inventory: &mut Inventory, limits: &smith_channel::Limits) -> Option<()> {
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
    inventory.layer = Layer::Channel;
    inventory.value("directory_name", Value::Bytes(u64::from(directory_name)))?;
    inventory.value("directory_path", Value::Bytes(u64::from(directory_path)))?;
    inventory.value("directory_conflicts", Value::Count(u64::from(directory_conflicts)))?;
    inventory.value("directory_conflicts_item", Value::Bytes(u64::from(directory_conflicts_item)))?;
    inventory.value("workspace_directories", Value::Count(u64::from(workspace_directories)))?;
    inventory.value("credential_value_bytes", Value::Bytes(u64::from(credential_value_bytes)))?;
    inventory.value("program_value_name", Value::Bytes(u64::from(program_value_name)))?;
    inventory.value("account_refused_code", Value::Bytes(u64::from(account_refused_code)))?;
    inventory.value("account_refused_reason", Value::Bytes(u64::from(account_refused_reason)))?;
    inventory.value("completion_fault_code", Value::Bytes(u64::from(completion_fault_code)))?;
    inventory.value("completion_fault_request_id", Value::Bytes(u64::from(completion_fault_request_id)))?;
    inventory.value("completion_fault_field", Value::Bytes(u64::from(completion_fault_field)))?;
    inventory.value("refused_fault_code", Value::Bytes(u64::from(refused_fault_code)))?;
    inventory.value("program_fault_name", Value::Bytes(u64::from(program_fault_name)))?;
    inventory.value("program_fault_detail", Value::Bytes(u64::from(program_fault_detail)))?;
    inventory.value("receipt_text", Value::Bytes(u64::from(receipt_text)))?;
    inventory.value("delivered_receipts", Value::Count(u64::from(delivered_receipts)))?;
    inventory.value("marker_path", Value::Bytes(u64::from(marker_path)))?;
    inventory.value("delivery_refusal_explanation", Value::Bytes(u64::from(delivery_refusal_explanation)))?;
    inventory.value("delivery_failure_diagnostic", Value::Bytes(u64::from(delivery_failure_diagnostic)))?;
    inventory.value("host_reply_text", Value::Bytes(u64::from(host_reply_text)))?;
    inventory.value("answered_call_tool", Value::Bytes(u64::from(answered_call_tool)))?;
    inventory.value("message_label", Value::Bytes(u64::from(message_label)))?;
    inventory.value("message_text", Value::Bytes(u64::from(message_text)))?;
    inventory.value("start_charter", Value::Bytes(u64::from(start_charter)))?;
    inventory.value("start_transcript", Value::Bytes(u64::from(start_transcript)))?;
    inventory.value("start_answered", Value::Count(u64::from(start_answered)))?;
    inventory.value("start_grants", Value::Count(u64::from(start_grants)))?;
    inventory.value("start_messages", Value::Count(u64::from(start_messages)))?;
    inventory.value("host_ask_tool", Value::Bytes(u64::from(host_ask_tool)))?;
    inventory.value("host_ask_input", Value::Bytes(u64::from(host_ask_input)))?;
    inventory.value("field_name", Value::Bytes(u64::from(field_name)))?;
    inventory.value("field_text", Value::Bytes(u64::from(field_text)))?;
    inventory.value("deliver_ask_fields", Value::Count(u64::from(deliver_ask_fields)))?;
    inventory.value("turn_body", Value::Bytes(u64::from(turn_body)))?;
    inventory.value("accepted_result", Value::Bytes(u64::from(accepted_result)))?;
    inventory.value("live_call_id", Value::Bytes(u64::from(live_call_id)))?;
    inventory.value("live_call_tool", Value::Bytes(u64::from(live_call_tool)))?;
    inventory.value("live_bytes", Value::Bytes(u64::from(live_bytes)))?;
    Some(())
}

pub(super) fn channel(inventory: &mut Inventory, limits: &smith_protocol_channel::Limits) -> Option<()> {
    let smith_protocol_channel::Limits {
        bodies: _,
        start_bytes,
        charter: _,
        transcript: _,
        resume_turns,
        search_hits,
        channel: _,
        endpoints,
        calls,
        turns,
        fact_reserve_frames,
        fact_reserve_bytes,
        grants,
    } = *limits;
    inventory.layer = Layer::Channel;
    inventory.value("start_bytes", Value::Bytes(u64::from(start_bytes)))?;
    inventory.value("resume_turns", Value::Count(u64::from(resume_turns)))?;
    inventory.value("search_hits", Value::Count(u64::from(search_hits)))?;
    inventory.value("endpoints", Value::Count(u64::from(endpoints)))?;
    inventory.value("calls", Value::Count(u64::from(calls)))?;
    inventory.value("turns", Value::Count(u64::from(turns)))?;
    inventory.value("fact_reserve_frames", Value::Count(u64::from(fact_reserve_frames)))?;
    inventory.value("fact_reserve_bytes", Value::Bytes(u64::from(fact_reserve_bytes)))?;
    inventory.value("grants", Value::Count(u64::from(grants)))?;
    Some(())
}

pub(super) fn framing(inventory: &mut Inventory, limits: &skein_channel::Limits) -> Option<()> {
    let skein_channel::Limits { chunk, credential, skip, output_bytes, output_frames, kinds } = *limits;
    inventory.layer = Layer::Channel;
    inventory.value("framing.chunk", Value::Bytes(u64::from(chunk)))?;
    inventory.value("framing.credential", Value::Bytes(u64::from(credential)))?;
    inventory.value("framing.skip", Value::Bytes(u64::from(skip)))?;
    inventory.value("framing.output_bytes", Value::Bytes(u64::from(output_bytes)))?;
    inventory.value("framing.output_frames", Value::Count(u64::from(output_frames)))?;
    inventory.value("framing.kinds", Value::Count(u64::from(kinds)))?;
    Some(())
}

fn transcript_bytes(inventory: &mut Inventory, limits: &smith_transcript::v7::Limits) -> Option<()> {
    inventory.value("program_name", Value::Bytes(u64::from(limits.program_name)))?;
    inventory.value("replay_bytes", Value::Bytes(u64::from(limits.replay_bytes)))?;
    inventory.value("text_text", Value::Bytes(u64::from(limits.text_text)))?;
    inventory.value("opaque_bytes", Value::Bytes(u64::from(limits.opaque_bytes)))?;
    inventory.value("call_id", Value::Bytes(u64::from(limits.call_id)))?;
    inventory.value("call_name", Value::Bytes(u64::from(limits.call_name)))?;
    inventory.value("call_name_name", Value::Bytes(u64::from(limits.call_name_name)))?;
    inventory.value("call_input", Value::Bytes(u64::from(limits.call_input)))?;
    inventory.value("entry_name", Value::Bytes(u64::from(limits.entry_name)))?;
    inventory.value("hit_path", Value::Bytes(u64::from(limits.hit_path)))?;
    inventory.value("hit_text", Value::Bytes(u64::from(limits.hit_text)))?;
    inventory.value("read_content", Value::Bytes(u64::from(limits.read_content)))?;
    inventory.value("found_hits", Value::Bytes(u64::from(limits.found_hits)))?;
    inventory.value("command_end_head", Value::Bytes(u64::from(limits.command_end_head)))?;
    inventory.value("command_end_last", Value::Bytes(u64::from(limits.command_end_last)))?;
    inventory.value("command_running_head", Value::Bytes(u64::from(limits.command_running_head)))?;
    inventory.value("command_running_last", Value::Bytes(u64::from(limits.command_running_last)))?;
    inventory.value("job_read_head", Value::Bytes(u64::from(limits.job_read_head)))?;
    inventory.value("job_read_last", Value::Bytes(u64::from(limits.job_read_last)))?;
    inventory.value("working_path_path", Value::Bytes(u64::from(limits.working_path_path)))?;
    inventory.value("other_path_directory", Value::Bytes(u64::from(limits.other_path_directory)))?;
    inventory.value("other_path_path", Value::Bytes(u64::from(limits.other_path_path)))?;
    inventory.value("outside_path_path", Value::Bytes(u64::from(limits.outside_path_path)))?;
    inventory.value("field_problem_field", Value::Bytes(u64::from(limits.field_problem_field)))?;
    inventory.value("said_text", Value::Bytes(u64::from(limits.said_text)))?;
    inventory.value("tool_result_id", Value::Bytes(u64::from(limits.tool_result_id)))?;
    inventory.value("turn_endpoint", Value::Bytes(u64::from(limits.turn_endpoint)))?;
    inventory.value("turn_model", Value::Bytes(u64::from(limits.turn_model)))?;
    inventory.value("turn_effort_some", Value::Bytes(u64::from(limits.turn_effort_some)))?;
    inventory.value("turn_system_some", Value::Bytes(u64::from(limits.turn_system_some)))?;
    Some(())
}

fn transcript_counts(inventory: &mut Inventory, limits: &smith_transcript::v7::Limits) -> Option<()> {
    inventory.value("allowed_calls_names", Value::Count(u64::from(limits.allowed_calls_names)))?;
    inventory.value("listed_entries", Value::Count(u64::from(limits.listed_entries)))?;
    inventory.value("ambiguous_lines", Value::Count(u64::from(limits.ambiguous_lines)))?;
    inventory.value("message_blocks", Value::Count(u64::from(limits.message_blocks)))?;
    inventory.value("turn_messages", Value::Count(u64::from(limits.turn_messages)))?;
    Some(())
}
