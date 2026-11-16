//! Selected landing probes and their durable outcomes (domain/landing-search-by-test.md, 3–6, 9).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_domain_forge_tiers::Failures;
use temper_engine_forge_world::{
    REPO, World,
    landings::{self, LANDING, PROJECT, Trains},
    translate,
};

fn workflow() -> raw::Workflow {
    raw::Workflow {
        select: Some(b"temper_tests".as_slice().into()),
        report: Some(raw::Report {
            artifact: b"failed".as_slice().into(),
            format: raw::ReportFormat::Lines,
            deflate: false,
        }),
        cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
        ..super::landing_batches::workflow(false)
    }
}
fn batch(count: u64, breaking: u64, report: bool) -> World {
    let contents: Vec<_> =
        (1..=count).map(|index| if index == breaking { Some(b"a\n".as_slice()) } else { None }).collect();
    configured_batch(&contents, report, workflow(), false)
}
fn configured_batch(contents: &[Option<&[u8]>], report: bool, workflow: raw::Workflow, second: bool) -> World {
    let mut workflows = vec![workflow];
    if second {
        workflows
            .push(raw::Workflow { file: b"green.yml".as_slice().into(), ..super::landing_batches::workflow(false) });
    }
    let mut world = landings::turn_in(World::search_workflows(31, workflows.into()));
    let mut gates = vec![gate(report)];
    if second {
        gates.insert(
            0,
            landing::Gate { name: b"green".as_slice().into(), workflow: b"green.yml".as_slice().into(), ..gate(false) },
        );
    }
    assert!(world.project_landing(
        PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates { repository: REPO, tiers: Box::new([]), gates: gates.into() }])
    ));
    world.batch(u16::try_from(contents.len()).expect("fixture value"));
    let mut previous = world.branch(b"main");
    for (index, content) in contents.iter().enumerate() {
        let task = 51 + u64::try_from(index).expect("fixture value");
        let branch = format!("temper/c{task}");
        world.external(
            raw::Op::Git(raw::Git::Create {
                branch: branch.as_bytes().into(),
                commit: if content.is_some() { previous } else { world.branch(b"main") },
            }),
            2,
        );
        if let Some(content) = content {
            world.push_file(branch.as_bytes(), b"failures", content);
            previous = world.branch(branch.as_bytes());
        } else {
            world.push_file(branch.as_bytes(), format!("safe{index}").as_bytes(), b"safe");
        }
        landings::ready_change(&mut world, task, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    world
}
fn gate(report: bool) -> landing::Gate {
    landing::Gate {
        name: b"gate".as_slice().into(),
        workflow: b"gate.yml".as_slice().into(),
        ignored: Box::new([]),
        blocking: true,
        base: false,
        report: report.then(|| landing::GateReport {
            artifact: b"failed".as_slice().into(),
            format: landing::Format::Lines,
            fuzzy: None,
        }),
    }
}
fn until_dispatch(world: &mut World, member: u16, start: u64) {
    for entry in start..start + 600 {
        let decision = landings::step_landing(world, LANDING, entry);
        if matches!(decision, landing::Decision::Dispatch { member: m, .. } if m == member) {
            return;
        }
    }
    let row = world.landing(LANDING);
    panic!("no dispatch at {member}: phase {:?}, search {:?}", row.landing.phase, row.landing.round.search);
}
type Dispatch = (Box<[u8]>, Box<[api::Input]>);
fn dispatches(calls: &[api::Op]) -> Vec<Dispatch> {
    calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Dispatch { branch, inputs, .. }) => Some((branch.clone(), inputs.clone())),
            _ => None,
        })
        .collect()
}
fn value<'a>(inputs: &'a [api::Input], name: &[u8]) -> Option<&'a [u8]> {
    inputs.iter().find(|i| i.name.as_ref() == name).map(|i| i.value.as_ref())
}
fn receipt(world: &World, commit: api::Commit) -> Option<&top::Receipt> {
    world.stored().iter().find_map(|(_, row)| match row {
        top::Stored::Receipt { receipt, .. } if receipt.candidate == commit => Some(receipt),
        _ => None,
    })
}

