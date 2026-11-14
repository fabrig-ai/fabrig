//! Smith's event conformance observations over the independently read file.
//! This referee owns only decoded record identities and sums; Skein owns the
//! scheduling, machine effects, resource accounting and settlement.

use smith_events::{Event, Record};

/// Check the direct provider's stream identities, cardinalities and response sums.
///
/// # Errors
/// Returns the first stream difference, naming its record position.
pub fn check(records: &[Record]) -> Result<(), String> {
    check_transport(records, false)
}

/// Check program stream conformance with the unknown request measures of smith/docs/design/protocol/events.md 3.5.
///
/// # Errors
/// Returns the first stream or sent-message difference, naming its position.
pub fn check_program(records: &[Record], received: &[serde_json::Value]) -> Result<(), String> {
    check_transport(records, true)?;
    let mut captured = Vec::<Vec<Vec<u8>>>::new();
    for (position, record) in records.iter().enumerate() {
        if let Event::ResponseStarted(started) = &record.event
            && started.model.backend == smith_events::Backend::ClaudeCode
            && let Some(prompt) = &started.prompt
        {
            for message in &prompt.messages {
                let message_position = captured.len();
                if message.role != smith_events::Role::User {
                    return Err(format!("record {position}, message {message_position}: sent user role"));
                }
                let mut blocks = Vec::new();
                for (block_position, block) in message.blocks.iter().enumerate() {
                    let smith_events::Block::Text { text } = block else {
                        return Err(format!(
                            "record {position}, message {message_position}, block {block_position}: sent text block"
                        ));
                    };
                    blocks.push(text.to_vec());
                }
                captured.push(blocks);
            }
        }
    }
    for (position, message) in received.iter().enumerate() {
        if message["type"] != "user" || message["message"]["role"] != "user" {
            return Err(format!("message {position}: sent user role"));
        }
        let blocks = match &message["message"]["content"] {
            serde_json::Value::String(text) => vec![text.as_bytes().to_vec()],
            serde_json::Value::Array(blocks) => {
                let mut texts = Vec::new();
                for (block_position, block) in blocks.iter().enumerate() {
                    if block["type"] != "text" {
                        return Err(format!("message {position}, block {block_position}: sent text block"));
                    }
                    let text = block["text"]
                        .as_str()
                        .ok_or_else(|| format!("message {position}, block {block_position}: recorded text block"))?;
                    texts.push(text.as_bytes().to_vec());
                }
                texts
            }
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Object(_) => return Err(format!("message {position}: sent user content")),
        };
        if captured.get(position) != Some(&blocks) {
            return Err(format!(
                "message {position}: captured user messages equal independently received contents in order: contents differ"
            ));
        }
    }
    if captured.len() != received.len() {
        return Err(format!(
            "message {}: captured user messages equal independently received contents in order: message missing",
            received.len()
        ));
    }
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive Smith record referee keeps all18 pairings and count/charge checks together"
)]
fn check_transport(records: &[Record], program: bool) -> Result<(), String> {
    let record_limits = std::cell::OnceCell::new();
    let position = records.len().saturating_sub(1);
    let Event::SessionStarted(header) = &records.first().ok_or("record 0: stream header")?.event else {
        return Err("record 0: session.started is first".into());
    };
    let Event::SessionEnded(end) = &records.last().ok_or("record 0: stream end")?.event else {
        return Err(format!("record {position}: session.ended is last"));
    };
    if end.loss.events != 0 {
        return Err(format!("record {position}: complete destination loses no record"));
    }
    let mut observed = std::collections::BTreeMap::<&[u8], u64>::new();
    let mut reported = std::collections::BTreeMap::<&[u8], u64>::new();
    let mut responses = Vec::new();
    let mut conversations = Vec::new();
    let mut calls = Vec::new();
    let mut checks = Vec::new();
    let mut runs = Vec::new();
    let mut answered = Vec::new();
    let mut spent = 0_u64;
    let mut usage = [None; 5];
    let mut reported_usage = false;
    for (position, record) in records.iter().enumerate() {
        *observed.entry(name(&record.event)).or_default() += 1;
        match &record.event {
            Event::SessionStarted(_) => {
                if observed.get(b"session.started".as_slice()) != Some(&1) {
                    return Err(format!(
                        "record {position}: observed.get(b\"session.started\".as_slice()), Some(&1)\nleft: {:?}\nright: {:?}",
                        observed.get(b"session.started".as_slice()),
                        Some(&1)
                    ));
                }
            }
            Event::SessionEnded(_) => {
                if observed.get(b"session.ended".as_slice()) != Some(&1) {
                    return Err(format!(
                        "record {position}: observed.get(b\"session.ended\".as_slice()), Some(&1)\nleft: {:?}\nright: {:?}",
                        observed.get(b"session.ended".as_slice()),
                        Some(&1)
                    ));
                }
            }
            Event::RunStarted(run) => {
                if runs.contains(&run.run) {
                    return Err(format!("record {position}: !runs.contains(&run.run)"));
                }
                runs.push(run.run);
            }
            Event::RunCompleted(run) => {
                if answered.contains(&run.run) {
                    return Err(format!("record {position}: !answered.contains(&run.run)"));
                }
                answered.push(run.run);
                if run.spent != spent {
                    return Err(format!(
                        "record {position}: run spend equals its response charges\nleft: {:?}\nright: {:?}",
                        run.spent, spent
                    ));
                }
                if [
                    run.usage.input_tokens,
                    run.usage.cache_read_tokens,
                    run.usage.cache_write_tokens,
                    run.usage.output_tokens,
                    run.usage.reasoning_tokens,
                ] != usage
                {
                    return Err(format!(
                        "record {position}: run totals equal reported response usage; no observation stays unknown\nleft: {:?}\nright: {:?}",
                        [
                            run.usage.input_tokens,
                            run.usage.cache_read_tokens,
                            run.usage.cache_write_tokens,
                            run.usage.output_tokens,
                            run.usage.reasoning_tokens
                        ],
                        usage
                    ));
                }
            }
            Event::ConversationOpened(conversation) => {
                let identity = (conversation.run, conversation.conversation);
                if conversations.contains(&identity) {
                    return Err(format!("record {position}: !conversations.contains(&identity)"));
                }
                conversations.push(identity);
            }
            Event::ConversationClosed(conversation) => {
                let identity = (conversation.run, conversation.conversation);
                let index = conversations
                    .iter()
                    .position(|opened| *opened == identity)
                    .ok_or_else(|| format!("record {position}: one opening before closure"))?;
                conversations.remove(index);
            }
            Event::ResponseStarted(response) => {
                let identity = (response.run, response.conversation, response.response);
                if responses.contains(&identity) {
                    return Err(format!("record {position}: !responses.contains(&identity)"));
                }
                responses.push(identity);
                if response.model.backend == smith_events::Backend::ClaudeCode
                    && let Some(prompt) = &response.prompt
                {
                    for message in &prompt.messages {
                        if message.role != smith_events::Role::User {
                            return Err(format!(
                                "record {position}: message.role, smith_events::Role::User\nleft: {:?}\nright: {:?}",
                                message.role,
                                smith_events::Role::User
                            ));
                        }
                        if !message.blocks.iter().all(|block| matches!(block, smith_events::Block::Text { .. })) {
                            return Err(format!(
                                "record {position}: message.blocks.iter().all(|block| matches!(block, smith_events::Block::Text {{ .. }}))"
                            ));
                        }
                    }
                    if prompt.system.is_some() && prompt.messages.is_empty() {
                        return Err(format!("record {position}: first request carries start text"));
                    }
                }
            }
            Event::ResponseCompleted(response) => {
                if header.capture == smith_events::Capture::Everything
                    && response.outcome == smith_events::Outcome::Completed
                    && response.completion.is_none()
                {
                    return Err(format!(
                        "record {position}: everything capture includes the completed response's blocks"
                    ));
                }
                let identity = (response.run, response.conversation, response.response);
                let index = responses
                    .iter()
                    .position(|started| *started == identity)
                    .ok_or_else(|| format!("record {position}: one provider handoff before terminal"))?;
                responses.remove(index);
                spent =
                    spent.checked_add(response.spent).ok_or_else(|| format!("record {position}: bounded charges"))?;
                let seen = [
                    response.usage.input_tokens,
                    response.usage.cache_read_tokens,
                    response.usage.cache_write_tokens,
                    response.usage.output_tokens,
                    response.usage.reasoning_tokens,
                ];
                if reported_usage {
                    for (sum, value) in usage.iter_mut().zip(seen) {
                        *sum = match (*sum, value) {
                            (Some(sum), Some(value)) => Some(
                                sum.checked_add(value).ok_or_else(|| format!("record {position}: bounded usage"))?,
                            ),
                            (None, Some(value)) => Some(value),
                            (sum, None) => sum,
                        };
                    }
                } else {
                    usage = seen;
                    reported_usage = true;
                }
                let started = records[..position]
                    .iter()
                    .find_map(|record| match &record.event {
                        Event::ResponseStarted(started) if started.response == response.response => Some(started),
                        _ => None,
                    })
                    .ok_or_else(|| format!("record {position}: response start"))?;
                if program || started.model.backend == smith_events::Backend::ClaudeCode {
                    if (response.request_bytes, response.first_byte_ms, response.largest_gap_ms) != (None, None, None) {
                        return Err(format!(
                            "record {position}: (response.request_bytes, response.first_byte_ms, response.largest_gap_ms), (None, None, None)\nleft: {:?}\nright: (None, None, None)",
                            (response.request_bytes, response.first_byte_ms, response.largest_gap_ms)
                        ));
                    }
                } else if response.outcome == smith_events::Outcome::Completed {
                    if response
                        .request_bytes
                        .ok_or_else(|| format!("record {position}: completed request was encoded"))?
                        == 0
                    {
                        return Err(format!(
                            "record {position}: response.request_bytes.expect(\"completed request was encoded\") > 0"
                        ));
                    }
                    if response.first_byte_ms.is_none() {
                        return Err(format!("record {position}: the native peer sent plaintext response bytes"));
                    }
                    if response.largest_gap_ms.is_none() {
                        return Err(format!("record {position}: the native peer sent multiple SSE events"));
                    }
                }
            }
            Event::ToolStarted(call) => {
                let identity = (call.run, call.conversation, call.call.clone());
                if calls.contains(&identity) {
                    return Err(format!("record {position}: !calls.contains(&identity)"));
                }
                calls.push(identity);
                match header.capture {
                    smith_events::Capture::Calls | smith_events::Capture::Everything => {
                        let input = call
                            .input
                            .as_ref()
                            .ok_or_else(|| format!("record {position}: the call carries its provider arguments"))?;
                        if skein_lib::len_u64(input.len()) != call.input_bytes {
                            return Err(format!(
                                "record {position}: skein_lib::len_u64(input.len()), call.input_bytes\nleft: {:?}\nright: {:?}",
                                skein_lib::len_u64(input.len()),
                                call.input_bytes
                            ));
                        }
                        if header.capture == smith_events::Capture::Everything {
                            let arguments = records
                                .iter()
                                .take(position)
                                .rev()
                                .find_map(|record| match &record.event {
                                    Event::ResponseCompleted(response)
                                        if response.run == call.run && response.conversation == call.conversation =>
                                    {
                                        response.completion.as_ref().and_then(|completion| {
                                            completion.blocks.iter().find_map(|block| match block {
                                                smith_events::Block::Call { call: name, tool, input }
                                                    if name == &call.call && tool == &call.tool =>
                                                {
                                                    Some(input)
                                                }
                                                _ => None,
                                            })
                                        })
                                    }
                                    _ => None,
                                })
                                .ok_or_else(|| format!("record {position}: the independently decoded provider completion contains the call"))?;
                            if input != arguments {
                                return Err(format!(
                                    "record {position}: call inputs equal the provider's original arguments: inputs differ"
                                ));
                            }
                        }
                    }
                    smith_events::Capture::None => {
                        if call.input.is_some() {
                            return Err(format!("record {position}: call.input.is_none()"));
                        }
                    }
                    smith_events::Capture::Unknown(_) => {
                        return Err(format!("record {position}: the fixture selects a known capture policy"));
                    }
                }
            }
            Event::ToolCompleted(call) => {
                let identity = (call.run, call.conversation, call.call.clone());
                let index = calls
                    .iter()
                    .position(|started| *started == identity)
                    .ok_or_else(|| format!("record {position}: one call start before terminal"))?;
                calls.remove(index);
                let line = smith_events::write(
                    record,
                    &smith_events::Capture::Everything,
                    record_limits.get_or_init(|| crate::tracing::event_limits(&crate::limits()).sinks.records),
                )
                .map_err(|_| format!("record {position}: tool terminal fits its receiving bounds"))?
                .ok_or_else(|| format!("record {position}: tool terminal is emitted"))?;
                if skein_lib::bytes::find(&line, b"\"result\":").is_some() {
                    return Err(format!(
                        "record {position}: tool.completed carries no pre-render result under the binding contract"
                    ));
                }
            }
            Event::CheckStarted(check) => {
                if checks.contains(&check.run) {
                    return Err(format!("record {position}: !checks.contains(&check.run)"));
                }
                checks.push(check.run);
            }
            Event::CheckCompleted(check) => {
                let index = checks
                    .iter()
                    .position(|started| *started == check.run)
                    .ok_or_else(|| format!("record {position}: one check start before terminal"))?;
                checks.remove(index);
            }
            _ => {}
        }
    }
    if !responses.is_empty() && conversations.is_empty() && calls.is_empty() && checks.is_empty() {
        return Err(format!("record {position}: every start settles once"));
    }
    if answered.len() != 1 {
        return Err(format!(
            "record {position}: one answer includes the entrance refusal\nleft: {:?}\nright: {:?}",
            answered.len(),
            1
        ));
    }
    if !runs.iter().all(|run| answered.contains(run)) {
        return Err(format!("record {position}: every admitted run answers"));
    }
    for count in &end.emitted {
        if reported.insert(count.record.as_ref(), count.count).is_some() {
            return Err(format!("record {position}: one emitted counter per type"));
        }
    }
    if observed != reported {
        return Err(format!(
            "record {position}: every emitted type, including authoritative records, equals file observations\nleft: {observed:?}\nright: {reported:?}"
        ));
    }
    Ok(())
}

