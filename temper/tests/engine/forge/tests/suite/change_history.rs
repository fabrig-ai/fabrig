//! Change effects are committed once at their deciding steps (domain/landing-history.md, 3.2).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge::{self as top, history as h};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_tiers as tiers;
use temper_engine_forge_world::{
    REPO, World,
    changes::{change_row, change_step},
    landings, translate,
};
const TASK: u64 = 51;
const CHILD: u64 = 71;
const BRANCH: &[u8] = b"temper/51";

fn effects(world: &World) -> Vec<h::Event> {
    world
        .history(TASK)
        .into_iter()
        .filter(|event| {
            matches!(
                event.what,
                h::What::Update { .. }
                    | h::What::Updated { .. }
                    | h::What::Conflict { .. }
                    | h::What::Asked { .. }
                    | h::What::Answered { .. }
                    | h::What::Pushed
                    | h::What::Turn { .. }
                    | h::What::Landed
                    | h::What::Recorded { .. }
                    | h::What::After { .. }
            )
        })
        .collect()
}
fn kinds(world: &World) -> Vec<h::What> {
    effects(world).into_iter().map(|event| event.what).collect()
}
fn watch_pull(world: &mut World, pull: u64) {
    world.event(top::Event::Client(skein_forge_client::Event::Keep {
        owner: skein_lib::Token::new(444),
        watches: Box::new([skein_forge_client::Watch {
            resource: skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Pull(pull) },
            participating: false,
        }]),
    }));
}
fn queued() -> World {
    let mut world = World::new(401);
    world.adopt();
    world.produce(BRANCH);
    world.status(BRANCH, raw::Check::Passed);
    let pull = world.open_pull(BRANCH, b"main");
    let head = translate::commit(world.branch(BRANCH));
    let mut row = change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = change::State::Queued { head, ready_since: world.wall() };
    world.event(top::Event::Change { row });
    watch_pull(&mut world, pull);
    world.run_for(1);
    world
}
fn step(world: &mut World, entry: u64) -> change::Decision {
    change_step(world, entry, change::Status::Unknown, Box::new([]))
}
fn result(world: &mut World, failed: bool, passed: bool) {
    world.event(top::Event::DelegateResult {
        task: TASK,
        child: CHILD,
        status: if passed { change::Status::Passed } else { change::Status::Failed },
        failed,
        cancelled: false,
        no_change: false,
        words: b"delegate result".as_slice().into(),
    });
}
fn verification(world: &mut World, commit: api::Commit, blame: Option<tiers::Blame>, repaired: bool) {
    world.event(top::Event::Restore {
        record: top::Stored::Verification(top::Verification {
            repository: REPO,
            commit,
            changes: Box::new([TASK]),
            tiers: Box::new([top::TierVerification {
                tier: b"browser".as_slice().into(),
                verdict: blame.map(|blame| top::TierVerdict { run: 1, blame }),
                repaired: repaired.then_some(top::TierRepair { run: 2, tip: commit, by: None }),
            }]),
        }),
    });
}
fn merged(world: &mut World) -> api::Commit {
    assert!(matches!(step(world, 10), change::Decision::Effect(change::Effect::Merge { .. })));
    translate::commit(world.branch(b"main"))
}
fn name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    }
}
fn report(world: &mut World, lost: bool) {
    let name = name();
    world.event(top::Event::Names { task: TASK, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: TASK, resource: name.clone(), from: None });
    world.event(top::Event::Claim {
        task: CHILD,
        attempt: 1,
        writes: Box::new([name.clone()]),
        designs: Box::new([]),
        holders: Box::new([TASK]),
    });
    if lost {
        world.event(top::Event::Lost { task: CHILD, attempt: 1 });
    } else {
        let commit = translate::commit(world.branch(BRANCH));
        world.event(top::Event::Answered { task: CHILD, attempt: 1, pushed: Box::new([(name, commit)]) });
    }
    world.run_for(1);
}

