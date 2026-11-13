//! Landing history is written by transitions, not by rediscovery (landing-history.md, 3).
use temper_engine_domain_forge::{self as top, history as h};
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World,
    landings::{self, LANDING, Trains},
    translate,
};

fn relevant(event: &h::Event) -> bool {
    matches!(
        event.what,
        h::What::Joined { line: h::Line::Landing, .. }
            | h::What::Departed { line: h::Line::Landing, .. }
            | h::What::Formed { .. }
            | h::What::RoundEnded { .. }
            | h::What::Job { job: h::Job::Prepare, .. }
            | h::What::JobEnded { job: h::Job::Prepare, .. }
            | h::What::Conflict { .. }
            | h::What::LandingEnded { .. }
            | h::What::Held { on: h::On::Landing, .. }
            | h::What::Released { on: h::On::Landing, .. }
    )
}
fn events(world: &World, change: u64) -> Vec<h::Event> {
    let events: Vec<_> = world.history(change).into_iter().filter(relevant).collect();
    let indexed: Vec<_> =
        world.landing_history(LANDING).into_iter().filter(relevant).filter(|e| e.change == change).collect();
    assert_eq!(events, indexed, "both history indexes tell the same steps");
    for (index, event) in events.iter().enumerate() {
        assert!(
            !events[..index].iter().any(|old| std::mem::discriminant(&old.what) == std::mem::discriminant(&event.what)
                && old.operation == event.operation
                && old.attempt == event.attempt),
            "repeated event: {event:?}"
        );
    }
    events
}
fn batch(count: u64) -> World {
    batch_workflow(count, None)
}
fn batch_workflow(count: u64, workflow: Option<skein_fake_forge_domain::api::Workflow>) -> World {
    let mut world = landings::turn_in(World::batch_turn(11, workflow));
    world.batch(u16::try_from(count).expect("small batch"));
    for task in 51..51 + count {
        let branch = format!("temper/c{task}");
        world.produce_file(branch.as_bytes(), format!("file{task}").as_bytes(), b"member");
        landings::ready_change(&mut world, task, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    world
}
fn prepare(world: &mut World) {
    for entry in 100..140 {
        if matches!(landings::step_landing_raw(world, LANDING, entry), landing::Decision::Prepare { .. }) {
            return;
        }
    }
    panic!("no prepare");
}
fn job(world: &mut World, task: u64, attempt: u32) {
    world.event(top::Event::JobHistory {
        task: LANDING,
        job: h::JobStep { task, attempt, job: h::Job::Prepare, outcome: None, head: None },
    });
    world.run_for(1);
}
fn heard(outcome: Option<(u64, h::JobOutcome)>) -> top::LandingHeard {
    let mut heard = super::landing_batches::heard(landing::Preparation::Idle, b"");
    heard.job = outcome.map(|(task, outcome)| h::JobStep {
        task,
        attempt: 1,
        job: h::Job::Prepare,
        outcome: Some(outcome),
        head: None,
    });
    heard
}
fn proving(world: &mut World) {
    for entry in 200..240 {
        landings::step_landing(world, LANDING, entry);
        if world.landing(LANDING).landing.phase == landing::Phase::Proving {
            return;
        }
    }
    panic!("not proving");
}
fn count_formed(events: &[h::Event], round: u16) -> usize {
    events
        .iter()
        .filter(|e| matches!(e.what, h::What::Formed { .. }) && e.landing.expect("landing event").round == round)
        .count()
}

#[test]
fn one_member_records_membership_job_round_and_end_in_order() {
    let mut world = batch(1);
    prepare(&mut world);
    let member = world.landing(LANDING).landing.members[0].clone();
    job(&mut world, 701, 1);
    world.prepare_landing(LANDING, false);
    landings::step_landing_heard(&mut world, LANDING, 150, heard(Some((701, h::JobOutcome::Done))));
    landings::drive_landing(&mut world, LANDING);
    let prefix = world.landing(LANDING).landing.members[0].candidate;
    let got = events(&world, 51);
    assert_eq!(
        got.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        vec![
            h::What::Joined { line: h::Line::Landing, place: Some(1), priority: None },
            h::What::Formed {
                members: Box::new([h::Member { change: 51, unit: member.unit, head: member.head.unwrap() }]),
                place: 1
            },
            h::What::Job { job: h::Job::Prepare, task: 701 },
            h::What::JobEnded { job: h::Job::Prepare, outcome: h::JobOutcome::Done },
            h::What::RoundEnded { cause: h::RoundCause::Turned },
            h::What::LandingEnded { outcome: h::LandingOutcome::Done },
        ]
    );
    assert_eq!(got[0].landing, Some(h::InLanding { task: LANDING, round: 1, unit: member.unit }));
    assert_eq!((got[1].head, got[1].base, got[1].operation, got[1].attempt), (member.head, member.base, LANDING, 1));
    assert_eq!((got[2].operation, got[2].attempt), (701, 1));
    assert_eq!((got[3].operation, got[3].attempt, got[3].head), (701, 1, prefix));
    assert_eq!(got[4].base, member.base);
    assert!(got.windows(2).all(|pair| pair[0].at <= pair[1].at));
}

#[test]
fn three_members_each_record_the_whole_stack_and_their_place() {
    let mut world = batch(3);
    prepare(&mut world);
    let members: Box<[_]> = world
        .landing(LANDING)
        .landing
        .members
        .iter()
        .map(|m| h::Member { change: m.change, unit: m.unit, head: m.head.unwrap() })
        .collect();
    for (index, member) in members.iter().enumerate() {
        let got = events(&world, member.change);
        assert_eq!(count_formed(&got, 1), 1);
        assert_eq!(got[1].what, h::What::Formed { members: members.clone(), place: u16::try_from(index + 1).unwrap() });
    }
    assert_eq!(world.landing_history(LANDING).iter().filter(|e| matches!(e.what, h::What::Formed { .. })).count(), 3);
}

#[test]
fn moved_member_departs_once_and_the_remaining_stack_forms_round_two() {
    let mut world = batch(3);
    proving(&mut world);
    world.push_file(b"temper/c52", b"moved", b"new head");
    let moved = translate::commit(world.branch(b"temper/c52"));
    for entry in 300..360 {
        if matches!(landings::step_landing_raw(&mut world, LANDING, entry), landing::Decision::Prepare { .. }) {
            break;
        }
    }
    let got = events(&world, 52);
    let joined = got.iter().find(|e| matches!(e.what, h::What::Joined { .. })).unwrap();
    let left = got.iter().find(|e| matches!(e.what, h::What::Departed { .. })).unwrap();
    assert_eq!(
        left.what,
        h::What::Departed {
            line: h::Line::Landing,
            cause: h::Departure::Landing(landing::Leave::HeadMoved { head: moved })
        }
    );
    assert_eq!((left.operation, left.attempt), (joined.operation, joined.attempt));
    for change in 51..54 {
        let got = events(&world, change);
        assert_eq!(got.iter().filter(|e| e.what == h::What::RoundEnded { cause: h::RoundCause::Left }).count(), 1);
        assert_eq!(count_formed(&got, 2), usize::from(change != 52));
    }
}

#[test]
fn prepare_conflicts_distinguish_the_branch_from_an_earlier_member() {
    for (with, expected) in [("branch", h::With::Branch), ("member", h::With::Earlier)] {
        let mut world = batch(3);
        prepare(&mut world);
        world.prepare_subset(LANDING, false, &[0, 2]);
        let bytes = format!(
            r#"{{"combined":[{{"member":1,"by":"merge"}},{{"member":3,"by":"merge"}}],"conflicts":[{{"member":2,"with":"{with}","files":["conflicted"],"cut":false}}]}}"#
        );
        let mut heard = heard(None);
        heard.failure = bytes.as_bytes().into();
        heard.outcomes = top::preparation::outcomes(bytes.as_bytes(), 3).unwrap();
        landings::step_landing_heard(&mut world, LANDING, 150, heard);
        let got = events(&world, 52);
        let conflict = got.iter().position(|e| matches!(e.what, h::What::Conflict { .. })).unwrap();
        assert_eq!(got[conflict].what, h::What::Conflict { with: expected, files: 0 });
        assert!(matches!(got[conflict + 1].what, h::What::Departed { line: h::Line::Landing, .. }));
    }
}

#[test]
fn stale_turn_ends_the_old_round_before_forming_the_next() {
    let mut world = batch(3);
    proving(&mut world);
    for entry in 300..350 {
        landings::step_landing_raw(&mut world, LANDING, entry);
        if matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }) {
            break;
        }
    }
    assert!(matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }));
    world.push_file(b"main", b"outside", b"outside push");
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    for change in 51..54 {
        let got = events(&world, change);
        let stale = got.iter().position(|e| e.what == h::What::RoundEnded { cause: h::RoundCause::Stale }).unwrap();
        let formed = got
            .iter()
            .position(|e| matches!(e.what, h::What::Formed { .. }) && e.landing.expect("landing event").round == 2)
            .unwrap();
        assert!(stale < formed);
        assert_eq!(got[stale].attempt, 1);
    }
}

