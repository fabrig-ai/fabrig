//! Decisions open before the forge catches up (domain/root.md, 8.1).
use temper_engine_domain_world::forge_routes::change_world;

#[test]
fn decisions_and_sign_in_open_while_all_forge_reads_are_held() {
    let (mut world, _, _, _) = change_world(false, 1000, None);
    world.oracle_settle();
    world.held_reads = Some(Vec::new());
    world.hold_reads_at_any_step = true;
    world.restart_same();
    for _ in 0..200 {
        world.tick();
        if world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()) {
            break;
        }
    }
    assert!(world.root.ready(), "the forge holds no opening phase");
    assert_ne!(world.held_reads.as_ref().expect("forge reads held").as_slice(), []);
    assert_eq!(world.root.start_phase(), None);
    assert!(!world.root.caught_up(0));
    world.signed_in = None;
    world.sign_in();
    assert!(world.signed_in.is_some(), "sign-in answers without a forge report");
    assert!(!world.root.caught_up(0));
    for read in world.held_reads.take().expect("held fresh reads") {
        world.out.push(read);
    }
    for _ in 0..200 {
        world.tick();
        if world.root.caught_up(0) {
            break;
        }
    }
    assert!(world.root.caught_up(0), "both forge reports reached the core");
    world.close();
}
