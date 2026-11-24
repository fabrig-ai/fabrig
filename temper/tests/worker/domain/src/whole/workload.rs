//! Concurrent worker stories (`temper/docs/design/testing/workloads.md`, 4.4).
//! Worker-wide endings are a cohort's story; parks replace only their struck run's ending.
use skein_lib::Token;
use skein_world::{
    faults::{FaultLedger, Kind, Struck},
    streams::{Stream, Streams},
};
use temper_faults::parks::{DELIVERED_THEN_PARKED, RESUMED_ON_ANOTHER_WORKER};
use temper_worker_domain::{Event, agent, host, wire};

use super::{Case, protocol, translate};

mod world;
use world::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub story: Case,
    pub start: u64,
    pub checks: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workload {
    pub runs: Vec<Run>,
}

/// All workload choices belong to Workload/worker, independently of the park draw.
#[must_use]
pub fn draw(streams: &Streams) -> Workload {
    let mut rng = streams.rng(Stream::Workload, "worker");
    let cases = [
        Case::Delivered,
        Case::CancelledDelivery,
        Case::ContactWithin,
        Case::ContactPast,
        Case::Drained,
        Case::DrainExpired,
        Case::LinkDrained,
        Case::LinkDrainExpired,
    ];
    let first = cases[usize::try_from(rng.next_u64() % 8).expect("eight stories")];
    let count = 1 + rng.below(3);
    let mut runs = Vec::new();
    for index in 0..count {
        let story = if index == 0 || global(first) {
            first
        } else {
            cases[usize::try_from(rng.below(3)).expect("three run-scoped stories")]
        };
        runs.push(Run { story, start: rng.below(4), checks: 2 + rng.below(3) });
    }
    Workload { runs }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Park {
    pub run: usize,
    /// Quiet steps after the run's check ends; the other runs keep advancing.
    pub step: u64,
    pub kind: Kind,
}

/// A kind, target and step, all from Faults/worker. No story is ineligible.
#[must_use]
pub fn draw_park(streams: &Streams, workload: &Workload) -> Park {
    let mut rng = streams.rng(Stream::Faults, "worker");
    let kind = if rng.below(2) == 0 { DELIVERED_THEN_PARKED } else { RESUMED_ON_ANOTHER_WORKER };
    Park {
        run: usize::try_from(rng.below(u64::try_from(workload.runs.len()).expect("three runs"))).expect("run index"),
        step: 1 + rng.below(3),
        kind,
    }
}

fn global(story: Case) -> bool {
    matches!(story, Case::ContactPast | Case::Drained | Case::DrainExpired | Case::LinkDrained | Case::LinkDrainExpired)
}

/// Run one seed, checking every quiet point and each run's complete remote tree and ending.
pub fn run(seed: u64, parks: bool) -> Result<FaultLedger, String> {
    let streams = Streams::new(seed);
    let workload = draw(&streams);
    let park = parks.then(|| draw_park(&streams, &workload));
    let mut world = World::new(seed, &workload);
    world.event(0, Event::Ready)?;
    let mut started = vec![None; workload.runs.len()];
    let mut checked = vec![None; workload.runs.len()];
    let mut finished = vec![false; workload.runs.len()];
    // A check owns the one build place until its drawn duration passes; the
    // other checks wait FIFO while the hosted runs share this worker's tables.
    for tick in 0..64 {
        world.tick(tick)?;
        for (index, run) in workload.runs.iter().enumerate() {
            if started[index].is_none() && tick >= run.start {
                world.assign(index, false, None)?;
                world.edit(index);
                world.say(
                    index,
                    agent::Up::Long { operation: agent::Operation::Checks, span: skein_lib::Duration::from_secs(60) },
                )?;
                world.ask_check(index)?;
                started[index] = Some(tick);
            }
        }
        for (index, run) in workload.runs.iter().enumerate() {
            let Some(start) = started[index] else { continue };
            if checked[index].is_none() && world.checking(index) && tick >= start + run.checks {
                world.finish_check(index)?;
                checked[index] = Some(tick);
            }
            let Some(ready) = checked[index] else { continue };
            if finished[index] {
                continue;
            }
            if let Some(park) = park.filter(|park| park.run == index) {
                if tick < ready + park.step {
                    continue;
                }
                park_run(&mut world, index, park.kind)?;
                finished[index] = true;
            } else if !global(run.story) {
                world.story(index, run.story)?;
                finished[index] = true;
            }
        }
        if checked.iter().all(Option::is_some)
            && park.is_none_or(|park| finished[park.run])
            && started.iter().all(Option::is_some)
        {
            let pending: Vec<_> = finished.iter().enumerate().filter_map(|(i, done)| (!done).then_some(i)).collect();
            if !pending.is_empty() {
                world.cohort(&pending, workload.runs[pending[0]].story)?;
                for index in pending {
                    finished[index] = true;
                }
            }
        }
        if finished.iter().all(|value| *value) {
            world.finish()?;
            return Ok(world.ledger);
        }
    }
    Err(format!("seed {seed}: workload did not end"))
}

fn park_run(world: &mut World, index: usize, kind: Kind) -> Result<(), String> {
    let run = world.runs[index].run;
    let attempt = world.runs[index].attempt;
    // Both parks first leave a durable branch; a delivery is checked, a save isn't.
    world.call(
        index,
        if kind == DELIVERED_THEN_PARKED { protocol::delivery(b"Delivery", b"Worker story") } else { agent::Ask::Save },
    )?;
    let head = world.head(index, false);
    assert_ne!(head, world.first, "the first attempt pushed its work");
    world.tree(index, head);
    world.say(
        index,
        agent::Up::Answer {
            answer: agent::Answer { read: None, turns: 0, spent: 0, result: agent::RunResult::Parked },
        },
    )?;
    world.gone(index)?;
    let (answer, work, preparation) = world.answer(index);
    assert!(matches!(answer.ending, host::Ending::Parked { .. }));
    assert_eq!(preparation, None);
    assert_eq!(
        work.landed.as_ref(),
        [wire::Landed { tag: 1, commit: translate::commit(head).raw(), checked: kind == DELIVERED_THEN_PARKED }]
    );
    let entry = world.ledger.inject(
        kind,
        world.now(),
        Struck { request: Some(format!("run {} attempt {}", run.raw(), attempt.raw())), ..Struck::default() },
    );
    world.ack(index)?;
    if kind == RESUMED_ON_ANOTHER_WORKER {
        world.move_worker(index)?;
    }
    world.runs[index].attempt = Token::new(attempt.raw() + 1);
    world.assign(index, true, (kind == DELIVERED_THEN_PARKED).then_some(head))?;
    world.checkout_tree(index);
    if kind == DELIVERED_THEN_PARKED {
        world.finish_change(index)?;
    } else {
        // Continue the fetched work, retaining the first attempt's entire edit.
        // A clean unchecked save is not itself a new checked delivery.
        world.runs[index].edit.extend_from_slice(b"continued on another worker\n");
        world.edit(index);
        world.call(index, protocol::delivery(b"Delivery", b"Worker story"))?;
        world.finish_report(index)?;
    }
    world.gone(index)?;
    let (answer, work, preparation) = world.answer(index);
    assert!(matches!(answer.ending, host::Ending::Ended { .. }));
    assert_eq!(preparation, None);
    let final_head = world.head(index, false);
    if kind == DELIVERED_THEN_PARKED {
        assert_eq!(final_head, head, "finishing a delivered tree creates no new commit");
    }
    assert_eq!(
        work.landed.as_ref(),
        [wire::Landed { tag: 1, commit: translate::commit(final_head).raw(), checked: true }]
    );
    world.tree(index, final_head);
    world.ack(index)?;
    world.ledger.stop(entry, world.now())?;
    Ok(())
}
