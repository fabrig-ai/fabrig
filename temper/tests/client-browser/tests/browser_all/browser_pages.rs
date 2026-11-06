//! The pages a browser moves between (client/browser.md, 4.1, stories 2 to 4): back and forward between the chats
//! and a chat, an address that names no page opens the inbox at `/`, Enter starts one chat where Shift+Enter breaks
//! its line, and a panic shows the failure page.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep, Target};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Tier, WITHIN, first_chat};

#[test]
fn enter_starts_one_chat_keeping_a_shift_enter_break() {
    let tier = Tier::new("composer").with_model();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
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
    ];
    steps.extend(byline_steps());
    steps.extend([
        chats(),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, b"T2"), within: ABSENT }),
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
    steps.extend(change_and_close_steps(&tier));
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
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
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

// client/README.md, 5.7: the same change card changes from Running to Landed in the chat.
fn change_and_close_steps(tier: &Tier) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let card = |find: Find| {
        find.within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }]))
    };
    let mut steps = vec![
        see(card(Find::exactly_named(Role::Paragraph, b"Change"))),
        see(card(Find::exactly_named(Role::Heading { level: 3 }, b"Export widget's table as CSV"))),
        see(card(Find::exactly_named(Role::Status, b"Running"))),
        see(card(Find::exactly_named(Role::Link, b"Task 2"))),
        see(card(
            Find::text(b"Open")
                .within(Box::new([Target::ExactRole { role: Role::Link, name: b"Task 2".as_slice().into() }])),
        )),
        // Meta and progress steps stay in the terminal; Chromium reads one tool line.
        Step::Person(PersonStep::Gone { find: card(Find::exactly_named(Role::List, b"Steps")), within: ABSENT }),
    ];
    steps.extend(super::browser_task_pages::plan_and_agent_steps(tier));
    steps.extend([
        press(Role::Link, FIRST_CHAT),
        see(first_chat(b"Add CSV export to widget")),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        playground_landed(card(Find::exactly_named(Role::Status, b"Landed"))),
        see(card(Find::text("CI passed. Landed · merge commit ".as_bytes()))),
        press(Role::Button, b"Close"),
        see(Find::exactly_named(Role::Dialog, b"Close this chat?")),
        press(Role::Button, b"Close chat"),
        see(Find::named(Role::Status, b"Closed")),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::TextArea, b"Message"), within: ABSENT }),
        press(Role::Link, b"Chats"),
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        press(Role::Button, b"Closed"),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        see(Find::exactly_named(Role::Status, b"Closed")
            .within(Box::new([Target::ExactRole { role: Role::List, name: b"Your chats".as_slice().into() }]))),
        see(Find::exactly_named(Role::Paragraph, b"Add CSV export to widget")),
        Step::Address(b"/chats?state=closed".as_slice().into()),
    ]);
    steps
}

// The native story owns the completion allowance for this same scripted change.
fn playground_landed(find: Find) -> Step {
    let within = temper_playground_world::stories::a()
        .into_iter()
        .find_map(|step| {
            if let PersonStep::See { find, within } = step
                && find == Find::exactly_named(Role::Status, b"Landed")
            {
                Some(within)
            } else {
                None
            }
        })
        .expect("the playground story observes its landed change");
    Step::Person(PersonStep::See { find, within })
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
