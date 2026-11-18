//! The agent's turn allowances (temper/docs/design/playground.md, 3).
use serde_json::Value;

/// Derive the turn body bound from a credential-free agent document.
#[must_use]
pub fn turn_body(agent: &Value) -> u32 {
    // Only the profile and model declarations determine T. Give the reader
    // local, credential-free destinations so it opens no trust or login files.
    let mut agent = agent.clone();
    if agent["programs"].is_string() {
        agent["programs"] = serde_json::json!({});
    } else if agent["programs"]["claude_code"]["path"] == "fetched" {
        agent["programs"]["claude_code"]["path"] = "/fixture/claude".into();
    }
    for endpoint in agent["endpoints"].as_array_mut().expect("agent endpoints") {
        endpoint["address"] = "127.0.0.1:9".into();
        endpoint["transport"] = "plaintext".into();
        let endpoint = endpoint.as_object_mut().expect("endpoint");
        endpoint.remove("server_name");
        endpoint.remove("trust_der");
    }
    smith_agent_shell::config::from_value(agent)
        .unwrap_or_else(|why| panic!("agent profile: {why:?}"))
        .service
        .limits
        .channel
        .bodies
        .turn_body
}

/// Apply the agent's turn allowance to the engine and workers.
pub fn apply(document: &mut Value, turn: u32) {
    let turn = u64::from(turn);
    let resume = (turn + 4).max(4_194_304);
    let box_bytes = u64::try_from(size_of::<Box<[u8]>>()).expect("turn box slot");
    let window = resume + (box_bytes - 4) * (resume / 4);
    let loaded = turn + u64::try_from(size_of::<temper_engine_domain::Record>()).expect("record slot");

    if !document["engine"].is_null() {
        let output = worker_output(&document["engine"]["limits"], turn);
        let limits = &mut document["engine"]["limits"]["overrides"];
        hub(limits, turn, window);
        limits["core.resume_bytes"] = resume.into();
        limits["io.output"] = output.into();
    }
    if !document["application"]["limits"].is_null() {
        let limits = &mut document["application"]["limits"]["overrides"];
        for (name, bytes) in [
            ("root.journal.transcript_bytes", resume),
            ("root.loads.transcript_bytes", turn),
            ("root.loads.bytes", loaded),
            ("root.loads.reply_bytes", loaded),
            ("root.loads.read_bytes", 2 * turn),
        ] {
            limits[name] = bytes.into();
        }
    }
}
fn worker_output(limits: &Value, turn: u64) -> u64 {
    use jig_protocol_worker_wire::v17;
    let defaults = jig_protocol_config::stock();
    let overrides = &limits["overrides"];
    let inbox = overrides["tasks.run_inbox"].as_u64().unwrap_or(u64::from(defaults.core.tasks.run_inbox));
    let bodies = v17::Limits {
        turn_body: u32::try_from(turn).expect("turn body"),
        turn_refused: u32::try_from(inbox).expect("run inbox"),
        ..v17::CEILINGS
    };
    let frame = v17::Turn::encoded_bound(&bodies).expect("turn frame") + 8;
    let output = overrides["io.output"].as_u64().unwrap_or(u64::from(defaults.io.output));
    output.max(frame)
}
fn hub(limits: &mut Value, turn: u64, window: u64) {
    limits["hub.turn_bytes"] = turn.into();
    limits["hub.transcript_bytes"] = window.into();
    limits["hub.turn_queue_bytes"] = (8 * turn).into();
}
