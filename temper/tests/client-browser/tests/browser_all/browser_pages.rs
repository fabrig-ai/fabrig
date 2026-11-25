//! Only a browser shows Back and Forward, a replaced history entry, the composer's focus,
//! a filter's address across a reload, and an address that's no page. Ask temper's boxes at
//! phone width, disclosures, keys in a text area, where project rename returns focus and
//! the panic page stay here too (client/web/testing.md, 6, stories 2, 4, 5, 11 and 15).
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Stories, Tier, WITHIN, first_chat, reading};

mod serve;
use serve::ReadingSteps;

/// Sending keeps the focus in the composer, by a press or a key (client/pages/README.md).
#[test]
fn enter_starts_one_chat_and_the_composer_keeps_the_focus() {
    let tier = Tier::new("composer").with_model().with_model_picker();
    let mut owner = tier.project_face();
    let mut stories = Stories::of_person(1);
    let mut steps = signed_in_steps();
    stories.begin("composer", &steps);
    steps.extend(composer_steps());
    steps.extend(model_picker_steps());
    steps.extend([
        go(b"/chats"),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        press(Role::Link, FIRST_CHAT),
        see(message()),
        keep_node("main h1:focus", "focused-chat-heading"),
    ]);
    stories.begin("rename", &steps);
    steps.extend(rename_steps());
    stories.begin("panic", &steps);
    steps.extend([
        go(b"/test/panic"),
        Step::Person(PersonStep::See {
            find: Find::text(b"The application failed."),
            within: skein_lib::Duration::from_secs(6),
        }),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut setup = tier.held_setup();
    setup.expected.push(b"Uncaught".as_slice().into());
    stories.assert_done(temper_client_browser_world::harness(setup).run(&mut [&mut owner, &mut script]));
}

fn composer_steps() -> Vec<Step> {
    vec![
        go(b"/chats/new"),
        see(composer()),
        Step::Type { find: composer(), words: b"Hello".as_slice().into() },
        Step::KeyWithShift(Key::Enter),
        Step::Type { find: composer(), words: b"there".as_slice().into() },
        Step::Key(Key::Enter),
        // The first-line title distinguishes a newline from an ignored Shift+Enter.
        see(first_chat(b"Hello")),
        see(message()),
        address(FIRST_CHAT_ADDRESS),
        Step::Focused(message()),
        Step::Person(PersonStep::Type { find: message(), words: b"By the button".as_slice().into() }),
        press(Role::Button, b"Send"),
        see(paragraph(b"By the button")),
        Step::Focused(message()),
        Step::Person(PersonStep::Type { find: message(), words: b"By the key".as_slice().into() }),
        Step::Key(Key::Enter),
        see(paragraph(b"By the key")),
        Step::Focused(message()),
    ]
}

const HOME: &[u8] = b"/projects/1";

#[test]
fn history_keeps_the_chat_focus_and_filter_across_reload_and_back() {
    const WORDS: &str = "Add CSV export to widget";
    let tier = Tier::demo_asked(
        "history-chat-filter-address",
        WORDS,
        serde_json::json!([
            {"lines":[{"call":"read_docs","arguments":{"topic":"hold"}}],"finish":"tool_calls","tokens":8},
            {"lines":[{"call":"read_docs","arguments":{"topic":"chat"}}],"finish":"tool_calls","tokens":8},
            "I read the hold and chat pages."
        ]),
    );
    let _model = tier.scripted_model();
    let mut owner = tier.playground_face();
    let mut steps = signed_in_steps();
    let mut stories = Stories::of_person(1);
    stories.begin("phone ask", &steps);
    steps.extend(phone_ask_steps());
    stories.begin("history", &steps);
    steps.push(go(b"/chats"));
    steps.extend(temper_client_browser_world::start_change(true));
    // One browser start supplies both the replaced NewChat entry and the scripted call line.
    steps.extend([
        see(composer()),
        Step::Focused(composer()),
        address(b"/chats/new"),
        Step::Type { find: composer(), words: WORDS.as_bytes().into() },
        Step::Key(Key::Enter),
        see(message()),
        address(FIRST_CHAT_ADDRESS),
        Step::Focused(message()),
    ]);
    stories.begin("call line", &steps);
    steps.extend(call_line_steps());
    stories.begin("history", &steps);
    steps.extend([Step::Back, address(b"/chats"), Step::Forward, address(FIRST_CHAT_ADDRESS)]);
    stories.begin("night across a reload", &steps);
    steps.extend(night_steps());
    stories.begin("history", &steps);
    steps.extend([
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        press(Role::Link, FIRST_CHAT),
        see(message()),
        address(FIRST_CHAT_ADDRESS),
        Step::Back,
        address(b"/chats"),
        Step::Forward,
        address(FIRST_CHAT_ADDRESS),
    ]);
    // The phone steps need To begin's Start; the Home history checks need the
    // full rail. Seed Ask temper after the T1 checks so its chat completes To begin.
    steps.extend(tier.ctl(&[
        "chat",
        "start",
        "--project",
        "1",
        "--template",
        "5",
        "--model",
        "demo",
        "--key",
        "history-ask",
        "Tell temper how you work",
    ]));
    steps.extend(home_replaces(b"/", FIRST_CHAT_ADDRESS));
    steps.extend(filter_steps());
    steps.extend(home_replaces(b"/nowhere", b"/chats"));
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = temper_client_browser_world::harness(tier.held_setup());
    stories.assert_done(harness.run(&mut [&mut owner, &mut script]));
    phone_ask_geometry(&harness);
    let lines: Vec<_> = harness.kept().iter().filter(|kept| kept.name.as_ref() == b"call-line").collect();
    let [before, after] = lines.as_slice() else {
        panic!("call line: the open call line is kept before and after the reply")
    };
    assert_eq!(before.value, after.value, "call line: new words preserve the open call line's node");
}

fn phone_ask_steps() -> Vec<Step> {
    let ask = Find::exactly_named(Role::TextArea, b"Ask temper, or tell it how you work");
    let picker = Find::named(Role::Disclosure, b"Model");
    let tabs = Find::exactly_named(Role::Navigation, b"Sections");
    let start = Find::exactly_named(Role::Button, b"Start");
    vec![
        Step::Viewport { width: 420, height: 900 },
        go(HOME),
        see(start.clone()),
        Step::Person(PersonStep::Press { find: start }),
        see(ask.clone()),
        address(b"/projects/1/ask"),
        see(picker.clone()),
        see(tabs.clone()),
        keep_box(ask, "phone-ask/composer"),
        keep_box(picker, "phone-ask/picker"),
        keep_box(tabs, "phone-ask/tab-bar"),
        Step::Back,
        address(HOME),
        Step::Viewport { width: 1280, height: 800 },
    ]
}

fn phone_ask_geometry(harness: &Harness) {
    let (composer_y, composer_height) = box_of(harness, "phone-ask", "composer");
    let (picker_y, picker_height) = box_of(harness, "phone-ask", "picker");
    let (tab_bar_y, _) = box_of(harness, "phone-ask", "tab-bar");
    assert!(
        composer_y + composer_height <= tab_bar_y,
        "phone ask: composer {composer_y} + {composer_height} overlaps the tab bar at {tab_bar_y}"
    );
    assert!(
        picker_y + picker_height <= tab_bar_y,
        "phone ask: picker {picker_y} + {picker_height} overlaps the tab bar at {tab_bar_y}"
    );
}

fn night_steps() -> [Step; 8] {
    const NIGHT: &[u8] = b"Night mode";
    let away = Find::exactly_named(Role::Button, b"Away");
    [
        go(b"/chats"),
        // Away arrives with the project facts and moves Night to its settled position.
        see(away.clone()),
        press(Role::Button, NIGHT),
        pressed(NIGHT),
        go(b"/chats"),
        pressed(NIGHT),
        see(away),
        press(Role::Button, NIGHT),
    ]
}

fn home_replaces(path: &[u8], previous: &[u8]) -> [Step; 8] {
    [
        go(path),
        see(Find::exactly_named(Role::Button, b"New draft or change")),
        Step::Keep {
            selector: b"main.temper-home".as_slice().into(),
            attribute: Some(b"class".as_slice().into()),
            name: b"home-page".as_slice().into(),
        },
        address(HOME),
        Step::Back,
        address(previous),
        Step::Forward,
        address(HOME),
    ]
}

fn filter_steps() -> [Step; 11] {
    const CLOSED: &[u8] = b"/chats?state=closed";
    [
        go(b"/chats"),
        // Drafts arrives before Closed and moves it; wait for the settled row before pressing.
        see(Find::exactly_named(Role::Button, b"Drafts")),
        press(Role::Button, b"Closed"),
        address(CLOSED),
        go(CLOSED),
        pressed(b"Closed"),
        address(CLOSED),
        // The fresh load with internal settings added an entry of its own.
        Step::Back,
        Step::Back,
        pressed(b"All"),
        address(b"/chats"),
    ]
}

fn call_line_steps() -> Vec<Step> {
    let row = |topic: &[u8]| Find::named(Role::Paragraph, &[b"read_docs ".as_slice(), topic].concat());
    // Chromium includes the run's elapsed time in the disclosure's accessible name.
    let line = || Find::named(Role::Disclosure, b"Read 2 docs");
    let kept =
        || keep_node("details.temper-work-run[open]:not(.temper-work-running):not(.temper-work-failed)", "call-line");
    vec![
        see(Find::text(b"I read the hold and chat pages.")),
        see(line()),
        gone(row(b"hold")),
        gone(row(b"chat")),
        Step::Person(PersonStep::Press { find: line() }),
        see(row(b"hold")),
        see(row(b"chat")),
        kept(),
        Step::Person(PersonStep::Type { find: message(), words: b"What about those pages?".as_slice().into() }),
        press(Role::Button, b"Send"),
        see(paragraph(b"What about those pages?")),
        see(Find::text(b"I am the scripted model,")),
        kept(),
    ]
}

fn model_picker_steps() -> Vec<Step> {
    let picker = || Step::Person(PersonStep::Press { find: Find::named(Role::Disclosure, b"Model") });
    let folded = |open| {
        keep_node(if open { "details.temper-model[open]" } else { "details.temper-model:not([open])" }, "model-picker")
    };
    vec![
        Step::Viewport { width: 420, height: 900 },
        picker(),
        folded(true),
        picker(),
        folded(false),
        picker(),
        folded(true),
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

fn rename_steps() -> Vec<Step> {
    let summary = |name: &[u8]| Find::named(Role::Disclosure, name);
    let main_unfocused = || Step::Keep {
        selector: b"main:not(:focus)".as_slice().into(),
        attribute: Some(b"class".as_slice().into()),
        name: b"main-without-focus".as_slice().into(),
    };
    let rename = || Find::exactly_named(Role::TextField, b"Rename");
    vec![
        go(HOME),
        see(summary(b"Project 1")),
        Step::Person(PersonStep::Press { find: summary(b"Project 1") }),
        press(Role::Button, b"Rename Project 1"),
        Step::Focused(rename()),
        Step::Key(Key::Escape),
        gone(rename()),
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
    ]
}

fn signed_in_steps() -> Vec<Step> {
    let mut steps = temper_client_browser_world::page_opens();
    // Wait for the reader's cookie before leaving the sign-in page.
    steps.push(see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a forge")));
    steps
}

fn go(path: &[u8]) -> Step {
    Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(path) })
}
fn address(path: &[u8]) -> Step {
    Step::Address(path.into())
}
fn press(role: Role, name: &[u8]) -> Step {
    Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) })
}
fn pressed(name: &[u8]) -> Step {
    Step::Pressed { find: Find::exactly_named(Role::Button, name), within: WITHIN }
}
fn composer() -> Find {
    Find::exactly_named(Role::TextArea, b"What do you want done?")
}
fn message() -> Find {
    Find::exactly_named(Role::TextArea, b"Message")
}
fn keep_node(selector: &str, name: &str) -> Step {
    Step::Keep {
        selector: selector.as_bytes().into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: name.as_bytes().into(),
    }
}

