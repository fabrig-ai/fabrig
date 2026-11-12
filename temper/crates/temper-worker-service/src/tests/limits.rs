use super::fixtures;
#[test]
fn the_worker_takes_its_agents_host_reply_bound() {
    let input = fixtures::owned_worker_alone_document();
    let part = crate::startup::worker_part(&input, &jig_protocol_config::Directories::working()).unwrap();
    let agent = fixtures::owned_worker_agent();
    assert_eq!(agent.limits.bodies.host_reply_text, 4_098);
    assert_eq!(crate::worker_settings::root_limits(&part.worker, &agent, &part.limits).reply_bytes, 4_098);
}

