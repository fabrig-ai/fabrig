//! The playground's native world (temper/docs/design/playground.md, 9).

use skein_lib::Wall;
use skein_sim_world::{Host, Printed, StateDigests, Stream, World};
use temper_playground::{machine::Machine, programs::Process, stories::Story};

/// The composition without a harness.
pub type Playground = World<Process<()>, Machine>;

// The composition's own startup tests use this step bound.
const STARTUP_STEPS: u64 = 10_000;

/// A story waiting for its visitor, with its standard output read as whole lines.
pub struct Started {
    pub world: Playground,
    pub printed: Vec<Printed>,
}

/// Start a story and step until it waits idle with nothing due now.
#[must_use]
pub fn start_idle(story: &Story, seed: u64, wall: Wall) -> Started {
    let mut world = temper_playground::world(story.start(seed, wall));
    for step in 1..=STARTUP_STEPS {
        temper_playground::story::step(&mut world);
        if !world.busy() && world.next_due().is_none_or(|due| due > world.now()) {
            eprintln!("story {} seed {seed}: idle after {} steps", story.name, world.steps());
            let printed = world.printed();
            return Started { world, printed };
        }
        assert!(
            step < STARTUP_STEPS,
            "story {} seed {seed}: not idle at step {step}, now {:?}, next due {:?}",
            story.name,
            world.now(),
            world.next_due()
        );
        if !world.busy()
            && let Some(due) = world.next_due()
        {
            world.advance(due);
        }
    }
    unreachable!("the startup bound fails on its last step")
}

/// Read the service's single sign-in link from the world's captured output.
#[must_use]
pub fn printed_link(lines: &[Printed]) -> &str {
    assert_eq!(lines.len(), 1, "one sign-in link on standard output");
    let line = &lines[0];
    assert_eq!(line.process.as_ref(), b"temper serve");
    assert_eq!(line.stream, Stream::Output);
    std::str::from_utf8(&line.line).expect("the service prints a UTF-8 link")
}

/// Snapshot the idle world's process digests and its shared fake disk.
#[must_use]
pub fn digests(world: &Playground) -> StateDigests {
    let Process::Service(service) = &world.processes()[0] else { panic!("the service is process zero") };
    StateDigests {
        procs: world.processes().iter().map(Host::state_digest).collect(),
        machine: Some(service.parameters.disk.borrow().state_digest()),
    }
}