/// One `temper box`, three chats through ctl, a page load for each story (client/web/testing.md, 6, item 14).
#[test]
fn a_chat_opens_at_its_end_and_reads_back_keeping_its_place() {
    let tier = Tier::reading("reading");
    let _model = tier.scripted_model();
    let mut calls = tier.ctl_chat(reading::SHORT_START, &[]);
    let position: Vec<_> = (1..=reading::POSITION_MESSAGES).map(reading::position_words).collect();
    let position: Vec<_> = position.iter().map(|words| std::str::from_utf8(words).expect("position words")).collect();
    calls.extend(tier.ctl_next_chat(2, position[0], &position[1..]));
    let long: Vec<_> = (1..=reading::LONG_MESSAGES).map(reading::long_words).collect();
    let long: Vec<_> = long.iter().map(|words| std::str::from_utf8(words).expect("long words")).collect();
    calls.extend(tier.ctl_next_chat(3, long[0], &long[1..]));
    let mut ctl = tier.ctl_face(calls);
    let mut stories = Stories::of_person(1);
    let mut steps = temper_client_browser_world::page_opens();
    steps.extend(temper_client_browser_world::scripted_playground_ready());
    steps.push(Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(FIRST_CHAT_ADDRESS) }));
    stories.begin("short", &steps);
    steps.extend(short_steps());
    stories.begin("latest", &steps);
    steps.extend(latest_steps(&tier));
    let mut reading = ReadingSteps::new(&tier, steps);
    stories.begin("unanchored", &reading.browser);
    unanchored_steps(&mut reading);
    stories.begin("long", &reading.browser);
    long_steps(&mut reading);
    let setup = tier.held_setup();
    let mut reader = reading.player(&setup);
    let mut harness = temper_client_browser_world::harness(setup);
    let outcome = harness.run(&mut [&mut ctl, &mut reader]);
    drop(reader);
    stories.assert_done(outcome);

    in_log(&harness, "short", "start");
    in_log(&harness, "short", "end");
    in_log(&harness, "latest", "before-incoming");
    held(&harness, "latest", "before-incoming", "after-incoming");
    in_log(&harness, "latest", "latest-words");
    in_log(&harness, "latest", "sent-words");
    reading_measures(&harness, "unanchored");
    reading_measures(&harness, "long");
    in_log(&harness, "long/later", "before");
    held(&harness, "long/later", "before", "after");
    in_log(&harness, "long/later", "latest-end");
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
        see(paragraph(reading::SHORT_START.as_bytes())),
        see(Find::text(reading::SHORT_END.as_bytes())),
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
        // The pending line gets its message number when the send answers.
        gone(Find::exactly_named(Role::Status, b"Being sent")),
        gone(latest()),
        keep_box(paragraph(SENT), "latest/sent-words"),
    ]);
    steps
}

