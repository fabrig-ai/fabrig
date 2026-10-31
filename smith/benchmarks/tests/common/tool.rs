//! Tool pairs share their run, conversation and call identity.

use super::{edit, events::record};
use smith_events::Record;

pub fn tool_pair(tool: &str, call: &str) -> (Record, Record) {
    static START: std::sync::LazyLock<Record> = std::sync::LazyLock::new(|| {
        record(include_str!("../../../crates/smith-events/golden/v4/tool_started_everything.jsonl"))
    });
    static END: std::sync::LazyLock<Record> = std::sync::LazyLock::new(|| {
        record(include_str!("../../../crates/smith-events/golden/v4/tool_completed_everything.jsonl"))
    });
    let mut start = START.clone();
    let mut end = END.clone();
    let value = edit::tool_started(&mut start);
    value.run = 0;
    value.conversation = 0;
    value.call = call.as_bytes().into();
    value.tool = tool.as_bytes().into();
    let value = edit::tool_completed(&mut end);
    value.run = 0;
    value.conversation = 0;
    value.call = call.as_bytes().into();
    value.tool = tool.as_bytes().into();
    (start, end)
}
