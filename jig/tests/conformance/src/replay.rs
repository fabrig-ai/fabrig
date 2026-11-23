//! Cold-cut continuations contain only durable rows and independent neighbours.
//! Fresh and resumed runs share both the story program and concrete actions.
use crate::{Testing, actions::World, scenarios::Continuation};
use jig_conformance::scenarios::{Scenario, ScenarioApplication, Step, program};
use jig_conformance::{Cut, Harness, Outcome, referee::Violation};
use std::{cell::RefCell, rc::Rc};

mod observations;
pub use observations::OutsideText;

#[derive(Clone, Debug, Default)]
struct Position {
    step: usize,
    action: Option<Continuation>,
    draining: bool,
    drain_started: u64,
}

fn finish_steps(
    world: &mut World,
    scenario: Scenario,
    mut position: Position,
    position_seen: &RefCell<Position>,
) -> Result<(), Violation> {
    let steps = program(scenario);
    loop {
        if position.draining {
            position_seen.replace(position.clone());
            world.drain_from(position.drain_started)?;
            position.draining = false;
        }
        let Some(&step) = steps.get(position.step) else { break };
        match step {
            Step::Action(action) => {
                let continuation = position.action.get_or_insert_with(|| Continuation::new(action));
                if continuation.advance(world).map_err(|error| world.diagnostic(error))? {
                    position.draining = true;
                    position.drain_started = world.outcome().iterations;
                } else {
                    position.action = None;
                    position.step += 1;
                }
            }
            Step::Drain => {
                position.step += 1;
                position.draining = true;
                position.drain_started = world.outcome().iterations;
            }
            Step::Advance(nanos) => {
                world.advance(nanos);
                position.step += 1;
            }
            Step::Crash => {
                world.crash().map_err(|error| world.diagnostic(error))?;
                position.step += 1;
            }
        }
    }
    Testing::check(world, scenario).map_err(|error| world.diagnostic(error))
}

fn diagnose(
    world: &mut World,
    seed: u64,
    scenario: Scenario,
    play: impl FnOnce(&mut World) -> Result<(), Violation>,
) -> Result<(), Violation> {
    eprintln!("scenario {scenario:?}, seed {seed}");
    play(world)
}

fn finish(
    world: &mut World,
    scenario: Scenario,
    seed: u64,
    position: Position,
    position_seen: &RefCell<Position>,
) -> Result<(), Violation> {
    diagnose(world, seed, scenario, |world| finish_steps(world, scenario, position, position_seen))
}

/// Replay every original numbered durable cut from its outside snapshot.
/// Snapshots are consumed immediately, so retained copies cannot inflate the
/// baseline's heap check. The seeded random run remains a separate fresh run.
///
/// # Errors
/// Returns the same scenario/referee failure as fresh cut replay.
pub fn every_cut(scenario: Scenario, seed: u64, visit: impl FnMut(u64, World) + 'static) -> Result<Outcome, Violation> {
    every_cut_traced(scenario, seed, None, visit)
}

/// Explicit complete traces for the independent snapshot/fresh equivalence proof.
///
/// # Errors
/// Returns a scenario/referee failure from baseline or a resumed cut.
pub fn every_cut_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    cuts_traced(scenario, seed, enabled, None, false, visit)
}

/// Compare selected durable snapshots without replaying the other cuts.
///
/// # Errors
/// Returns the same baseline or resumed scenario failure as all-cut replay.
pub fn chosen_cuts_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    cuts: &[u64],
    visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    cuts_traced(scenario, seed, enabled, Some(cuts.to_vec()), false, visit)
}

/// Hold the baseline once; each snapshot continuation and its visitor run in a child.
/// No retained observation is allocated in the baseline's metered heap.
///
/// # Errors
/// Returns a baseline violation or names the snapshot cut whose child failed.
pub fn fork_snapshots(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    cuts: Option<&[u64]>,
    visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    cuts_traced(scenario, seed, enabled, cuts.map(<[u64]>::to_vec), true, visit)
}

fn cuts_traced(
    scenario: Scenario,
    seed: u64,
    enabled: Option<bool>,
    selected: Option<Vec<u64>>,
    fork: bool,
    mut visit: impl FnMut(u64, World) + 'static,
) -> Result<Outcome, Violation> {
    let config = Testing::configuration(scenario, seed).expect("testing binds every shared story");
    let mut world = enabled.map_or_else(
        || Harness::<Testing>::new(config, seed),
        |enabled| Harness::<Testing>::with_trace(config, seed, enabled),
    );
    let position = Rc::new(RefCell::new(Position::default()));
    let hook_position = Rc::clone(&position);
    let failure = Rc::new(RefCell::new(None));
    let hook_failure = Rc::clone(&failure);
    let passed = Rc::new(RefCell::new(Vec::new()));
    let hook_passed = Rc::clone(&passed);
    world.on_commit(move |world| {
        if hook_failure.borrow().is_some() {
            return;
        }
        let cut = world.store.applied;
        if selected.as_ref().is_some_and(|cuts| !cuts.contains(&cut)) {
            return;
        }
        if fork {
            match skein_fork::split() {
                skein_fork::Split::Child => {}
                skein_fork::Split::Parent(skein_fork::Exit::Code(0)) => {
                    hook_passed.borrow_mut().push(cut);
                    return;
                }
                skein_fork::Split::Parent(exit) => {
                    hook_failure.replace(Some(Violation {
                        promise: jig_conformance::referee::Promise::Lost,
                        why: format!("snapshot cut {cut} failed: {exit:?}"),
                    }));
                    return;
                }
            }
        }
        let continuation = hook_position.borrow().clone();
        let result = world.cold_snapshot().and_then(World::from_snapshot).and_then(|mut replay| {
            finish(&mut replay, scenario, seed, continuation, &RefCell::new(Position::default()))?;
            visit(cut, replay);
            Ok(())
        });
        if fork {
            if let Err(error) = &result {
                panic!("snapshot cut {cut}: {error:?}");
            }
            skein_fork::passed();
        }
        if let Err(error) = result {
            hook_failure
                .replace(Some(Violation { promise: error.promise, why: format!("snapshot cut {cut}: {}", error.why) }));
        }
    });
    finish(&mut world, scenario, seed, Position::default(), &position)?;
    if let Some(error) = failure.take() {
        return Err(error);
    }
    let mut outcome = world.outcome();
    outcome.forked = passed.take();
    Ok(outcome)
}

