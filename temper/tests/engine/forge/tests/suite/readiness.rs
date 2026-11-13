//! Readiness is history of decisions, not of polls (domain/landing-history.md, 2.4 and 3).
use skein_fake_forge_domain::api as raw;
use temper_engine_domain_forge::{self as top, history as h, settings};
use temper_engine_domain_forge_change as c;
use temper_engine_forge_world::{World, changes, landings, translate};

const CHANGE: u64 = 51;
const BRANCH: &[u8] = b"temper/51";

fn history(world: &World, change: u64) -> Vec<h::Event> {
    world
        .history(change)
        .into_iter()
        .filter(|event| {
            matches!(
                event.what,
                h::What::Ready
                    | h::What::Unready { .. }
                    | h::What::Selected
                    | h::What::Joined { line: h::Line::Queue, .. }
                    | h::What::Departed { line: h::Line::Queue, .. }
                    | h::What::Paused { .. }
                    | h::What::Resumed
                    | h::What::Held { on: h::On::Change, .. }
                    | h::What::Released { on: h::On::Change, .. }
                    | h::What::Ended { .. }
            )
        })
        .collect()
}

fn kinds(world: &World, change: u64) -> Vec<h::What> {
    history(world, change).into_iter().map(|event| event.what).collect()
}

fn step(world: &mut World, task: u64, entry: u64) -> c::Decision {
    changes::change_step_task(world, task, entry, c::Status::Unknown, Box::new([]), false, false)
}

fn queue() -> World {
    let mut world = World::new(2563);
    world.adopt();
    add(&mut world, CHANGE, BRANCH);
    world
}

fn add(world: &mut World, task: u64, branch: &[u8]) {
    world.produce_file(branch, branch, b"change");
    world.status(branch, raw::Check::Passed);
    let head = translate::commit(world.branch(branch));
    let mut row = changes::change_row();
    row.task = task;
    row.change.task = task;
    row.branch = branch.into();
    row.pull = Some(world.open_pull(branch, b"main"));
    row.change.last_head = Some(head);
    row.change.state = c::State::Gating { head, asked: Box::new([]) };
    world.event(top::Event::Change { row });
    assert_eq!(step(world, task, task), c::Decision::Ready);
}

fn turn() -> World {
    let mut world = landings::turn_world();
    world.produce(BRANCH);
    landings::ready_change(&mut world, CHANGE, BRANCH);
    world
}

fn hold(world: &mut World) {
    world.event(top::Event::RelayRefused {
        task: CHANGE,
        questions: world.change(CHANGE).questions.clone(),
        words: b"relay refused".as_slice().into(),
    });
}

fn own(event: &h::Event) {
    assert_eq!(event.operation, event.number);
    assert_eq!(event.attempt, 0);
    assert!(event.source_at.is_none());
}

fn unique(world: &World, change: u64) {
    let events = history(world, change);
    for pair in events.windows(2) {
        assert!(pair[0].number < pair[1].number);
    }
    for (i, event) in events.iter().enumerate() {
        assert!(
            !events[..i]
                .iter()
                .any(|old| old.what == event.what && old.operation == event.operation && old.attempt == event.attempt)
        );
    }
}

#[test]
fn an_idle_queue_records_ready_joined_selected_and_not_a_departure() {
    for restart in [false, true] {
        let mut world = queue();
        let head = world.change(CHANGE).change.last_head;
        let tip = translate::commit(world.branch(b"main"));
        if restart {
            world.restart();
        }
        assert!(matches!(step(&mut world, CHANGE, 101), c::Decision::Effect(c::Effect::Merge { .. })));
        if restart {
            world.restart();
        }
        assert!(matches!(step(&mut world, CHANGE, 102), c::Decision::Finish { .. }));
        let events = history(&world, CHANGE);
        assert_eq!(
            events.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
            [
                h::What::Ready,
                h::What::Joined { line: h::Line::Queue, place: Some(1), priority: Some(1) },
                h::What::Selected,
            ]
        );
        for event in &events {
            own(event);
            assert_eq!(event.head, head);
            assert_eq!(event.landing, None);
        }
        assert_eq!(events[0].base, None);
        assert_eq!(events[1].base, None);
        assert_eq!(events[2].base, Some(tip));
        unique(&world, CHANGE);
    }
}

