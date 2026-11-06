//! Only a browser shows Back and Forward, a replaced history entry, the composer's focus,
//! a filter's address across a reload, and an address that's no page; the page's own bounds
//! on the box's Me (client/browser.md, 4.6, item 11). Keys in a text area, a details dismissed by
//! a press or Escape, the panic page, and where project rename returns focus stay here too.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Tier, WITHIN, first_chat};

/// Sending keeps the focus in the composer, by a press or a key (client/README.md, 3.5).
#[test]
fn enter_starts_one_chat_and_the_composer_keeps_the_focus() {
    let tier = Tier::new("composer").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    let message = || Find::exactly_named(Role::TextArea, b"Message");
    let chats = || Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") });
    let mut steps = vec![
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")),
        Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        chats(),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"+ New chat") }),
        see(composer()),
        Step::Type { find: composer(), words: Box::from(b"Hello".as_slice()) },
        Step::KeyWithShift(Key::Enter),
        Step::Type { find: composer(), words: Box::from(b"there".as_slice()) },
        Step::Key(Key::Enter),
        see(first_chat(b"Hello")),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
        see(message()),
        Step::Focused(message()),
        Step::Person(PersonStep::Type { find: message(), words: b"By the button".as_slice().into() }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
        see(Find::exactly_named(Role::Paragraph, b"By the button")),
        Step::Focused(message()),
        Step::Person(PersonStep::Type { find: message(), words: b"By the key".as_slice().into() }),
        Step::Key(Key::Enter),
        see(Find::exactly_named(Role::Paragraph, b"By the key")),
        Step::Focused(message()),
    ];
    steps.extend(byline_steps());
    steps.extend([
        chats(),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, b"T2"), within: ABSENT }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, FIRST_CHAT) }),
        see(message()),
        Step::Keep {
            selector: b"main h1:focus".as_slice().into(),
            attribute: Some(b"data-node".as_slice().into()),
            name: b"focused-chat-heading".as_slice().into(),
        },
        Step::Person(PersonStep::Go { address: b"/test/panic".as_slice().into() }),
        Step::Person(PersonStep::See {
            find: Find::text(b"The application failed."),
            within: skein_lib::Duration::from_secs(6),
        }),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut setup = tier.setup();
    setup.expected.push(b"Uncaught".as_slice().into());
    assert_eq!(Harness::new(setup).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn history_keeps_the_chat_focus_and_filter_across_reload_and_back() {
    let tier = Tier::demo("history-chat-filter-address");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let pressed = |position| Step::Keep {
        selector: format!("main .temper-filters > button[aria-pressed='true']:nth-child({position})")
            .into_bytes()
            .into_boxed_slice(),
        attribute: Some(b"aria-pressed".as_slice().into()),
        name: b"pressed-filter".as_slice().into(),
    };
    let needs_page = || Step::Keep {
        selector: b"main.temper-needs".as_slice().into(),
        attribute: Some(b"class".as_slice().into()),
        name: b"needs-page".as_slice().into(),
    };
    let words = b"Add CSV export to widget";
    let mut steps = temper_client_browser_world::start_playground_chat(words);
    steps.extend([
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        // Starting replaces NewChat: Back returns to the list that opened it.
        Step::Back,
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        Step::Address(b"/chats".as_slice().into()),
        Step::Forward,
        see(first_chat(words)),
    ]);
    steps.extend(reload_chats(&tier));
    steps.extend([
        press(Role::Link, FIRST_CHAT),
        see(first_chat(words)),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::Back,
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        Step::Forward,
        see(first_chat(words)),
        Step::Person(PersonStep::Go { address: b"/".as_slice().into() }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        see(Find::exactly_named(Role::Main, b"")),
        needs_page(),
        press(Role::Link, b"New chat"),
        see(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Focused(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Address(b"/chats/new".as_slice().into()),
        press(Role::Link, b"Chats"),
        press(Role::Button, b"Closed"),
        Step::Address(b"/chats?state=closed".as_slice().into()),
        Step::Person(PersonStep::Reload),
        see(Find::exactly_named(Role::Button, b"Closed")),
        pressed(3),
        Step::Back,
        see(Find::exactly_named(Role::Button, b"All")),
        Step::Address(b"/chats".as_slice().into()),
        pressed(1),
        Step::Person(PersonStep::Go { address: b"/nowhere".as_slice().into() }),
        see(Find::exactly_named(Role::Main, b"")),
        needs_page(),
        Step::Address(b"/".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

fn byline_steps() -> Vec<Step> {
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
    vec![
        press(Role::Disclosure, b"Change model"),
        folded(true),
        press(Role::TextArea, b"Message"),
        folded(false),
        press(Role::Disclosure, b"Change model"),
        folded(true),
        Step::Key(Key::Escape),
        folded(false),
    ]
}

fn reload_chats(tier: &Tier) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let heading = |level, name: &[u8]| Find::exactly_named(Role::Heading { level }, name);
    let behind = || Find::exactly_named(Role::Status, b"Behind");
    let chat = || Find::exactly_named(Role::Link, FIRST_CHAT);
    vec![
        // The playground's project made, the link stays up across a reload: the page's own
        // client bounds receive the box's Me (temper/docs/design/client/README.md, 2).
        Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
        see(heading(1, b"Talk to an agent about playground/widget.")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
        see(chat()),
        Step::Person(PersonStep::Reload),
        see(heading(1, b"Talk to an agent about playground/widget.")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
        see(chat()),
    ]
}

#[test]
fn closing_project_rename_returns_focus_to_the_summary() {
    let tier = Tier::new("rename-focus").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let summary = |name: &[u8]| Find::named(Role::Disclosure, name);
    let main_unfocused = || Step::Keep {
        selector: b"main:not(:focus)".as_slice().into(),
        attribute: Some(b"class".as_slice().into()),
        name: b"main-without-focus".as_slice().into(),
    };
    let rename = || Find::exactly_named(Role::TextField, b"Rename");
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")),
        Step::Person(PersonStep::Go { address: b"/projects/1".as_slice().into() }),
        see(summary(b"Project 1")),
        Step::Person(PersonStep::Press { find: summary(b"Project 1") }),
        press(Role::Button, b"Rename Project 1"),
        Step::Focused(rename()),
        Step::Key(Key::Escape),
        Step::Person(PersonStep::Gone { find: rename(), within: ABSENT }),
        Step::Focused(summary(b"Project 1")),
        main_unfocused(),
        press(Role::Button, b"Rename Project 1"),
        Step::Person(PersonStep::Type { find: rename(), words: b"New name".as_slice().into() }),
        Step::Key(Key::Enter),
        see(summary(b"New name")),
        Step::Focused(summary(b"New name")),
        main_unfocused(),
        press(Role::Button, b"Rename New name"),
        Step::Person(PersonStep::Type { find: rename(), words: b"Saved name".as_slice().into() }),
        press(Role::Button, b"Save"),
        see(summary(b"Saved name")),
        Step::Focused(summary(b"Saved name")),
        main_unfocused(),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

/// Scrolling with the browser's keys, and the view staying put.
#[test]
fn latest_returns_a_chat_to_its_end_after_new_words_while_reading_back() {
    // Six bounded messages fill the browser tier's log viewport.
    const MESSAGES: u32 = 6;
    const REPEATS: usize = 4;
    // Twenty arrow presses move the view away from its end at the tier's viewport.
    const SCROLL_KEYS: u32 = 20;
    let tier = Tier::demo("chat-latest");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let latest = || Find::exactly_named(Role::Button, b"Latest");
    let check = |selector: &[u8]| Step::Keep {
        selector: selector.into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: selector.into(),
    };
    let mut steps = temper_client_browser_world::start_playground_chat(b"Add CSV export to widget");
    steps.push(see(Find::exactly_named(Role::Region, b"Task 2")));
    for number in 1..=MESSAGES {
        let words = format!(
            "Reading paragraph {number}. {}",
            "Keep these words available while reading back. ".repeat(REPEATS)
        )
        .into_bytes()
        .into_boxed_slice();
        steps.extend([
            Step::Person(PersonStep::Type {
                find: Find::exactly_named(Role::TextArea, b"Message"),
                words: words.clone(),
            }),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
            see(Find::exactly_named(Role::Paragraph, &words)),
            Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Being sent"), within: WITHIN }),
        ]);
    }
    steps.extend([
        Step::Person(PersonStep::Gone { find: latest(), within: ABSENT }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::TextArea, b"Message") }),
        Step::KeyWithShift(Key::Tab),
    ]);
    // Focus the preceding control in the log, then scroll its container with the browser's keys.
    for _ in 0..SCROLL_KEYS {
        steps.push(Step::Key(Key::ArrowUp));
    }
    steps.extend([
        see(latest()),
        check(b".temper-thread > [role=log][data-log-away] + .temper-latest"),
        Step::Person(PersonStep::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"Tell me what changed.".as_slice().into(),
        }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
        see(Find::exactly_named(Role::Paragraph, b"Tell me what changed.")),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Being sent"), within: WITHIN }),
        see(latest()),
        check(b".temper-thread > [role=log][data-log-away] + .temper-latest"),
        Step::Person(PersonStep::Press { find: latest() }),
        Step::Person(PersonStep::Gone { find: latest(), within: ABSENT }),
        check(b".temper-thread > [role=log]:not([data-log-away]) + .temper-latest"),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
