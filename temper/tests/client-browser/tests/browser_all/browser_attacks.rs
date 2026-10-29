//! The attacks that need a browser (client/browser.md, 4.1 story 5; jig's protocol/browser.md, 15):
//! Chromium sends the cross-site fetches and enforces the page's frame policy.
use jig_client_fake_person_browser::attacks;
use skein_fake_person_browser::Outcome;
use temper_client_browser_world::Tier;
fn signed_in(tier: &Tier) -> jig_client_fake_person_browser::States {
    let mut states = tier.states();
    // The welcome's bare frame and the work pages both offer Sign out once signed in.
    states.live = skein_fake_person::Find::exactly_named(skein_ui::Role::Button, b"Sign out");
    states
}

#[test]
fn a_cross_site_request_from_another_port_does_nothing() {
    let tier = Tier::new("another-port");
    let outcome =
        attacks::a_cross_site_request_from_another_port_does_nothing(&tier.setup(), &signed_in(&tier), &tier.target());
    assert_eq!(outcome, Outcome::Done);
}

#[test]
fn a_cross_site_request_from_another_host_does_nothing() {
    let tier = Tier::new("another-host");
    let outcome =
        attacks::a_cross_site_request_from_another_host_does_nothing(&tier.setup(), &signed_in(&tier), &tier.target());
    assert_eq!(outcome, Outcome::Done);
}

#[test]
fn a_framed_page_shows_nothing() {
    let tier = Tier::new("framed-page");
    let outcome = attacks::a_framed_page_shows_nothing(&tier.setup(), &signed_in(&tier), &tier.target());
    assert_eq!(outcome, Outcome::Done);
}