#[test]
fn queue_merge_records_the_merge_base_and_clearing_in_order() {
    let mut world = queued();
    let head = translate::commit(world.branch(BRANCH));
    let base = translate::commit(world.branch(b"main"));
    let merge = merged(&mut world);
    world.run_for(10);
    verification(&mut world, merge, Some(tiers::Blame::Cleared), false);
    assert!(matches!(step(&mut world, 11), change::Decision::Finish { .. }));
    assert_eq!(
        kinds(&world),
        [
            h::What::Pushed,
            h::What::Turn { members: 1 },
            h::What::Landed,
            h::What::Recorded { state: h::RecordedState::Merged },
            h::What::After { verdict: h::AfterVerdict::Cleared }
        ]
    );
    let events = effects(&world);
    assert_eq!(events[0].head, Some(head));
    assert_eq!((events[1].head, events[1].base, events[1].operation), (Some(merge), Some(base), 10));
    assert_eq!(events[2].head, Some(merge));
    assert_eq!(events[3].operation, world.change(TASK).pull.expect("pull"));
    let before = effects(&world);
    world.restart();
    step(&mut world, 12);
    assert_eq!(effects(&world), before);
}

#[test]
fn a_lost_update_pairs_by_outbox_entry_across_restarts() {
    let mut world = queued();
    world.push_file(b"main", b"base", b"new base");
    let base = translate::commit(world.branch(b"main"));
    world.land_writes_late(Duration::from_secs(3));
    assert!(matches!(step(&mut world, 20), change::Decision::Effect(change::Effect::Update { .. })));
    world.restart();
    world.run_for(12);
    world.answer_writes();
    world.status(BRANCH, raw::Check::Pending);
    step(&mut world, 21);
    let updates: Vec<_> = effects(&world)
        .into_iter()
        .filter(|event| matches!(event.what, h::What::Update { .. } | h::What::Updated { .. }))
        .collect();
    assert_eq!(updates.len(), 2);
    assert_eq!(updates[0].what, h::What::Update { style: api::UpdateStyle::Merge });
    assert_eq!(updates[1].what, h::What::Updated { outcome: h::UpdateOutcome::Updated });
    assert_eq!((updates[0].operation, updates[1].operation), (20, 20));
    assert_eq!((updates[0].base, updates[1].base), (Some(base), Some(base)));
    assert_eq!(updates[1].head, Some(translate::commit(world.branch(BRANCH))));
    let before = effects(&world);
    world.restart();
    step(&mut world, 22);
    world.run_for(10);
    assert_eq!(effects(&world), before);
}

#[test]
fn a_retried_update_does_not_repeat_its_start_or_settlement() {
    let mut world = queued();
    world.push_file(b"main", b"base", b"base moved");
    world.land_writes_late(Duration::from_secs(40));
    assert!(matches!(step(&mut world, 25), change::Decision::Effect(change::Effect::Update { .. })));
    world.answer_writes();
    world.run_for(13);
    assert_eq!(world.writes(), 2, "lost physical write and retry");
    let updates: Vec<_> = effects(&world)
        .into_iter()
        .filter(|event| matches!(event.what, h::What::Update { .. } | h::What::Updated { .. }))
        .collect();
    assert_eq!(
        updates.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [h::What::Update { style: api::UpdateStyle::Merge }, h::What::Updated { outcome: h::UpdateOutcome::Updated }]
    );
    assert!(updates.iter().all(|e| e.operation == 25));
    let before = effects(&world);
    world.restart();
    world.run_for(45);
    assert_eq!(effects(&world), before, "late first copy changes no step");
}

#[test]
fn an_unfollowed_change_records_verified_when_its_ending_is_kept() {
    let mut world = queued();
    let merge = merged(&mut world);
    world.event(top::Event::ChangeEnding { task: TASK, ending: top::Ending::Landed });
    let after: Vec<_> =
        effects(&world).into_iter().filter(|event| matches!(event.what, h::What::After { .. })).collect();
    assert_eq!(
        after.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [h::What::After { verdict: h::AfterVerdict::Verified }]
    );
    assert_eq!(after[0].head, Some(merge));
    let before = effects(&world);
    world.restart();
    world.event(top::Event::ChangeEnding { task: TASK, ending: top::Ending::Landed });
    assert_eq!(effects(&world), before);
}

