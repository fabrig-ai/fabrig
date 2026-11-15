//! Independent branch trains against the fake forge (temper/docs/design/domain/landing.md, 9, 15, 16).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, Trains},
    translate,
};

fn branch(world: &mut World, name: &[u8]) {
    assert_eq!(
        world.external(raw::Op::Git(raw::Git::Create { branch: name.into(), commit: world.branch(b"main") }), 1),
        raw::Answer::Branch(raw::Created::Created)
    );
}

#[test]
fn drill_17_an_outside_push_during_proof_restacks_and_reproves_the_same_member() {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.latency_min = skein_lib::Duration::from_secs(60);
    workflow.latency_max = workflow.latency_min;
    let mut world = landings::turn_in(World::batch_turn(17, Some(workflow)));
    assert!(world.project_landing(
        landings::PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: temper_engine_forge_world::REPO,
            tiers: Box::new([]),
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: None,
            }]),
        }])
    ));
    ready(&mut world, 51, b"main");
    let since = world.change(51).change.ready_since;
    let mut trains = Trains::default();
    trains.judge(&mut world);
    let landing = task(&world, 51);
    let mut dispatched = false;
    for entry in 100..160 {
        if matches!(landings::step_landing(&mut world, landing, entry), landing::Decision::Dispatch { .. }) {
            dispatched = true;
            break;
        }
    }
    assert!(dispatched);
    let old = world.landing(landing).landing.members[0].candidate.expect("first candidate");
    let raw::Answer::Runs(runs) =
        world.inspect(&raw::Read::Runs { workflow: None, commit: None, number: None, page: 1, limit: 8 })
    else {
        panic!("runs")
    };
    assert_eq!(runs.len(), 1);
    assert_ne!(runs[0].state, raw::RunState::Completed);
    ready(&mut world, 52, b"main");
    world.push_file(b"main", b"docs/note", b"hand delivered documentation");
    let outside = world.branch(b"main");
    trains.judge(&mut world);
    assert_eq!(world.change(52).change.landing, None);
    landings::drive_landing(&mut world, landing);
    let row = world.landing(landing);
    let member = &row.landing.members[0];
    assert_eq!(row.landing.phase, landing::Phase::Landed);
    assert_eq!(row.landing.preparation, 2);
    assert_eq!(row.landing.stale_proofs, 1);
    assert_ne!(member.candidate, Some(old));
    assert_eq!(member.base, Some(translate::commit(outside)));
    assert_eq!(world.change(51).change.ready_since, since);
    assert_eq!(translate::commit(world.branch(b"main")), member.landed.expect("landed candidate"));
    assert_eq!(world.change(52).change.landing, None);
    let raw::Answer::Commits { commits } =
        world.inspect(&raw::Read::Commits { before: outside, after: world.branch(b"main") })
    else {
        panic!("new main")
    };
    assert_eq!(commits.len(), 2); // The delivered member and its merge onto the outside push.
    let candidate = commits.iter().find(|commit| commit.id == world.branch(b"main")).expect("candidate");
    assert_eq!(candidate.parents[0], outside);
    let calls = world.take_calls();
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 2);
    assert_eq!(
        calls
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. })))
            .count(),
        1
    );
    let receipts =
        world
            .stored()
            .iter()
            .filter_map(|(_, record)| {
                if let top::Stored::Receipt { key, receipt } = record { Some((key.key, receipt)) } else { None }
            })
            .collect::<Vec<_>>();
    assert_eq!(receipts.len(), 2);
    assert!(receipts.iter().all(|(_, receipt)| receipt.passed));
    assert_ne!(receipts[0].0, receipts[1].0);
}

fn ready(world: &mut World, task: u64, destination: &[u8]) {
    let name = format!("temper/c{task}");
    world.produce_file(name.as_bytes(), name.as_bytes(), b"change");
    landings::ready_change_into(world, task, name.as_bytes(), destination);
}

fn until_turn(world: &mut World, task: u64) {
    for entry in task * 100..task * 100 + 80 {
        let decision = landings::step_landing_raw(world, task, entry);
        if matches!(decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(task, world.landing(task).landing.branch.as_ref() != b"main");
        }
        if matches!(world.landing(task).landing.phase, landing::Phase::Turn { .. }) {
            return;
        }
    }
    panic!("train did not reach its turn: {:?}", world.landing(task));
}

fn task(world: &World, change: u64) -> u64 {
    world.change(change).change.landing.expect("train formed")
}