#[test]
fn landing_holds_and_releases_pair_for_every_member() {
    for timeout in [false, true] {
        let mut workflow = super::landing_batches::workflow(false);
        if timeout {
            workflow.latency_min = skein_lib::Duration::from_secs(100);
            workflow.latency_max = workflow.latency_min;
        }
        let mut world = batch_workflow(3, Some(workflow));
        if timeout {
            world
                .gate_policy(landing::Project { gate_timeout: skein_lib::Duration::ZERO, ..landing::Project::DEFAULT });
        } else {
            world.gate_policy(landing::Project { gate_retries: 0, ..landing::Project::DEFAULT });
            world.runner_setup_faults(1000);
        }
        for entry in 100..240 {
            if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Hold(_)) {
                break;
            }
        }
        let reason = world.landing_hold_words(LANDING).expect("landing held");
        let before: Vec<_> = (51..54).map(|change| events(&world, change)).collect();
        world.restart();
        landings::step_landing_raw(&mut world, LANDING, 300);
        for (index, change) in (51..54).enumerate() {
            assert_eq!(events(&world, change), before[index]);
        }
        let mut released = heard(None);
        released.released = true;
        landings::step_landing_heard(&mut world, LANDING, 301, released);
        for change in 51..54 {
            let got = events(&world, change);
            let held = got.iter().find(|e| matches!(e.what, h::What::Held { .. })).unwrap();
            let released = got.iter().find(|e| matches!(e.what, h::What::Released { .. })).unwrap();
            assert!(matches!(&held.what, h::What::Held { on: h::On::Landing, reason: why, .. } if *why == reason));
            assert_eq!((released.operation, released.attempt), (held.operation, 0));
            assert!(world.change(change).history.unwrap().landing_held.is_none());
        }
    }
}

