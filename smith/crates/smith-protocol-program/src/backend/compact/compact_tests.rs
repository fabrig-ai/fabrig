//! Summary init accepts only tool-free sessions (smith/docs/design/protocol/programs.md, 5.8).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust fixtures, programming-model.md 10.2"
)]

use super::super::Settings;
use super::checked_init;
use serde_json::{Value, json};

const LINE_BYTES: u32 = 4096;
const DEPTH: u32 = 32;

fn init() -> Value {
    let mut init: Value =
        serde_json::from_slice(&super::super::stream::tests::init_line(&[])).expect("recorded init JSON");
    init["mcp_servers"] = json!([]);
    init
}

#[test]
fn a_summary_init_with_no_tools_and_no_servers_passes() {
    let line = serde_json::to_vec(&init()).expect("summary init JSON");
    assert_eq!(checked_init(&line, DEPTH, Settings::stream_tokens(LINE_BYTES)), Some(()));
}

#[test]
fn a_summary_init_with_a_tool_or_a_server_fails() {
    for (tools, servers) in [
        (json!(["Read"]), json!([])),
        (json!([]), json!([{"name":"smith","status":"connected","source":"dynamic"}])),
        (json!(["mcp__smith__finish"]), json!([{"name":"smith","status":"connected","source":"dynamic"}])),
    ] {
        let mut init = init();
        init["tools"] = tools;
        init["mcp_servers"] = servers;
        let line = serde_json::to_vec(&init).expect("summary init JSON");
        assert_eq!(
            checked_init(&line, DEPTH, Settings::stream_tokens(LINE_BYTES)),
            None,
            "summary init tools={}, mcp_servers={}",
            init["tools"],
            init["mcp_servers"]
        );
    }
}
