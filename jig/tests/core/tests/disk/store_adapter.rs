//! The owner-set closed-window fixture runs over the simulated disk.
use jig_domain_core as core;

use crate::closed_proofs;

#[test]
fn a_closed_proof_restores_until_the_next_attempt() {
    use closed_proofs as f;
    let mut world = f::parked();
    let original = f::proof(&world).clone();
    let tries = f::tries(&world);
    f::close(&mut world);
    f::restart(&mut world);
    assert_eq!(f::tries(&world), tries);
    assert!(world.domain.ready(), "the real root restored the closed proof");
    let closed = f::proof(&world).clone();
    let mut expected = original;
    expected.window.as_mut().expect("parked window").closed = true;
    expected.crossed = true;
    assert_eq!(closed, expected, "restart preserves every other proof field");
    let limits = jig_test_profile::limits().core;
    let mut engine =
        core::Core::new(jig_core_world::world::fixture(f::SEED, false).0.core, &limits).expect("core configuration");
    let _loading = engine.start_begin(false, skein_lib::Wall::EPOCH);
    assert_eq!(
        engine.restore(
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits },
            core::Record::Core(core::CoreRecord::RunProof(closed.clone())),
        ),
        core::Restore::Kept
    );
    assert_eq!(engine.proofs.get(&closed.task), Some(&closed));
    world.restart(f::SEED, false);
    assert_eq!(f::proof(&world), &closed, "a later open keeps it closed");
    f::hello(&mut world);
    world.grant_account(1);
    world.say(87);
    f::fresh(&world);
    assert_eq!(f::tail(&world), Some(f::TAIL));
    world.window_turn(1, 1, None, &f::turn(b"new window"));
    let attempt = f::proof(&world).attempt;
    assert_eq!(f::proof(&world).window, Some(core::CurrentWindow { attempt, turn: 1, closed: false }));
}
