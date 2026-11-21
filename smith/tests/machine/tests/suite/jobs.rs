//! Commands held under their original token, on the real kernel (protocol/agent.md, 2).

use skein_lib::Time;
use smith_domain::tools;
use smith_machine_world::{
    World,
    job_requests::{done, far, spawn},
};

#[test]
fn a_job_held_at_the_machines_close_is_closed_within_one_close() {
    let mut world = World::new(75, &[]);
    let replies = world.jobs(vec![spawn(world.root(), b"sleep 5", 8, 8, far())]);
    assert_eq!(done(&replies[0]), tools::Done::Held { head: Box::default(), tail: Box::default(), dropped: 0 });
    assert!(world.now() < Time::from_nanos(1_030_000_000), "yield plus one io close");
}
