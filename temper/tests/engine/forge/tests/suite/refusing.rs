//! Open other-user merge scenario (client/forge/README.md, 5.2).
use skein_fake_forge_domain::api as raw;
use skein_lib::Duration;
use skein_world::faults::Kind;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    landings::{self, LANDING},
    translate,
};

#[test]
#[ignore = "landing train: one owner for a landing's members, and a landing that can always move"]
fn a_member_merged_by_another_lands_like_any_landing() {
    let mut world = super::landing_batches::batch(2);
    let pull = world.landing(LANDING).landing.members[1].pull;
    world.other_users_after_member_reads(
        vec![(52, Kind { family: "refusing", name: "merged_by_another" }, true)],
        Duration::ZERO,
    );
    let mut finished = false;
    for entry in 100..140 {
        let decision = landings::step_landing_raw(&mut world, LANDING, entry);
        if matches!(decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(LANDING, true);
        }
        assert!(
            !matches!(
                world.landing(LANDING).landing.phase,
                landing::Phase::Held { why: landing::Hold::Record { member: 1 }, .. }
            ),
            "another user's merge is landed, not held: {:?}",
            world.landing(LANDING)
        );
        if decision == landing::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(finished, "the landing with another user's merged member finishes");
    let raw::Answer::Pull(merged) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("member pull") };
    let commit = translate::commit(merged.merged.expect("other user merged"));
    assert!(
        matches!(world.change(52).change.state, change::State::Landed { merge, .. } if merge == commit),
        "the external member landed at the other user's commit"
    );
    let row = &world.landing(LANDING).landing;
    assert_eq!(row.members[1].landed, Some(commit));
    assert!(row.members[0].landed.is_some() && row.members[0].recorded, "the other member lands and is recorded");
}
