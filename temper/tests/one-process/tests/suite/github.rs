use temper_one_process_world::lower::github::{self, Scenario};
#[test]
fn uninstalled_adoption_starts_no_git_checkout_or_installation() {
    github::run(Scenario::Uninstalled);
}
#[test]
fn another_accounts_refusal_preserves_the_installed_read_only_checkout() {
    github::run(Scenario::ReadOnly);
}
#[test]
fn a_delivered_edit_authenticates_every_git_request_with_its_installation() {
    github::run(Scenario::Delivered);
}