#[test]
fn four_units_probe_selected_boundaries_and_land_only_the_next_whole_top() {
    let mut world = batch(4, 3, true);
    until_dispatch(&mut world, 1, 1000);
    let initial = world.landing(LANDING).clone();
    let prefixes: Vec<_> = initial.landing.members.iter().map(|m| m.candidate.expect("fixture value")).collect();
    let pull = initial.landing.round.pull.expect("fixture value");
    let search = initial.landing.round.search.as_ref().expect("fixture value");
    assert_eq!((search.green, search.red), (0, 4));
    assert!(
        matches!(&search.tests, landing::SearchTests::ByTest { gates, .. } if gates[0].as_deref() == Some(&[Box::from(b"a".as_slice())]))
    );
    until_dispatch(&mut world, 2, 2000);
    assert!(receipt(&world, prefixes[1]).is_none(), "a selected pass is no whole receipt");
    let search = world.landing(LANDING).landing.round.search.as_ref().expect("fixture value");
    assert_eq!((search.green, search.red), (2, 4));
    assert!(
        matches!(&search.tests, landing::SearchTests::ByTest { probed, .. } if probed.iter().any(|p| p.member == 2 && p.gate == 0 && p.passed && p.run > 0))
    );
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING);
    assert_eq!(row.landing.preparation, 2);
    assert!(
        matches!(row.landing.members[2].left, Some(landing::Leave::Blamed { green, red, .. }) if green == prefixes[1] && red == prefixes[2])
    );
    assert_eq!(row.landing.members[0].landed, Some(prefixes[0]));
    assert_eq!(row.landing.members[1].landed, Some(prefixes[1]));
    let next_top = row.landing.members[3].landed.expect("fixture value");
    assert_ne!(next_top, prefixes[3]);
    assert_eq!(translate::commit(world.branch(b"main")), next_top);
    let red = receipt(&world, prefixes[2]).expect("selected red's failed receipt");
    assert!(!red.passed);
    assert_eq!(red.failures, Some(Failures::Tests(Box::new([b"a".as_slice().into()]))));
    let raw::Answer::Pull(closed) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("round pull") };
    assert_eq!((closed.state, closed.merged), (raw::State::Closed, None));
    assert!(world.has_branch(&initial.landing.members[1].branch));
    assert!(!world.has_branch(&initial.landing.members[2].branch));
    let calls = world.take_calls();
    let dispatches = dispatches(&calls);
    assert_eq!(dispatches.len(), 4);
    for (index, (_, inputs)) in dispatches.iter().enumerate() {
        assert_eq!(
            value(inputs, b"temper_tests"),
            if index == 1 || index == 2 { Some(b"a\n".as_slice()) } else { None }
        );
        assert_eq!(
            value(inputs, b"temper_probe"),
            if index == 1 || index == 2 { Some(b"yes".as_slice()) } else { None }
        );
    }
    let merges: Vec<_> = calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. }) => Some(*head),
            _ => None,
        })
        .collect();
    assert_eq!(merges, vec![next_top], "no turn on the selected green prefix");
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::DeleteBranch { .. }))).count(), 2);
    assert!(
        !calls
            .iter()
            .any(|op| matches!(op, api::Op::Write(api::Write::Status { commit, .. }) if *commit == prefixes[1]))
    );

    // A later landing prepares the same tree under another commit and fails by that exact key.
    let raw::Answer::Tree(files) = world
        .inspect(&raw::Read::Tree { commit: skein_fake_forge_domain::commit_count(&prefixes[2]).expect("red commit") })
    else {
        panic!("red tree")
    };
    world.external(
        raw::Op::Git(raw::Git::Create { branch: b"temper/c55".as_slice().into(), commit: world.branch(b"main") }),
        2,
    );
    world.push_tree(b"temper/c55", files);
    landings::ready_change(&mut world, 55, b"temper/c55");
    let mut later = landings::landing_row(world.change(55));
    later.landing.task = 62;
    later.landing.round = landing::Round::new(b"temper/62/l62".as_slice().into());
    later.landing.members[0].branch = later.landing.round.branch.clone();
    landings::register(&mut world, later);
    world.take_calls();
    landings::drive_landing(&mut world, 62);
    assert!(matches!(world.landing(62).landing.members[0].left, Some(landing::Leave::Failed { gate: 0, .. })));
    assert_eq!(
        self::dispatches(&world.take_calls()),
        [],
        "a later top takes the selected failed receipt without a run"
    );
}

