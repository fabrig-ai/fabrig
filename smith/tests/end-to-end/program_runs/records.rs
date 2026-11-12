//! Independent decoding of each process stream (smith/docs/design/protocol/events.md, sections 2 and 6).
#[expect(clippy::disallowed_types, reason = "bounded outside observation of the settled agent trace")]
pub(super) fn agent(trace: &std::path::Path) -> Vec<smith_events::Record> {
    use std::io::Read;
    let mut path = trace.as_os_str().to_os_string();
    path.push(".agent");
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .expect("agent trace exists")
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .expect("read agent trace");
    assert!(bytes.len() <= 1_048_576, "agent trace stays within the process stream bound");
    read(&bytes)
}

pub(super) fn read(bytes: &[u8]) -> Vec<smith_events::Record> {
    // The binary harness captures at most one MiB per stream.
    const STREAM_BYTES: u32 = 1_048_576;
    let limits = {
        let mut fixture_limits = smith_events_fixtures::SMALL;
        fixture_limits.string = STREAM_BYTES;
        fixture_limits.content = STREAM_BYTES;
        fixture_limits.items = STREAM_BYTES;
        fixture_limits
    };
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            smith_events::read_with_ceiling(line, &limits, STREAM_BYTES)
                .expect("typed event record")
                .expect("known event kind")
        })
        .collect()
}

pub(super) fn answer(records: &[smith_events::Record]) -> &smith_events::RunCompleted {
    use smith_events::Event;
    assert!(
        matches!(records.first().map(|record| &record.event), Some(Event::SessionStarted(_))),
        "product stream starts with its header"
    );
    assert!(
        matches!(records.last().map(|record| &record.event), Some(Event::SessionEnded(_))),
        "product stream ends with its terminal"
    );
    let completed: Vec<_> = records
        .iter()
        .filter_map(|record| match &record.event {
            Event::RunCompleted(answer) => Some(answer),
            Event::SessionStarted(_) | Event::SessionEnded(_) | Event::Notice(_) => None,
            Event::RunStarted(_)
            | Event::Budget(_)
            | Event::ConversationOpened(_)
            | Event::ConversationClosed(_)
            | Event::ResponseStarted(_)
            | Event::ResponseCompleted(_)
            | Event::TextDelta(_)
            | Event::ToolStarted(_)
            | Event::ToolCompleted(_)
            | Event::JobEnded(_)
            | Event::CheckStarted(_)
            | Event::CheckCompleted(_)
            | Event::MessageReceived(_)
            | Event::MessageRead(_)
            | Event::MessageUnread(_) => {
                panic!("detailed records belong to the child process");
            }
        })
        .collect();
    assert_eq!(completed.len(), 1, "one product run terminal");
    completed.into_iter().next().expect("one checked product terminal")
}