#[test]
fn an_update_keeps_the_same_selection_through_a_restart() {
    let mut world = queue();
    world.push_file(b"main", b"base", b"base moved");
    let tip = translate::commit(world.branch(b"main"));
    assert!(
        matches!(step(&mut world, CHANGE, 101), c::Decision::Effect(c::Effect::Update { base, .. }) if base == tip)
    );
    let before = history(&world, CHANGE);
    assert_eq!(
        before.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [
            h::What::Ready,
            h::What::Joined { line: h::Line::Queue, place: Some(1), priority: Some(1) },
            h::What::Selected
        ]
    );
    assert_eq!(before[2].base, Some(tip));
    world.restart();
    world.status(BRANCH, raw::Check::Passed);
    assert_eq!(step(&mut world, CHANGE, 102), c::Decision::Ready);
    assert_eq!(
        history(&world, CHANGE),
        before,
        "a clean update keeps its existing turn, readiness and queue membership"
    );
    step(&mut world, CHANGE, 103);
    step(&mut world, CHANGE, 104);
    assert_eq!(history(&world, CHANGE), before);
}

#[test]
fn a_queued_head_move_closes_both_intervals_and_the_next_ready_is_new() {
    let mut world = queue();
    let initial = history(&world, CHANGE);
    world.restart();
    world.push_file(BRANCH, b"new", b"new head");
    world.status(BRANCH, raw::Check::Passed);
    assert_eq!(step(&mut world, CHANGE, 101), c::Decision::Hold(c::Hold::BranchMoved));
    let events = history(&world, CHANGE);
    let unready = events
        .iter()
        .find(|e| e.what == h::What::Unready { why: h::Unready::HeadMoved })
        .expect("head moved closes ready");
    let departed = events
        .iter()
        .find(|e| e.what == h::What::Departed { line: h::Line::Queue, cause: h::Departure::Unready })
        .expect("head moved leaves queue");
    assert_eq!(unready.operation, initial[0].number);
    assert_eq!(departed.operation, initial[1].number);
    world.restart();
    let before = history(&world, CHANGE);
    step(&mut world, CHANGE, 102);
    assert_eq!(history(&world, CHANGE), before, "held poll writes nothing");
    changes::change_step_with_release(&mut world, 103, c::Status::Unknown, Box::new([]), true);
    let ready: Vec<_> = history(&world, CHANGE).into_iter().filter(|e| e.what == h::What::Ready).collect();
    assert_eq!(ready.len(), 2);
    assert!(ready[1].number > ready[0].number);
    assert_ne!(ready[1].head, ready[0].head);
    for event in ready {
        own(&event);
    }
    unique(&world, CHANGE);
}

fn approval_rule(world: &mut World) {
    world.keep_settings(
        1,
        &[settings::Asked {
            kind: 2,
            scope: 5,
            number: 1,
            edit: settings::Edit::Set(settings::SizedValue {
                size: 128,
                value: settings::Value::Rule(settings::LandingRule {
                    sets: Box::new([]),
                    pattern: top::GatePattern {
                        segments: Box::new([b"branch".as_slice().into()]),
                        last: top::GateLast::Exact(b"main".as_slice().into()),
                    },
                    enforce: true,
                    ci: true,
                    up_to_date: true,
                    gates: Box::new([]),
                    approvals: Box::new([settings::RuleApproval {
                        role: 1,
                        people: 1,
                        freshness: top::JudgeFreshness::Exact,
                    }]),
                }),
            }),
        }],
    );
}