#[test]
fn reportless_twin_keeps_whole_probes_and_the_green_turn() {
    let mut world = batch(4, 3, false);
    landings::drive_landing(&mut world, LANDING);
    let calls = world.take_calls();
    assert!(dispatches(&calls).iter().all(|(_, inputs)| value(inputs, b"temper_tests").is_none()));
    assert_eq!(
        calls
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. })))
            .count(),
        2
    );
}

#[test]
fn every_selected_probe_green_keeps_the_prefix_without_another_prepare_job() {
    let mut world = batch(4, 4, true);
    until_dispatch(&mut world, 1, 1000);
    let prefix = world.landing(LANDING).landing.members[2].candidate.expect("fixture value");
    let mut whole_top = false;
    for entry in 2000..2100 {
        let decision = landings::step_landing_raw(&mut world, LANDING, entry);
        assert!(!matches!(decision, landing::Decision::Prepare { .. }), "kept green prefix needs no job");
        if matches!(decision, landing::Decision::Dispatch { member: 2, .. })
            && world.landing(LANDING).landing.round.search.is_none()
        {
            whole_top = true;
            break;
        }
    }
    assert!(whole_top);
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING);
    assert!(matches!(row.landing.members[3].left, Some(landing::Leave::Blamed { green, .. }) if green == prefix));
    assert_eq!(row.landing.members[2].landed, Some(prefix));
    assert_eq!(row.landing.preparation, 2, "new round reuses the green prefix");
    assert!(receipt(&world, prefix).expect("fixture value").passed, "the next top proved whole");
    let calls = world.take_calls();
    let dispatches = dispatches(&calls);
    assert_eq!(dispatches.len(), 4);
    assert_eq!(value(&dispatches[3].1, b"temper_tests"), None);
    assert_eq!(value(&dispatches[3].1, b"temper_commit"), Some(top::commit_text(&prefix).as_ref()));
}

#[test]
fn a_selected_first_unit_leaves_no_green_turn() {
    let mut world = batch(4, 1, true);
    let base = translate::commit(world.branch(b"main"));
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING);
    assert!(matches!(row.landing.members[0].left, Some(landing::Leave::Blamed { green, .. }) if green == base));
    assert!(row.landing.members[1..].iter().all(|m| m.landed.is_some()));
    let calls = world.take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. })))
            .count(),
        1
    );
    assert!(dispatches(&calls).iter().any(|(_, inputs)| value(inputs, b"temper_tests") == Some(b"a\n")));
}

#[test]
fn a_gate_green_at_the_top_is_never_dispatched_at_a_selected_probe() {
    let mut world = configured_batch(&[None, None, Some(b"a\n"), None], true, workflow(), true);
    landings::drive_landing(&mut world, LANDING);
    let calls = world.take_calls();
    let green: Vec<_> = calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Dispatch { workflow, inputs, .. }) if workflow.as_ref() == b"green.yml" => {
                Some(inputs)
            }
            _ => None,
        })
        .collect();
    assert_eq!(green.len(), 2, "only the two whole tops run the green gate");
    assert!(green.iter().all(|inputs| value(inputs, b"temper_probe").is_none()));
    assert!(matches!(world.landing(LANDING).landing.members[2].left, Some(landing::Leave::Blamed { gate: 1, .. })));
    assert!(dispatches(&calls).iter().any(|(_, inputs)| value(inputs, b"temper_tests") == Some(b"a\n")));
}