#[test]
fn cancellation_and_the_last_departure_record_the_landings_end() {
    let mut world = batch(3);
    proving(&mut world);
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    world.run_for(1);
    for change in 51..54 {
        assert!(
            events(&world, change)
                .iter()
                .any(|e| e.what == h::What::LandingEnded { outcome: h::LandingOutcome::Cancelled })
        );
    }
    let before = world.landing_history(LANDING);
    world.restart();
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    world.run_for(1);
    assert_eq!(world.landing_history(LANDING), before);

    let mut world = batch(3);
    proving(&mut world);
    // The last leaver is first in stack order, not the last member of the stack.
    for task in [53, 52, 51] {
        world.event(top::Event::SettleEffects { task, root: task, ending: top::ReleaseEnding::Cancelled });
        world.run_for(1);
    }
    assert_eq!(world.landing(LANDING).landing.last_left, Some(0));
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    for change in 51..54 {
        let count = events(&world, change)
            .iter()
            .filter(|e| e.what == h::What::LandingEnded { outcome: h::LandingOutcome::Done })
            .count();
        assert_eq!(count, usize::from(change == 51));
    }
}

#[test]
fn recovery_of_partial_preparation_and_each_phase_never_repeats_history() {
    for checkpoint in ["preparing", "proving", "turn", "recording", "ended"] {
        let mut world = batch(3);
        prepare(&mut world);
        job(&mut world, 701, 1);
        world.prepare_landing(LANDING, checkpoint == "recording");
        landings::step_landing_heard(&mut world, LANDING, 150, heard(Some((701, h::JobOutcome::Done))));
        if checkpoint == "ended" {
            landings::drive_landing(&mut world, LANDING);
        } else if checkpoint != "preparing" {
            let mut reached = false;
            for entry in 300..360 {
                let decision = landings::step_landing_raw(&mut world, LANDING, entry);
                reached = match checkpoint {
                    "proving" => matches!(decision, landing::Decision::Dispatch { .. }),
                    "turn" => matches!(decision, landing::Decision::Merge { .. }),
                    "recording" => matches!(decision, landing::Decision::Record { member: 0, .. }),
                    _ => unreachable!(),
                };
                if reached {
                    break;
                }
            }
            assert!(reached, "{checkpoint}");
        }
        world.restart();
        landings::drive_landing(&mut world, LANDING);
        for change in 51..54 {
            let got = events(&world, change);
            assert_eq!(count_formed(&got, 1), 1);
            assert_eq!(got.iter().filter(|e| matches!(e.what, h::What::Job { .. })).count(), 1);
            assert_eq!(got.iter().filter(|e| matches!(e.what, h::What::JobEnded { .. })).count(), 1);
            assert_eq!(got.iter().filter(|e| matches!(e.what, h::What::RoundEnded { .. })).count(), 1);
            assert_eq!(got.iter().filter(|e| matches!(e.what, h::What::LandingEnded { .. })).count(), 1);
        }
    }
    let mut world = batch(3);
    prepare(&mut world);
    job(&mut world, 701, 1);
    world.prepare_subset(LANDING, false, &[0, 1]);
    world.restart();
    let mut heard = heard(Some((701, h::JobOutcome::Failed)));
    heard.preparation = landing::Preparation::Failed;
    landings::step_landing_heard(&mut world, LANDING, 150, heard);
    assert!(world.landing(LANDING).landing.members[1].candidate.is_some());
    for change in 51..54 {
        assert_eq!(events(&world, change).iter().filter(|e| matches!(e.what, h::What::Job { .. })).count(), 1);
    }
}

