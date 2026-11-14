//! Lost updates and their copies (domain/forge/README.md, 6 and 10;
//! domain/landing.md, 5.2 and 16; jig/docs/design/domain/connectors.md, 15).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    LIMITS, World,
    changes::{change_row, change_step, change_step_with_release},
    landings::{self, CHANGE, LANDING},
    translate,
};

const BRANCH: &[u8] = b"temper/51";

fn queued() -> World {
    let mut world = World::new(17);
    world.adopt();
    world.produce(BRANCH);
    let pull = world.open_pull(BRANCH, b"main");
    let head = translate::commit(world.branch(BRANCH));
    let mut row = change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = change::State::Queued { head, ready_since: world.wall() };
    world.event(top::Event::Change { row });
    world.push_file(b"main", b"base", b"base moved");
    world
}

fn step(world: &mut World, entry: u64) -> change::Decision {
    let decision = change_step(world, entry, change::Status::Passed, Box::new([]));
    assert!(!matches!(decision, change::Decision::Hold(_)), "unexpected hold: {decision:?}");
    assert!(
        world.seen().iter().all(|request| !matches!(
            request,
            top::Request::Outcome { outcome: skein_forge_client::Outcome::Held, .. }
        )),
        "an update's recovery must hold no one"
    );
    decision
}

fn finish(world: &mut World, first: u64) {
    for entry in first..first + 30 {
        if matches!(step(world, entry), change::Decision::Finish { .. }) {
            assert!(matches!(world.change(CHANGE).change.state, change::State::Landed { .. }));
            return;
        }
    }
    panic!("change did not land: {:?}", world.change(CHANGE));
}

fn updates(world: &mut World, style: api::UpdateStyle) -> usize {
    assert_eq!(
        top::effect_shape(11),
        Some((top::EffectForm::Transition, top::Recovery::Idempotent)),
        "the update kind declares the class this world exercises"
    );
    world
        .take_calls()
        .iter()
        .filter(|op| {
            matches!(
                op, api::Op::Write(api::Write::Update { style: sent, .. }) if *sent == style
            )
        })
        .count()
}

#[test]
fn a_lost_update_is_found_under_a_queue() {
    let mut world = queued();
    let before = world.branch(BRANCH);
    world.land_writes_late(Duration::from_secs(3));
    assert!(matches!(step(&mut world, 1), change::Decision::Effect(change::Effect::Update { .. })));
    world.run_for(12);
    world.answer_writes();
    world.status(BRANCH, raw::Check::Pending);
    assert!(matches!(step(&mut world, 2), change::Decision::Wait { .. }));
    let head = translate::commit(world.branch(BRANCH));
    assert_ne!(world.branch(BRANCH), before);
    assert!(matches!(world.change(CHANGE).change.state, change::State::Checking { head: checked } if checked == head));
    assert_eq!(world.change(CHANGE).pending, None);
    assert_eq!(world.writes(), 1);
    assert_eq!(updates(&mut world, api::UpdateStyle::Merge), 1);
}

#[test]
fn a_lost_update_is_made_again_and_lands_under_a_queue() {
    let mut world = queued();
    world.land_writes_late(Duration::from_secs(40));
    assert!(matches!(step(&mut world, 1), change::Decision::Effect(change::Effect::Update { .. })));
    world.answer_writes();
    world.run_for(13);
    assert_eq!(world.writes(), 2, "the lost copy and its answered retry");
    finish(&mut world, 2);
    assert_eq!(updates(&mut world, api::UpdateStyle::Merge), 2);
    assert_eq!(world.writes(), 3, "two updates and the merge");
    let landed = world.branch(b"main");
    let candidate = world.branch(BRANCH);
    world.run_for(45);
    assert_eq!(world.branch(b"main"), landed, "the old copy cannot change the landed head");
    assert_eq!(world.branch(BRANCH), candidate, "the merge copy changes nothing once its base is held");
}

