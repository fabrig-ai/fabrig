//! Signing in, cookies and reconnects share one browser and server (client/web/testing.md, 6, client/web/README.md, 1).
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Outcome, Script, Step};
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
    let row = || Find::exactly_named(Role::Button, b"Owner");
    let item = |name: &[u8]| Find::exactly_named(Role::MenuItem, name);
    let press = |find| Step::Person(PersonStep::Press { find });
    let gone = |find| Step::Person(PersonStep::Gone { find, within: WITHIN });
    let mut steps = temper_client_browser_world::first_chat_steps(b"Hello");
    steps.extend([
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(FIRST_CHAT_ADDRESS) }),
        see(first_chat(b"Hello")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::StopServer,
        see(Find::exactly_named(Role::Status, b"Offline")),
        Step::StartServer,
        Step::Person(PersonStep::Gone { find: tier.states().offline, within: WITHIN }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        see(first_chat(b"Hello")),
    ]);
    for _ in 0..RAPID_RELOADS {
        steps.push(Step::Person(PersonStep::Go {
            address: temper_client_browser_world::page_address(FIRST_CHAT_ADDRESS),
        }));
    }
    steps.extend([
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        press(row()),
        see(item(b"Accounts")),
        see(item(b"What temper remembers about you")),
        see(item(b"Sign out")),
        Step::Key(Key::Escape),
        gone(item(b"Sign out")),
        Step::Focused(row()),
        press(row()),
        press(first_chat(b"Hello")),
        gone(item(b"Sign out")),
        Step::Focused(row()),
        press(row()),
        press(item(b"Accounts")),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"What agents run on.")),
        gone(Find::exactly_named(Role::Button, b"Sign out")),
        press(row()),
        press(item(b"Sign out")),
        see(tier.states().signed_out),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/accounts") }),
        see(tier.states().signed_out),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(FIRST_CHAT_ADDRESS) }),
        see(tier.states().signed_out),
        Step::Reopen,
        see(Find::text(b"This link isn't from this temper. Use the one its terminal printed.")),
        Step::Address(b"/welcome".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(temper_client_browser_world::harness(setup).run(&mut [&mut script]), Outcome::Done);
}
