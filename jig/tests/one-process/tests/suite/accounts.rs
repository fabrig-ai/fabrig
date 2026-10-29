use jig_one_process_world::accounts::{self, Story};

// Successful lifecycles request one chat and one sign-out. Restart adds one refresh.
const ONE_CHAT: u32 = 1;
const ONE_REQUEST: usize = 1;
const CODE_GRANT: usize = 1;
const REFRESH_GRANT: usize = 1;
const REVOCATION: usize = 1;

/// Replaces `process::a_person_signs_in_at_the_fake_issuer_and_the_live_story_answers_and_signs_out`.
#[test]
fn a_person_signs_in_uses_the_plan_and_signs_out() {
    sign_in(jig_one_process_world::fixtures::SEED);
}
pub(super) fn sign_in(seed: u64) {
    let outcome = accounts::run(seed, Story::SignIn);
    assert_eq!(outcome.chats, ONE_CHAT);
    assert_eq!(outcome.credentials, [Some(0)]);
    assert_eq!(outcome.posts, CODE_GRANT);
    assert_eq!(outcome.revokes, REVOCATION);
    assert!(outcome.accounts_empty && outcome.confirmed);
    assert_eq!(outcome.queries.len(), ONE_REQUEST);
    let query = &outcome.queries[0];
    assert_eq!(query.model.as_ref(), b"fake");
    assert!(query.system.windows(b"Reply to the person.".len()).any(|w| w == b"Reply to the person."));
    assert!(query.system.windows(b"Hello".len()).any(|w| w == b"Hello"));
}

/// Replaces `process::a_provider_refusing_the_plan_s_grant_fails_the_live_story_in_words`.
#[test]
fn a_refused_plan_grant_fails_the_run() {
    refused(jig_one_process_world::fixtures::SEED);
}
pub(super) fn refused(seed: u64) {
    let outcome = accounts::run(seed, Story::Refused);
    assert_eq!(outcome.run_failure, Some(jig_domain_tasks::Class::Run));
    let Some(smith_channel::RunResult::Failed(failure)) = outcome.provider_failure else { panic!("provider failure") };
    let smith_channel::RunFailure::Model(model) = failure.reason() else { panic!("model failure") };
    let smith_channel::ModelFault::Completion(fault) = model.value() else { panic!("completion failure: {model:?}") };
    assert_eq!(fault.failure(), &smith_channel::CompletionFailure::Unauthorized);
    let smith_channel::CompletionEvidence::Response(response) = fault.evidence() else {
        panic!("HTTP response evidence")
    };
    assert_eq!(response.status(), &Some(401));
    assert!(!outcome.credentials.is_empty());
    assert!(outcome.credentials.iter().all(Option::is_none));
    assert!(outcome.queries.is_empty());
}

/// Replaces `process::a_person_s_sign_in_outlives_a_restart_is_refreshed_at_the_issuer_and_signs_out`.
#[test]
fn a_restored_sign_in_is_refreshed_and_signs_out() {
    restored(jig_one_process_world::fixtures::SEED);
}
pub(super) fn restored(seed: u64) {
    let outcome = accounts::run(seed, Story::Restart);
    assert_eq!(outcome.restarts, 1);
    assert_eq!(outcome.credentials, [None, Some(0)]);
    assert_eq!(outcome.posts, CODE_GRANT + REFRESH_GRANT);
    assert_eq!(outcome.revokes, REVOCATION);
    assert_eq!(outcome.chats, ONE_CHAT);
    assert!(outcome.accounts_empty && outcome.confirmed);
}
