//! Smith's side of the host-call relation (smith/docs/design/testing.md, 6).

use skein_world::inspect::{Inspection, Moment, Related, check_tables, check_waits};
use smith_agent_world::{Job, Settings, World};
use smith_domain::{
    Event, Start,
    run::{HostAnswer, HostReply},
};
use smith_inspect::{KINDS, host_calls};

#[test]
fn inspection_names_a_relay_until_the_host_answers() {
    let settings = Settings { job: Job::HostTools, ..Settings::calm(811) };
    let start = Start {
        activation: 4,
        ..smith_agent_world::fixture::start(smith_agent_world::scripted_charter(&settings), true)
    };
    let mut world = World::with_start(settings, Event::Start(start));
    world.enable_parent_host_calls();
    assert!(!world.drive(20_000), "the host has not answered");
    let pending = world.pending_host_calls();
    let [call] = pending.as_slice() else { panic!("one relayed call") };
    let relay = call.relay;
    let name = call.name;
    assert_ne!(name.activation, u64::from(relay.attempt), "activation is not the relay attempt");
    let mut reading = Inspection::new("smith", KINDS);
    smith_inspect::inspect(world.domain(), &mut reading);
    assert!(reading.tables().iter().any(|table| table.name == "runs" && table.entries.len() == 1));
    assert!(reading.tables().iter().any(|table| table.name == "open calls" && table.entries.len() == 1));
    let moment = Moment { seed: settings.seed, at: call.at, iteration: 1 };
    check_waits(&[&reading], moment).expect("the run has a host wake");
    check_tables(&[&reading], moment).expect("the live call owns its entries");
    host_calls(world.domain(), 7, &mut reading);
    assert_eq!(
        reading.relateds(),
        &[Related {
            relation: "a run's host calls",
            key: format!(
                "task 7 attempt {} completion {} position {}",
                name.activation, name.completion, name.position
            ),
            holds: "an open call",
        }]
    );

    let answer = HostAnswer::new(b"parent decision".as_slice().into(), false).expect("bounded answer");
    world.return_host_reply(relay, HostReply::Answered(answer)).expect("pending relay");
    assert!(!world.drive(1), "the terminal reaches smith before the run settles");
    let mut reading = Inspection::new("smith", KINDS);
    host_calls(world.domain(), 7, &mut reading);
    assert!(reading.relateds().is_empty(), "no unanswered relay after the host's terminal");
    assert!(world.drive(20_000));
}
