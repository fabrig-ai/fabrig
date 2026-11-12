//! The agent's engine-facing declaration (protocol/services.md, 3).
use super::fixtures;

#[test]
fn declaration_preserves_the_agents_engine_facing_fields() {
    let mut agent = fixtures::agent();
    agent.limits.bodies.host_reply_text = 40_960;
    agent.programs = Box::new([crate::settings::Program {
        adapter: crate::settings::Adapter::ClaudeCode,
        path: b"/bin/claude".as_slice().into(),
        directory: b"/programs".as_slice().into(),
    }]);
    let mut endpoints = skein_lib::List::with_capacity(2);
    for endpoint in agent.endpoints.entries() {
        endpoints.push(endpoint.clone()).unwrap();
    }
    endpoints
        .push(smith_protocol_channel::Endpoint {
            name: b"alias".as_slice().into(),
            number: 1,
            dialect: fixtures::DIALECT,
        })
        .unwrap();
    agent.endpoints = smith_protocol_channel::Endpoints::new(endpoints);

    let declaration = agent.declaration();
    assert_eq!(declaration.charter_bytes, agent.host_limits.charter_bytes);
    assert_eq!(declaration.tool_payload, agent.tool_payload);
    assert_eq!(declaration.host_ask_input, agent.limits.bodies.host_ask_input);
    assert_eq!(declaration.host_reply_text, agent.limits.bodies.host_reply_text);
    assert_eq!(declaration.turn_body, agent.limits.bodies.turn_body);
    assert_eq!(declaration.outcome_bytes, agent.host_limits.outcome_bytes);
    assert_eq!(declaration.run_limits, agent.run_limits);
    assert_eq!(declaration.bounds, agent.bounds);
    assert_eq!(declaration.programs.len(), agent.programs.len());
    assert_eq!(declaration.programs[0].as_ref(), agent.programs[0].adapter.name());
    assert_eq!(declaration.endpoints.len(), 2);
    assert_eq!(declaration.endpoints[0].as_ref(), b"fake");
    assert_eq!(declaration.endpoints[1].as_ref(), b"alias");
    for name in &declaration.endpoints {
        assert!(agent.endpoints.resolve(name).is_some());
    }
}
