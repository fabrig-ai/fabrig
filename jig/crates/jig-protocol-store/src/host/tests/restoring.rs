//! A restore adoption owns the safety checkpoint and gives one terminal (restoring.md, 3–4).
use super::super::Adoption;
use super::checkpoints::adopt;
use super::*;
use crate::{CheckpointRefusal, Request, RestoreRefusal};
#[test]
fn restore_adoption_keeps_the_lock_through_safety_and_answers_only_after_close() {
    let mut world = World::new();
    world.open();
    let mut source = None;
    for number in 1..=2 {
        world.host.down(
            &mut world.store,
            &fixtures::env(),
            fixtures::commit(number, Box::new([fixtures::header(number)])),
            &mut world.above,
        );
        world.settle();
        let Some(Event::Committed { .. }) = world.above.pop() else { panic!("adoption terminal") };
        if number == 1 {
            world.host.down(
                &mut world.store,
                &fixtures::env(),
                Request::Checkpoint { owner: Token::new(41), draw: [1; 16] },
                &mut world.above,
            );
            world.settle();
            let Some(Event::Checkpointed { checkpoint, .. }) = world.above.pop() else { panic!("source checkpoint") };
            source = Some(checkpoint);
        }
    }
    let checkpoint = source.expect("source at first commit");
    world.close();
    adopt(&mut world, Adoption::Restore { name: checkpoint.name, draw: [2; 16] });
    let event = world.above.pop();
    let Some(Event::Restored { owner, restored }) = event else { panic!("restore: {event:?}") };
    assert_eq!(owner, Token::new(41));
    assert_eq!((restored.name, restored.generation, restored.safety.store), (checkpoint.name, 2, 2));
    assert!(world.above.is_empty(), "one adoption terminal");
    assert!(world.store.checkpoint_open.is_none());
    world.close();
    adopt(&mut world, Adoption::Back);
    let Some(Event::BackRefused { why: crate::BackRefusal::NoPrevious, .. }) = world.above.pop() else {
        panic!("adoption terminal")
    };
    world.close();
    adopt(&mut world, Adoption::Open);
    let Some(Event::Opened { last: 1, .. }) = world.above.pop() else { panic!("adoption terminal") };
    while world.above.pop().is_some() {}
    world.close();
    adopt(&mut world, Adoption::Restore { name: restored.safety, draw: [3; 16] });
    let Some(Event::Restored { restored: crate::Restored { generation: 3, .. }, .. }) = world.above.pop() else {
        panic!("adoption terminal")
    };
    world.close();
    adopt(&mut world, Adoption::Open);
    let Some(Event::Opened { last: 2, .. }) = world.above.pop() else { panic!("adoption terminal") };
    while world.above.pop().is_some() {}
    world.close();
    adopt(&mut world, Adoption::Restore { name: crate::CheckpointName { store: 999, suffix: 0 }, draw: [4; 16] });
    let Some(Event::RestoreRefused { why: RestoreRefusal::Source { why: CheckpointRefusal::Missing }, .. }) =
        world.above.pop()
    else {
        panic!("adoption terminal")
    };
    world.close();
}