fn reading_chat(address: &[u8], last: &[u8], second_last: &[u8]) -> Vec<Step> {
    vec![
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(address) }),
        see(paragraph(last)),
        // Opening holds only a weight; the second-last message comes by the log's near read.
        see(paragraph(second_last)),
        gone(loading_earlier()),
        gone(Find::exactly_named(Role::Button, b"Latest")),
    ]
}

fn unanchored_steps(steps: &mut ReadingSteps<'_>) {
    steps.browser.extend(reading_chat(
        b"/chats/2",
        &reading::position_words(reading::POSITION_MESSAGES),
        &reading::position_words(reading::POSITION_MESSAGES - 1),
    ));
    steps.browser.extend([
        gone(Find::exactly_named(Role::Button, b"Older")),
        gone(paragraph(&reading::position_words(1))),
        keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-default"),
        reading::anchoring_off(),
    ]);
    reading_back_steps(steps, "unanchored", reading::POSITION_MESSAGES, reading::position_words, reply_end(1));
    steps
        .browser
        .extend([keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-end"), reading::anchoring_on()]);
}

fn long_steps(steps: &mut ReadingSteps<'_>) {
    const UP_TO_OVERLAP: u32 = 3;
    let messages = reading::LONG_MESSAGES;
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    steps.browser.extend(reading_chat(b"/chats/3", &reading::long_words(messages), &reading::long_words(messages - 1)));
    reading_back_steps(steps, "long", messages, reading::long_words, paragraph(&reading::long_words(messages - 2)));
    // Stop short of the opening. A shorter log lets the next PgUp read leave us
    // inside the retained reply but outside the foot's read margin. One PgDown
    // then reaches that margin without leaving the new window near its head.
    let latest = || Find::exactly_named(Role::Button, b"Latest");
    steps.browser.push(Step::Viewport { width: 1280, height: reading::LATER_HEIGHT });
    steps.stop();
    for _ in 0..UP_TO_OVERLAP {
        steps.browser.push(Step::Key(Key::PageUp));
    }
    steps.browser.push(see(loading_earlier()));
    steps.resume();
    steps.browser.extend([
        see(paragraph(&reading::long_words(messages - 3))),
        gone(loading_earlier()),
        gone(reply_end(messages)),
        see(latest()),
        keep_box(Find::named(Role::Log, b"Conversation"), "long/later/log"),
    ]);

    // Both replies carry the numbered paragraph. It belongs to the newer
    // reply before the slide, and to the older reply after it.
    let overlap = || Find::paragraph_text(b"Numbered line 26. Keep this paragraph available while reading.");
    steps.stop();
    steps.browser.extend([Step::Key(Key::PageDown), keep_box(overlap().nth(1), "long/later/before")]);
    steps.resume();
    steps.browser.extend([
        see(reply_end(messages - 1)),
        keep_box(overlap(), "long/later/after"),
        Step::Focused(composer()),
        see(latest()),
        Step::Person(PersonStep::Press { find: latest() }),
        see(reply_end(messages)),
        gone(latest()),
        keep_box(reply_end(messages), "long/later/latest-end"),
    ]);
}

fn reading_back_steps(
    steps: &mut ReadingSteps<'_>,
    story: &str,
    messages: u32,
    words: fn(u32) -> Box<[u8]>,
    earlier: Find,
) {
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let anchor = || paragraph(&words(messages - 1));
    let loading = loading_earlier;
    let keep = |find, name: &str| keep_box(find, &format!("{story}/{name}"));
    steps.browser.extend([
        see(reply_end(messages)),
        keep(Find::named(Role::Log, b"Conversation"), "log"),
        keep(reply_end(messages), "end"),
        keep(anchor(), "head"),
        Step::Person(PersonStep::Press { find: composer() }),
    ]);
    // Opening and the viewport change must finish reading before this read is held.
    steps.browser.push(gone(loading()));
    steps.stop();
    steps.browser.extend([Step::Key(Key::PageUp), Step::Focused(composer()), keep(reply_end(messages), "up")]);
    for _ in 0..5 {
        steps.browser.push(Step::Key(Key::PageUp));
    }
    steps.browser.extend([Step::Focused(composer()), see(loading()), keep(anchor(), "before-earlier")]);
    steps.resume();
    steps.browser.extend([see(earlier), gone(loading()), keep(anchor(), "after-earlier"), Step::Focused(composer())]);
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
fn loading_earlier() -> Find {
    Find::exactly_named(Role::Status, b"Loading earlier messages")
}
fn held(harness: &Harness, story: &str, before: &str, after: &str) {
    let before = box_of(harness, story, before);
    let after = box_of(harness, story, after);
    assert!((before.0 - after.0).abs() <= 1, "{story}: {before:?} -> {after:?}: words moved");
    assert_eq!(before.1, after.1, "{story}: line height stays");
}
