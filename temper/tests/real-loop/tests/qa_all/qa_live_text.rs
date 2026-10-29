//! Design-only QA of the worker-to-terminal text path (domain/testing.md, 2.4).
use skein_fake_llm_domain::api::{Finish, Line, Piece, Script, Turn};
use temper_real_loop_world::{calls, run, scripts, terminal};

const REQUEST: &[u8] = b"QA live text request";
const PREFIX: &str = "QA prefix caf\u{e9}";
const MIDDLE: &[u8] = b"second paragraph";
const END: &[u8] = b"final sentence.";
const FIRST_HOLD: &[u8] = b"qa-middle";
const LAST_HOLD: &[u8] = b"qa-last";
const WRITING: &[u8] = b"Writing...";
const ONCE: usize = 1;

fn pieces() -> Box<[Script]> {
    Box::new([Script {
        cue: REQUEST.into(),
        turns: Box::new([Turn {
            lines: Box::new([
                Line::Pieces {
                    pieces: Box::new([
                        Piece { text: PREFIX.as_bytes().into(), hold: None },
                        Piece { text: b"\n\nsecond paragraph".as_slice().into(), hold: Some(FIRST_HOLD.into()) },
                        Piece { text: b" final sentence.".as_slice().into(), hold: Some(LAST_HOLD.into()) },
                    ]),
                },
                Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            ]),
            finish: Finish::ToolCalls,
            tokens: 16,
        }]),
    }])
}

fn open_chat(request: &'static [u8]) -> Vec<calls::Step> {
    vec![
        calls::sign_in(calls::D1),
        calls::Step::FixtureReady,
        terminal::open(),
        terminal::see(b"Chats"),
        terminal::type_on(b"Chats", b""),
        terminal::see(b"Start a chat"),
        terminal::type_on(b"Start a chat", request),
        terminal::see(b"Message"),
    ]
}

fn start_chat() -> Vec<calls::Step> {
    let mut steps = open_chat(REQUEST);
    steps.extend([terminal::see(WRITING), terminal::see_without(PREFIX.as_bytes(), MIDDLE)]);
    steps
}

#[test]
fn held_prefix_grows_and_first_committed_page_replaces_it() {
    // domain/testing.md, 2.4; client/README.md, 3.5 and 6.9.
    let ran = run(pieces, || {
        let mut steps = start_chat();
        steps.extend([
            calls::release_hold(FIRST_HOLD),
            terminal::see_without(MIDDLE, END),
            calls::release_hold(LAST_HOLD),
            terminal::see_without(END, WRITING),
            terminal::close(),
        ]);
        steps
    });
    let pages = &ran.terminal().pages;
    let first = pages
        .iter()
        .position(|page| calls::occurrences(&page.bytes, PREFIX.as_bytes()) > 0)
        .expect("the held prefix must reach the terminal without a later delta");
    let committed = pages
        .iter()
        .position(|page| calls::occurrences(&page.bytes, END) > 0 && calls::occurrences(&page.bytes, WRITING) == 0)
        .expect("the committed reply must replace Writing\u{2026}");
    let mut saw_middle = false;
    for page in &pages[first..=committed] {
        let text = String::from_utf8_lossy(&page.bytes);
        assert_eq!(calls::occurrences(&page.bytes, PREFIX.as_bytes()), ONCE, "prefix lost or duplicated: {text}");
        let has_middle = calls::occurrences(&page.bytes, MIDDLE) > 0;
        assert!(!saw_middle || has_middle, "a printed prefix shrank: {text}");
        saw_middle |= has_middle;
        assert!(calls::occurrences(&page.bytes, END) == 0 || has_middle, "suffix overtook middle: {text}");
    }
    assert!(saw_middle, "the independently held second prefix was not read");
    assert_eq!(calls::occurrences(&pages[committed].bytes, MIDDLE), ONCE);
    assert_eq!(calls::occurrences(&pages[committed].bytes, END), ONCE);
}

#[test]
fn navigation_command_works_while_the_reply_is_held() {
    // client/README.md, 6.5 and 6.9: commands work while text streams.
    let ran = run(pieces, || {
        let mut steps = start_chat();
        steps.extend([
            terminal::type_on(b"Chats", b""),
            terminal::see_without(b"Your chats", PREFIX.as_bytes()),
            terminal::see(b"T1"),
            terminal::type_on(b"T1", b""),
            terminal::see_without(PREFIX.as_bytes(), MIDDLE),
            calls::release_hold(FIRST_HOLD),
            terminal::see_without(MIDDLE, END),
            calls::release_hold(LAST_HOLD),
            terminal::see_without(END, WRITING),
            terminal::close(),
        ]);
        steps
    });
    let committed = ran
        .terminal()
        .pages
        .iter()
        .find(|page| calls::occurrences(&page.bytes, END) > 0 && calls::occurrences(&page.bytes, WRITING) == 0)
        .expect("committed page");
    assert_eq!(calls::occurrences(&committed.bytes, PREFIX.as_bytes()), ONCE);
}