fn search_example(count: u64, breaking: u64, expected_probes: &[u16]) {
    let mut world = landings::turn_in(World::search_turn(18, super::landing_batches::workflow(true)));
    world.batch(u16::try_from(count).expect("small batch"));
    world.push_file(b"main", b"file", b"fixed");
    let base = world.branch(b"main");
    for index in 1..=count {
        let change = 50 + index;
        let name = format!("temper/c{change}");
        branch(&mut world, name.as_bytes());
        world.push_file(name.as_bytes(), format!("safe{index}").as_bytes(), b"safe");
        if index == breaking {
            world.push_file(name.as_bytes(), b"file", b"broken");
        }
        landings::ready_change(&mut world, change, name.as_bytes());
    }
    Trains::default().judge(&mut world);
    let landing = task(&world, 51);
    let mut probes = Vec::new();
    let mut reached_second = false;
    let mut prefix_turns = 0;
    for entry in 1000..1200 {
        let decision = landings::step_landing(&mut world, landing, entry);
        for change in 51..51 + count {
            if !matches!(world.change(change).change.state, change::State::Repairing { requested: true, .. }) {
                changes::change_step_task(
                    &mut world,
                    change,
                    entry * 10 + change,
                    change::Status::Passed,
                    Box::new([]),
                    false,
                    false,
                );
            }
        }
        let row = &world.landing(landing).landing;
        if row.preparation == 2 {
            let landed = row.members.iter().filter(|member| member.landed.is_some()).count();
            assert_eq!(landed, usize::try_from(breaking - 1).expect("green prefix size"));
            let blamed = &row.members[usize::try_from(breaking - 1).expect("blamed index")];
            assert!(matches!(blamed.left, Some(landing::Leave::Blamed { .. })));
            assert_eq!(world.change(blamed.change).change.repairs, 1);
            assert!(
                row.members
                    .iter()
                    .skip(usize::try_from(breaking).expect("suffix"))
                    .all(|member| member.landed.is_none())
            );
            assert_eq!(prefix_turns, u32::from(breaking > 1));
            assert_eq!(probes, expected_probes);
            if breaking == 1 {
                assert_eq!(world.branch(b"main"), base);
            }
            reached_second = true;
            break;
        }
        if let landing::Decision::Dispatch { member, .. } = decision {
            if row.round.search.is_some() {
                probes.push(member + 1);
            }
            let raw::Answer::Runs(runs) =
                world.inspect(&raw::Read::Runs { workflow: None, commit: None, number: None, page: 1, limit: 8 })
            else {
                panic!("runs")
            };
            assert!(
                runs.iter().filter(|run| run.state != raw::RunState::Completed).count() <= 1,
                "one proof or probe at a time"
            );
        }
        if matches!(decision, landing::Decision::Merge { .. }) {
            prefix_turns += 1;
            assert_eq!(probes, expected_probes, "green prefix lands on the probe receipts");
            let calls = world.take_calls();
            assert_eq!(
                calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(),
                expected_probes.len() + 1
            );
        }
    }
    assert!(reached_second, "first round ends");
    let maximum = u64::BITS - (count - 1).leading_zeros();
    assert!(probes.len() <= usize::try_from(maximum).expect("logarithmic bound"));
    landings::drive_landing(&mut world, landing);
    assert_eq!(world.landing(landing).landing.phase, landing::Phase::Landed);
    assert_eq!(world.landing(landing).landing.preparation, 2);
    for member in &world.landing(landing).landing.members {
        if member.change == 50 + breaking {
            assert!(member.landed.is_none());
            assert!(matches!(world.change(member.change).change.state, change::State::Repairing { .. }));
        } else {
            assert!(member.landed.is_some());
        }
    }
}

#[test]
fn drill_18_five_units_with_the_third_breaking_probe_two_then_three() {
    search_example(5, 3, &[2, 3]);
}

#[test]
fn drill_18_three_units_with_the_first_breaking_land_nothing_in_round_one() {
    search_example(3, 1, &[1]);
}

fn workflow_runs(world: &World) -> Box<[raw::Run]> {
    let mut all = Vec::new();
    // Four retained runs fit two pages in the small fake.
    for page in 1..=2 {
        let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
            workflow: Some(b"gate.yml".as_slice().into()),
            commit: None,
            number: None,
            page,
            limit: 8,
        }) else {
            panic!("workflow runs")
        };
        all.extend(runs);
    }
    all.into_boxed_slice()
}

fn report(world: &World, run: u64) -> Box<[u8]> {
    let raw::Answer::Artifact { artifact, .. } =
        world.inspect(&raw::Read::Artifact { run, name: b"report".as_slice().into() })
    else {
        panic!("run report")
    };
    artifact.bytes
}