#[test]
fn a_conflicting_update_and_its_resolution_have_distinct_intervals() {
    let mut world = queued();
    world.push(b"main", b"conflicting base");
    assert!(matches!(step(&mut world, 30), change::Decision::Effect(change::Effect::Update { .. })));
    let base = translate::commit(world.branch(b"main"));
    assert_eq!(step(&mut world, 31), change::Decision::Delegate(change::Delegate::Resolve { base }));
    world.event(top::Event::Delegated { task: TASK, child: CHILD, kind: change::Delegate::Resolve { base } });
    world.restart();
    world.event(top::Event::Delegated { task: TASK, child: CHILD, kind: change::Delegate::Resolve { base } });
    world.resolve(BRANCH, b"settled both sides");
    let head = translate::commit(world.branch(BRANCH));
    report(&mut world, false);
    result(&mut world, false, true);
    let before = effects(&world);
    world.restart();
    result(&mut world, false, true);
    assert_eq!(effects(&world), before);
    let events: Vec<_> = effects(&world).into_iter().filter(|event| !matches!(event.what, h::What::Pushed)).collect();
    assert_eq!(
        events.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [
            h::What::Update { style: api::UpdateStyle::Merge },
            h::What::Updated { outcome: h::UpdateOutcome::Conflict },
            h::What::Conflict { with: h::With::Branch, files: 0 },
            h::What::Asked { ask: h::Ask::Resolution, task: CHILD, gate: None },
            h::What::Answered { ask: h::Ask::Resolution, outcome: h::Answer::Pushed },
        ]
    );
    assert_eq!((events[0].operation, events[1].operation), (30, 30));
    assert_eq!(events[3].base, Some(base));
    assert_eq!((events[3].operation, events[4].operation), (CHILD, CHILD));
    assert_eq!(events[4].head, Some(head));
}

#[test]
fn repair_and_judgement_results_record_their_own_outcomes_once() {
    for (kind, ask, gate, failed, passed, outcome) in [
        (change::Delegate::Repair(change::Repair::Ci), h::Ask::Repair, None, false, true, h::Answer::Pushed),
        (change::Delegate::Repair(change::Repair::Gate(9)), h::Ask::Repair, Some(9), true, false, h::Answer::Failed),
        (
            change::Delegate::Gate { number: 9, head: [0; 32] },
            h::Ask::Judgement,
            Some(9),
            false,
            true,
            h::Answer::Passed,
        ),
        (
            change::Delegate::Gate { number: 9, head: [0; 32] },
            h::Ask::Judgement,
            Some(9),
            false,
            false,
            h::Answer::Refused,
        ),
    ] {
        let mut world = queued();
        let head = translate::commit(world.branch(BRANCH));
        let kind = if matches!(kind, change::Delegate::Gate { .. }) {
            change::Delegate::Gate { number: 9, head }
        } else {
            kind
        };
        let mut row = world.change(TASK).clone();
        row.change.state = match kind {
            change::Delegate::Repair(why) => change::State::Repairing { requested: false, why },
            change::Delegate::Gate { .. } => {
                row.change.gates = Box::new([change::Gate {
                    number: 9,
                    kind: change::GateKind::Agent,
                    blocking: true,
                    freshness: change::Freshness::Exact,
                    eager: false,
                }]);
                change::State::Gating { head, asked: Box::new([]) }
            }
            _ => unreachable!("repair or judgement"),
        };
        world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
        if kind == change::Delegate::Repair(change::Repair::Ci) {
            world.status(BRANCH, raw::Check::Failed);
        }
        assert_eq!(step(&mut world, 70), change::Decision::Delegate(kind));
        world.event(top::Event::Delegated { task: TASK, child: CHILD, kind });
        if outcome == h::Answer::Pushed {
            world.push(BRANCH, b"repair");
            report(&mut world, false);
        }
        result(&mut world, failed, passed);
        let answers: Vec<_> = effects(&world)
            .into_iter()
            .filter(|e| matches!(e.what, h::What::Asked { .. } | h::What::Answered { .. }))
            .collect();
        assert_eq!(
            answers.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
            [h::What::Asked { ask, task: CHILD, gate }, h::What::Answered { ask, outcome }]
        );
        assert_eq!((answers[0].operation, answers[1].operation), (CHILD, CHILD));
        if outcome == h::Answer::Pushed {
            assert_eq!(answers[1].head, Some(translate::commit(world.branch(BRANCH))));
        }
        let before = effects(&world);
        world.restart();
        result(&mut world, failed, passed);
        assert_eq!(effects(&world), before);
    }
}

