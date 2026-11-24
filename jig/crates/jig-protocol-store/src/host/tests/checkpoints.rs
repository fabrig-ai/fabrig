//! Stopped checkpoint and named adoptions use the production host (checkpoints.md, 5–6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary Rust simulator fixture and outcome assertions (programming-model.md, 10.2)"
)]
use super::super::Adoption;
use super::*;
use crate::{CheckpointRefusal, Request};
use skein_fake_machine::How;

pub(super) fn adopt(world: &mut World, mode: Adoption) {
    let source = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(source.raw()));
    world.host = Drivers::new(&fixtures::store_limits(), 2);
    world.store = Store::on_directory(&world.store.checked(), fixtures::APPLICATION, &[], world.host.seeded());
    world.host.adopt_mode(world.fd, 0, Token::new(41), fixtures::INSTALLATION, mode).unwrap();
    world.settle();
}
#[test]
fn checkpoint_adoption_links_the_stopped_segments_at_c_and_closes_before_answering() {
    let mut world = World::new();
    world.open();
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        fixtures::commit(1, Box::new([fixtures::header(1)])),
        &mut world.above,
    );
    world.settle();
    let event = world.above.pop();
    let Some(Event::Committed { number: 1, .. }) = event else { panic!("adoption terminal: {event:?}") };
    world.close();
    adopt(&mut world, Adoption::Checkpoint { draw: [1; 16] });
    let event = world.above.pop();
    let Some(Event::Checkpointed { checkpoint, .. }) = event else { panic!("checkpoint adoption: {event:?}") };
    assert!(world.above.is_empty(), "no implicit serving Opened or Census");
    assert_eq!(checkpoint.name.store, 1);
    assert!(checkpoint.linked > 0);
    assert_eq!(checkpoint.copied, 0, "recovery sealed every selected segment");
    assert!(world.store.checkpoint_open.is_none(), "logical Close precedes the terminal");
    let root = world.machine.reopen_root(0);
    let base = [b"checkpoints/".as_slice(), &checkpoint.name.bytes()].concat();
    let mut records = Vec::new();
    for suffix in [b"/checkpoint".as_slice(), b"/generations/1/manifest"] {
        let file = world.machine.open(root, &[base.as_slice(), suffix].concat(), How::Read).unwrap();
        records.push(world.machine.read(file, 0, super::super::max_whole(&fixtures::store_limits())).unwrap());
        world.machine.close(file);
    }
    let record = crate::codec::component::Checkpoint::decode(&records[0]).unwrap();
    let manifest = crate::codec::component::Manifest::decode(&records[1]).unwrap();
    assert_eq!((manifest.store, record.store), (1, 1));
    assert!(record.files.iter().filter(|file| file.part == 2).all(|file| file.way == 1));
    world.machine.close(root);
    world.close();
    // The deployment lock was released, and serving still recovers the same C.
    adopt(&mut world, Adoption::Open);
    let event = world.above.pop();
    let Some(Event::Opened { last: 1, .. }) = event else { panic!("adoption terminal: {event:?}") };
    while world.above.pop().is_some() {}
    world.close();
    adopt(&mut world, Adoption::HoldCheckpoint { name: checkpoint.name });
    let event = world.above.pop();
    let Some(Event::CheckpointHeld { owner }) = event else { panic!("held: {event:?}") };
    assert_eq!(owner, Token::new(41));
    world.host.down(&mut world.store, &fixtures::env(), Request::Close { owner: Token::new(41) }, &mut world.above);
    world.settle();
    let event = world.above.pop();
    let Some(Event::Closed { removed: false, .. }) = event else { panic!("adoption terminal: {event:?}") };
    world.close();
    adopt(&mut world, Adoption::RemoveCheckpoint { name: checkpoint.name });
    let event = world.above.pop();
    let Some(Event::CheckpointRemoved { .. }) = event else { panic!("adoption terminal: {event:?}") };
    world.close();
    adopt(&mut world, Adoption::HoldCheckpoint { name: checkpoint.name });
    let event = world.above.pop();
    let Some(Event::CheckpointRefused { why: CheckpointRefusal::Missing, .. }) = event else {
        panic!("adoption terminal: {event:?}")
    };
    world.close();
}