fn lost_prepare(found: bool) {
    let mut world = landings::turn_world();
    world.produce(BRANCH);
    world.push_file(b"main", b"base", b"main moved");
    landings::ready_change(&mut world, CHANGE, BRANCH);
    let original = world.branch(BRANCH);
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    assert_eq!(landings::step_landing_raw(&mut world, LANDING, 100), landing::Decision::None);
    assert!(matches!(landings::step_landing_raw(&mut world, LANDING, 101), landing::Decision::Prepare { .. }));
    if found {
        world.prepare_landing(LANDING, false);
    }
    world.restart();
    world.run_for(1);
    let next = landings::step_landing_raw(&mut world, LANDING, 102);
    assert_eq!(
        matches!(next, landing::Decision::Prepare { .. }),
        !found,
        "read the landing branch before making another job"
    );
    if !found {
        world.prepare_landing(LANDING, false);
    }
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    assert_eq!(world.branch(BRANCH), original);
    assert_eq!(updates(&mut world, api::UpdateStyle::Rebase), 0);
}
#[test]
fn a_lost_prepare_is_found_and_the_landing_goes_on() {
    lost_prepare(true);
}
#[test]
fn a_lost_prepare_is_made_again_and_the_landing_goes_on() {
    lost_prepare(false);
}
#[test]
fn a_job_finding_the_branch_already_there_is_undelivered_and_the_read_takes_it() {
    let mut world = landings::turn_world();
    world.produce(BRANCH);
    landings::ready_change(&mut world, CHANGE, BRANCH);
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    landings::step_landing_raw(&mut world, LANDING, 100);
    assert!(matches!(landings::step_landing_raw(&mut world, LANDING, 101), landing::Decision::Prepare { .. }));
    let candidate = world.prepare_landing(LANDING, false);
    let branch = world.landing(LANDING).landing.round.branch.clone();
    assert_eq!(
        world.external(raw::Op::Git(raw::Git::PushNew { branch, commit: candidate }), 1),
        raw::Answer::PushNew { existing: Some(candidate) }
    );
    assert!(!matches!(landings::step_landing_raw(&mut world, LANDING, 102), landing::Decision::Prepare { .. }));
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.branch(b"main"), candidate);
}

#[test]
fn an_update_lost_for_good_holds_at_its_count_and_a_release_lands_it() {
    let mut world = queued();
    world.lose_writes();
    let mut held = false;
    for entry in 1..=u64::from(LIMITS.change_policy.updates) + 1 {
        let decision = change_step(&mut world, entry, change::Status::Passed, Box::new([]));
        if let change::Decision::Hold(why) = decision {
            assert_eq!(why, change::Hold::Updates);
            held = true;
            break;
        }
        world.run_for(35);
    }
    assert!(
        held,
        "the update count bounds writes lost for good: {:?}; writes {}",
        world.change(CHANGE),
        world.writes()
    );
    assert!(matches!(world.change(CHANGE).change.state, change::State::Held { why: change::Hold::Updates, .. }));
    assert_eq!(world.change(CHANGE).change.updates, LIMITS.change_policy.updates);
    assert_eq!(world.writes(), LIMITS.change_policy.updates * LIMITS.client.write_attempts);
    let account = &world.change(CHANGE).effect_account.as_ref().expect("last update's account").words;
    assert!(String::from_utf8_lossy(account).contains("update pull request: forge timed out"));
    world.answer_writes();
    world.event(top::Event::ReleaseChange { task: CHANGE });
    let decision = change_step_with_release(&mut world, 100, change::Status::Passed, Box::new([]), true);
    assert!(!matches!(decision, change::Decision::Hold(_)));
    finish(&mut world, 101);
}

#[test]
fn an_update_whose_base_went_on_while_lost_takes_the_new_tip() {
    let mut world = queued();
    let first_tip = world.branch(b"main");
    world.land_writes_late(Duration::from_secs(3));
    assert!(matches!(step(&mut world, 1), change::Decision::Effect(change::Effect::Update { .. })));
    world.run_for(4);
    world.push_file(b"main", b"next", b"next base tip");
    let next_tip = world.branch(b"main");
    world.answer_writes();
    world.run_for(12);
    finish(&mut world, 2);
    assert_eq!(world.change(CHANGE).change.updates, 2);
    assert_eq!(updates(&mut world, api::UpdateStyle::Merge), 2);
    assert!(contains(&world, first_tip, world.branch(BRANCH)));
    assert!(contains(&world, next_tip, world.branch(BRANCH)));
}

fn hold_branch(world: &mut World) {
    let name = top::Name {
        forge: temper_engine_forge_world::REPO.forge,
        repository: temper_engine_forge_world::REPO.repository,
        what: top::What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"51".as_slice())])),
    };
    world.event(top::Event::Names { task: CHANGE, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: CHANGE, resource: name.clone(), from: None });
    world.run_for(1);
    world.event(top::Event::Claim {
        designs: Box::new([]),
        task: CHANGE,
        attempt: 1,
        writes: Box::new([name.clone()]),
        holders: Box::new([]),
    });
    let head = translate::commit(world.branch(BRANCH));
    world.event(top::Event::Answered { task: CHANGE, attempt: 1, pushed: Box::new([(name, head, true)]) });
    world.run_for(1);
}

