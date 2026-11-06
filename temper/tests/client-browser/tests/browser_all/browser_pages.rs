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

// Each reader signals the next stage without a timing allowance between tabs.
struct Stage<'a> {
    script: Script,
    wait: Option<&'a std::cell::Cell<bool>>,
    signal: Option<&'a std::cell::Cell<bool>>,
}
struct Together<'a> {
    stages: std::collections::VecDeque<Stage<'a>>,
}
impl skein_fake_person_browser::Play for Together<'_> {
    fn next(
        &mut self,
        face: &mut skein_fake_person_browser::BrowserFace<'_>,
        now: skein_lib::Time,
    ) -> skein_fake_person::Next<Box<[u8]>> {
        use skein_fake_person::Next;
        let Some(stage) = self.stages.front_mut() else {
            return Next::Done;
        };
        if stage.wait.is_some_and(|ready| !ready.get()) {
            return Next::Wait { until: None };
        }
        match stage.script.next(face, now) {
            Next::Done => {
                if let Some(signal) = stage.signal {
                    signal.set(true);
                }
                self.stages.pop_front();
                Next::Wait { until: None }
            }
            next => next,
        }
    }
    fn at(&self) -> u32 {
        self.stages.front().map_or(0, |stage| stage.script.at())
    }
    fn check_deadline(&mut self, face: &mut skein_fake_person_browser::BrowserFace<'_>, now: skein_lib::Time) {
        if let Some(stage) = self.stages.front_mut() {
            stage.script.check_deadline(face, now);
        }
    }
}
#[test]
fn latest_returns_a_chat_to_its_end_after_new_words_while_reading_back() {
    const MESSAGES: u32 = 6;
    const INCOMING: &[u8] = b"New words from the other reader.";
    const SENT: &[u8] = b"Sent while far back in the conversation.";
    let tier = Tier::demo("chat-latest");
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let latest = || Find::exactly_named(Role::Button, b"Latest");
    let words = |number| {
        format!("Reading paragraph {number}. {}", "Keep these words available while reading back. ".repeat(4))
            .into_bytes()
            .into_boxed_slice()
    };
    let mut opening = temper_client_browser_world::start_playground_chat(b"Add CSV export to widget");
    opening.extend([see(Find::exactly_named(Role::Region, b"Task 2")), Step::Reopen]);
    let mut steps = vec![
        Step::Person(PersonStep::Go { address: FIRST_CHAT_ADDRESS.into() }),
        see(first_chat(b"Add CSV export to widget")),
    ];
    for number in 1..=MESSAGES {
        steps.extend([
            Step::Person(PersonStep::Type { find: composer(), words: words(number) }),
            Step::Key(Key::Enter),
            see(paragraph(&words(number))),
            Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Being sent"), within: WITHIN }),
        ]);
    }
    steps.extend([
        gone(latest()),
        Step::Person(PersonStep::Press { find: composer() }),
        Step::Key(Key::PageUp),
        Step::Key(Key::PageUp),
        Step::Focused(composer()),
        see(latest()),
        keep_box(Find::named(Role::Log, b"Conversation"), "log"),
        keep_box(paragraph(&words(2)), "before-incoming"),
    ]);
    let following = vec![
        see(paragraph(INCOMING)),
        keep_box(paragraph(&words(2)), "after-incoming"),
        see(latest()),
        Step::Person(PersonStep::Press { find: latest() }),
        gone(latest()),
        keep_box(paragraph(INCOMING), "latest-words"),
        Step::Person(PersonStep::Press { find: composer() }),
        Step::Key(Key::PageUp),
        Step::Key(Key::PageUp),
        see(latest()),
        Step::Person(PersonStep::Type { find: composer(), words: SENT.into() }),
        Step::Key(Key::Enter),
        see(paragraph(SENT)),
        gone(latest()),
        keep_box(paragraph(SENT), "sent-words"),
    ];
    let linked = std::cell::Cell::new(false);
    let ready = std::cell::Cell::new(false);
    let stage = |steps: Vec<Step>, wait, signal| Stage { script: Script::new(steps.into_boxed_slice()), wait, signal };
    let mut reader = Together {
        stages: [stage(opening, None, None), stage(steps, Some(&linked), Some(&ready)), stage(following, None, None)]
            .into(),
    };
    let mut sender = Together {
        stages: [
            stage(
                vec![
                    see(Find::exactly_named(Role::Button, b"Try the playground")),
                    Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Try the playground") }),
                    Step::Person(PersonStep::Go { address: FIRST_CHAT_ADDRESS.into() }),
                    see(first_chat(b"Add CSV export to widget")),
                ],
                None,
                Some(&linked),
            ),
            stage(
                vec![
                    Step::Person(PersonStep::Go { address: FIRST_CHAT_ADDRESS.into() }),
                    see(paragraph(&words(MESSAGES))),
                    Step::Person(PersonStep::Type { find: composer(), words: INCOMING.into() }),
                    Step::Key(Key::Enter),
                    see(paragraph(INCOMING)),
                ],
                Some(&ready),
                None,
            ),
        ]
        .into(),
    };
    let mut harness = Harness::new(tier.setup());
    let outcome = harness.run(&mut [&mut reader, &mut sender]);
    assert_eq!(outcome, Outcome::Done, "linked {}, ready {}", linked.get(), ready.get());
    in_log(&harness, "before-incoming");
    held(&harness, "before-incoming", "after-incoming");
    in_log(&harness, "latest-words");
    in_log(&harness, "sent-words");
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
fn box_of(harness: &Harness, name: &str) -> (i64, i64) {
    let kept = harness.kept().iter().find(|kept| kept.name.as_ref() == name.as_bytes()).expect("kept box");
    let value = std::str::from_utf8(&kept.value).expect("box coordinates");
    let (y, height) = value.split_once(' ').expect("y and height");
    (y.parse().expect("signed y"), height.parse().expect("height"))
}
fn in_log(harness: &Harness, name: &str) {
    let (top, height) = box_of(harness, "log");
    let (y, h) = box_of(harness, name);
    assert!(y >= top && y + h <= top + height, "{name}: {y} + {h} outside {top} + {height}");
}
#[test]
fn a_short_chat_opens_whole_without_a_reading_status() {
    const START: &[u8] = b"A short conversation";
    const END: &[u8] = b"The whole answer.";
    let tier = Tier::demo("reading-short").with_replies(&[("A short conversation", "The whole answer.".into())]);
    let mut steps = temper_client_browser_world::start_playground_chat(START);
    steps.extend([
        see(Find::text(END)),
        Step::Person(PersonStep::Reload),
        see(Find::text(END)),
        gone(Find::exactly_named(Role::Status, b"Loading earlier messages")),
        gone(Find::exactly_named(Role::Button, b"Older")),
        gone(Find::exactly_named(Role::Button, b"Latest")),
        keep_box(Find::named(Role::Log, b"Conversation"), "log"),
        keep_box(paragraph(START), "start"),
        keep_box(Find::text(END), "end"),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(tier.setup());
    assert_eq!(harness.run(&mut [&mut script]), Outcome::Done);
    in_log(&harness, "start");
    in_log(&harness, "end");
}

const READING_MESSAGES: u32 = 16;
fn reading_words(number: u32) -> Box<[u8]> {
    format!("Reading paragraph {number:02}.").into_bytes().into_boxed_slice()
}
fn reply_end(number: u32) -> Find {
    Find::text(format!("End of reply {number}.").as_bytes())
}
fn reading_history(messages: u32) -> Vec<Step> {
    let mut steps = temper_client_browser_world::start_playground_chat(&reading_words(1));
    steps.push(see(Find::text(b"End of the large reply.")));
    for number in 2..=messages {
        steps.extend([
            Step::Person(PersonStep::Type {
                find: Find::exactly_named(Role::TextArea, b"Message"),
                words: reading_words(number),
            }),
            Step::Key(Key::Enter),
            see(paragraph(&reading_words(number))),
            Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Being sent"), within: WITHIN }),
            see(Find::exactly_named(Role::Status, b"waiting on your words")),
        ]);
    }
    steps.extend([
        Step::Person(PersonStep::Reload),
        see(paragraph(&reading_words(messages))),
        gone(Find::exactly_named(Role::Status, b"Loading earlier messages")),
        gone(Find::exactly_named(Role::Button, b"Older")),
        gone(Find::exactly_named(Role::Button, b"Latest")),
    ]);
    steps
}
fn reading_tier(anchoring: bool, messages: u32) -> Tier {
    use std::fmt::Write as _;
    let tier =
        if anchoring { Tier::demo("reading-anchored") } else { Tier::demo_without_anchoring("reading-unanchored") };
    // A long, horizontally scrollable code line crosses the byte bound without thousands of page keys.
    let mut reply = format!("```text\n{}\n```\n\nEnd of the large reply.", "data ".repeat(12_700));
    for line in 1..=30 {
        write!(reply, "\n\nNumbered line {line:02}. Keep this paragraph available while reading.").expect("reply text");
    }
    tier.with_replies(&[
        (&format!("Reading paragraph {:02}.", messages - 1), format!("{reply}\n\nEnd of reply {}.", messages - 1)),
        (&format!("Reading paragraph {messages:02}."), format!("{reply}\n\nEnd of reply {messages}.")),
        ("Reading paragraph", format!("{reply}\n\nEnd of reply 1.")),
    ])
}
#[test]
fn a_long_chat_opens_at_its_end_and_reads_back_from_the_composer() {
    reading_position(true);
}
#[test]
fn earlier_words_stay_put_without_browser_scroll_anchoring() {
    reading_position(false);
}
fn reading_position(anchoring: bool) {
    let messages = if anchoring { READING_MESSAGES } else { 3 };
    let tier = reading_tier(anchoring, messages);
    let mut steps = reading_history(messages);
    let composer = || Find::exactly_named(Role::TextArea, b"Message");
    let anchor = || paragraph(&reading_words(messages - 1));
    let loading = || Find::exactly_named(Role::Status, b"Loading earlier messages");
    steps.extend([
        keep_box(Find::named(Role::Log, b"Conversation"), "log"),
        keep_box(reply_end(messages), "end"),
        keep_box(anchor(), "head"),
        Step::Person(PersonStep::Press { find: composer() }),
    ]);
    steps.extend(serve_signal(&tier, "-STOP"));
    steps.extend([Step::Key(Key::PageUp), Step::Focused(composer()), keep_box(reply_end(messages), "up")]);
    for _ in 0..5 {
        steps.push(Step::Key(Key::PageUp));
    }
    steps.extend([
        Step::Focused(composer()),
        see(loading()),
        see(Find::text("Loading earlier messages…".as_bytes())),
        keep_box(anchor(), "before-earlier"),
    ]);
    steps.extend(serve_signal(&tier, "-CONT"));
    steps.extend([
        see(if anchoring { paragraph(&reading_words(messages - 2)) } else { reply_end(1) }),
        gone(loading()),
        keep_box(anchor(), "after-earlier"),
        Step::Focused(composer()),
    ]);
    if anchoring {
        read_to_start(&mut steps);
        steps.extend([gone(reply_end(messages)), see(Find::exactly_named(Role::Button, b"Latest"))]);
        // Hold the later read while PageDown reaches the retained window's foot.
        steps.extend(serve_signal(&tier, "-STOP"));
        for _ in 0..40 {
            steps.push(Step::Key(Key::PageDown));
        }
        steps.push(keep_box(Find::text(b"End of reply 15."), "before-later"));
        steps.extend(serve_signal(&tier, "-CONT"));
        steps.extend([
            see(reply_end(messages)),
            keep_box(Find::text(b"End of reply 15."), "after-later"),
            Step::Focused(composer()),
            see(Find::exactly_named(Role::Button, b"Latest")),
        ]);
        // Read the dropped head again, dropping the newest reply before Latest restores it.
        for _ in 0..40 {
            steps.push(Step::Key(Key::PageUp));
        }
        steps.extend([
            see(paragraph(&reading_words(1))),
            gone(reply_end(messages)),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Latest") }),
            see(reply_end(messages)),
            gone(Find::exactly_named(Role::Button, b"Latest")),
            keep_box(reply_end(messages), "latest-end"),
        ]);
    }
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(tier.setup());
    let outcome = harness.run(&mut [&mut script]);
    for kept in harness.kept() {
        eprintln!("{}: {}", String::from_utf8_lossy(&kept.name), String::from_utf8_lossy(&kept.value));
    }
    assert_eq!(outcome, Outcome::Done);
    in_log(&harness, "end");
    in_log(&harness, "before-earlier");
    let (top, height) = box_of(&harness, "log");
    let (head, _) = box_of(&harness, "head");
    assert!((top - head) * 2 >= height * 3, "head {head}, log {top} + {height}");
    assert!(box_of(&harness, "up").0 > box_of(&harness, "end").0, "PageUp scrolls from the composer");
    held(&harness, "before-earlier", "after-earlier");
    if anchoring {
        in_log(&harness, "before-later");
        held(&harness, "before-later", "after-later");
        in_log(&harness, "latest-end");
    }
}
fn read_to_start(steps: &mut Vec<Step>) {
    // Reach the held head, then wait for the next numbered message before scrolling again.
    for number in (1..=READING_MESSAGES - 2).rev() {
        for _ in 0..6 {
            steps.push(Step::Key(Key::PageUp));
        }
        steps.push(see(paragraph(&reading_words(number))));
    }
}
fn held(harness: &Harness, before: &str, after: &str) {
    let before = box_of(harness, before);
    let after = box_of(harness, after);
    assert!((before.0 - after.0).abs() <= 1, "{before:?} -> {after:?}: words moved");
    assert_eq!(before.1, after.1, "line height stays");
}

// Stop only this scratch deployment's serve process, leaving its browser running.
// This holds a read out long enough to measure the words before its answer lands.
fn serve_signal(tier: &Tier, signal: &str) -> [Step; 2] {
    use skein_fake_person_browser::{Command, Stdin};
    let setup = tier.setup();
    let flags = setup.args.iter().skip(1).map(|arg| arg.to_str().expect("fixture flag")).collect::<Vec<_>>().join(" ");
    let command = format!("{} internal serve {flags} --box", setup.server.display());
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
