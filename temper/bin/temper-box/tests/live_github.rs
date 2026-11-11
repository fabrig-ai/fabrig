//! Owner-run only: real GitHub, fake model, real engine and git, through the guarded relay.
pub mod support;
#[test]
#[ignore = "live: github.com, on the probe App's fixtures the owner holds; scripts/proof-live-github.sh"]
fn a_chat_lands_a_change_on_github_com_through_a_restart() {
    support::live_github::run();
}
