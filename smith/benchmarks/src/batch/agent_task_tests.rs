use super::*;

#[test]
fn a_prepared_pin_runs_only_its_selected_tasks() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let suite_file = root.join("suites/probes.toml");
    let mut suite = crate::read_suite(&suite_file).expect("suite");
    suite.tasks = vec!["probes/smoke-tools".into(), "probes/large-write".into()];
    suite.agents.retain(|agent| agent.agent == crate::Agent::Codex);
    suite.agents[0].tasks = vec!["probes/large-write".into()];
    let pin_file = root.join("agents/codex/standard.pin.toml");
    let pin = crate::read_configuration(&pin_file).expect("pin");
    let mut identity =
        crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("identity fixture").identity;
    identity.task = "probes/large-write".into();
    identity.agent = crate::Agent::Codex;
    identity.provider = crate::Provider::Codex;
    identity.agent_version = pin.pin.version.clone();
    identity.configuration_sha256 = pin.sha256.clone();
    identity.model.requested_model = "gpt-6-luna".into();
    identity.model.requested_effort = "low".into();
    let mut request = Request {
        program: "/nonexecuted".into(),
        configuration: Some(pin_file),
        teardown_ms: 0,
        grace_ms: 1,
        settlement_ms: 1,
        max_stdout_bytes: 1,
        max_stderr_bytes: 1,
        disk_estimate_bytes: 1,
        ..crate::request_fixture::request(root.join("tasks/probes/large-write/task.toml"), identity)
    };
    assert_eq!(selected_configuration_matches(&suite.agents, std::slice::from_ref(&pin), &request), Ok(true));
    request.identity.task = "probes/smoke-tools".into();
    request.task = root.join("tasks/probes/smoke-tools/task.toml");
    assert_eq!(selected_configuration_matches(&suite.agents, &[pin], &request), Ok(false));
}
