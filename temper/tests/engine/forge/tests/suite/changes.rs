//! Change history and catch-up production (temper/docs/design/domain/forge/change/README.md, 2, and temper/docs/design/domain/forge/README.md, 6).
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes};
const PROJECT: u32 = 5;
const SEED: u64 = 17;
const CHANGE: u64 = 51;
const BRANCH: &[u8] = b"temper/51";
const TITLE: &[u8] = b"Small fix";
fn view(world: &World) -> top::ChangesView {
    let top::Document::Changes(view) = world.document(PROJECT, top::changes_subject(PROJECT)).expect("changes watch")
    else {
        panic!("changes document")
    };
    view
}
#[test]
fn a_landed_change_keeps_its_title_after_its_close() {
    let mut world = World::new(SEED);
    world.adopt();
    world.produce(BRANCH);
    world.status(BRANCH, skein_fake_forge_domain::api::Check::Passed);
    let pull = world.open_pull(BRANCH, b"main");
    let head = temper_engine_forge_world::translate::commit(world.branch(BRANCH));
    let mut row = changes::change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = change::State::Queued { head, ready_since: world.wall() };
    world.event(top::Event::Change { row });
    let queued = view(&world);
    assert_eq!(queued.queues.len(), 1);
    assert_eq!(queued.queues[0].changes.len(), 1);
    assert_eq!((queued.queues[0].changes[0].task, queued.queues[0].changes[0].title.as_ref()), (CHANGE, TITLE));
    for entry in 1..=8 {
        changes::change_step(&mut world, entry, change::Status::Passed, Box::new([]));
        if matches!(world.change(CHANGE).change.state, change::State::Landed { .. }) {
            break;
        }
    }
    assert!(matches!(world.change(CHANGE).change.state, change::State::Landed { .. }));
    let landed = view(&world);
    assert!(landed.queues.is_empty());
    assert_eq!(landed.landed.len(), 1);
    world.event(top::Event::SettleEffects { task: CHANGE, root: CHANGE, ending: top::ReleaseEnding::Done });
    world.event(top::Event::Release {
        task: CHANGE,
        root: CHANGE,
        ending: top::ReleaseEnding::Done,
        reason: None,
        entry: 99,
    });
    world.run_for(1);
    let closed = view(&world);
    assert_eq!(closed.landed, landed.landed);
    assert_eq!((closed.landed[0].task, closed.landed[0].title.as_ref()), (CHANGE, TITLE));
    world.restart();
    world.run_for(1);
    assert_eq!(view(&world), closed);

    let mut restored = World::new(SEED);
    for (_, record) in world.stored() {
        if matches!(record, top::Stored::Repository(_) | top::Stored::LandingLog(_)) {
            restored.event(top::Event::Restore { record: record.clone() });
        }
    }
    assert!(restored.document(PROJECT, CHANGE).is_none());
    assert_eq!(view(&restored), closed);
}

// Catch-ups use ancestry without the goal's full diff (temper/docs/design/domain/forge/change/README.md, 2, and temper/docs/design/domain/forge/README.md, 6).
fn catch_up_world(advance_main: bool) -> (World, skein_forge::api::Commit, skein_forge::api::Commit) {
    let mut world = World::with_comparison_room(SEED);
    world.adopt();
    world.produce_on(b"temper/g", b"main");
    for merge in 0_u8..12 {
        world.produce_on(b"temper/incoming", b"temper/g");
        world.push_file(b"temper/incoming", b"file", &[merge]);
        world.status(b"temper/incoming", skein_fake_forge_domain::api::Check::Passed);
        let pull = world.open_pull(b"temper/incoming", b"temper/g");
        world.merge(pull, b"temper/incoming");
        world.delete_branch(b"temper/incoming");
    }
    if advance_main {
        world.push_file(b"main", b"main-only", b"independent");
    }
    let commit = temper_engine_forge_world::translate::commit(world.branch(b"main"));
    let tip = temper_engine_forge_world::translate::commit(world.branch(b"temper/g"));
    let mut row = changes::change_row();
    row.base = b"temper/g".as_slice().into();
    row.catch_up = Some(commit);
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    world.reject_comparisons();
    world.take_calls();
    (world, commit, tip)
}

