//! A worker's slots compose smith's real single-agent host limits (temper/docs/design/domain/worker/README.md, 5).
use super::fixtures;
use jig_domain_host as hub;
use jig_protocol_worker_link as link;
use skein_lib::{Queue, Token};
use temper_worker_domain as root;

#[test]
fn two_agents_run_at_once_under_the_real_host_limits() {
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).expect("stock agent"));
    assert_eq!(agent.host_limits.agents, 1, "smith sizes one agent's host");
    let mut settings = fixtures::worker_settings();
    settings.jig.slots = 2;
    settings.jig.limits = jig_protocol_config::profile(jig_protocol_config::Profile::Standard);
    let hub = &mut settings.jig.limits.hub;
    hub.slots = 2;
    hub.accounts = agent.host_limits.accounts;
    hub.turns = agent.host_limits.turns;
    hub.turn_bytes = u64::from(agent.limits.bodies.turn_body);
    hub.turn_queue_bytes = hub.turn_bytes.checked_mul(u64::from(hub.turns)).unwrap();
    hub.event_bytes = agent.host_limits.message_bytes.checked_sub(2).unwrap();
    hub.run_calls = agent.host_limits.calls;
    hub.held = agent.host_limits.messages;
    hub.inbox = agent.bounds.inbox;
    let mut own = fixtures::worker_limits();
    own.checkout.workspaces = 2;
    let payload = crate::worker_settings::payload_limits(&own).unwrap();
    let link = crate::worker_settings::link_limits(&settings.jig, &payload, own.peer.forges).unwrap();
    settings.jig.limits.io.output =
        jig_protocol_worker_wire::sizes::largest_body(&link.sizes, jig_protocol_worker_wire::Kind::Turn)
            .unwrap()
            .checked_add(8)
            .unwrap();
    let mut worker =
        crate::Worker::new(&settings, &agent, &own, &fixtures::worker_files(), fixtures::worker_runtime(), 7)
            .expect("two slots fit the real agent");
    let first = assigned(&mut worker, &agent, Token::new(101));
    let second = assigned(&mut worker, &agent, Token::new(102));
    assert_ne!(first, second, "two distinct smith process owners spawn");
    assert_eq!(worker.root.agent().agents(), 2, "both agents remain held at once");
    assert_eq!(worker.root.kept_answers(), 0, "neither assignment was answered busy");
}

fn assigned(worker: &mut crate::Worker, agent: &crate::settings::Agent, run: Token) -> Token {
    let link::Request::Down { down: hub::LinkDown::Assign { mut assignment }, .. } = fixtures::chat_assignment() else {
        panic!("chat assignment")
    };
    assignment.assignment.run = run;
    let fixture_agent = fixtures::worker_agent();
    let charter = smith_protocol_channel::decode_charter(
        &assignment.assignment.charter,
        &agent.limits.charter,
        &fixture_agent.endpoints,
    )
    .expect("fixture charter");
    assignment.assignment.charter =
        smith_protocol_channel::encode_charter(&charter, &agent.limits.charter, &agent.endpoints)
            .expect("charter uses the stock endpoint names");
    let mut out = Queue::with_capacity(256);
    root::step(&mut worker.root, &worker.env, root::Event::Link(hub::LinkDown::Assign { assignment }), &mut out);
    let mut spawned = None;
    while let Some(request) = out.pop() {
        if let root::Request::Decode { agent: owner, run, start } = request {
            let start = crate::worker_route::decode(run, *start, agent);
            assert!(start.is_ok(), "the stock agent decodes the assignment: {start:?}");
            root::step(&mut worker.root, &worker.env, root::Event::Decoded { agent: owner, start }, &mut out);
        } else {
            if let root::Request::Process(ref process) = request
                && let smith_host_domain::process::Request::Spawn { owner, .. } = process.as_ref()
            {
                assert!(spawned.replace(*owner).is_none(), "one process per assignment");
            }
            worker.route(request);
        }
    }
    spawned.expect("the assigned run spawns rather than answering busy")
}
