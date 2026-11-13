//! Proof intervals and their round transitions (domain/landing-history.md, 2–3).
use skein_fake_forge_domain::api as raw;
use skein_lib::Duration;
use temper_engine_domain_forge::{
    self as top,
    history::{self, GateVerdict, What},
};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, CHANGE, LANDING, Trains},
};

fn ours(event: &history::Event) -> bool {
    matches!(
        event.what,
        What::Gate { .. }
            | What::GateEnded { .. }
            | What::Proved { .. }
            | What::Search
            | What::Searched { .. }
            | What::Probe
            | What::ProbeEnded { .. }
            | What::Turn { .. }
            | What::Recorded { .. }
    )
}
fn events(world: &World, change: u64) -> Vec<history::Event> {
    let events: Vec<_> = world.history(change).into_iter().filter(ours).collect();
    let indexed: Vec<_> =
        world.landing_history(LANDING).into_iter().filter(|e| e.change == change).filter(ours).collect();
    assert_eq!(events, indexed, "both indexes expose the same proof history");
    events
}
fn workflow(seconds: u64) -> raw::Workflow {
    let mut workflow = super::landing_batches::workflow(false);
    workflow.latency_min = Duration::from_secs(seconds);
    workflow.latency_max = workflow.latency_min;
    workflow
}
fn start(seconds: u64) -> World {
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow(seconds))));
    world.produce(b"temper/51");
    world.push_file(b"main", b"base", b"main moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    world.event(top::Event::Landing { row: landings::landing_row(world.change(CHANGE)) });
    world
}
fn step(world: &mut World, entry: u64) -> landing::Decision {
    let decision = landings::step_landing(world, LANDING, entry);
    changes::change_step_task(world, CHANGE, entry + 10000, change::Status::Passed, Box::new([]), false, false);
    decision
}
fn until_dispatch(world: &mut World) {
    for entry in 100..160 {
        if matches!(step(world, entry), landing::Decision::Dispatch { .. }) {
            return;
        }
    }
    panic!("no dispatch");
}
fn until_gate(world: &mut World) {
    for entry in 200..250 {
        step(world, entry);
        if world.history(CHANGE).iter().any(|e| matches!(e.what, What::Gate { .. })) {
            return;
        }
    }
    panic!("run not found while running: {:?}", world.history(CHANGE));
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
#[test]
fn green_one_records_the_run_proof_and_turn_in_order() {
    let mut world = start(60);
    landings::drive_landing(&mut world, LANDING);
    let events = events(&world, CHANGE);
    assert_eq!(events.len(), 5, "{events:?}");
    let run = &runs(&world)[0];
    let member = &world.landing(LANDING).landing.members[0];
    let What::Gate { gate, key } = events[0].what else { panic!("gate start") };
    assert_eq!(gate, 0);
    assert_eq!(events[1].what, What::GateEnded { gate, key, verdict: GateVerdict::Passed });
    assert_eq!(events[2].what, What::Proved { proofs: Box::new([history::Proof { gate, run: Some(run.id) }]) });
    assert_eq!(events[3].what, What::Turn { members: 1 });
    assert_eq!(events[4].what, What::Recorded { state: history::RecordedState::Merged });
    assert_eq!((events[4].operation, events[4].attempt), (member.pull, 0));
    assert_eq!(events[0].source_at.map(skein_lib::Wall::as_nanos), Some(run.created.as_nanos()));
    for event in &events[..2] {
        assert_eq!((event.operation, event.attempt), (run.id, 1));
        assert_eq!(event.base, None);
    }
    for event in &events {
        assert_eq!(event.head, member.candidate);
        assert_eq!(event.landing, Some(history::InLanding { task: LANDING, round: 1, unit: member.unit }));
        assert!(event.source_at.is_none() || matches!(event.what, What::Gate { .. }));
    }
    assert_eq!(events[2].base, member.base);
    assert_eq!(events[3].base, member.base);
}
#[test]
fn receipts_prove_each_gate_without_a_run() {
    let mut world = landings::first_landing();
    landings::documentation_landing(&mut world, 52, 62, b"temper/52");
    let events: Vec<_> = world.history(52).into_iter().filter(ours).collect();
    assert_eq!(events, world.landing_history(62).into_iter().filter(ours).collect::<Vec<_>>());
    assert_eq!(
        events.iter().map(|e| &e.what).collect::<Vec<_>>(),
        vec![
            &What::Proved { proofs: Box::new([history::Proof { gate: 0, run: None }]) },
            &What::Turn { members: 1 },
            &What::Recorded { state: history::RecordedState::Merged },
        ]
    );
}
#[test]
fn a_runner_fault_and_its_same_id_rerun_have_distinct_attempts() {
    let mut world = start(30);
    world.runner_setup_faults(1000);
    world.gate_policy(landing::Project { gate_retries: 1, ..landing::Project::DEFAULT });
    until_dispatch(&mut world);
    world.lose_rerun_answer();
    let mut restarted = false;
    for entry in 200..400 {
        if matches!(step(&mut world, entry), landing::Decision::Hold(_)) {
            break;
        }
        if !restarted && world.history(CHANGE).iter().any(|e| matches!(e.what, What::Gate { .. }) && e.attempt == 2) {
            let before = landing_events(&world);
            world.restart();
            world.run_for(1);
            assert_eq!(landing_events(&world), before);
            restarted = true;
        }
    }
    assert!(restarted, "relearned the second attempt while running");
    let events = events(&world, CHANGE);
    let ends: Vec<_> =
        events.iter().filter(|e| matches!(e.what, What::GateEnded { verdict: GateVerdict::Fault, .. })).collect();
    assert_eq!(ends.len(), 2, "{events:?}");
    assert_eq!((ends[0].operation, ends[0].attempt), (ends[1].operation, 1));
    assert_eq!(ends[1].attempt, 2);
    assert!(
        events
            .iter()
            .any(|e| matches!(e.what, What::Gate { .. }) && e.operation == ends[1].operation && e.attempt == 2)
    );
    let before = events.clone();
    world.restart();
    for entry in 500..505 {
        step(&mut world, entry);
    }
    assert_eq!(events_for(&world, CHANGE), before);
}
fn events_for(world: &World, change: u64) -> Vec<history::Event> {
    events(world, change)
}
fn landing_events(world: &World) -> Vec<history::Event> {
    world.landing_history(LANDING).into_iter().filter(ours).collect()
}
#[test]
fn timeout_closes_the_running_gate_once() {
    let mut world = start(300);
    world.gate_policy(landing::Project { gate_timeout: Duration::from_secs(10), ..landing::Project::DEFAULT });
    until_dispatch(&mut world);
    for entry in 200..240 {
        if matches!(step(&mut world, entry), landing::Decision::Hold(_)) {
            break;
        }
    }
    let events = events(&world, CHANGE);
    assert_eq!(
        events.iter().filter(|e| matches!(e.what, What::GateEnded { verdict: GateVerdict::Timeout, .. })).count(),
        1,
        "{events:?}"
    );
    world.restart();
    world.run_for(400);
    for entry in 500..505 {
        step(&mut world, entry);
    }
    assert_eq!(events_for(&world, CHANGE), events);
}
#[test]
fn leaving_mid_proof_supersedes_the_open_interval_once() {
    for delay in [0, 400] {
        let mut world = start(300);
        until_dispatch(&mut world);
        until_gate(&mut world);
        world.push_file(b"temper/51", b"moved", b"new head");
        world.run_for(delay);
        for entry in 300..330 {
            step(&mut world, entry);
            if world
                .history(CHANGE)
                .iter()
                .any(|e| matches!(e.what, What::GateEnded { verdict: GateVerdict::Superseded, .. }))
            {
                break;
            }
        }
        let events = events(&world, CHANGE);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.what, What::GateEnded { verdict: GateVerdict::Superseded, .. }))
                .count(),
            1,
            "{events:?}"
        );
        world.restart();
        world.run_for(400);
        for entry in 500..510 {
            step(&mut world, entry);
        }
        assert_eq!(events_for(&world, CHANGE), events);
    }
}
#[test]
fn a_completed_run_first_relearned_has_only_its_end() {
    let mut world = start(60);
    until_dispatch(&mut world);
    assert_eq!(events(&world, CHANGE), []);
    world.restart();
    world.run_for(100);
    landings::drive_landing(&mut world, LANDING);
    let events = events(&world, CHANGE);
    assert!(matches!(events[0].what, What::GateEnded { verdict: GateVerdict::Passed, .. }), "{events:?}");
    assert!(!events.iter().any(|e| matches!(e.what, What::Gate { .. })));
    assert_eq!(events.len(), 4);
    assert_eq!(events[3].what, What::Recorded { state: history::RecordedState::Merged });
}
fn batch(count: u64, red: bool) -> World {
    let mut workflow = workflow(180);
    if red {
        workflow.cue = Some(raw::Cue::File { path: b"file".as_slice().into(), green: b"fixed".as_slice().into() });
    }
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow)));
    world.batch(u16::try_from(count).expect("fixture"));
    world.push_file(b"main", b"file", b"fixed");
    for task in 51..51 + count {
        let branch = format!("temper/c{task}");
        world.external(
            raw::Op::Git(raw::Git::Create { branch: branch.as_bytes().into(), commit: world.branch(b"main") }),
            2,
        );
        if red && task == 52 {
            world.push_file(branch.as_bytes(), b"file", b"broken");
        } else {
            world.push_file(branch.as_bytes(), format!("safe{task}").as_bytes(), b"safe");
        }
        landings::ready_change(&mut world, task, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    world
}
#[test]
fn a_batch_gets_one_proof_interval_and_turn_per_member() {
    let mut world = batch(3, false);
    landings::drive_landing(&mut world, LANDING);
    let candidate = world.landing(LANDING).landing.members[2].candidate;
    for task in 51..=53 {
        let events = events(&world, task);
        assert_eq!(events.len(), 5, "{events:?}");
        assert!(matches!(events[0].what, What::Gate { .. }));
        assert!(matches!(events[1].what, What::GateEnded { verdict: GateVerdict::Passed, .. }));
        assert!(matches!(events[2].what, What::Proved { .. }));
        assert_eq!(events[3].what, What::Turn { members: 3 });
        assert_eq!(events[4].what, What::Recorded { state: history::RecordedState::Merged });
        let member = world.landing(LANDING).landing.members.iter().find(|m| m.change == task).expect("member");
        assert_eq!((events[4].operation, events[4].attempt), (member.pull, 0));
        assert!(events[..4].iter().all(|e| e.head == candidate));
        assert!(events.iter().all(|e| e.landing.expect("fixture").unit == u16::try_from(task - 51).expect("fixture")));
    }
    assert_eq!(runs(&world).len(), 1);
}
#[test]
fn a_squashed_member_is_recorded_at_its_prefix_once() {
    let mut world = start(60);
    for entry in 100..120 {
        if matches!(landings::step_landing_raw(&mut world, LANDING, entry), landing::Decision::Prepare { .. }) {
            break;
        }
    }
    world.prepare_landing(LANDING, true);
    landings::drive_landing(&mut world, LANDING);
    let member = &world.landing(LANDING).landing.members[0];
    let events = events(&world, CHANGE);
    let records: Vec<_> =
        events.iter().filter(|e| matches!(e.what, What::Recorded { state: history::RecordedState::Merged })).collect();
    assert_eq!(records.len(), 1);
    assert_eq!((records[0].head, records[0].operation, records[0].attempt), (member.candidate, member.pull, 0));
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(events_for(&world, CHANGE), events);
}
#[test]
fn a_red_round_records_search_probes_boundary_and_the_green_turn() {
    let mut world = batch(3, true);
    let mut prefixes = Vec::new();
    for entry in 1000..2000 {
        landings::step_landing(&mut world, LANDING, entry);
        if prefixes.is_empty() && world.landing(LANDING).landing.phase == landing::Phase::Proving {
            prefixes = world.landing(LANDING).landing.members.iter().map(|m| m.candidate.expect("fixture")).collect();
        }
        if matches!(world.landing(LANDING).landing.round.ending, Some(landing::RoundEnd::Cut)) {
            break;
        }
    }
    assert_eq!(prefixes.len(), 3);
    for task in 51..=53 {
        let events = events(&world, task);
        let search: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.what, What::Search | What::Probe | What::ProbeEnded { .. } | What::Searched { .. }))
            .collect();
        assert_eq!(
            search.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
            vec![
                What::Search,
                What::Probe,
                What::ProbeEnded { green: true },
                What::Probe,
                What::ProbeEnded { green: false },
                What::Searched { green: Some(prefixes[0]), red: prefixes[1], blamed: 1 },
            ]
        );
        assert_eq!(search[0].head, Some(prefixes[2]));
        assert_eq!(search[1].head, Some(prefixes[0]));
        assert_eq!(search[2].head, search[1].head);
        assert_eq!(search[3].head, Some(prefixes[1]));
        assert_eq!(search[4].head, search[3].head);
        for prefix in &prefixes[..2] {
            assert!(events.iter().any(|e| e.head == Some(*prefix) && matches!(e.what, What::Gate { .. })));
            assert!(events.iter().any(|e| e.head == Some(*prefix) && matches!(e.what, What::GateEnded { .. })));
        }
    }
    landings::drive_landing(&mut world, LANDING);
    let events = events(&world, 51);
    assert!(events.iter().any(|e| e.head == Some(prefixes[0]) && matches!(e.what, What::Proved { .. })));
    assert!(events.iter().any(|e| e.head == Some(prefixes[0]) && e.what == What::Turn { members: 1 }));
    assert!(
        events.iter().all(|e| e.landing.expect("landing").round == 1),
        "the green member is not in round two: {events:?}"
    );
    let suffix = events_for(&world, 53);
    for event in suffix
        .iter()
        .filter(|e| e.landing.expect("landing").round == 2 && matches!(e.what, What::Proved { .. } | What::Turn { .. }))
    {
        assert_eq!(event.base, Some(prefixes[0]));
    }
}
#[test]
fn restarts_in_proving_probing_and_turn_keep_every_identity_once() {
    let mut world = batch(3, true);
    let mut proving = false;
    let mut search_seen = false;
    let mut turning = false;
    for entry in 1000..2000 {
        let decision = landings::step_landing(&mut world, LANDING, entry);
        let row = world.landing(LANDING);
        let checkpoint = if !proving && row.entries.iter().any(|e| e.run.is_some()) {
            proving = true;
            true
        } else if !search_seen
            && row.landing.round.search.is_some()
            && row.entries.iter().any(|e| e.member == 0 && e.run.is_some())
        {
            search_seen = true;
            true
        } else if !turning && matches!(decision, landing::Decision::Merge { .. }) {
            turning = true;
            true
        } else {
            false
        };
        if checkpoint {
            let before = landing_events(&world);
            world.restart();
            world.run_for(1);
            assert_eq!(landing_events(&world), before);
        }
        if turning {
            break;
        }
    }
    assert!(proving && search_seen && turning);
    landings::drive_landing(&mut world, LANDING);
    for task in 51..=53 {
        let events = events(&world, task);
        assert_ne!(events, []);
        for (index, event) in events.iter().enumerate() {
            assert!(!events[..index].iter().any(|other| std::mem::discriminant(&other.what)
                == std::mem::discriminant(&event.what)
                && other.operation == event.operation
                && other.attempt == event.attempt
                && other.head == event.head));
        }
        // Proving's first run and both probes still have their starts and ends.
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.what, What::Gate { .. }) && e.landing.expect("fixture").round == 1)
                .count(),
            3
        );
    }
}