fn copy_world(style: api::UpdateStyle) -> (World, u64) {
    let mut world = match style {
        api::UpdateStyle::Merge => queued(),
        api::UpdateStyle::Rebase => {
            let mut world = landings::turn_world();
            world.produce(BRANCH);
            world.push_file(b"main", b"base", b"main moved");
            landings::ready_change(&mut world, CHANGE, BRANCH);
            let row = landings::landing_row(world.change(CHANGE));
            world.event(top::Event::Landing { row });
            world
        }
    };
    let original = world.branch(BRANCH);
    hold_branch(&mut world);
    world.land_writes_late(Duration::from_secs(40));
    match style {
        api::UpdateStyle::Merge => {
            assert!(matches!(step(&mut world, 10), change::Decision::Effect(change::Effect::Update { .. })));
        }
        api::UpdateStyle::Rebase => {
            let mut sent = false;
            for position in 0..10 {
                landings::step_landing(&mut world, LANDING, 10 + position * 2);
                if matches!(
                    step(&mut world, 11 + position * 2),
                    change::Decision::Effect(change::Effect::Prepare { .. })
                ) {
                    sent = true;
                    break;
                }
            }
            assert!(sent);
        }
    }
    world.answer_writes();
    world.run_for(13);
    assert_eq!(world.writes(), 2);
    (world, original)
}

fn contains(world: &World, base: u64, head: u64) -> bool {
    let raw::Answer::Comparison { contains_base, .. } =
        world.inspect(&raw::Read::Compare { base, head, page: 1, limit: 16 })
    else {
        panic!("comparison")
    };
    contains_base
}

#[test]
fn a_copy_landing_after_the_change_moved_on_holds_and_lands_nothing_unchecked() {
    {
        let style = api::UpdateStyle::Merge;
        let (mut world, original) = copy_world(style);
        let candidate = world.branch(BRANCH);
        if style == api::UpdateStyle::Merge {
            world.status(BRANCH, raw::Check::Pending);
        }
        step(&mut world, 100);
        assert_eq!(world.change(CHANGE).pending, None, "the update's slot is free");
        assert!(!matches!(
            world.change(CHANGE).change.state,
            change::State::Updating { .. } | change::State::Preparing { .. }
        ));
        let pushed = match style {
            api::UpdateStyle::Merge => {
                world.push_file(b"main", b"next", b"next base tip");
                None
            }
            api::UpdateStyle::Rebase => {
                world.push_as(BRANCH, b"later", b"a later push", 1);
                let pushed = world.branch(BRANCH);
                // The fake reads at landing. An outside hand resets the branch
                // to its pre-prepare head so the copy can show a dropped push.
                world.external(raw::Op::Write(raw::Write::DeleteBranch { branch: BRANCH.into() }), 1);
                world.external(raw::Op::Write(raw::Write::CreateBranch { branch: BRANCH.into(), commit: original }), 1);
                world.external(
                    raw::Op::Write(raw::Write::Reopen { number: world.change(CHANGE).pull.expect("pull") }),
                    1,
                );
                Some(pushed)
            }
        };
        let main = world.branch(b"main");
        world.run_for(30);
        let moved = world.branch(BRANCH);
        assert_ne!(candidate, moved);
        assert_eq!(contains(&world, candidate, moved), style == api::UpdateStyle::Merge);
        if let Some(pushed) = pushed {
            assert!(!contains(&world, pushed, moved), "the rewrite drops the later push");
        }
        assert_eq!(
            change_step(&mut world, 101, change::Status::Passed, Box::new([])),
            change::Decision::Hold(change::Hold::BranchMoved)
        );
        assert_eq!(world.change(CHANGE).pending, None);
        let before = world.take_calls();
        assert!(
            !before.iter().any(|op| matches!(op, api::Op::Write(api::Write::Merge { .. }))),
            "no unchecked head is merged"
        );
        assert_eq!(world.branch(b"main"), main, "the copy lands nothing into main");
        world.event(top::Event::ReleaseChange { task: CHANGE });
        if style == api::UpdateStyle::Merge {
            world.status(BRANCH, raw::Check::Pending);
        }
        let decision = change_step_with_release(&mut world, 102, change::Status::Passed, Box::new([]), true);
        assert!(!matches!(decision, change::Decision::Hold(_)));
        assert_eq!(world.change(CHANGE).change.last_head, Some(translate::commit(moved)));
        if style == api::UpdateStyle::Merge {
            assert!(
                matches!(world.change(CHANGE).change.state, change::State::Checking { head } if head == translate::commit(moved))
            );
            assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Merge { .. }))));
            world.status(BRANCH, raw::Check::Passed);
            finish(&mut world, 103);
        } else {
            landings::drive_landing(&mut world, LANDING);
            assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
        }
        assert!(world.take_calls().iter().any(
            |op| matches!(op, api::Op::Write(api::Write::Merge { head, .. }) if *head == translate::commit(moved))
        ));
        assert_eq!(world.branch(BRANCH), moved, "release keeps the observed head");
    }
}
