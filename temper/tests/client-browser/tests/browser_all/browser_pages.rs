//! The pages a browser moves between (client/browser.md, 4.1, stories 2 to 4): back and forward between the chats
//! and a chat, an address that names no page opens the inbox at `/`, Enter starts one chat where Shift+Enter breaks
//! its line, and a panic shows the failure page.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step, stories};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Tier, WITHIN, first_chat};

#[test]
fn enter_starts_one_chat_keeping_a_shift_enter_break() {
    let tier = Tier::new("composer").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    let chats = || Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") });
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"temper chats") }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        chats(),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"New chat") }),
        see(composer()),
        Step::Type { find: composer(), words: Box::from(b"Hello".as_slice()) },
        Step::KeyWithShift(Key::Enter),
        Step::Type { find: composer(), words: Box::from(b"there".as_slice()) },
        Step::Key(Key::Enter),
        see(first_chat(b"Hello")),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
        chats(),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, b"T2"), within: ABSENT }),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn a_panic_shows_the_failure_page() {
    let tier = Tier::new("panic");
    assert_eq!(stories::panic(&tier.setup(), b"/test/panic", Find::text(b"The application failed.")), Outcome::Done);
}

#[test]
fn history_keeps_the_chat_focus_and_filter_across_reload_and_back() {
    let tier = Tier::new("history-chat-filter-address").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let pressed = |position| Step::Keep {
        selector: format!("main .temper-filters > button[aria-pressed='true']:nth-child({position})")
            .into_bytes()
            .into_boxed_slice(),
        attribute: Some(b"aria-pressed".as_slice().into()),
        name: b"pressed-filter".as_slice().into(),
    };
    let mut steps = temper_client_browser_world::start_chat(b"Hello");
    steps.extend([
        press(Role::Link, b"Chats"),
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        press(Role::Link, FIRST_CHAT),
        see(first_chat(b"Hello")),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::Back,
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        Step::Forward,
        see(first_chat(b"Hello")),
        Step::Person(PersonStep::Go { address: b"/".as_slice().into() }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
        press(Role::Link, b"New chat"),
        see(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Focused(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Address(b"/chats/new".as_slice().into()),
        press(Role::Link, b"Chats"),
        press(Role::Button, b"Live"),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Address(b"/chats?state=live".as_slice().into()),
        Step::Person(PersonStep::Reload),
        see(Find::exactly_named(Role::Button, b"Live")),
        pressed(2),
        Step::Back,
        see(Find::exactly_named(Role::Button, b"All")),
        Step::Address(b"/chats".as_slice().into()),
        pressed(1),
        Step::Person(PersonStep::Go { address: b"/nowhere".as_slice().into() }),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
        Step::Address(b"/".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn the_byline_panel_folds_on_an_outside_press_and_escape() {
    let tier = Tier::new("chat-byline").with_model();
    let mut steps = temper_client_browser_world::start_chat(b"Hello");
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::named(role, name) });
    let folded = |open| Step::Keep {
        selector: Box::from(if open {
            b"details.temper-byline[open]".as_slice()
        } else {
            b"details.temper-byline:not([open])".as_slice()
        }),
        attribute: Some(Box::from(b"data-node".as_slice())),
        name: Box::from(b"model-panel".as_slice()),
    };
    steps.extend([
        press(Role::Disclosure, b"Change model"),
        folded(true),
        press(Role::TextArea, b"Message"),
        folded(false),
        press(Role::Disclosure, b"Change model"),
        folded(true),
        Step::Key(Key::Escape),
        folded(false),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