#[test]
fn a_failure_outside_the_selection_discards_its_verdict_and_restarts_whole_from_both_bounds() {
    let mut unselected = workflow();
    unselected.select = None;
    let mut world = configured_batch(&[None, Some(b"b\n"), Some(b"a\n"), None], true, unselected, false);
    until_dispatch(&mut world, 1, 1000);
    let prefix = world.landing(LANDING).landing.members[1].candidate.expect("fixture value");
    world.take_calls();
    until_dispatch(&mut world, 1, 2000);
    assert!(receipt(&world, prefix).is_none(), "no receipt for b outside selection a");
    let search = world.landing(LANDING).landing.round.search.as_ref().expect("fixture value");
    assert_eq!((search.green, search.red), (0, 4));
    assert_eq!(search.tests, landing::SearchTests::WholeAgain);
    let calls = world.take_calls();
    let whole = dispatches(&calls);
    assert_eq!(whole.len(), 1);
    assert_eq!(value(&whole[0].1, b"temper_tests"), None);
    assert_eq!(value(&whole[0].1, b"temper_probe"), Some(b"yes".as_slice()));
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    assert!(
        receipt(&world, prefix)
            .is_some_and(|r| !r.passed && r.failures == Some(Failures::Tests(Box::new([b"b".as_slice().into()]))))
    );
}

fn slow_batch(breaking: u64) -> World {
    let mut slow = workflow();
    slow.latency_min = skein_lib::Duration::from_secs(300);
    slow.latency_max = slow.latency_min;
    let contents = if breaking == 1 { [Some(b"a\n".as_slice()), None] } else { [None, Some(b"a\n".as_slice())] };
    configured_batch(&contents, true, slow, false)
}
fn runs(world: &World) -> Box<[raw::Run]> {
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(b"gate.yml".as_slice().into()),
        commit: None,
        number: None,
        page: 1,
        limit: 16,
    }) else {
        panic!("runs")
    };
    runs
}
fn next_round(world: &mut World, start: u64) {
    for entry in start..start + 60 {
        landings::step_landing(world, LANDING, entry);
        if world.landing(LANDING).landing.round.search.is_none() {
            return;
        }
    }
    panic!("round did not end");
}

#[test]
fn a_late_selected_pass_makes_nothing_and_the_next_top_waits_then_dispatches_whole() {
    let mut world = slow_batch(2);
    until_dispatch(&mut world, 0, 1000);
    let prefix = world.landing(LANDING).landing.members[0].candidate.expect("fixture value");
    world.push_file(b"temper/c52", b"moved", b"new head");
    next_round(&mut world, 2000);
    world.restart();
    world.take_calls();
    for entry in 3000..3005 {
        assert!(!matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Dispatch { .. }));
    }
    assert_eq!(runs(&world).len(), 2, "top and still-running selected probe only");
    assert!(receipt(&world, prefix).is_none());
    world.run_for(400);
    until_dispatch(&mut world, 0, 4000);
    assert!(receipt(&world, prefix).is_none(), "late selected pass did not prove the new whole top");
    assert!(world.landing(LANDING).landing.round.search.is_none());
    let calls = world.take_calls();
    let whole = dispatches(&calls);
    assert_eq!(whole.len(), 1);
    assert_eq!(value(&whole[0].1, b"temper_tests"), None);
    assert_eq!(value(&whole[0].1, b"temper_commit"), Some(top::commit_text(&prefix).as_ref()));
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.members[0].landed, Some(prefix));
}

#[test]
fn a_late_selected_red_keeps_its_ids_and_a_later_top_fails_on_its_receipt_without_a_run() {
    let mut world = slow_batch(1);
    until_dispatch(&mut world, 0, 1000);
    assert!(matches!(
        world.landing(LANDING).landing.round.search.as_ref().expect("search").tests,
        landing::SearchTests::ByTest { .. }
    ));
    let prefix = world.landing(LANDING).landing.members[0].candidate.expect("fixture value");
    world.push_file(b"temper/c52", b"moved", b"new head");
    next_round(&mut world, 2000);
    world.restart();
    world.run_for(400);
    world.take_calls();
    let mut failed = false;
    for entry in 3000..3060 {
        if matches!(
            landings::step_landing(&mut world, LANDING, entry),
            landing::Decision::Leave { member: 0, why: landing::Leave::Failed { .. } }
        ) {
            failed = true;
            break;
        }
    }
    assert!(failed, "a later whole top takes the late selected red");
    let red = receipt(&world, prefix).expect("fixture value");
    assert!(!red.passed);
    assert_eq!(red.failures, Some(Failures::Tests(Box::new([b"a".as_slice().into()]))));
    assert_eq!(dispatches(&world.take_calls()), []);
}

