//! The changes watch retains a landed title after cleanup (temper/docs/design/client/forge.md, 3).
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes};
const PROJECT: u32 = 5;
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
    let mut world = World::new(17);
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
    assert!(!world.stored().contains_key(&top::Key::Change(CHANGE)));
}
