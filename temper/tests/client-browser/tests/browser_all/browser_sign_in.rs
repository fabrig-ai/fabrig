//! Signing in, cookies and reconnects share one browser and server (client/browser.md, 4.6, 1).
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{FIRST_CHAT_ADDRESS, Tier, WITHIN, first_chat};

// Several loads begin before the preceding one reaches Live.
const RAPID_RELOADS: usize = 5;

#[test]
fn signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract() {
    let tier = Tier::reserved("sign-in-cookie").with_model();
    let mut setup = tier.setup();
    setup.expected = vec![
        b"net::ERR_CONNECTION_REFUSED".as_slice().into(),
        b"net::ERR_INCOMPLETE_CHUNKED_ENCODING".as_slice().into(),
        b"401".as_slice().into(),
    ];
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut steps = temper_client_browser_world::start_chat(b"Hello");
    steps.extend([
        Step::Person(PersonStep::Reload),
        see(first_chat(b"Hello")),
        see(tier.states().live),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::StopServer,
        see(Find::exactly_named(Role::Status, b"Offline")),
        Step::StartServer,
        see(tier.states().live),
        see(first_chat(b"Hello")),
    ]);
    for _ in 0..RAPID_RELOADS {
        steps.push(Step::Person(PersonStep::Reload));
    }
    steps.extend([
        see(tier.states().live),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign out") }),
        see(tier.states().signed_out),
        Step::Person(PersonStep::Reload),
        see(tier.states().signed_out),
        Step::Person(PersonStep::Go { address: FIRST_CHAT_ADDRESS.into() }),
        see(tier.states().signed_out),
        Step::Reopen,
        see(tier.states().signed_out),
        Step::Address(b"/welcome".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(setup).run(&mut [&mut script]), Outcome::Done);
}
