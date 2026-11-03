//! The negative walk (temper/docs/design/protocol/sign-in.md, 2.5).
use temper_one_process_world::public;
const SEED: u64 = 7;
#[test]
fn each_broken_relation_refuses_the_start() {
    public::walk(SEED, "each_broken_relation_refuses_the_start");
}
#[test]
fn the_proxy_keeps_age_and_serves_nothing_stale() {
    public::walk(SEED, "the_proxy_keeps_age_and_serves_nothing_stale");
}
#[test]
fn the_proxy_keeps_only_public_200_and_404() {
    public::walk(SEED, "the_proxy_keeps_only_public_200_and_404");
}
#[test]
fn a_stopped_upstream_serves_copies_until_their_freshness_ends() {
    public::walk(SEED, "a_stopped_upstream_serves_copies_until_their_freshness_ends");
}
#[test]
fn two_deployments_behind_one_proxy_keep_their_own_copies() {
    public::walk(SEED, "two_deployments_behind_one_proxy_keep_their_own_copies");
}
#[test]
fn a_refused_host_is_never_answered_from_the_cache() {
    public::walk(SEED, "a_refused_host_is_never_answered_from_the_cache");
}
#[test]
fn malformed_paths_are_404_no_store_and_never_kept() {
    public::walk(SEED, "malformed_paths_are_404_no_store_and_never_kept");
}
#[test]
fn no_credential_anywhere_else_is_signed_out() {
    public::walk(SEED, "no_credential_anywhere_else_is_signed_out");
}
#[test]
fn no_answer_carries_an_access_control_header() {
    public::walk(SEED, "no_answer_carries_an_access_control_header");
}

#[test]
fn a_cookie_or_bearer_on_a_public_read_changes_no_byte() {
    public::walk(SEED, "a_cookie_or_bearer_on_a_public_read_changes_no_byte");
}
#[test]
fn every_owner_version_and_kind_answers_as_declared() {
    public::walk(SEED, "every_owner_version_and_kind_answers_as_declared");
}

#[test]
fn a_members_set_visibility_is_refused() {
    public::walk(SEED, "a_members_set_visibility_is_refused");
}
