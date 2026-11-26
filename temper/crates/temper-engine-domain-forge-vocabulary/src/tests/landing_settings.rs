//! Kind 4's judgment of test reruns (temper/docs/design/domain/landing-settings.md, 1.1).
use crate::settings::{Bounds, Flakes, Item, ProjectLanding, Refusal, Value, judge};

const PROJECT: u32 = 7;
const RERUN_PROCESSES: u32 = 3;
const BOUNDS: Bounds = Bounds {
    landing_members: 1,
    rerun_processes: RERUN_PROCESSES,
    repositories: 1,
    landing_gates: 1,
    tiers: 1,
    ignored_paths: 1,
    landing_rules: 1,
    gates: 1,
    roles: 1,
    segments: 1,
    segment_bytes: 16,
    follows: 1,
    dependencies: 1,
    follow_paths: 1,
    name_bytes: 16,
};

fn landing(flakes: Option<Flakes>) -> Result<(), Refusal> {
    let value = ProjectLanding { flakes, ..ProjectLanding::default() };
    judge(&Item { kind: 4, scope: PROJECT, number: 1, value: Some(Value::Landing(value)) }, &BOUNDS)
}

#[test]
fn landing_flakes_accepts_the_deployment_bound_and_no_reruns() {
    for flakes in [None, Some(Flakes::Never), Some(Flakes::Rerun(0)), Some(Flakes::Rerun(RERUN_PROCESSES))] {
        assert_eq!(landing(flakes), Ok(()), "{flakes:?}");
    }
}

#[test]
fn landing_flakes_refuses_one_past_the_deployment_bound() {
    let past = RERUN_PROCESSES.checked_add(1).unwrap();
    let refused = landing(Some(Flakes::Rerun(past))).unwrap_err();
    assert_eq!(refused.words().as_ref(), b"flakes: rerun passes checkout.rerun_processes");
}
