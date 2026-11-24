//! A prepare refused at the holds bound must be admitted when those holds free
//! (`temper/docs/design/testing/engine-checks.md`, 5.2).
use skein_world::{streams::Streams, sweep::Sweep};
use temper_engine_domain_world::{forge_routes, workload};

#[test]
#[ignore = "failure worlds: a branch hold refused while the holds table frees stays held instead of admitted, repaired as a red"]
fn a_prepare_is_admitted_when_branch_holds_are_released() {
    // The concurrent goals fill sixteen holds while a landing's prepare asks
    // for its prefixes. Other goals release their holds, but the prepare stays
    // ResourceUnavailable and its goal cannot end. No exclusion masks the run.
    let mut limits = forge_routes::box_limits();
    limits.forge.holds = 16;
    workload::run_bounds(&Streams::new(1), &Sweep::new("prepare waits for freed holds"), &limits)
        .expect("the prepare is admitted after room frees, and every goal ends");
}