#[test]
fn selected_jobs_rerun_and_fresh_dispatch_keep_their_inputs_through_restart() {
    for cancel in [false, true] {
        let mut world = slow_batch(2);
        until_dispatch(&mut world, 0, 1000);
        let first = runs(&world)
            .iter()
            .find(|r| r.inputs.iter().any(|(name, _)| name.as_ref() == b"temper_tests"))
            .expect("fixture value")
            .clone();
        assert_eq!(
            first.inputs.iter().find(|(name, _)| name.as_ref() == b"temper_tests").expect("fixture value").1.as_ref(),
            b"a\n"
        );
        if cancel {
            assert_eq!(world.external(raw::Op::Write(raw::Write::Cancel { run: first.id }), 2), raw::Answer::Done);
        } else {
            world.runner_setup_faults(1000);
        }
        world.run_for(400);
        let mut retried = false;
        for entry in 2000..2060 {
            if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Retry { member: 0, .. })
            {
                retried = true;
                break;
            }
        }
        assert!(retried);
        world.runner_setup_faults(0);
        world.restart();
        landings::drive_landing(&mut world, LANDING);
        let retried = runs(&world)
            .iter()
            .find(|r| {
                r.commit == first.commit
                    && r.state == raw::RunState::Completed
                    && r.conclusion == Some(raw::Conclusion::Success)
                    && r.inputs.iter().any(|(name, _)| name.as_ref() == b"temper_tests")
            })
            .expect("fixture value")
            .clone();
        assert_eq!(retried.inputs, first.inputs);
        assert_eq!(retried.id == first.id, !cancel);
        assert!(
            receipt(&world, translate::commit(first.commit)).is_some_and(|r| r.passed),
            "next top proved whole after the selected pass"
        );
        let selected: Vec<_> = dispatches(&world.take_calls())
            .into_iter()
            .filter(|(_, inputs)| value(inputs, b"temper_tests").is_some())
            .collect();
        assert_eq!(selected.len(), if cancel { 2 } else { 1 });
        assert!(selected.iter().all(|(_, inputs)| value(inputs, b"temper_tests") == Some(b"a\n")));
    }
}

#[test]
fn restarts_keep_selections_passes_and_boundary_cleanup_once() {
    for checkpoint in ["probing", "deleting", "cut"] {
        let mut world = batch(4, 3, true);
        until_dispatch(&mut world, 2, 1000);
        if checkpoint == "probing" {
            let search = world.landing(LANDING).landing.round.search.as_ref().expect("fixture value");
            assert_eq!((search.green, search.red), (2, 4));
            assert!(
                matches!(&search.tests, landing::SearchTests::ByTest { probed, .. } if probed.len() == 1 && probed[0].passed)
            );
        } else {
            for entry in 2000..2060 {
                let decision = landings::step_landing(&mut world, LANDING, entry);
                let row = world.landing(LANDING);
                if (checkpoint == "deleting" && matches!(decision, landing::Decision::Delete { .. }))
                    || (checkpoint == "cut"
                        && row.landing.round.ending.is_none()
                        && row.landing.round.search.is_none()
                        && row.landing.members[2].candidate.is_none())
                {
                    break;
                }
            }
            assert!(world.landing(LANDING).landing.searched.is_some());
        }
        let saved = world.landing(LANDING).clone();
        world.restart();
        assert_eq!(world.landing(LANDING).landing.round.search, saved.landing.round.search);
        assert_eq!(world.landing(LANDING).landing.searched, saved.landing.searched);
        landings::drive_landing(&mut world, LANDING);
        let calls = world.take_calls();
        assert_eq!(dispatches(&calls).len(), 4, "no repeated top or probe");
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::DeleteBranch { .. }))).count(), 2);
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Close { .. }))).count(), 1);
    }
}

