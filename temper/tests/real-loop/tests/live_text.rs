//! A person reads a held reply's live prefix before reading its committed turn.
use skein_world::domain::heap::Counting;
use temper_real_loop_world::{
    Exit,
    calls::{D1, release_hold, sign_in},
    run, scripts, terminal,
};
#[global_allocator]
static HEAP: Counting = Counting;
// The production view's Provisional word (cards/words.rs).
const PROVISIONAL: &str = "Writing...";
const PROVISIONAL_LINE: &str = "  * Writing...\n";
const REPLY: &[u8] = b"The answer is forty-two.";
// The ctl call that signs in and provisions the deployment's key.
const SIGN_IN: usize = 0;

#[test]
fn a_person_reads_a_chats_reply_as_it_is_written_then_its_committed_turn() {
    let ran = run(scripts::live_scripts, || {
        vec![
            sign_in(D1),
            terminal::open(),
            terminal::type_on(b"Chats", b""),
            terminal::see(b"Start a chat"),
            terminal::type_on(b"Start a chat", scripts::WORDS.as_bytes()),
            terminal::see(scripts::FIRST),
            release_hold(scripts::LIVE),
            terminal::see_without(REPLY, PROVISIONAL.as_bytes()),
            terminal::close(),
        ]
    });
    assert_eq!(ran.calls()[SIGN_IN].exit, Some(Exit::Code(0)), "ctl signed in before provisioning the key");
    let read = ran.terminal();
    let first = read
        .pages
        .iter()
        .find(|page| page.bytes.windows(scripts::FIRST.len()).any(|part| part == scripts::FIRST))
        .expect("a page shows the held reply's first piece");
    let first = std::str::from_utf8(&first.bytes).expect("printed UTF-8");
    assert_eq!(
        provisional(first).as_deref(),
        Some(std::str::from_utf8(scripts::FIRST).expect("prefix UTF-8")),
        "first piece is provisional:\n{first}"
    );
    assert!(!first.contains(std::str::from_utf8(scripts::SECOND).expect("suffix UTF-8")), "hold kept suffix:\n{first}");
    let reply = [scripts::FIRST, scripts::SECOND].concat();
    let reply = std::str::from_utf8(&reply).expect("reply UTF-8");
    let mut previous = 0;
    let mut committed = false;
    for page in &read.pages {
        let text = std::str::from_utf8(&page.bytes).expect("printed UTF-8");
        if let Some(prefix) = provisional(text) {
            assert!(reply.starts_with(&prefix), "provisional text is a reply prefix:\n{text}");
            assert!(prefix.len() >= previous, "provisional text never shrinks:\n{text}");
            assert!(!committed, "provisional text never returns after commitment:\n{text}");
            previous = prefix.len();
        }
        if let Some((_, assistant)) = text.split_once("Conversation:\n").and_then(|(_, c)| c.split_once("Temper\n"))
            // The committed turn's lines follow its speaker and clock.
            && assistant.lines().any(|line| line.trim_ascii_start() == reply)
        {
            assert!(
                !text.contains(PROVISIONAL),
                "the first committed page and every later one lack provisional text:\n{text}"
            );
            committed = true;
        }
    }
    assert!(committed, "a page shows the whole reply under Temper");
    assert_eq!(read.exit, Some(Exit::Code(0)), "the terminal closed and drained");
}

fn provisional(text: &str) -> Option<String> {
    let (_, text) = text.split_once(PROVISIONAL_LINE)?;
    Some(text.lines().map_while(|line| line.strip_prefix("  ")).collect::<Vec<_>>().join("\n"))
}
