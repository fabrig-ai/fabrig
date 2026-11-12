//! Only a browser shows Back and Forward, a replaced history entry, the composer's focus,
//! a filter's address across a reload, and an address that's no page; the page's own bounds
//! on the box's Me (client/web/testing.md, 6, item 11). A run of calls' folded line, the picker's
//! menu, keys in a text area, a details dismissed by a press or Escape, the panic page,
//! and where project rename returns focus stay here too.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{
    ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Stories, Tier, WITHIN, first_chat, reader_opens, reading, temper,
};

/// Sending keeps the focus in the composer, by a press or a key (client/pages/README.md).
#[test]
fn enter_starts_one_chat_and_the_composer_keeps_the_focus() {
    let tier = Tier::new("composer").with_model().with_model_picker();
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    let message = || Find::exactly_named(Role::TextArea, b"Message");
    // These chats have no changes, so Work has no Chats filter.
    let chats =
        || Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) });
    let mut stories = Stories::of_person(0);
    let mut steps = Vec::new();
    stories.begin("composer", &steps);
    steps.extend([
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        chats(),
    ]);
    steps.extend(temper_client_browser_world::start_change(true));
    steps.extend([
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
    ]);
    stories.begin("model picker", &steps);
    steps.extend(model_picker_steps());
    stories.begin("composer", &steps);
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
    ]);
    stories.begin("panic", &steps);
    steps.extend([
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/test/panic".as_slice()) }),
        Step::Person(PersonStep::See {
            find: Find::text(b"The application failed."),
            within: skein_lib::Duration::from_secs(6),
        }),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut setup = tier.setup();
    setup.expected.push(b"Uncaught".as_slice().into());
    stories.assert_done(temper_client_browser_world::harness(setup).run(&mut [&mut script]));
}

#[test]
#[expect(clippy::too_many_lines, reason = "the named stories share one playground and harness")]
fn history_keeps_the_chat_focus_and_filter_across_reload_and_back() {
    let tier = Tier::demo_asked(
        "history-chat-filter-address",
        "Add CSV export to widget",
        serde_json::json!([
            {"lines":[{"call":"read_docs","arguments":{"topic":"hold"}}],"finish":"tool_calls","tokens":8},
            {"lines":[{"call":"read_docs","arguments":{"topic":"chat"}}],"finish":"tool_calls","tokens":8},
            "I read the hold and chat pages."
        ]),
    );
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let pressed = |name: &[u8]| Step::Pressed { find: Find::exactly_named(Role::Button, name), within: WITHIN };
    let needs_page = || Step::Keep {
        selector: b"main.temper-needs".as_slice().into(),
        attribute: Some(b"class".as_slice().into()),
        name: b"needs-page".as_slice().into(),
    };
    let words = b"Add CSV export to widget";
    let mut steps = temper_client_browser_world::playground_ready();
    let mut stories = Stories::of_person(0);
    stories.begin("phone ask", &steps);
    let ask = || Find::exactly_named(Role::TextArea, b"Ask temper, or tell it how you work");
    let tabs = || Find::exactly_named(Role::Navigation, b"Sections");
    steps.extend([
        Step::Viewport { width: 420, height: 900 },
        // NewChat hides its tab bar on a phone; open the project's Home by address.
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/projects/1".as_slice()) }),
        see(Find::exactly_named(Role::Region, b"To begin")),
        temper_client_browser_world::connected(),
        press(Role::Button, b"Start"),
        Step::Address(b"/projects/1/ask".as_slice().into()),
        see(ask()),
        see(Find::exactly_named(Role::Tabs, b"Ways of working")),
        press(Role::Tab, b"Draft first"),
        see(Find::exactly_named(Role::Button, b"Use this")),
        see(tabs()),
        keep_box(ask(), "phone-ask/composer"),
        keep_box(Find::named(Role::Disclosure, b"Model"), "phone-ask/picker"),
        keep_box(tabs(), "phone-ask/tabs"),
        Step::Viewport { width: 1280, height: 800 },
        // Start adds an entry: Back through Home to the ready playground's NewChat.
        Step::Back,
        Step::Address(b"/projects/1".as_slice().into()),
        Step::Back,
        Step::Address(b"/chats/new".as_slice().into()),
        see(tier.states().live),
        temper_client_browser_world::connected(),
    ]);
    stories.begin("call line", &steps);
    steps.extend(temper_client_browser_world::playground_chat(words));
    steps.extend(call_line_steps());
    stories.begin("history", &steps);
    steps.extend([
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        // Starting replaces NewChat: Back returns to the list that opened it.
        Step::Back,
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        Step::Address(b"/chats".as_slice().into()),
        Step::Forward,
        see(first_chat(words)),
    ]);
    stories.begin("project across a reload", &steps);
    steps.extend(reload_chats(&tier));
    stories.begin("night across a reload", &steps);
    steps.extend([
        press(Role::Button, b"Night mode"),
        pressed(b"Night mode"),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats") }),
        pressed(b"Night mode"),
        press(Role::Button, b"Night mode"),
    ]);
    stories.begin("history", &steps);
    steps.extend([
        press(Role::Link, FIRST_CHAT),
        see(first_chat(words)),
        Step::Address(FIRST_CHAT_ADDRESS.into()),
        Step::Back,
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
        Step::Forward,
        see(first_chat(words)),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/".as_slice()) }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        // The rail comes with the page once `me` is read; until then the main is empty and busy
        // (temper/docs/design/client/view.md, 1).
        see(Find::exactly_named(Role::Button, b"New draft or change")),
        see(Find::exactly_named(Role::Main, b"")),
        needs_page(),
    ]);
    steps.extend(temper_client_browser_world::start_change(true));
    steps.extend([
        see(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Focused(Find::exactly_named(Role::TextArea, b"What do you want done?")),
        Step::Address(b"/chats/new".as_slice().into()),
        // This chat read docs without starting work; its Work filter is empty.
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(Find::exactly_named(Role::Button, b"Closed")),
        press(Role::Button, b"Closed"),
        Step::Address(b"/chats?state=closed".as_slice().into()),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats?state=closed") }),
        see(Find::exactly_named(Role::Button, b"Closed")),
        pressed(b"Closed"),
        // Loading with settings again made a history entry before the page cleaned it.
        Step::Back,
        Step::Back,
        see(Find::exactly_named(Role::Button, b"All")),
        Step::Address(b"/chats".as_slice().into()),
        pressed(b"All"),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/nowhere".as_slice()) }),
        see(Find::exactly_named(Role::Button, b"New draft or change")),
        see(Find::exactly_named(Role::Main, b"")),
        needs_page(),
        Step::Address(b"/".as_slice().into()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = temper_client_browser_world::harness(tier.setup());
    stories.assert_done(harness.run(&mut [&mut script]));
    let (composer_y, composer_height) = box_of(&harness, "phone-ask", "composer");
    let (picker_y, picker_height) = box_of(&harness, "phone-ask", "picker");
    let (tabs_y, _) = box_of(&harness, "phone-ask", "tabs");
    assert!(
        composer_y + composer_height <= tabs_y,
        "phone ask: composer {composer_y} + {composer_height} overlaps tabs at {tabs_y}"
    );
    assert!(
        picker_y + picker_height <= tabs_y,
        "phone ask: picker {picker_y} + {picker_height} overlaps tabs at {tabs_y}"
    );
    let lines: Vec<_> = harness.kept().iter().filter(|kept| kept.name.as_ref() == b"call-line").collect();
    let [before, after] = lines.as_slice() else {
        panic!("call line: the open call line is kept before and after the reply")
    };
    assert_eq!(before.value, after.value, "call line: new words preserve the open call line's node");
}

fn call_line_steps() -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let gone = |find| Step::Person(PersonStep::Gone { find, within: ABSENT });
    let row = |topic: &[u8]| Find::named(Role::Paragraph, &[b"read_docs ".as_slice(), topic].concat());
    let line = || Find::exactly_named(Role::Disclosure, b"Read 2 docs");
    let kept = || Step::Keep {
        selector: b"details.temper-work-run[open]".as_slice().into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: b"call-line".as_slice().into(),
    };
    // The read_docs answer starts with its topic's title; its page begins on the second line.
    let body = || Find::text(b"# Hold");
    vec![
        see(Find::text(b"I read the hold and chat pages.")),
        see(line()),
        gone(row(b"hold")),
        gone(row(b"chat")),
        gone(body()),
        Step::Person(PersonStep::Press { find: line() }),
        see(row(b"hold")),
        see(row(b"chat")),
        kept(),
        gone(body()),
        Step::Person(PersonStep::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"What about those pages?".as_slice().into(),
        }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
        see(Find::exactly_named(Role::Paragraph, b"What about those pages?")),
        see(Find::text(b"I am the scripted model,")),
        see(row(b"hold")),
        see(row(b"chat")),
        kept(),
        gone(body()),
    ]
}

fn model_picker_steps() -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let picker = || Step::Person(PersonStep::Press { find: Find::named(Role::Disclosure, b"Model") });
    let folded = |open| keep(if open { "details.temper-model[open]" } else { "details.temper-model:not([open])" });
    vec![
        Step::Viewport { width: 420, height: 900 },
        picker(),
        folded(true),
        see(Find::text(b"A change applies from the next turn.")),
        press(Role::Button, b"GPT-6 Astra"),
        see(Find::text(b"GPT-6 Astra \xc2\xb7 high")),
        // A single press changes the line; no Apply control is involved.
        picker(),
        folded(false),
        picker(),
        folded(true),
        keep(".temper-model-menu li > button[aria-pressed=true]"),
        press(Role::Button, b"low"),
        see(Find::text(b"GPT-6 Astra \xc2\xb7 low")),
        keep(".temper-model-effort button[aria-pressed=true]"),
        Step::Viewport { width: 1440, height: 900 },
        press(Role::TextArea, b"Message"),
        folded(false),
        picker(),
        folded(true),
        Step::Key(Key::Escape),
        folded(false),
        Step::Viewport { width: 1280, height: 800 },
    ]
}

fn keep(selector: &str) -> Step {
    Step::Keep {
        selector: selector.as_bytes().into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: b"model-picker".as_slice().into(),
    }
}

fn reload_chats(tier: &Tier) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let heading = |level, name: &[u8]| Find::exactly_named(Role::Heading { level }, name);
    let behind = || Find::exactly_named(Role::Status, b"Behind");
    let chat = || Find::exactly_named(Role::Link, FIRST_CHAT);
    vec![
        // The playground's project made, the link stays up across a reload: the page's own
        // client bounds receive the box's Me (temper/docs/design/client/README.md, 2).
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(heading(1, b"Talk to an agent about playground/widget.")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
        see(chat()),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats") }),
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
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/projects/1".as_slice()) }),
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
    assert_eq!(temper_client_browser_world::harness(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

/// One `temper box`, three chats through ctl, a page load for each story (client/web/testing.md, 6, item 14).
#[test]
fn a_chat_opens_at_its_end_and_reads_back_keeping_its_place() {
    let tier = Tier::reading("reading");
    let mut calls = tier.ctl_chat(reading::SHORT_START, &[]);
    let position: Vec<_> = (1..=reading::POSITION_MESSAGES).map(reading::position_words).collect();
    let position: Vec<_> = position.iter().map(|words| std::str::from_utf8(words).expect("position words")).collect();
    calls.extend(tier.ctl_next_chat(2, position[0], &position[1..]));
    let long: Vec<_> = (1..=reading::LONG_MESSAGES).map(reading::long_words).collect();
    let long: Vec<_> = long.iter().map(|words| std::str::from_utf8(words).expect("long words")).collect();
    calls.extend(tier.ctl_next_chat(3, long[0], &long[1..]));
    let mut ctl = tier.ctl_face(calls);
    let mut stories = Stories::of_person(1);
    let mut steps = reader_opens(FIRST_CHAT_ADDRESS);
    stories.begin("short", &steps);
    steps.extend(short_steps());
    stories.begin("latest", &steps);
    steps.extend(latest_steps(&tier));
    stories.begin("unanchored", &steps);
    steps.extend(unanchored_steps(&tier));
    stories.begin("long", &steps);
    steps.extend(long_steps(&tier));
    let mut reader = Script::new(steps.into_boxed_slice());
    let mut harness = temper_client_browser_world::harness(tier.held_setup());
    let outcome = harness.run(&mut [&mut ctl, &mut reader]);
    eprintln!("kept boxes: {:?}", harness.kept());
    stories.assert_done(outcome);

    in_log(&harness, "short", "start");
    in_log(&harness, "short", "end");
    in_log(&harness, "latest", "before-incoming");
    held(&harness, "latest", "before-incoming", "after-incoming");
    in_log(&harness, "latest", "latest-words");
    in_log(&harness, "latest", "sent-words");
    reading_measures(&harness, "unanchored");
    reading_measures(&harness, "long");
    in_log(&harness, "long", "before-later");
    held(&harness, "long", "before-later", "after-later");
    in_log(&harness, "long", "latest-end");
    // Read at unanchored's end, after every read, so the rule held throughout that story.
    assert_eq!(
        box_of(&harness, "unanchored", "log-end").0,
        box_of(&harness, "unanchored", "log-default").0 + 7,
        "unanchored: the rule holds at its height"
    );
    assert_eq!(
        box_of(&harness, "long", "log").0,
        box_of(&harness, "unanchored", "log-default").0,
        "long: the browser's anchoring is back"
    );
}

fn short_steps() -> Vec<Step> {
    vec![
        see(Find::text(reading::SHORT_END.as_bytes())),
        gone(Find::exactly_named(Role::Status, b"Loading earlier messages")),
        gone(Find::exactly_named(Role::Button, b"Older")),
        gone(Find::exactly_named(Role::Button, b"Latest")),
        keep_box(Find::named(Role::Log, b"Conversation"), "short/log"),
        keep_box(paragraph(reading::SHORT_START.as_bytes()), "short/start"),
        keep_box(Find::text(reading::SHORT_END.as_bytes()), "short/end"),
    ]
}

fn latest_steps(tier: &Tier) -> Vec<Step> {
    const INCOMING: &[u8] = b"New words from the other reader.";
    const SENT: &[u8] = b"Sent while far back in the conversation.";
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let latest = || Find::exactly_named(Role::Button, b"Latest");
    let words = reading::latest_words;
    let mut steps = vec![
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats".as_slice()) }),
        see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
    ];
    let history: Vec<_> = (1..=reading::LATEST_MESSAGES).map(words).collect();
    let then: Vec<_> = history.iter().map(|words| std::str::from_utf8(words).expect("latest words")).collect();
    steps.extend(tier.ctl_messages(1, 0, &then));
    steps.push(Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(FIRST_CHAT_ADDRESS) }));
    steps.extend([see(paragraph(&words(reading::LATEST_MESSAGES))), see(paragraph(&words(1)))]);
    steps.extend([
        gone(latest()),
        Step::Person(PersonStep::Press { find: composer() }),
        Step::Key(Key::PageUp),
        Step::Key(Key::PageUp),
        Step::Focused(composer()),
        see(latest()),
        keep_box(Find::named(Role::Log, b"Conversation"), "latest/log"),
        keep_box(paragraph(&words(2)), "latest/before-incoming"),
    ]);
    steps.extend(tier.ctl(&[
        "task",
        "message",
        "1",
        std::str::from_utf8(INCOMING).expect("incoming words"),
        "--key",
        "history-incoming",
    ]));
    steps.extend([
        see(paragraph(INCOMING)),
        keep_box(paragraph(&words(2)), "latest/after-incoming"),
        see(latest()),
        Step::Person(PersonStep::Press { find: latest() }),
        gone(latest()),
        keep_box(paragraph(INCOMING), "latest/latest-words"),
        Step::Person(PersonStep::Press { find: composer() }),
        Step::Key(Key::PageUp),
        Step::Key(Key::PageUp),
        see(latest()),
        Step::Person(PersonStep::Type { find: composer(), words: SENT.into() }),
        Step::Key(Key::Enter),
        see(paragraph(SENT)),
        gone(latest()),
        keep_box(paragraph(SENT), "latest/sent-words"),
    ]);
    steps
}

fn reading_chat(address: &[u8], last: &[u8]) -> Vec<Step> {
    vec![
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(address) }),
        see(paragraph(last)),
        gone(Find::exactly_named(Role::Status, b"Loading earlier messages")),
        gone(Find::exactly_named(Role::Button, b"Older")),
        gone(Find::exactly_named(Role::Button, b"Latest")),
    ]
}

fn unanchored_steps(tier: &Tier) -> Vec<Step> {
    let mut steps = reading_chat(b"/chats/2", &reading::position_words(reading::POSITION_MESSAGES));
    steps.extend([
        keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-default"),
        reading::anchoring_off(),
    ]);
    steps.extend(reading_back_steps(
        tier,
        "unanchored",
        reading::POSITION_MESSAGES,
        reading::position_words,
        reply_end(1),
    ));
    steps.extend([keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-end"), reading::anchoring_on()]);
    steps
}

fn long_steps(tier: &Tier) -> Vec<Step> {
    let messages = reading::LONG_MESSAGES;
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let mut steps = reading_chat(b"/chats/3", &reading::long_words(messages));
    steps.extend(reading_back_steps(
        tier,
        "long",
        messages,
        reading::long_words,
        paragraph(&reading::long_words(messages - 2)),
    ));
    read_to_start(tier, &mut steps, messages - 2);
    steps.extend([gone(reply_end(messages)), see(Find::exactly_named(Role::Button, b"Latest"))]);
    // Hold the later read while PageDown reaches the retained window's foot.
    steps.extend(serve_signal(tier, "-STOP"));
    for _ in 0..6 {
        steps.push(Step::Key(Key::PageDown));
    }
    steps.push(keep_box(reply_end(2), "long/before-later"));
    steps.extend(serve_signal(tier, "-CONT"));
    steps.extend([
        see(reply_end(3)),
        keep_box(reply_end(2), "long/after-later"),
        Step::Focused(composer()),
        see(Find::exactly_named(Role::Button, b"Latest")),
    ]);
    read_to_end(tier, &mut steps, messages);
    steps.push(gone(paragraph(&reading::long_words(1))));
    // Read the dropped head again, dropping the newest reply before Latest restores it.
    read_to_start(tier, &mut steps, messages - 1);
    steps.extend([
        see(paragraph(&reading::long_words(1))),
        gone(reply_end(messages)),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Latest") }),
        see(reply_end(messages)),
        gone(Find::exactly_named(Role::Button, b"Latest")),
        keep_box(reply_end(messages), "long/latest-end"),
    ]);
    steps
}

fn reading_back_steps(
    tier: &Tier,
    story: &str,
    messages: u32,
    words: fn(u32) -> Box<[u8]>,
    earlier: Find,
) -> Vec<Step> {
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let anchor = || paragraph(&words(messages - 1));
    let loading = || Find::exactly_named(Role::Status, b"Loading earlier messages");
    let keep = |find, name: &str| keep_box(find, &format!("{story}/{name}"));
    let mut steps = Vec::new();
    steps.extend([
        keep(Find::named(Role::Log, b"Conversation"), "log"),
        keep(reply_end(messages), "end"),
        keep(anchor(), "head"),
        Step::Person(PersonStep::Press { find: composer() }),
    ]);
    steps.extend(serve_signal(tier, "-STOP"));
    steps.extend([Step::Key(Key::PageUp), Step::Focused(composer()), keep(reply_end(messages), "up")]);
    for _ in 0..5 {
        steps.push(Step::Key(Key::PageUp));
    }
    steps.extend([
        Step::Focused(composer()),
        see(loading()),
        see(Find::text("Loading earlier messages…".as_bytes())),
        keep(anchor(), "before-earlier"),
    ]);
    steps.extend(serve_signal(tier, "-CONT"));
    steps.extend([see(earlier), gone(loading()), keep(anchor(), "after-earlier"), Step::Focused(composer())]);
    steps
}

fn reading_measures(harness: &Harness, story: &str) {
    in_log(harness, story, "end");
    in_log(harness, story, "before-earlier");
    let (top, height) = box_of(harness, story, "log");
    let (head, _) = box_of(harness, story, "head");
    assert!((top - head) * 2 >= height * 3, "{story}: head {head}, log {top} + {height}");
    assert!(
        box_of(harness, story, "up").0 > box_of(harness, story, "end").0,
        "{story}: PageUp scrolls from the composer"
    );
    held(harness, story, "before-earlier", "after-earlier");
}

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}
fn gone(find: Find) -> Step {
    Step::Person(PersonStep::Gone { find, within: ABSENT })
}
fn paragraph(words: &[u8]) -> Find {
    Find::exactly_named(Role::Paragraph, words)
}
fn keep_box(find: Find, name: &str) -> Step {
    Step::KeepBox { find, name: name.as_bytes().into() }
}
fn box_of(harness: &Harness, story: &str, name: &str) -> (i64, i64) {
    let name = format!("{story}/{name}");
    let kept = harness
        .kept()
        .iter()
        .find(|kept| kept.name.as_ref() == name.as_bytes())
        .unwrap_or_else(|| panic!("{story}: {name}: kept box"));
    let value = std::str::from_utf8(&kept.value).unwrap_or_else(|_| panic!("{story}: {name}: box coordinates"));
    let (y, height) = value.split_once(' ').unwrap_or_else(|| panic!("{story}: {name}: y and height"));
    (
        y.parse().unwrap_or_else(|_| panic!("{story}: {name}: signed y")),
        height.parse().unwrap_or_else(|_| panic!("{story}: {name}: height")),
    )
}
fn in_log(harness: &Harness, story: &str, name: &str) {
    let (top, height) = box_of(harness, story, "log");
    let (y, h) = box_of(harness, story, name);
    assert!(y >= top && y + h <= top + height, "{story}: {name}: {y} + {h} outside {top} + {height}");
}

fn reply_end(number: u32) -> Find {
    Find::text(format!("End of reply {number}.").as_bytes())
}
fn read_to_start(tier: &Tier, steps: &mut Vec<Step>, from: u32) {
    // Hold each read until the keys reach the retained head, then await the next message.
    steps.push(see(paragraph(&reading::long_words(from))));
    for number in (2..from).rev() {
        read_earlier(tier, steps, paragraph(&reading::long_words(number)));
    }
    // The first reply and the first message cross separate activity pages.
    read_earlier(tier, steps, reply_end(1));
    read_earlier(tier, steps, paragraph(&reading::long_words(1)));
}
fn read_earlier(tier: &Tier, steps: &mut Vec<Step>, next: Find) {
    steps.extend(serve_signal(tier, "-STOP"));
    for _ in 0..4 {
        steps.push(Step::Key(Key::PageUp));
    }
    steps.extend(serve_signal(tier, "-CONT"));
    steps.push(see(next));
}
fn read_to_end(tier: &Tier, steps: &mut Vec<Step>, messages: u32) {
    for number in 4..=messages {
        steps.extend(serve_signal(tier, "-STOP"));
        for _ in 0..4 {
            steps.push(Step::Key(Key::PageDown));
        }
        steps.extend(serve_signal(tier, "-CONT"));
        steps.push(see(reply_end(number)));
    }
}
fn held(harness: &Harness, story: &str, before: &str, after: &str) {
    let before = box_of(harness, story, before);
    let after = box_of(harness, story, after);
    assert!((before.0 - after.0).abs() <= 1, "{story}: {before:?} -> {after:?}: words moved");
    assert_eq!(before.1, after.1, "{story}: line height stays");
}

// Stop only this scratch deployment's serve process, leaving its browser running.
// This holds a read out long enough to measure the words before its answer lands.
fn serve_signal(tier: &Tier, signal: &str) -> [Step; 2] {
    use skein_fake_person_browser::{Command, Stdin};
    let args = tier.box_serve();
    let args = args.iter().map(|arg| arg.to_str().expect("fixture argument")).collect::<Vec<_>>().join(" ");
    let command = format!("{} {args}", temper().display());
    let mut pattern = String::from("^");
    for ch in command.chars() {
        if ".^$*+?()[]{}|\\".contains(ch) {
            pattern.push('\\');
        }
        pattern.push(ch);
    }
    pattern.push('$');
    [
        Step::Run(Command {
            program: "/usr/bin/pkill".into(),
            args: vec![signal.into(), "-f".into(), pattern.into()],
            env: vec![],
            stdin: Stdin::Closed,
            opens: None,
            within: WITHIN,
        }),
        Step::Exited { code: 0, within: WITHIN },
    ]
}