#[test]
fn a_tool_only_turn_leaves_no_provisional_text() {
    // client/README.md, 3.5: a conversation draws the agent's words, not run mechanics.
    let ran = run(
        || {
            Box::new([Script {
                cue: REQUEST.into(),
                turns: Box::new([Turn {
                    lines: Box::new([Line::Call {
                        name: b"wait".as_slice().into(),
                        arguments: b"{}".as_slice().into(),
                    }]),
                    finish: Finish::ToolCalls,
                    tokens: 1,
                }]),
            }])
        },
        || {
            let mut steps = open_chat(REQUEST);
            steps.extend([terminal::see_without(b"1 tool call", WRITING), terminal::close()]);
            steps
        },
    );
    for page in &ran.terminal().pages {
        assert_eq!(
            calls::occurrences(&page.bytes, WRITING),
            0,
            "a tool-only turn must not manufacture a provisional text turn: {}",
            String::from_utf8_lossy(&page.bytes)
        );
    }
}

#[test]
fn the_public_live_reply_script_commits_without_losing_its_prefix() {
    // domain/testing.md, 2.4: the harness's documented two-piece smoke.
    let ran = run(scripts::live_scripts, || {
        let mut steps = open_chat(scripts::WORDS.as_bytes());
        steps.extend([
            terminal::see(WRITING),
            terminal::see_without(scripts::FIRST, scripts::SECOND),
            calls::release_hold(scripts::LIVE),
            terminal::see_without(scripts::SECOND, WRITING),
            terminal::close(),
        ]);
        steps
    });
    let pages = &ran.terminal().pages;
    let first = pages.iter().position(|page| calls::occurrences(&page.bytes, scripts::FIRST) > 0).expect("live prefix");
    let committed = pages
        .iter()
        .position(|page| {
            calls::occurrences(&page.bytes, scripts::SECOND) > 0 && calls::occurrences(&page.bytes, WRITING) == 0
        })
        .expect("committed reply");
    for page in &pages[first..=committed] {
        assert_eq!(calls::occurrences(&page.bytes, scripts::FIRST), ONCE, "{}", String::from_utf8_lossy(&page.bytes));
    }
}

#[test]
fn reply_notification_arrives_after_leaving_the_held_chat() {
    // client/README.md, 6.5 and 6.9: commands work while text streams.
    // client/README.md, 3.5: turns are seen through the newest the window holds.
    // One hold lets the inbox's reply notification acknowledge its release.
    let ran = run(scripts::live_scripts, || {
        let mut steps = open_chat(scripts::WORDS.as_bytes());
        steps.extend([
            terminal::see(WRITING),
            terminal::see_without(scripts::FIRST, scripts::SECOND),
            terminal::type_on(b"Inbox", b""),
            terminal::see_without(b"Needs you", scripts::FIRST),
            calls::release_hold(scripts::LIVE),
            terminal::see(b"replied"),
            terminal::close(),
        ]);
        steps
    });
    let first = ran
        .terminal()
        .pages
        .iter()
        .find(|page| calls::occurrences(&page.bytes, scripts::FIRST) > 0)
        .expect("prefix before navigation");
    assert!(
        ran.terminal().pages.iter().any(|page| page.commands > first.commands
            && calls::occurrences(&page.bytes, b"Needs you") > 0
            && calls::occurrences(&page.bytes, scripts::FIRST) == 0),
        "the command must leave the chat while the reply is held"
    );
}

#[test]
fn a_composer_command_is_processed_while_the_reply_is_held() {
    // client/README.md, 6.5 and 6.9: a command works during streaming.
    let ran = run(scripts::live_scripts, || {
        let mut steps = open_chat(scripts::WORDS.as_bytes());
        steps.extend([
            terminal::see(WRITING),
            terminal::see_without(scripts::FIRST, scripts::SECOND),
            terminal::type_on(b"Reset choice", b""),
            terminal::see_without(scripts::FIRST, scripts::SECOND),
            calls::release_hold(scripts::LIVE),
            terminal::see_without(scripts::SECOND, WRITING),
            terminal::close(),
        ]);
        steps
    });
    let first = ran
        .terminal()
        .pages
        .iter()
        .find(|page| calls::occurrences(&page.bytes, scripts::FIRST) > 0)
        .expect("the held prefix before the command");
    assert!(
        ran.terminal().pages.iter().any(|page| page.commands > first.commands
            && calls::occurrences(&page.bytes, scripts::FIRST) > 0
            && calls::occurrences(&page.bytes, scripts::SECOND) == 0),
        "a fresh page must follow the command before the hold is released"
    );
}
