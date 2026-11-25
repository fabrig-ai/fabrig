//! A refused round cleanup must be visible and held (landing-prepare.md, 1.1).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::landings::{self, CHANGE, LANDING};

#[test]
fn a_refused_release_is_accounted() {
    let mut world = landings::turn_world();
    world.produce(b"temper/51");
    world.push_file(b"main", b"base", b"base moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    landings::register(&mut world, row);
    for entry in 100..140 {
        if world.landing(LANDING).landing.phase == landing::Phase::Proving {
            break;
        }
        landings::step_landing(&mut world, LANDING, entry);
    }
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Proving);
    let branch = world.landing(LANDING).landing.round.branch.clone();
    world.push_file(b"main", b"moved-again", b"outside push");
    let mut closed = false;
    for entry in 200..260 {
        if landings::step_landing(&mut world, LANDING, entry) == landing::Decision::Close {
            closed = true;
            break;
        }
    }
    assert!(closed, "stale round closes its pull before releasing branches");
    // The close has settled; only the branch deletion meets Forbidden.
    world.permission(raw::Permission::Read);
    world.take_calls();
    // Forty turns exceed the fake's response and client retry/settle bounds.
    for entry in 300..340 {
        if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Hold(_)) {
            break;
        }
    }
    let calls = world.take_calls();
    assert!(
        calls
            .iter()
            .any(|op| matches!(op, api::Op::Write(api::Write::DeleteBranch { branch: deleted }) if deleted == &branch)),
        "round cleanup attempted the branch deletion: {calls:?}"
    );
    assert!(world.has_branch(&branch), "the forge refused deletion");
    assert!(
        matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { .. }),
        "a refused release must hold, not repeat: {:?}",
        world.landing(LANDING).landing.phase
    );
    let words = world.landing_hold_words(LANDING).expect("refused release has words");
    super::words_name(&words, &["delet", "permission"]);
    assert!(String::from_utf8_lossy(&words).contains(String::from_utf8_lossy(&branch).as_ref()), "{words:?}");
}
