//! Browser printed-link sign-in (client/browser.md 2.4): a QA pass run by `--profile qa`.
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use temper_client_browser_world::{FIRST_CHAT_ADDRESS, Tier};

#[test]
fn browser_printed_link_reaches_the_signed_in_page() {
    let tier = Tier::new("qa-sign-in-browser").with_model();
    let mut harness = Harness::new(tier.setup());
    let mut steps = temper_client_browser_world::start_chat(b"QA printed-link sign-in");
    steps.push(Step::Address(FIRST_CHAT_ADDRESS.into()));
    let mut person = Script::new(steps.into_boxed_slice());
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
}