fn approval(world: &mut World, present: bool) {
    let head = world.change(CHANGE).change.last_head.expect("pushed head");
    let reviewers = if present {
        vec![top::Reviewer { at: world.wall(), person: 7, role: 1, head, verdict: skein_forge::api::Verdict::Approve }]
    } else {
        vec![]
    };
    assert!(world.keep_review_facts(
        CHANGE,
        top::ReviewFacts {
            size: None,
            head: Some(head),
            reviews_complete: true,
            reviewers: reviewers.into_boxed_slice(),
            unmapped: Box::new([]),
        }
    ));
    world.event(top::Event::ReviewsChanged { task: CHANGE });
}

#[test]
fn an_approval_lapse_closes_readiness_but_not_the_counted_queue_interval() {
    let mut world = queue();
    approval_rule(&mut world);
    approval(&mut world, true);
    approval(&mut world, false);
    let events = history(&world, CHANGE);
    assert_eq!(
        kinds(&world, CHANGE),
        [
            h::What::Ready,
            h::What::Joined { line: h::Line::Queue, place: Some(1), priority: Some(1) },
            h::What::Unready { why: h::Unready::Approval }
        ]
    );
    assert_eq!(events[2].operation, events[0].number);
    assert!(matches!(world.change(CHANGE).change.state, c::State::Queued { .. }));
    assert!(world.change(CHANGE).history.expect("history").queued.is_some());
    world.restart();
    approval(&mut world, false);
    step(&mut world, CHANGE, 101);
    assert_eq!(history(&world, CHANGE), events, "relearned lapse is not another transition");
}

#[test]
fn turn_availability_tracks_approval_without_changing_its_ready_state() {
    let mut world = turn();
    approval_rule(&mut world);
    approval(&mut world, false);
    assert_eq!(kinds(&world, CHANGE), [h::What::Ready, h::What::Unready { why: h::Unready::Approval }]);
    let events = history(&world, CHANGE);
    world.restart();
    step(&mut world, CHANGE, 101);
    assert_eq!(history(&world, CHANGE), events, "unread reviews do not reopen a lapsed approval");
    approval(&mut world, false);
    assert_eq!(history(&world, CHANGE), events);
    approval(&mut world, true);
    let events = history(&world, CHANGE);
    assert_eq!(events.len(), 3);
    assert_eq!(events[2].what, h::What::Ready);
    own(&events[2]);
    assert!(matches!(world.change(CHANGE).change.state, c::State::Ready { .. }));
}

#[test]
fn a_turn_dependency_flips_the_readiness_interval_not_its_policy_state() {
    let mut world = turn();
    changes::change_step_task(&mut world, CHANGE, 101, c::Status::Unknown, Box::new([]), false, true);
    assert_eq!(kinds(&world, CHANGE), [h::What::Ready, h::What::Unready { why: h::Unready::Dependency }]);
    let events = history(&world, CHANGE);
    assert_eq!(events[1].operation, events[0].number);
    world.restart();
    changes::change_step_task(&mut world, CHANGE, 102, c::Status::Unknown, Box::new([]), false, true);
    assert_eq!(history(&world, CHANGE), events);
    step(&mut world, CHANGE, 103);
    assert_eq!(
        kinds(&world, CHANGE),
        [h::What::Ready, h::What::Unready { why: h::Unready::Dependency }, h::What::Ready]
    );
    assert!(matches!(world.change(CHANGE).change.state, c::State::Ready { .. }));
    unique(&world, CHANGE);
}