fn name(event: &Event) -> &'static [u8] {
    match event {
        Event::Budget(_) => b"budget",
        Event::SessionStarted(_) => b"session.started",
        Event::SessionEnded(_) => b"session.ended",
        Event::RunStarted(_) => b"run.started",
        Event::RunCompleted(_) => b"run.completed",
        Event::ConversationOpened(_) => b"conversation.opened",
        Event::ConversationClosed(_) => b"conversation.closed",
        Event::ResponseStarted(_) => b"response.started",
        Event::ResponseCompleted(_) => b"response.completed",
        Event::TextDelta(_) => b"text.delta",
        Event::ToolStarted(_) => b"tool.started",
        Event::JobEnded(_) => b"job.ended",
        Event::ToolCompleted(_) => b"tool.completed",
        Event::CheckStarted(_) => b"check.started",
        Event::CheckCompleted(_) => b"check.completed",
        Event::MessageReceived(_) => b"message.received",
        Event::MessageRead(_) => b"message.read",
        Event::MessageUnread(_) => b"message.unread",
        Event::Notice(_) => b"notice",
    }
}

/// Decode the closed append file under its producer's receiving ceiling.
#[must_use]
pub fn read(bytes: &[u8], events: &smith_protocol_events::EventLimits) -> Vec<Record> {
    let ceiling = events.sinks.line_ceiling(events.capture).expect("producer line ceiling");
    let mut reader = smith_events::Reader::with_ceiling(&events.sinks.records, ceiling).expect("reader limits");
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let line = reader.feed(&bytes[offset..]).expect("valid event stream");
        assert!(line.consumed > 0);
        offset += line.consumed;
        if let Some(record) = line.record {
            records.push(record);
        }
    }
    reader.finish().expect("newline-terminated stream");
    records
}
