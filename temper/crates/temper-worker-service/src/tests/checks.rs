use super::fixtures;
use crate::startup::lines;
use jig_protocol_config as config;
#[test]
fn a_worker_part_names_the_agents_program() {
    let part =
        crate::startup::worker_part(&fixtures::worker_alone_document(), &config::Directories::working()).unwrap();
    assert_eq!(part.named.program.as_ref(), fixtures::TEMPER);
}

#[test]
fn worker_checkout_capacity_names_stdout_and_its_room_without_an_engine() {
    let document = fixtures::with(
        &fixtures::worker_alone_document(),
        b"application.limits.overrides",
        br#"{"checkout.conflicts":1024,"checkout.path_bytes":4096,"git_component.stdout_bytes":262144}"#,
    );
    let mut agent = fixtures::startup_agent();
    agent.host_limits.path_bytes = 4096;
    let part = crate::startup::worker_part(&document, &config::Directories::working()).unwrap();
    let found = crate::startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7)
        .err()
        .expect("checkout capacity is refused");
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        String::from_utf8_lossy(&lines(&found)),
        "checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 2: raise these with them\ngit_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes\nworkers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401142 bytes\n"
    );
}