#[test]
fn a_stale_selected_round_stacks_every_survivor_again() {
    let mut world = batch(4, 3, true);
    until_dispatch(&mut world, 2, 1000);
    assert!(matches!(
        world.landing(LANDING).landing.round.search.as_ref().expect("search").tests,
        landing::SearchTests::ByTest { .. }
    ));
    let old: Vec<_> =
        world.landing(LANDING).landing.members.iter().map(|m| m.candidate.expect("fixture value")).collect();
    world.push_file(b"main", b"outside", b"base moved");
    let base = translate::commit(world.branch(b"main"));
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING);
    assert_eq!(row.landing.stale_proofs, 1);
    assert_eq!(row.landing.members[0].base, Some(base));
    assert!(
        row.landing
            .members
            .iter()
            .enumerate()
            .filter(|(_, m)| m.left.is_none())
            .all(|(index, m)| m.candidate != Some(old[index]))
    );
    assert!(row.landing.members[3].landed.is_some());
}

#[test]
fn dispatch_room_counts_the_selection_slot_and_each_ids_line_feed() {
    for room in [1, 2] {
        let mut world = batch(4, 3, true);
        until_dispatch(&mut world, 3, 1000);
        let top = world.landing(LANDING).landing.members[3].clone();
        let calls = world.take_calls();
        let (_, mut inputs) = dispatches(&calls).pop().expect("fixture value");
        let mut values = inputs.to_vec();
        values.push(api::Input { name: b"temper_probe".as_slice().into(), value: b"yes".as_slice().into() });
        values.push(api::Input { name: b"temper_tests".as_slice().into(), value: Box::new([]) });
        inputs = values.into();
        let dispatch = api::Op::Write(api::Write::Dispatch {
            workflow: b"gate.yml".as_slice().into(),
            branch: top.branch,
            inputs,
        });
        let bytes = u32::try_from(skein_forge::measure::op(&dispatch).expect("fixture value")).expect("fixture value");
        world.dispatch_room(bytes + room);
        // Observe the search before its dispatch can reach the smaller entrance.
        for entry in 2000..2060 {
            landings::step_landing(&mut world, LANDING, entry);
            if world.landing(LANDING).landing.round.search.is_some() {
                break;
            }
        }
        let search = world.landing(LANDING).landing.round.search.as_ref().expect("fixture value");
        assert_eq!(
            matches!(search.tests, landing::SearchTests::ByTest { .. }),
            room == 2,
            "a plus LF needs exactly two bytes of room"
        );
        if room == 1 {
            assert_eq!(search.tests, landing::SearchTests::Whole);
        }
    }
}

fn key_at(world: &World, candidate: api::Commit) -> top::ReceiptKey {
    let limits = skein_forge_client::Limits { rows: 64, ..temper_engine_forge_world::LIMITS.client };
    let mut tree = Vec::new();
    for page in 1..=8 {
        let op = api::Op::Read(api::Read::Tree { object: candidate, page });
        let raw::Op::Read(read) = translate::op(&op, &limits) else { panic!("tree read") };
        let api::Answer::Tree { entries, more } =
            translate::answer(&op, world.inspect(&read), &limits).expect("tree answer")
        else {
            panic!("tree")
        };
        tree.extend(entries);
        if !more {
            break;
        }
    }
    let entries = tree
        .into_iter()
        .map(|entry| landing::Entry {
            name: entry.name,
            mode: entry.mode,
            object: entry.object,
            kind: match entry.kind {
                api::TreeEntryKind::Blob => landing::EntryKind::Blob,
                api::TreeEntryKind::Tree => landing::EntryKind::Tree,
                api::TreeEntryKind::Commit => landing::EntryKind::Commit,
            },
        })
        .collect();
    let gate = gate(true);
    let landing::Keyed::Key(key) = landing::key(
        landing::Repository { forge: REPO.forge, repository: REPO.repository },
        &gate.name,
        &landing::definition(&gate),
        &gate.ignored,
        candidate,
        &[landing::Tree { object: candidate, entries }],
        None,
        &landing::KeyLimits { entries: 64, path_bytes: 4096 },
    ) else {
        panic!("key")
    };
    top::ReceiptKey { project: PROJECT, repository: REPO, check: top::Check::Gate(gate.name), key }
}