#[test]
fn a_branch_red_pauses_each_waiter_and_green_resumes_it_once() {
    let mut world = queue();
    add(&mut world, 52, b"temper/52");
    world.landing_ci(b"main", true);
    world.status(b"main", raw::Check::Failed);
    assert_eq!(step(&mut world, CHANGE, 101), c::Decision::QueueRepair);
    world.event(top::Event::QueueRepairStarted { owner: CHANGE, repair: 91 });
    world.restart();
    assert_eq!(step(&mut world, CHANGE, 102), c::Decision::QueueRepair);
    world.status(b"main", raw::Check::Passed);
    step(&mut world, 52, 103);
    world.restart();
    step(&mut world, 52, 104);
    let tip = translate::commit(world.branch(b"main"));
    for change in [CHANGE, 52] {
        let events: Vec<_> = history(&world, change)
            .into_iter()
            .filter(|e| matches!(e.what, h::What::Paused { .. } | h::What::Resumed))
            .collect();
        assert_eq!(events.len(), 2, "one pause and one resume per waiter");
        assert_eq!(events[0].what, h::What::Paused { why: h::Pause::BranchRed });
        assert_eq!(events[1].what, h::What::Resumed);
        own(&events[0]);
        assert_eq!(events[1].operation, events[0].number);
        for event in events {
            assert_eq!(event.head, None);
            assert_eq!(event.base, Some(tip));
        }
        unique(&world, change);
    }
}

#[test]
fn a_yielded_turn_records_each_selection_with_its_own_identity() {
    let mut world = queue();
    world.landing_ci(b"main", true);
    world.status(b"main", raw::Check::Failed);
    step(&mut world, CHANGE, 101);
    approval_rule(&mut world);
    approval(&mut world, false);
    assert!(matches!(world.change(CHANGE).change.state, c::State::Queued { .. }));
    world.restart();
    approval(&mut world, true);
    step(&mut world, CHANGE, 102);
    let selected: Vec<_> = history(&world, CHANGE).into_iter().filter(|e| e.what == h::What::Selected).collect();
    assert_eq!(selected.len(), 2);
    assert!(selected[0].number < selected[1].number);
    for event in selected {
        own(&event);
    }
    unique(&world, CHANGE);
}

#[test]
fn a_turn_hold_closes_ready_and_its_release_pairs_with_the_hold() {
    let mut world = turn();
    world.restart();
    hold(&mut world);
    let events = history(&world, CHANGE);
    assert_eq!(
        events.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [
            h::What::Ready,
            h::What::Unready { why: h::Unready::Held },
            h::What::Held { on: h::On::Change, reason: b"failed".as_slice().into(), by: None }
        ]
    );
    assert_eq!(events[1].operation, events[0].number);
    own(&events[2]);
    world.restart();
    world.event(top::Event::ReleaseChange { task: CHANGE });
    let events = history(&world, CHANGE);
    assert_eq!(
        events.last().expect("release").what,
        h::What::Released { on: h::On::Change, reason: b"failed".as_slice().into(), by: None }
    );
    assert_eq!(events.last().expect("release").operation, events[2].number);
    world.restart();
    step(&mut world, CHANGE, 101);
    let ready: Vec<_> = history(&world, CHANGE).into_iter().filter(|e| e.what == h::What::Ready).collect();
    assert_eq!(ready.len(), 2);
    unique(&world, CHANGE);
}

#[test]
fn a_cancelled_ready_change_ends_without_becoming_unready() {
    for way in [c::Way::Queue, c::Way::Turn] {
        let mut world = if way == c::Way::Queue { queue() } else { turn() };
        world.restart();
        world.event(top::Event::StepChange {
            task: CHANGE,
            entry: 101,
            heard: c::Heard {
                review_request: None,
                start_conflict: None,
                lacking: None,
                spend: None,
                delegate: c::Status::Unknown,
                delegate_running: false,
                delegate_live: false,
                effect: c::EffectResult::None,
                released: false,
                cancelled: true,
            },
            gates: Box::new([]),
            queue_repair_active: false,
            kept: false,
            merge_job: false,
            pending: Box::new([]),
        });
        world.run_for(1);
        let events = history(&world, CHANGE);
        assert_eq!(events.last().expect("ending").what, h::What::Ended { ending: h::Ending::Cancelled });
        assert!(!events.iter().any(|e| matches!(e.what, h::What::Unready { .. } | h::What::Departed { .. })));
        own(events.last().expect("ending"));
        world.restart();
        step(&mut world, CHANGE, 102);
        assert_eq!(history(&world, CHANGE), events);
    }
}

