//! Acceptance of a run's heads, apart from observing its branch (domain/landing-history.md, 3.2).
use skein_forge::api;
use skein_lib::Token;
use temper_engine_domain_forge::{self as top, history as h};
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes::change_row, landings, translate};

const TASK: u64 = 7;
const CHILD: u64 = 71;
const BRANCH: &[u8] = b"temper/7";

fn name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into()])),
    }
}
fn deliveries(world: &World) -> Vec<h::Event> {
    world.history(TASK).into_iter().filter(|event| matches!(event.what, h::What::Delivered { .. })).collect()
}
fn producer(kind: change::Delegate) -> World {
    let mut world = World::new(407);
    world.adopt();
    world.produce(BRANCH);
    let mut row = change_row();
    row.task = TASK;
    row.change.task = TASK;
    row.branch = BRANCH.into();
    world.event(top::Event::Change { row });
    world.event(top::Event::Delegated { task: TASK, child: CHILD, kind });
    world.event(top::Event::Client(skein_forge_client::Event::Keep {
        owner: Token::new(444),
        watches: Box::new([skein_forge_client::Watch {
            resource: skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(BRANCH.into()),
            },
            participating: false,
        }]),
    }));
    world.run_for(1);
    world
}
fn claim(world: &mut World, attempt: u64) {
    let name = name();
    world.event(top::Event::Names { task: TASK, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: TASK, resource: name.clone(), from: None });
    world.event(top::Event::Claim {
        task: CHILD,
        attempt,
        writes: Box::new([name]),
        designs: Box::new([]),
        holders: Box::new([TASK]),
    });
}
fn answer(world: &mut World, attempt: u64, head: api::Commit, checked: bool) {
    world.event(top::Event::Answered { task: CHILD, attempt, pushed: Box::new([(name(), head, checked)]) });
}

#[test]
fn a_checked_answer_is_delivered_after_its_observed_push_only_once() {
    let mut world = producer(change::Delegate::Produce);
    let head = translate::commit(world.branch(BRANCH));
    let pushed: Vec<_> = world.history(TASK).into_iter().filter(|event| event.what == h::What::Pushed).collect();
    assert_eq!(pushed.len(), 1, "the branch read saw the head first");
    assert_eq!(pushed[0].head, Some(head));
    // The procedure may have read and retained this head before the attempt answers.
    let mut row = world.change(TASK).clone();
    row.change.last_head = Some(head);
    world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
    claim(&mut world, 1);
    answer(&mut world, 1, head, true);
    let delivered = deliveries(&world);
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        (delivered[0].head, &delivered[0].what, delivered[0].operation, delivered[0].attempt),
        (Some(head), &h::What::Delivered { checked: true }, delivered[0].number, 0)
    );
    assert!(pushed[0].number < delivered[0].number);
    answer(&mut world, 1, head, true);
    assert_eq!(deliveries(&world), delivered);
    world.restart();
    answer(&mut world, 1, head, true);
    assert_eq!(deliveries(&world), delivered);
    claim(&mut world, 2);
    answer(&mut world, 2, head, true);
    assert_eq!(deliveries(&world), delivered, "a later answer accepting the same head is no delivery");
    world.push(BRANCH, b"new delivery");
    let next = translate::commit(world.branch(BRANCH));
    claim(&mut world, 3);
    answer(&mut world, 3, next, true);
    let delivered = deliveries(&world);
    assert_eq!(delivered.len(), 2);
    assert_eq!(delivered[1].head, Some(next));
}

#[test]
fn an_unchecked_save_keeps_its_delivery_status_after_restart() {
    let mut world = producer(change::Delegate::Produce);
    let head = translate::commit(world.branch(BRANCH));
    claim(&mut world, 1);
    answer(&mut world, 1, head, false);
    let delivered = deliveries(&world);
    assert_eq!(delivered.len(), 1);
    assert_eq!((delivered[0].head, &delivered[0].what), (Some(head), &h::What::Delivered { checked: false }));
    world.restart();
    claim(&mut world, 2);
    answer(&mut world, 2, head, true);
    assert_eq!(deliveries(&world), delivered, "the same head is not delivered again even with different evidence");
}

#[test]
fn resolutions_and_repairs_deliver_their_heads_too() {
    for kind in [change::Delegate::Resolve { base: [1; 32] }, change::Delegate::Repair(change::Repair::Ci)] {
        let mut world = producer(kind);
        let head = translate::commit(world.branch(BRANCH));
        claim(&mut world, 1);
        answer(&mut world, 1, head, true);
        world.event(top::Event::DelegateResult {
            task: TASK,
            child: CHILD,
            status: change::Status::Passed,
            failed: false,
            cancelled: false,
            no_change: false,
            words: b"done".as_slice().into(),
        });
        let delivered = deliveries(&world);
        assert_eq!(delivered.len(), 1);
        assert_eq!((delivered[0].head, &delivered[0].what), (Some(head), &h::What::Delivered { checked: true }));
    }
}

#[test]
fn a_turned_members_change_event_names_its_round_or_the_older_rows_preparation() {
    for (round, expected) in [(Some(1), 1), (None, 2)] {
        let mut world = landings::first_landing();
        let mut row = world.change(landings::CHANGE).clone();
        row.change.landing = Some(landings::LANDING);
        world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
        let mut landing = world.landing(landings::LANDING).clone();
        landing.landing.preparation = 2;
        let member = &mut landing.landing.members[0];
        assert!(member.landed.is_some(), "the prefix member already turned");
        member.round = round;
        let unit = member.unit;
        world.event(top::Event::Restore { record: top::Stored::Landing(landing) });
        world.event(top::Event::Delegated {
            task: landings::CHANGE,
            child: CHILD,
            kind: change::Delegate::Repair(change::Repair::Ci),
        });
        let events: Vec<_> = world
            .history(landings::CHANGE)
            .into_iter()
            .filter(|event| matches!(event.what, h::What::Asked { task: CHILD, .. }))
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].landing, Some(h::InLanding { task: landings::LANDING, round: expected, unit }));
    }
}
