//! An existing GitHub installation accepted through ctl, without a browser.
use crate::support;

#[test]
fn ctl_accepts_an_existing_read_only_github_installation_and_adopts_its_repository() {
    support::github::accept_existing();
}
