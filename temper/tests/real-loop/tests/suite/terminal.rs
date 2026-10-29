//! A person starts a chat at the terminal and reads its committed answer.
use std::os::unix::fs::MetadataExt;
use temper_real_loop_world::{
    Exit,
    calls::{D1, sign_in},
    run, scripts, terminal,
};
// The production view's Provisional word (cards/words.rs).
const PROVISIONAL: &[u8] = b"Writing...";
// The ctl call that signs in and provisions the deployment's key.
const SIGN_IN: usize = 0;

#[test]
fn a_person_at_the_terminal_reads_a_chats_committed_reply() {
    let ran = run(
        || scripts::chat_scripts(1),
        || {
            vec![
                sign_in(D1),
                terminal::open(),
                terminal::type_on(b"Chats", b""),
                terminal::see(b"Start a chat"),
                terminal::type_on(b"Start a chat", scripts::WORDS.as_bytes()),
                terminal::see_without(scripts::REPLY, PROVISIONAL),
                terminal::close(),
            ]
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
