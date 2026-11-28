use core::convert::Infallible;
use skein_world::{sweep::cases, this_test};

use crate::accounts;

// Three simulator schedules supplement the focused seed for each composed story.
const SEEDS: std::ops::RangeInclusive<u64> = 1..=3;

/// Replaces `process::a_person_signs_in_at_the_fake_issuer_and_the_live_story_answers_and_signs_out` across seeds.
#[test]
fn signed_in_plans_answer_and_sign_out_across_seeds() {
    cases(this_test!("signed_in_plans_answer_and_sign_out_across_seeds"), SEEDS, |seed| {
        accounts::sign_in(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

/// Replaces `process::a_provider_refusing_the_plan_s_grant_fails_the_live_story_in_words` across seeds.
#[test]
fn refused_plan_grants_keep_the_failure_across_seeds() {
    cases(this_test!("refused_plan_grants_keep_the_failure_across_seeds"), SEEDS, |seed| {
        accounts::refused(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

/// Replaces `process::a_person_s_sign_in_outlives_a_restart_is_refreshed_at_the_issuer_and_signs_out` across seeds.
#[test]
fn restored_sign_ins_refresh_and_sign_out_across_seeds() {
    cases(this_test!("restored_sign_ins_refresh_and_sign_out_across_seeds"), SEEDS, |seed| {
        accounts::restored(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
