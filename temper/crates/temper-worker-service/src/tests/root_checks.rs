use super::fixtures;
use crate::startup::{Problems, worker::worker as check_worker, lines};
#[test]
fn the_checkouts_repositories_past_the_agents_directories_name_both_limits() {
    let mut agent = fixtures::startup_agent();
    agent.host_limits.directories = 1;
    let worker = fixtures::worker_settings();
    let own = crate::worker_settings::Limits::shared(temper_service_core::limits::profile(temper_service_core::limits::Profile::Small));
    let mut found = Problems::empty();
    check_worker(&crate::worker_settings::root_limits(&worker, &agent, &own), &mut found);
    assert_eq!(
        &*lines(&found),
        b"application.limits.overrides.checkout.repositories: must agree with the agent's directories\n"
    );
}