#[test]
fn an_unrelated_whole_failure_at_three_does_not_move_red_or_prevent_a_selected_dispatch() {
    let mut world = configured_batch(&[None, None, Some(b"b\n"), Some(b"a\n"), None], true, workflow(), false);
    until_dispatch(&mut world, 1, 1000);
    let prefix = world.landing(LANDING).landing.members[2].candidate.expect("prefix three");
    let key = key_at(&world, prefix);
    let unrelated = top::Receipt {
        base: None,
        jobs: Box::new([]),
        report: top::ReceiptReport::Unset,
        candidate: prefix,
        passed: false,
        failures: Some(Failures::Tests(Box::new([b"b".as_slice().into()]))),
        logs: b"whole failed b".as_slice().into(),
        ended: world.wall(),
        run: top::ReceiptRun { id: 900, number: 900, url: b"http://forge/runs/900".as_slice().into() },
    };
    world.restore_receipt(key, unrelated.clone());
    until_dispatch(&mut world, 2, 2000);
    let search = world.landing(LANDING).landing.round.search.as_ref().expect("selected search");
    assert_eq!((search.green, search.red), (2, 5), "b alone cannot move r for a");
    world.restart();
    world.take_calls();
    until_dispatch(&mut world, 3, 3000);
    let search = world.landing(LANDING).landing.round.search.as_ref().expect("selected search");
    assert_eq!((search.green, search.red), (3, 5), "selected a passes despite b's whole receipt");
    assert!(
        matches!(&search.tests, landing::SearchTests::ByTest { gates, probed } if gates[0].as_deref() == Some(&[Box::from(b"a".as_slice())]) && probed.iter().any(|p| p.member == 3 && p.passed))
    );
    let calls = world.take_calls();
    assert_eq!(receipt(&world, prefix), Some(&unrelated), "a selected pass leaves the whole failed receipt alone");
    assert!(dispatches(&calls).iter().all(|(_, inputs)| value(inputs, b"temper_tests") == Some(b"a\n")));
    landings::drive_landing(&mut world, LANDING);
    assert!(matches!(world.landing(LANDING).landing.members[3].left, Some(landing::Leave::Blamed { .. })));
}