#[test]
fn a_cut_ends_only_its_suffix_and_later_rounds_exclude_the_turned_prefix() {
    let mut world = super::landing_batches::red_batch(52);
    landings::drive_landing(&mut world, LANDING);
    let first = events(&world, 51);
    assert_eq!(
        first
            .iter()
            .filter(|e| matches!(e.what, h::What::RoundEnded { .. }))
            .map(|e| e.what.clone())
            .collect::<Vec<_>>(),
        vec![h::What::RoundEnded { cause: h::RoundCause::Turned }]
    );
    assert_eq!(count_formed(&first, 2), 0);
    assert!(!first.iter().any(|e| matches!(e.what, h::What::LandingEnded { .. })));
    let third = events(&world, 53);
    assert!(third.iter().any(|e| e.what == h::What::RoundEnded { cause: h::RoundCause::Cut } && e.attempt == 1));
    let formed =
        third.iter().find(|e| matches!(e.what, h::What::Formed { .. }) && e.attempt == 2).expect("suffix round");
    let h::What::Formed { members, place } = &formed.what else { unreachable!() };
    assert_eq!((*place, members.iter().map(|m| m.change).collect::<Vec<_>>()), (1, vec![53]));
    assert!(third.iter().any(|e| e.what == h::What::LandingEnded { outcome: h::LandingOutcome::Done }));
}

#[test]
fn a_stored_job_end_survives_restart_before_readback_and_retry_closes_loss() {
    let mut world = batch(3);
    prepare(&mut world);
    job(&mut world, 701, 1);
    // A new durable claim ends the previous uncertain attempt before starting the next.
    job(&mut world, 701, 2);
    world.prepare_landing(LANDING, false);
    let ended =
        h::JobStep { task: 701, attempt: 2, job: h::Job::Prepare, outcome: Some(h::JobOutcome::Done), head: None };
    world.event(top::Event::JobHistory { task: LANDING, job: ended });
    world.run_for(1);
    assert!(!world.landing(LANDING).job.as_ref().unwrap().written);
    world.restart();
    landings::step_landing_raw(&mut world, LANDING, 150);
    for change in 51..54 {
        let got = events(&world, change);
        let jobs: Vec<_> =
            got.iter().filter(|e| matches!(e.what, h::What::Job { .. } | h::What::JobEnded { .. })).collect();
        assert_eq!(jobs.len(), 4);
        assert_eq!(
            (jobs[1].what.clone(), jobs[1].attempt),
            (h::What::JobEnded { job: h::Job::Prepare, outcome: h::JobOutcome::Lost }, 1)
        );
        assert_eq!((jobs[2].what.clone(), jobs[2].attempt), (h::What::Job { job: h::Job::Prepare, task: 701 }, 2));
        assert_eq!(
            jobs[3].head,
            world.landing(LANDING).landing.members[usize::try_from(change - 51).unwrap()].candidate
        );
    }
    let before = world.landing_history(LANDING);
    world.restart();
    world.event(top::Event::JobHistory { task: LANDING, job: ended });
    world.run_for(1);
    landings::step_landing_raw(&mut world, LANDING, 151);
    assert_eq!(world.landing_history(LANDING), before);
}
