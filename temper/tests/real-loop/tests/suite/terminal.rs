//! A person starts a chat at the terminal and reads its committed answer.
use std::os::unix::fs::MetadataExt;
use temper_real_loop_world::{
    Exit, Scratch,
    calls::{D1, sign_in},
    fixtures, llm, run_on, scripts, terminal,
};
// The production view's Provisional word (cards/words.rs).
const PROVISIONAL: &[u8] = "▍".as_bytes();
// The ctl call that signs in and provisions the deployment's key.
const SIGN_IN: usize = 0;

#[test]
fn a_person_at_the_terminal_reads_a_run_of_calls() {
    let scratch = Scratch::new("call-line-terminal");
    fixtures::standing_chats(scratch.path()).expect("standing chat fixture");
    let ran = run_on(
        scratch,
        || llm::scripted(scripts::call_line_scripts()),
        || {
            [sign_in(D1), terminal::open()]
                .into_iter()
                .chain(terminal::start_change(true))
                .chain([
                    terminal::see(b"What do you want done?"),
                    terminal::type_on(b"What do you want done?", scripts::WORDS.as_bytes()),
                    terminal::see_without(scripts::REPLY, PROVISIONAL),
                    terminal::close(),
                ])
                .collect()
        },
    );
    assert_eq!(ran.calls()[SIGN_IN].exit, Some(Exit::Code(0)), "ctl signed in before provisioning the key");
    let read = ran.terminal();
    assert_eq!(read.exit, Some(Exit::Code(0)), "the terminal closed and drained");
    let page = read
        .pages
        .iter()
        .find(|page| {
            page.bytes.windows(scripts::REPLY.len()).any(|part| part == scripts::REPLY)
                && !page.bytes.windows(PROVISIONAL.len()).any(|part| part == PROVISIONAL)
        })
        .expect("a printed page has the committed reply without provisional text");
    let text = std::str::from_utf8(&page.bytes).expect("printed UTF-8");
    let (_, after_name) = text.split_once("Read 2 docs").unwrap_or_else(|| panic!("{text}"));
    let (_, after_first) = after_name.split_once("`read_docs` hold: hold: Hold").unwrap_or_else(|| panic!("{text}"));
    let (_, after_second) = after_first.split_once("`read_docs` chat: chat: Chats").unwrap_or_else(|| panic!("{text}"));
    assert!(after_second.contains(std::str::from_utf8(scripts::REPLY).expect("reply UTF-8")), "{text}");
    assert!(!text.contains("read_docs ×"), "{text}");
    // temper/docs/help/hold.md: the result's second line, after "hold: Hold".
    assert!(!text.contains("# Hold"), "{text}");
}

#[test]
fn a_person_at_the_terminal_reads_a_chats_committed_reply() {
    let scratch = Scratch::new("standing-terminal");
    fixtures::standing_chats(scratch.path()).expect("standing chat fixture");
    let ran = run_on(
        scratch,
        || llm::scripted(scripts::chat_scripts(1)),
        || {
            [sign_in(D1), terminal::open()]
                .into_iter()
                .chain(terminal::start_change(true))
                .chain([
                    terminal::see(b"What do you want done?"),
                    terminal::type_on(b"What do you want done?", scripts::WORDS.as_bytes()),
                    terminal::see_without(scripts::REPLY, PROVISIONAL),
                    terminal::close(),
                ])
                .collect()
        },
    );
    assert_eq!(ran.calls()[SIGN_IN].exit, Some(Exit::Code(0)), "ctl signed in before provisioning the key");
    let read = ran.terminal();
    assert_eq!(read.opens, 2, "ctl spent the first link; the terminal used the fresh one");
    assert_eq!(read.exit, Some(Exit::Code(0)), "the terminal closed and drained");
    let page = read
        .pages
        .iter()
        .find(|page| {
            page.bytes.windows(scripts::REPLY.len()).any(|part| part == scripts::REPLY)
                && !page.bytes.windows(PROVISIONAL.len()).any(|part| part == PROVISIONAL)
        })
        .expect("a printed page has the committed reply without provisional text");
    let text = std::str::from_utf8(&page.bytes).expect("printed UTF-8");
    assert!(text.lines().next().expect("chat title").contains(scripts::WORDS), "{text}");
    let (_, conversation) = text.split_once("Conversation:\n").expect("chat's conversation");
    let (_, assistant) = conversation.split_once("Temper\n").expect("committed assistant turn");
    assert!(assistant.contains(std::str::from_utf8(scripts::REPLY).expect("reply UTF-8")), "{text}");
    assert!(page.commands >= 2, "the reply was read after the person's chat command");
    let directory = terminal::directory(ran.path());
    for (path, mode) in [(directory.clone(), 0o700), (directory.join("token"), 0o600)] {
        let metadata = std::fs::metadata(&path).expect("terminal's signed-in metadata");
        assert_eq!(
            (metadata.mode() & 0o777, metadata.uid()),
            (mode, skein_shell::effective_user()),
            "{}",
            path.display()
        );
    }
    assert!(
        !std::fs::read(directory.join("token")).expect("kept credential").is_empty(),
        "closing keeps the credential"
    );
}