fn collect_step(world: &mut World, entry: u64, lines: &mut Vec<(u64, Box<[u8]>)>) -> landing::Decision {
    let result = landings::read_landing_step(
        world,
        LANDING,
        entry,
        super::landing_batches::heard(landing::Preparation::Idle, b""),
    );
    lines.extend(result.news);
    if let Some(entry) = result.entry {
        world.event(top::Event::Committed { entry });
        world.run_for(1);
    }
    if matches!(result.decision, landing::Decision::Prepare { .. }) {
        world.prepare_landing(LANDING, false);
    }
    result.decision
}
fn text(commit: api::Commit) -> String {
    String::from_utf8(top::commit_text(&commit).into_vec()).expect("fixture value")
}
#[test]
fn selected_probe_news_and_the_next_round_name_the_search_once_across_restart() {
    for (breaking, grouped) in [(1, false), (3, false), (3, true)] {
        let contents: Vec<_> = (1..=4).map(|index| (index == breaking).then_some(b"a\nb\n".as_slice())).collect();
        let mut world = configured_batch(&contents, true, workflow(), false);
        if grouped {
            let mut row = world.landing(LANDING).clone();
            row.landing.members[3].unit = row.landing.members[2].unit;
            world.keep_history_landing(row);
        }
        let mut lines = Vec::new();
        let mut prefixes = Vec::new();
        for entry in 1000..1600 {
            let decision = collect_step(&mut world, entry, &mut lines);
            if prefixes.is_empty() && world.landing(LANDING).landing.phase == landing::Phase::Proving {
                prefixes = world
                    .landing(LANDING)
                    .landing
                    .members
                    .iter()
                    .map(|m| m.candidate.expect("fixture value"))
                    .collect();
            }
            if matches!(decision, landing::Decision::Leave { .. }) {
                break;
            }
        }
        let base = world.landing(LANDING).landing.members[0].base.expect("fixture value");
        let probes = if grouped {
            vec![(1, 3)]
        } else if breaking == 1 {
            vec![(1, 3), (0, 1)]
        } else {
            vec![(1, 3), (2, 3)]
        };
        for change in 51..=54 {
            for &(probe, red) in &probes {
                let expected = format!(
                    "landing {LANDING}: change {change} probing at {} by 2 tests, red at {}, on {}",
                    text(prefixes[probe]),
                    text(prefixes[red]),
                    text(base)
                );
                assert_eq!(
                    lines
                        .iter()
                        .filter(|(task, words)| *task == change && words.as_ref() == expected.as_bytes())
                        .count(),
                    1,
                    "{expected}"
                );
            }
        }
        assert!(lines.iter().any(|(task, words)| *task == 50 + breaking
            && String::from_utf8_lossy(words).contains("left: gate gate failed in")));
        world.restart();
        for entry in 2000..2600 {
            let decision = collect_step(&mut world, entry, &mut lines);
            if decision == landing::Decision::Finish {
                break;
            }
        }
        for change in (51..=54).filter(|c| *c != 50 + breaking && !(grouped && *c == 54)) {
            assert_eq!(
                lines
                    .iter()
                    .filter(|(task, words)| *task == change && String::from_utf8_lossy(words).contains(" proving at "))
                    .count(),
                2,
                "change {change} proves in both rounds"
            );
        }
        let green = if breaking == 1 { "the base".to_owned() } else { text(prefixes[1]) };
        let blamed = if grouped { "53, 54".to_owned() } else { (50 + breaking).to_string() };
        let red = if grouped { prefixes[3] } else { prefixes[usize::try_from(breaking - 1).expect("fixture value")] };
        let expected = format!("Search by test: red at {}, change {blamed} blamed; green at {green}.\n", text(red));
        let bodies: Vec<_> = world
            .take_calls()
            .into_iter()
            .filter_map(|op| match op {
                api::Op::Write(api::Write::OpenPull { body, .. }) => {
                    Some(String::from_utf8(body.into_vec()).expect("fixture value"))
                }
                _ => None,
            })
            .collect();
        assert_eq!(bodies.len(), 2);
        assert!(!bodies[0].contains("Search by test:"));
        assert!(bodies[1].ends_with(&expected), "{}", bodies[1]);
        assert_eq!(bodies[1].matches("Search by test:").count(), 1);
        assert!(world.landing(LANDING).landing.searched.is_none());
    }
}
#[test]
fn a_search_going_whole_tells_each_member_once_without_a_change_prefix() {
    let mut unselected = workflow();
    unselected.select = None;
    let mut world = configured_batch(&[None, Some(b"b\n"), Some(b"a\n"), None], true, unselected, false);
    let mut lines = Vec::new();
    for entry in 1000..1600 {
        if collect_step(&mut world, entry, &mut lines) == landing::Decision::Finish {
            break;
        }
    }
    let base = world.landing(LANDING).landing.members[0].base.expect("fixture value");
    let expected = format!("landing {LANDING}: probing whole again from {}", text(base));
    for change in 51..=54 {
        assert_eq!(
            lines.iter().filter(|(task, words)| *task == change && words.as_ref() == expected.as_bytes()).count(),
            1,
            "{expected}"
        );
    }
}

#[test]
fn the_search_line_is_not_repeated_when_an_opened_round_goes_stale() {
    let mut world = batch(4, 3, true);
    let mut opened = false;
    for entry in 1000..1600 {
        landings::step_landing(&mut world, LANDING, entry);
        if world.landing(LANDING).landing.round.pull.is_some() && world.landing(LANDING).landing.preparation == 2 {
            opened = true;
            break;
        }
    }
    assert!(opened, "the next round opens after the boundary");
    world.push_file(b"main", b"outside", b"base moved");
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    let bodies: Vec<_> = world
        .take_calls()
        .into_iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::OpenPull { body, .. }) => {
                Some(String::from_utf8(body.into_vec()).expect("body"))
            }
            _ => None,
        })
        .collect();
    assert_eq!(bodies.len(), 3);
    assert_eq!(bodies[1].matches("Search by test:").count(), 1);
    assert!(!bodies[2].contains("Search by test:"));
}