#[test]
fn a_lost_push_found_again_and_restarted_is_written_once_per_head() {
    let mut world = queued();
    world.event(top::Event::Delegated { task: TASK, child: CHILD, kind: change::Delegate::Produce });
    report(&mut world, false);
    world.push(BRANCH, b"saved work");
    report(&mut world, true);
    let before = effects(&world);
    let pushes: Vec<_> = before.iter().filter(|e| e.what == h::What::Pushed).collect();
    assert_eq!(pushes.len(), 2);
    assert_ne!(pushes[0].head, pushes[1].head);
    report(&mut world, true);
    assert_eq!(effects(&world), before, "a reread accepting the same head is no push");
    world.restart();
    world.event(top::Event::Lost { task: CHILD, attempt: 1 });
    world.run_for(10);
    assert_eq!(effects(&world), before);
}

#[test]
fn a_turn_member_lands_at_its_prefix_with_its_round_and_unit() {
    let mut world = landings::first_landing();
    let pull = world.change(TASK).pull.expect("member pull");
    watch_pull(&mut world, pull);
    world.run_for(10);
    let landing = world.landing(landings::LANDING);
    let member = &landing.landing.members[0];
    let expected = h::InLanding { task: landings::LANDING, round: landing.landing.preparation, unit: member.unit };
    let events: Vec<_> =
        effects(&world).into_iter().filter(|e| matches!(e.what, h::What::Landed | h::What::Recorded { .. })).collect();
    assert_eq!(
        events.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
        [h::What::Recorded { state: h::RecordedState::Merged }, h::What::Landed]
    );
    assert_eq!(events[1].head, member.landed);
    assert_eq!(events[1].landing, Some(expected));
    let before = effects(&world);
    world.restart();
    step(&mut world, 900);
    assert_eq!(effects(&world), before);
}

#[test]
fn after_landing_records_red_then_cleared_and_timeout_uncleared() {
    for timeout in [false, true] {
        let mut world = queued();
        let merge = merged(&mut world);
        verification(&mut world, merge, None, false);
        step(&mut world, 40);
        if timeout {
            world.run_for(7201);
        } else {
            verification(&mut world, merge, Some(tiers::Blame::Owns(tiers::Failures::Many)), false);
            step(&mut world, 41);
            world.restart();
            step(&mut world, 42);
            verification(&mut world, merge, Some(tiers::Blame::Owns(tiers::Failures::Many)), true);
        }
        step(&mut world, 43);
        let after: Vec<_> = effects(&world).into_iter().filter(|e| matches!(e.what, h::What::After { .. })).collect();
        assert_eq!(
            after.iter().map(|e| e.what.clone()).collect::<Vec<_>>(),
            if timeout {
                vec![h::What::After { verdict: h::AfterVerdict::Uncleared }]
            } else {
                vec![
                    h::What::After { verdict: h::AfterVerdict::Red },
                    h::What::After { verdict: h::AfterVerdict::Cleared },
                ]
            }
        );
        assert!(after.iter().all(|e| e.head == Some(merge)));
        let before = effects(&world);
        world.restart();
        step(&mut world, 44);
        assert_eq!(effects(&world), before);
    }
}