#[test]
fn up_down_and_the_page_up_line_read_the_conversation() {
    use skein_fake_llm_domain::api;
    const FIRST: &[u8] = b"First terminal answer. This whole answer leaves the window as later replies arrive. This whole answer leaves the window as later replies arrive. This whole answer leaves the window as later replies arrive.";
    const MIDDLE: &[u8] = b"Middle terminal answer. This whole answer leaves the window as later replies arrive. This whole answer leaves the window as later replies arrive. This whole answer leaves the window as later replies arrive.";
    const LAST: &[u8] = b"Last terminal answer. This whole answer leaves the window as earlier replies arrive. This whole answer leaves the window as earlier replies arrive. This whole answer leaves the window as earlier replies arrive.";
    let scratch = Scratch::new("terminal-reading");
    fixtures::standing_chats(scratch.path()).expect("standing chat fixture");
    let ran = run_on(
        scratch,
        || {
            llm::played_by_input(
                Box::new([api::Script {
                    cue: Box::default(),
                    turns: [FIRST, MIDDLE, LAST]
                        .into_iter()
                        .map(|text| api::Turn {
                            lines: Box::new([
                                api::Line::Text { text: text.into() },
                                api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                            ]),
                            finish: api::Finish::ToolCalls,
                            tokens: 1,
                        })
                        .collect(),
                }]),
                Box::new([Box::new([
                    api::Rule { cue: b"third answer".as_slice().into(), turn: 2 },
                    api::Rule { cue: b"second answer".as_slice().into(), turn: 1 },
                    api::Rule { cue: Box::default(), turn: 0 },
                ])]),
            )
        },
        || {
            [sign_in(D1), terminal::open_reading()]
                .into_iter()
                .chain(terminal::start_change(true))
                .chain([
                    terminal::type_on(b"What do you want done?", b"Read our earlier words"),
                    terminal::see_without(FIRST, PROVISIONAL),
                    terminal::type_on(b"Message", b"Please continue the conversation with a second answer"),
                    terminal::see_without(MIDDLE, PROVISIONAL),
                    terminal::type_on(b"Message", b"Please continue the conversation with a third answer"),
                    terminal::see_without(LAST, PROVISIONAL),
                    terminal::line(b"up"),
                    terminal::line(b"up"),
                    terminal::line(b"up"),
                    terminal::see_without(FIRST, LAST),
                    terminal::line(b"down"),
                    terminal::line(b"down"),
                    terminal::line(b"down"),
                    terminal::see_without(LAST, FIRST),
                    terminal::line(b"\x1b[5~"),
                    terminal::line(b"\x1b[5~"),
                    terminal::line(b"\x1b[5~"),
                    terminal::see_without(FIRST, LAST),
                    terminal::close(),
                ])
                .collect()
        },
    );
    assert_eq!(ran.terminal().exit, Some(Exit::Code(0)));
    assert!(
        ran.terminal().pages.iter().any(|page| page
            .bytes
            .windows(b"(up, down: more lines)".len())
            .any(|part| part == b"(up, down: more lines)"))
    );
}
