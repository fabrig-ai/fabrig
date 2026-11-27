//! Stale views and legitimate outside writes (draft/failure-worlds.md, 4.5).
use super::opened::Moment;
use skein_fake_forge_domain::{self as forge, api as raw};
use skein_world::{
    faults::Struck,
    streams::{Stream, Streams},
    sweep::{Sweep, cases, gate},
};
use std::convert::Infallible;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{World, changes, fake_config, landings::LANDING, translate};
use temper_faults::{
    forge::{CLOSED_BY_ANOTHER as CLOSED, MERGE_LATE, MERGED_BY_ANOTHER as MERGED},
    stale::{MOVED_BASE, REOPENED, SQUASHED, STALE},
};

fn candidate(head: u64) -> Struck {
    Struck {
        candidate: Some(String::from_utf8(top::commit_text(&translate::commit(head)).into()).expect("hex")),
        ..Struck::default()
    }
}

fn step(world: &mut World, task: u64, entry: u64, released: bool) -> change::Decision {
    changes::change_step_task(world, task, entry, change::Status::Unknown, Box::new([]), released, false)
}

fn reopened(world: &mut World) {
    let source = b"other/source";
    world.external_now(
        raw::Op::Git(raw::Git::Create { branch: source.as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_now_as(source, b"other", b"cancelled", 2);
    let source_head = world.branch(source);
    changes::open_change(world, 91, source, None);
    assert!(matches!(step(world, 91, 91_001, false), change::Decision::Effect(change::Effect::CreateBranch { .. })));
    assert_eq!(step(world, 91, 91_002, false), change::Decision::Effect(change::Effect::Open));
    let pull = world.change(91).pull.expect("opened sacrificial change");
    world.after_write(
        raw::Op::Write(raw::Write::Close { number: pull }),
        REOPENED,
        raw::Op::Write(raw::Write::Reopen { number: pull }),
        2,
    );
    super::opened::release(world, 91, top::ReleaseEnding::Cancelled);
    assert_eq!(world.branch(source), source_head, "cancellation and outside reopen preserve the source");
}

fn dependency(world: &mut World) {
    let source = b"docs/x";
    let dependency = b"other/dependency";
    let original = world.branch(source);
    world.external_now(
        raw::Op::Git(raw::Git::Create { branch: dependency.as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_now_as(dependency, b"dependency", b"prior change", 2);
    let parent = world.branch(dependency);
    let raw::Answer::Tree(files) = world.inspect(&raw::Read::Tree { commit: original }) else { panic!("source tree") };
    let mut files = files.into_vec();
    files.push(raw::File { path: b"dependency".as_slice().into(), content: b"prior change".as_slice().into() });
    let mut config = fake_config();
    config.limits.files = 8;
    // The early source checkout has the dependency as its parent and retains
    // its own first revision as the other parent of this legitimate merge.
    let early = forge::commit_tree(world.forge_mut(), &config, parent, Some(original), files.into(), b"early checkout")
        .expect("early checkout");
    assert_eq!(
        world.external_now(
            raw::Op::Git(raw::Git::Push { branch: source.as_slice().into(), commit: early, expected: Some(original) }),
            2
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let pull = world.open_pull(dependency, b"main");
    let injection = world.inject(SQUASHED, candidate(early));
    let raw::Answer::Merged(squash) = world
        .external_now(raw::Op::Write(raw::Write::Merge { number: pull, head: parent, how: raw::MergeHow::Style }), 2)
    else {
        panic!("dependency squash")
    };
    world.stop(injection);
    assert!(!world.forge_mut().is_ancestor(parent, squash), "the squashed parent is outside main's ancestry");
    world.external_now(raw::Op::Write(raw::Write::DeleteBranch { branch: dependency.as_slice().into() }), 2);
    // A foreign pull following this source makes its later source pushes
    // exercise head lag; the engine must still never write that source.
    world.open_pull(source, b"main");
    world.push_now_as(source, b"member0", b"first", 2);
}

fn close_and_release(world: &mut World, task: u64) {
    let pull = world.change(task).pull.expect("ready opened change");
    let injection = world.inject(CLOSED, Struck { request: Some(pull.to_string()), ..Struck::default() });
    assert_eq!(world.external_now(raw::Op::Write(raw::Write::Close { number: pull }), 2), raw::Answer::Done);
    world.stop(injection);
    assert_eq!(step(world, task, 80_001, false), change::Decision::Hold(change::Cause::PullClosed));
    world.event(top::Event::ReleaseChange { task, release: None });
    assert_eq!(step(world, task, 80_002, true), change::Decision::Effect(change::Effect::Reopen));
    assert_eq!(step(world, task, 80_003, false), change::Decision::Ready);
}

#[test]
fn stale_state_keeps_the_opens_invariants() {
    let test = skein_world::this_test!("stale_state_keeps_the_opens_invariants");
    let at_gate = gate();
    let mut sweep = Sweep::new("stale_state_keeps_the_opens_invariants");
    for kind in STALE {
        sweep.configure(kind);
    }
    super::lagging::exclude_records(&mut sweep);
    let mut own_late_merges = 0;
    let mut skipped = 0;
    cases(test, 1..=8, |seed| {
        let streams = Streams::new(seed);
        let mut world =
            World::batch_turn_in(&streams, Some(super::super::landing_batches::workflow(false)), fake_config());
        super::lagging::lag_world(&mut world, &sweep);
        let (world, replay, reordered) = run(world, streams);
        if at_gate && reordered {
            own_late_merges += 1;
        }
        skipped += replay.skipped_pushes;
        sweep.ran(world.ledger());
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    super::opened::check_skipped_pushes(skipped);
    if at_gate {
        assert!(own_late_merges > 0, "an engine merge's record arrived after a later visible write");
        eprintln!("{own_late_merges} seeds observed the engine's own merge behind a later write");
        eprintln!("{}: {} draws left out", super::lagging::LATE_RECORD, sweep.left_out(super::lagging::LATE_RECORD));
    }
    sweep.check_fell().expect("configured stale sequences fell");
}

fn run(world: World, streams: Streams) -> (World, super::opened::Replay, bool) {
    let reorder = streams.seed() % 2 != 0;
    let mut merged = false;
    let mut reordered = false;
    // Sourced spends five commits (sacrificial change, dependency, early
    // checkout, squash, source update); Kept moves main; odd seeds can reorder.
    let outside_commits = 6 + u32::from(reorder);
    let (world, replay) = super::opened::run_with(
        world,
        streams.seed_of(Stream::Workload, "opened"),
        240,
        outside_commits,
        |world, moment| {
            match moment {
                Moment::Sourced(51) => {
                    reopened(world);
                    dependency(world);
                }
                Moment::Kept(51) => {
                    // Taking writes the kept head, not an update/resolution/save.
                    // This is the nearest moved-base race that this workload writes.
                    let injection = world.inject(MOVED_BASE, candidate(world.branch(b"docs/x")));
                    world.push_now_as(b"main", b"base_moved", b"after kept read, before make", 2);
                    world.stop(injection);
                }
                Moment::Ready(51) => close_and_release(world, 51),
                Moment::Landing if reorder && !reordered => {
                    let row = &world.landing(LANDING).landing;
                    if let Some(number) = row.round.pull {
                        let current = candidate(world.branch(b"main")).candidate;
                        let own_write_lagging = world.ledger().entries().iter().any(|entry| {
                            entry.kind == MERGE_LATE
                                && entry.struck.candidate == current
                                && entry.until.is_some_and(|until| world.now() < until)
                        });
                        if own_write_lagging {
                            let raw::Answer::Pull(before) = world.inspect(&raw::Read::Pull { number }) else {
                                panic!("round pull")
                            };
                            assert_eq!(
                                before.state,
                                raw::State::Open,
                                "the engine's own merge is not yet in the pull view"
                            );
                            let earlier = world.branch(b"docs/x");
                            world.push_now_as(b"docs/x", b"member0", b"later than the engine merge", 2);
                            assert_ne!(world.branch(b"docs/x"), earlier, "a later write is already visible");
                            let raw::Answer::Pull(after) = world.inspect(&raw::Read::Pull { number }) else {
                                panic!("round pull")
                            };
                            assert_eq!(after.merged, None, "the earlier merge record still lags the later write");
                            reordered = true;
                        }
                    }
                }
                Moment::Landing if !reorder && !merged => {
                    let row = &world.landing(LANDING).landing;
                    if matches!(row.phase, landing::Phase::Turn { .. }) {
                        let number = row.round.pull.expect("proved round pull");
                        let head = world.branch(&row.members.last().expect("member").branch);
                        let injection =
                            world.inject(MERGED, Struck { request: Some(number.to_string()), ..Struck::default() });
                        assert_eq!(
                            world.external_now(
                                raw::Op::Write(raw::Write::Merge { number, head, how: raw::MergeHow::FastForwardOnly }),
                                2
                            ),
                            raw::Answer::Merged(head)
                        );
                        world.stop(injection);
                        merged = true;
                    }
                }
                Moment::Sourced(_) | Moment::Kept(_) | Moment::Ready(_) | Moment::Landing => {}
            }
        },
    );
    (world, replay, reordered)
}
