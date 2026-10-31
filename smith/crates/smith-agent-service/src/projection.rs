//! Adapts composed source limits and program inventory to event projection.
//! No event component retains the service configuration or its domain owner.
//! Contract: smith/docs/design/protocol/events.md, sections 4–5.

use alloc::boxed::Box;
use skein_lib::Duration;
use smith_protocol_events::sinks::{Capture, SinkLimits};
use smith_protocol_events::{EventLimits, Sources};

/// Derive receiving and destination ownership from the admitted source bounds.
/// `effort_bytes` is the charter codec’s reasoning effort bound.
/// `header` supplies the startup inventory's count and longest configured name.
#[must_use]
pub fn event_limits(
    limits: &crate::Limits,
    capture: Capture,
    destinations: u32,
    effort_bytes: u32,
    header: &[smith_events::EffectiveLimit],
    write_deadline: Duration,
) -> Option<EventLimits> {
    let charter = limits.channel.charter;
    let mut header_string = 0;
    for entry in header {
        header_string = header_string.max(u32::try_from(entry.name.len()).ok()?);
    }
    let metadata_string = charter
        .llm_endpoint
        .max(charter.llm_model)
        .max(charter.host_tool_name)
        .max(charter.field_name)
        .max(charter.run_result_label_some)
        .max(effort_bytes)
        .max(header_string)
        .max(46);
    let client = limits.llm.adapter.client;
    let string = metadata_string.max(client.strings).max(limits.channel.bodies.message_label);
    let answer_bytes = u32::try_from(limits.domain.run.outcome_bytes).ok()?;
    let metadata_items =
        charter.run_result_fields.max(limits.domain.run.host_tools).max(18).max(u32::try_from(header.len()).ok()?);
    let content = answer_bytes
        .max(client.answer)
        .max(client.request)
        .max(limits.channel.bodies.message_text)
        .max(client.detail_bytes);
    let items = metadata_items.max(client.output_items).max(client.history_items);
    let records = smith_events::Limits { string, content, items };
    let scalar_content = answer_bytes.max(client.detail_bytes);
    let metadata = smith_events::Limits { string, content: scalar_content, items: metadata_items };
    let scalar = smith_protocol_events::sinks::largest_line(&metadata, Capture::None, answer_bytes, metadata_string)?;
    // One completion's blocks share the Client's answer bytes. A message
    // receipt owns only that channel message's exact label and text.
    let observed = client
        .answer
        .checked_mul(6)?
        .checked_add(client.output_items.checked_mul(128)?)?
        .max(limits.channel.bodies.message_text.checked_mul(6)?);
    // The component validates this exact native body before copying a new
    // part. Its payload is a subset of that body. At most history_items
    // messages and blocks add 128 bytes of event framing per total block;
    // tool names add only bounded array framing. No replay payload is copied.
    let prompt = client
        .request
        .checked_add(client.history_items.checked_mul(128)?)?
        .checked_add(client.tools.checked_mul(128)?)?
        .checked_add(metadata_string.checked_mul(18)?)?
        .checked_add(2048)?;
    let observation_line = match capture {
        Capture::None | Capture::Calls => scalar.checked_add(observed)?,
        Capture::Everything => scalar.checked_add(observed)?.max(prompt),
    };
    let mut sinks = SinkLimits {
        destinations,
        output: 0,
        records,
        answer_bytes,
        metadata_string,
        metadata_items,
        scalar_content,
        observation_line,
        fact_extra: observed,
        conversations: limits.domain.run.conversations,
    };
    sinks.output = smith_protocol_events::sinks::output_bound(&sinks, capture)?;
    if !smith_protocol_events::sinks::valid(&sinks, capture) {
        return None;
    }
    Some(EventLimits { sinks, capture, front: false, write_deadline })
}

pub(crate) fn sources(limits: &crate::Limits) -> Sources {
    Sources { session: limits.domain.session, output: limits.io.output, events: limits.events }
}

/// Construct the process event projection from the service's admitted configuration.
#[must_use]
pub fn event_producer(config: &crate::Config) -> Option<smith_protocol_events::Producer> {
    config.limits.events?;
    let programs = match crate::service::bound(config.program.as_ref()) {
        Some(program) => program.endpoints.clone(),
        None => Box::default(),
    };
    Some(smith_protocol_events::Producer::new(&sources(&config.limits), config.channel_endpoints.clone(), programs))
}
