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
pub fn apply(configuration: &mut super::Configuration, turn: u32) {
    let turn = u64::from(turn);
    let resume = (turn + 4).max(4_194_304);
    let box_bytes = u64::try_from(size_of::<Box<[u8]>>()).expect("turn box slot");
    let window = resume + (box_bytes - 4) * (resume / 4);
    let loaded = turn + u64::try_from(size_of::<temper_engine_domain::Record>()).expect("record slot");
    let output = configuration["workers"]
        .as_array()
        .and_then(|workers| workers.iter().map(|worker| worker_output(&worker["limits"], turn)).max())
        .unwrap_or_else(|| output(turn, configuration.jig.core.tasks.run_inbox, configuration.jig.io.output));
    configuration.jig.hub.turn_bytes = turn;
    configuration.jig.hub.transcript_bytes = window;
    configuration.jig.hub.turn_queue_bytes = 8 * turn;
    configuration.jig.core.resume_bytes = u32::try_from(resume).expect("resume");
    configuration.jig.io.output = u32::try_from(output).expect("output");
    configuration.limits.root.journal.transcript_bytes = u32::try_from(resume).expect("resume");
    configuration.limits.root.loads.transcript_bytes = u32::try_from(turn).expect("turn");
    configuration.limits.root.loads.bytes = u32::try_from(loaded).expect("loaded");
    configuration.limits.root.loads.reply_bytes = u32::try_from(loaded).expect("loaded");
    configuration.limits.root.loads.read_bytes = 2 * turn;
    apply_workers(&mut configuration.document, u32::try_from(turn).expect("turn"));
    configuration.derive_places();
}
/// Serialized subprocess worlds keep only the worker's document figures.
pub fn apply_workers(document: &mut Value, turn: u32) {
    let turn = u64::from(turn);
    let resume = (turn + 4).max(4_194_304);
    let box_bytes = u64::try_from(size_of::<Box<[u8]>>()).expect("turn box slot");
    let window = resume + (box_bytes - 4) * (resume / 4);
    if let Some(workers) = document["workers"].as_array_mut() {
        for worker in workers {
            hub(&mut worker["limits"]["overrides"], turn, window);
            let output = worker_output(&worker["limits"], turn);
            worker["limits"]["overrides"]["io.output"] = output.into();
        }
    }
}
fn worker_output(limits: &Value, turn: u64) -> u64 {
    let defaults = jig_protocol_config::stock();
    let overrides = &limits["overrides"];
    let inbox = overrides["tasks.run_inbox"].as_u64().unwrap_or(u64::from(defaults.core.tasks.run_inbox));
    output(
        turn,
        u32::try_from(inbox).expect("run inbox"),
        u32::try_from(overrides["io.output"].as_u64().unwrap_or(u64::from(defaults.io.output))).expect("output"),
    )
}
fn output(turn: u64, inbox: u32, initial: u32) -> u64 {
    use jig_protocol_worker_wire::v15;
    let bodies =
        v15::Limits { turn_body: u32::try_from(turn).expect("turn body"), turn_refused: inbox, ..v15::CEILINGS };
    let frame = v15::Turn::encoded_bound(&bodies).expect("turn frame") + 8;
    u64::from(initial).max(frame)
}
fn hub(limits: &mut Value, turn: u64, window: u64) {
    limits["hub.turn_bytes"] = turn.into();
    limits["hub.transcript_bytes"] = window.into();
    limits["hub.turn_queue_bytes"] = (8 * turn).into();
}
