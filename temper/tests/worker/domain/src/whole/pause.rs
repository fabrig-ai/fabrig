//! Link pauses with a live process and the worker's simulated clock.
use super::{ATTEMPT, Case, REMOTE, RUN, WORKSTREAM, World, assignment, host, wire};
use jig_domain_link::{PauseMode, PauseState};
use skein_lib::{Duration, Token};
use temper_worker_domain::Event;

fn ask(world: &mut World, mode: PauseMode) -> Result<(), String> {
    world.event(Event::Link(host::LinkDown::Pause { mode }))?;
    Ok(())
}

/// Pausing leaves the current run live, and a resume admits its next attempt.
pub fn pause_and_resume(seed: u64) -> Result<(), String> {
    let mut world = World::with_slots(seed, 2);
    world.start()?;
    ask(&mut world, PauseMode::Pause)?;
    let run = Token::new(82);
    let start = wire::Start::Base { branch: b"main".as_slice().into() };
    world.event(assignment(
        world.first,
        run,
        WORKSTREAM + 1,
        wire::Start::Base { branch: b"main".as_slice().into() },
    ))?;
    assert_eq!(world.busy.len(), 1, "a free slot does not admit while paused");
    assert_eq!(world.busy[0].0, run);
    assert_eq!(world.report.cancellations, 0);
    assert_eq!(world.report.drains, 0);
    world.story(Case::Delivered)?;
    assert!(matches!(world.answer.take().expect("paused run finishes").0.ending, host::Ending::Ended { .. }));
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.domain.host().hosted(), 0);
    ask(&mut world, PauseMode::Resume)?;
    world.process = None;
    world.attempt = Token::new(14);
    world.assign(start)?;
    world.say(temper_worker_domain::agent::Up::Admitted)?;
    assert_eq!(world.domain.host().unanswered(), 1, "the next assignment is admitted");
    assert_eq!(world.busy.len(), 1);
    world.park()?;
    world.answer.take().expect("resumed run answers");
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    assert_eq!(world.report.pauses, [PauseState::Paused, PauseState::Taking]);
    assert_eq!(world.root_pauses, world.report.pauses);
    assert!(world.up);
    assert_eq!(world.stops, 0);
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 1);
    Ok(())
}

/// A stop replaces the link drain's deadline with its own from the signal.
pub fn stop_during_link_drain(seed: u64) -> Result<(), String> {
    let mut world = World::new(seed);
    world.start()?;
    world.env.now = world.env.now.saturating_add(Duration::from_secs(7));
    ask(&mut world, PauseMode::Drain)?;
    let link_bound = world.env.now.saturating_add(world.env.limits.drain);
    world.env.now = world.env.now.saturating_add(Duration::from_secs(5));
    world.event(Event::Shutdown { drain: true })?;
    let stop_drain = world.env.now.saturating_add(world.env.limits.drain);
    assert_eq!(world.domain.next_deadline(), Some(stop_drain), "only the stop's drain deadline is armed");
    assert_eq!(world.shutdown_notices, 1);
    assert_eq!(world.report.drains, 1, "the agent is not drained twice");
    world.env.now = link_bound;
    world.settle()?;
    assert_eq!(world.report.cancellations, 0, "the link drain's earlier bound no longer applies");
    world.env.now = stop_drain;
    world.settle()?;
    assert_eq!(world.report.cancellations, 1, "the stop's drain bound is counted from the signal");
    world.gone()?;
    assert_eq!(world.report.saves, 1);
    assert!(matches!(
        world.answer.take().expect("shutdown answer").0.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Shutdown), .. }
    ));
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.stops, 1);
    assert_eq!(world.report.pauses, [PauseState::Draining], "no paused notice follows shutdown");
    assert!(world.forge.branch(REMOTE, b"saved").is_some());
    Ok(())
}
