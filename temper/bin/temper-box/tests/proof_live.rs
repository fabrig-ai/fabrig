//! Offline B0 driver proofs (temper/docs/design/testing/live-tier.md).
use temper_box_kit as support;
#[test]
fn rung_b0s_setup_reaches_account_import_with_the_fake_model_and_no_account() {
    let scratch = support::Scratch::new("rung-b0-offline-setup");
    support::forge_land::rung::offline_setup(scratch.path());
    scratch.remove();
}

#[test]
fn rung_b0s_walk_passes_offline_on_its_twin() {
    let scratch = support::Scratch::new("rung-b0-offline-walk");
    support::forge_land::rung::offline_walk(scratch.path());
    scratch.remove();
}