#[test]
fn a_no_change_producer_result_records_the_ending_once() {
    let mut world = World::new(2563);
    world.adopt();
    let mut row = changes::change_row();
    row.change.state = c::State::Producing { requested: true };
    row.delegate = Some((91, c::Delegate::Produce));
    world.event(top::Event::Change { row });
    let result = || top::Event::DelegateResult {
        task: CHANGE,
        child: 91,
        status: c::Status::Passed,
        no_change: true,
        failed: false,
        cancelled: false,
        words: b"Nothing needed".as_slice().into(),
    };
    world.event(result());
    assert_eq!(kinds(&world, CHANGE), [h::What::Ended { ending: h::Ending::NoChange }]);
    let events = history(&world, CHANGE);
    own(&events[0]);
    assert_eq!(events[0].head, None);
    world.restart();
    world.event(result());
    step(&mut world, CHANGE, 101);
    assert_eq!(history(&world, CHANGE), events);
}

#[test]
fn a_change_failed_for_good_records_failed_not_a_temporary_hold() {
    let mut world = turn();
    hold(&mut world);
    let ended = || top::Event::ChangeEnding {
        task: CHANGE,
        ending: top::Ending::Failed { line: b"failed for good".as_slice().into() },
    };
    world.event(ended());
    assert_eq!(history(&world, CHANGE).last().expect("ending").what, h::What::Ended { ending: h::Ending::Failed });
    let events = history(&world, CHANGE);
    own(events.last().expect("ending"));
    world.restart();
    world.event(ended());
    assert_eq!(history(&world, CHANGE), events);
}

#[test]
fn restoring_each_turn_interval_and_polling_writes_nothing_twice() {
    let mut world = turn();
    assert_eq!(kinds(&world, CHANGE), [h::What::Ready]);
    for phase in 0..3 {
        if phase == 1 {
            hold(&mut world);
        }
        if phase == 2 {
            world.event(top::Event::ReleaseChange { task: CHANGE });
            step(&mut world, CHANGE, 102);
        }
        let events = history(&world, CHANGE);
        world.restart();
        step(&mut world, CHANGE, 110 + phase);
        assert_eq!(history(&world, CHANGE), events, "restore at interval {phase}");
        step(&mut world, CHANGE, 120 + phase);
        assert_eq!(history(&world, CHANGE), events, "empty poll at interval {phase}");
    }
    unique(&world, CHANGE);
}

#[test]
fn a_change_event_in_a_live_landing_names_its_task_round_and_unit() {
    let mut world = turn();
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    landings::step_landing_raw(&mut world, landings::LANDING, 101);
    landings::step_landing_raw(&mut world, landings::LANDING, 102);
    let landing = world.landing(landings::LANDING);
    let in_landing = h::InLanding {
        task: landings::LANDING,
        round: landing.landing.preparation,
        unit: landing.landing.members[0].unit,
    };
    assert!(in_landing.round > 0);
    world.restart();
    hold(&mut world);
    let events: Vec<_> = history(&world, CHANGE)
        .into_iter()
        .filter(|e| matches!(e.what, h::What::Unready { .. } | h::What::Held { .. }))
        .collect();
    assert_eq!(events.len(), 2);
    for event in &events {
        assert_eq!(event.landing, Some(in_landing));
    }
    let indexed: Vec<_> = world
        .landing_history(landings::LANDING)
        .into_iter()
        .filter(|e| events.iter().any(|ours| ours.number == e.number))
        .collect();
    assert_eq!(indexed, events);
}
