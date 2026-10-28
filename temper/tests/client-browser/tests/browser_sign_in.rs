//! Signing in by the printed link (client/browser.md, 4.1, story 1): the page takes the secret from its own
//! address, puts `/welcome` in its place, and the browser keeps jig's cookie; a reload signs in by that cookie, and a
//! spent link shows the signed-out words.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{FIRST_CHAT_ADDRESS, Tier, WITHIN, first_chat};

#[test]
fn the_printed_link_signs_in() {
    let tier = Tier::new("printed-link");
    let welcome = Find::exactly_named(Role::Heading { level: 1 }, b"Welcome to temper");
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: welcome, within: WITHIN }),
        Step::Person(PersonStep::See { find: Find::exactly_named(Role::Link, b"Set up temper"), within: WITHIN }),
        Step::Address(Box::from(b"/welcome".as_slice())),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn a_reload_on_a_chat_signs_in_by_its_cookie() {
    let tier = Tier::new("reload-on-a-chat").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut steps = temper_client_browser_world::start_chat(b"Hello");
    steps.extend([
        Step::Person(PersonStep::Reload),
        see(first_chat(b"Hello")),
        see(tier.states().live),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn a_spent_link_shows_the_signed_out_words() {
    let tier = Tier::new("spent-link");
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: Find::exactly_named(Role::Link, b"Set up temper"), within: WITHIN }),
        Step::Reopen,
        Step::Person(PersonStep::See { find: tier.states().signed_out, within: WITHIN }),
        Step::Address(Box::from(b"/welcome".as_slice())),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
