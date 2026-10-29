//! Design-only QA of the worker-to-terminal text path (domain/testing.md, 2.4).
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use temper_real_loop_world::{calls, run, scripts, terminal};

const REQUEST: &[u8] = b"QA live text request";
const WRITING: &[u8] = b"Writing...";

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