#[test]
fn a_catch_up_starts_when_the_full_comparison_is_too_large() {
    use skein_forge::api::{Answer, CommitMessage, Error};
    for ancestry in [
        None,
        Some(Err(Error::TooLarge(skein_forge::api::Excess::unnamed()))),
        Some(Ok(Answer::Commits {
            commits: Box::new([CommitMessage { id: [0; 32], parents: Box::new([]), message: Box::new([]) }]),
            more: true,
        })),
    ] {
        let (mut world, commit, tip) = catch_up_world(true);
        world.script_ancestry(ancestry);
        assert_eq!(
            changes::change_step(&mut world, 1, change::Status::Unknown, Box::new([])),
            change::Decision::Delegate(change::Delegate::Merge { base: commit, from: None })
        );
        assert_eq!(world.change(CHANGE).change.resolutions, 0, "production spends no resolution");
        assert_eq!(world.change(CHANGE).change.state, change::State::Producing { requested: true });
        assert_eq!(world.change(CHANGE).change.last_head, None, "no producer branch yet");
        assert_catch_up_range(&mut world, commit, tip);
    }
}

fn assert_catch_up_range(world: &mut World, commit: skein_forge::api::Commit, tip: skein_forge::api::Commit) {
    use skein_forge::api::{Op, Read};
    let calls = world.take_calls();
    assert!(calls.contains(&Op::Read(Read::Commits { before: tip, after: commit, page: 1 })), "current goal tip");
    assert!(!calls.iter().any(|op| matches!(op, Op::Read(Read::Compare { .. }))), "no full comparison");
}

#[test]
fn a_contained_catch_up_ends_without_a_producer() {
    let (mut world, commit, tip) = catch_up_world(false);
    assert_eq!(changes::change_step(&mut world, 1, change::Status::Unknown, Box::new([])), change::Decision::NoChange);
    assert!(matches!(world.change(CHANGE).change.state, change::State::NoChange { .. }));
    assert_eq!(world.change(CHANGE).change.last_head, None);
    assert_eq!(world.change(CHANGE).change.resolutions, 0);
    assert_catch_up_range(&mut world, commit, tip);
}

#[test]
fn a_catch_up_with_uncertain_ancestry_stalls_and_a_release_retries_it() {
    use skein_forge::api::{Answer, Error};
    for result in
        [Err(Error::Refused), Err(Error::InvalidAnswer), Ok(Answer::Commits { commits: Box::new([]), more: true })]
    {
        let (mut world, commit, tip) = catch_up_world(true);
        world.script_ancestry(Some(result.clone()));
        let decision = changes::change_step(&mut world, 1, change::Status::Unknown, Box::new([]));
        let change::Decision::Wait { until: Some(until) } = decision else {
            panic!("uncertain ancestry must schedule a retry: {result:?}: {decision:?}")
        };
        assert!(until > world.wall(), "retry must be in the future");
        assert_eq!(world.change(CHANGE).change.state, change::State::Producing { requested: false });
        assert_catch_up_range(&mut world, commit, tip);
        world.run_for(31);
        assert_eq!(
            changes::change_step(&mut world, 2, change::Status::Unknown, Box::new([])),
            change::Decision::Hold(change::Hold::Stalled)
        );
        world.script_ancestry(None);
        world.event(top::Event::ReleaseChange { task: CHANGE });
        assert_eq!(
            changes::change_step(&mut world, 3, change::Status::Unknown, Box::new([])),
            change::Decision::Delegate(change::Delegate::Merge { base: commit, from: None }),
            "recovered ancestry: {result:?}"
        );
        assert_eq!(world.change(CHANGE).change.resolutions, 0);
        assert_catch_up_range(&mut world, commit, tip);
    }
}