#[test]
fn drill_19_named_flake_and_inherited_reports_are_still_a_failed_landing_gate() {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.fails = Box::new([b"case.inherited".as_slice().into()]);
    workflow.flaky = Box::new([b"case.flake".as_slice().into()]);
    workflow.select = Some(b"temper_tests".as_slice().into());
    workflow.report =
        Some(raw::Report { artifact: b"report".as_slice().into(), format: raw::ReportFormat::Lines, deflate: false });
    let mut world = landings::turn_in(World::search_turn(19, workflow));
    world.external(
        raw::Op::Write(raw::Write::Dispatch {
            workflow: b"gate.yml".as_slice().into(),
            branch: b"main".as_slice().into(),
            inputs: Box::new([(b"temper_tests".as_slice().into(), b"case.inherited".as_slice().into())]),
        }),
        1,
    );
    world.run_for(10);
    let base_run = workflow_runs(&world)[0].id;
    assert_eq!(report(&world, base_run).as_ref(), b"case.inherited\n");
    ready(&mut world, 51, b"main");
    Trains::default().judge(&mut world);
    let landing = task(&world, 51);
    let main = world.branch(b"main");
    landings::drive_landing(&mut world, landing);
    let member = &world.landing(landing).landing.members[0];
    let Some(landing::Leave::Failed { run, .. }) = member.left else { panic!("failed gate") };
    let candidate =
        translate::commit(workflow_runs(&world).iter().find(|known| known.id == run).expect("candidate run").commit);
    assert_eq!(report(&world, run).as_ref(), b"case.flake\ncase.inherited\n");
    assert_eq!(member.landed, None);
    assert_eq!(world.branch(b"main"), main);
    assert!(matches!(world.change(51).change.state, change::State::Repairing { .. }));
    let calls = world.take_calls();
    assert!(!calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::Artifact { .. }))));
    // The runner can prove the named flake passes alone at the same candidate;
    // landing gates classify workflow outcomes, without reading report artifacts
    // (temper/docs/design/domain/landing.md, 7.3).
    world.external(
        raw::Op::Git(raw::Git::Create {
            branch: b"temper/alone".as_slice().into(),
            commit: skein_fake_forge_domain::commit_count(&candidate).expect("candidate"),
        }),
        1,
    );
    world.external(
        raw::Op::Write(raw::Write::Dispatch {
            workflow: b"gate.yml".as_slice().into(),
            branch: b"temper/alone".as_slice().into(),
            inputs: Box::new([(b"temper_tests".as_slice().into(), b"case.flake".as_slice().into())]),
        }),
        1,
    );
    world.run_for(10);
    let runs = workflow_runs(&world);
    assert_eq!(runs.len(), 3);
    let alone = runs
        .iter()
        .find(|run| run.inputs.iter().any(|(_, tests)| tests.as_ref() == b"case.flake"))
        .expect("selected flake");
    assert_eq!(alone.commit, skein_fake_forge_domain::commit_count(&candidate).expect("candidate"));
    assert_eq!(alone.conclusion, Some(raw::Conclusion::Success));
    assert!(report(&world, alone.id).is_empty());
}

#[test]
fn drill_13_two_goal_trains_and_main_progress_independently() {
    let mut world = landings::turn_in(World::parallel_trains());
    world.batch(4);
    branch(&mut world, b"temper/g1");
    branch(&mut world, b"temper/g2");
    for (change, base) in
        [(51, b"temper/g1".as_slice()), (52, b"temper/g1"), (53, b"temper/g2"), (54, b"temper/g2"), (55, b"main")]
    {
        ready(&mut world, change, base);
    }
    let mut trains = Trains::default();
    trains.judge(&mut world);
    let (first, second, main) = (task(&world, 51), task(&world, 53), task(&world, 55));
    assert_ne!(first, second);
    assert_ne!(first, main);
    assert_ne!(second, main);
    assert_eq!(world.change(52).change.landing, None);
    assert_eq!(world.change(54).change.landing, None);
    for (landing, change) in [(first, 51), (second, 53), (main, 55)] {
        assert_eq!(world.landing(landing).landing.members.iter().map(|m| m.change).collect::<Vec<_>>(), [change]);
        until_turn(&mut world, landing);
    }
    // Main takes its turn while both goal trains still own theirs.
    trains.land(&mut world, main);
    landings::close_landing(&mut world, main);
    assert_eq!(world.landing(main).landing.phase, landing::Phase::Landed);
    assert!(matches!(world.landing(first).landing.phase, landing::Phase::Turn { .. }));
    assert!(matches!(world.landing(second).landing.phase, landing::Phase::Turn { .. }));
    trains.land(&mut world, second);
    landings::close_landing(&mut world, second);
    let next_second = task(&world, 54);
    assert_ne!(next_second, second);
    assert_eq!(world.change(52).change.landing, None);
    until_turn(&mut world, next_second);
    trains.land(&mut world, next_second);
    landings::close_landing(&mut world, next_second);
    trains.land(&mut world, first);
    landings::close_landing(&mut world, first);
    let next_first = task(&world, 52);
    until_turn(&mut world, next_first);
    trains.land(&mut world, next_first);
    for change in 51..=55 {
        assert!(matches!(world.change(change).change.state, change::State::Landed { .. }));
    }
    assert!(trains.held.is_empty(), "no train held: {:?}", trains.held);
    let calls = world.take_calls();
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1);
    assert_eq!(
        calls
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. })))
            .count(),
        5
    );
    assert_ne!(translate::commit(world.branch(b"main")), world.change(51).change.last_head.expect("goal head"));
}