/// Original fresh replay, retained for random cuts and independent comparison.
///
/// # Errors
/// Returns a scenario/referee failure.
pub fn fresh(scenario: Scenario, cut: Cut, seed: u64, enabled: bool) -> Result<World, Violation> {
    let config = Testing::configuration(scenario, seed).expect("testing binds every shared story");
    let mut world = Harness::<Testing>::with_trace(config, seed, enabled);
    world.cut(cut);
    finish_fresh(&mut world, scenario, seed)?;
    Ok(world)
}

/// A fresh prefix, never captured from a snapshot, held once for selected cuts.
/// Each child finishes its story and visits the complete outside state before exiting.
///
/// # Errors
/// Returns a scenario/referee failure or a child that did not pass its comparison.
pub fn fresh_cuts(
    scenario: Scenario,
    cuts: Option<&[u64]>,
    seed: u64,
    enabled: Option<bool>,
    mut visit: impl FnMut(u64, &World),
) -> Result<Outcome, Violation> {
    let config = Testing::configuration(scenario, seed).expect("testing binds every shared story");
    let mut world = enabled.map_or_else(
        || Harness::<Testing>::new(config, seed),
        |enabled| Harness::<Testing>::with_trace(config, seed, enabled),
    );
    match cuts {
        Some(cuts) => world.fork_cuts(cuts),
        None => world.cut(Cut::EachCommitFrom(1)),
    }
    finish_fresh(&mut world, scenario, seed)?;
    if let Some(cut) = world.forked_at() {
        visit(cut, &world);
    }
    jig_conformance::cut_passed();
    Ok(world.outcome())
}

fn finish_fresh(world: &mut World, scenario: Scenario, seed: u64) -> Result<(), Violation> {
    diagnose(world, seed, scenario, |world| {
        for step in program(scenario) {
            match step {
                Step::Action(action) => Testing::action(world, action).map_err(|error| world.diagnostic(error))?,
                Step::Drain => {
                    world.drain()?;
                }
                Step::Advance(nanos) => world.advance(nanos),
                Step::Crash => world.crash().map_err(|error| world.diagnostic(error))?,
            }
        }
        Testing::check(world, scenario).map_err(|error| world.diagnostic(error))?;
        Ok(())
    })
}

macro_rules! outside_state {
    ($world:ident; $($field:ident: $kind:ty = $value:expr),+ $(,)?) => {
        /// Owned outside state compared by snapshot equivalence proofs.
        #[derive(PartialEq, Eq, Debug)]
        pub struct Outside { $($field: $kind),+ }
        /// Complete outside state, including durable store fields and trace lines.
        #[must_use]
        pub fn outside($world: &World) -> Outside { Outside { $($field: Clone::clone($value)),+ } }
        /// Serialize borrowed outside fields after the run's last heap check.
        #[must_use]
        pub fn outside_text($world: &World) -> OutsideText {
            OutsideText::new(vec![$(observations::field_text($value)),+])
        }
        impl Outside {
            /// Full field text for a process boundary, after the run's last heap check.
            #[must_use]
            pub fn text(&self) -> OutsideText {
                OutsideText::new(vec![$(observations::field_text(&self.$field)),+])
            }
            /// The field that differs and its first differing `{:#?}` line.
            #[must_use]
            pub fn first_difference(&self, other: &Self) -> Option<String> {
                $(if self.$field != other.$field {
                    let left = format!("{:#?}", self.$field);
                    let right = format!("{:#?}", other.$field);
                    let mut left = left.lines().map(Some).chain(std::iter::repeat(None));
                    let difference = right.lines().map(Some).chain([None]).zip(&mut left).find(|(b, a)| a != b);
                    return Some(match difference {
                        Some((b, a)) => format!("{}: {a:?} != {b:?}", stringify!($field)),
                        None => format!("{}: values differ with identical Debug lines", stringify!($field)),
                    });
                })+
                None
            }
        }
        pub(super) const OUTSIDE_FIELDS: &[&str] = &[$(stringify!($field)),+];
    };
}
outside_state!(world;
    systems: [jig_test_system::System; 2] = &world.systems,
    peers: crate::Neighbours = &world.peers,
    applied: u64 = &world.store.applied,
    rows: std::collections::BTreeMap<jig_test_domain::Key, jig_test_domain::Record> = &world.store.rows,
    pending: std::collections::VecDeque<jig_test_store_files::Pending> = &world.store.pending,
    held: std::collections::VecDeque<u64> = &world.store.held,
    referee: jig_conformance::referee::Referee = &world.referee,
    clock: u64 = &world.clock().now,
    // Coverage belongs to the run: a sweep's preceding children checked more
    // commits than a sampled snapshot prefix, without changing application state.
    outcome: Outcome = &Outcome { checked: 0, ..world.outcome() },
    trace: Vec<String> = &world.trace.to_vec(),
);
