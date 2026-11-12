//! A person reads a held reply's live prefix before reading its committed turn.
use skein_world::domain::heap::Counting;
use temper_real_loop_world::{
    Exit, Scratch,
    calls::{D1, release_hold, sign_in},
    fixtures, llm, run_on, scripts, terminal,
};
#[global_allocator]
static HEAP: Counting = Counting;
// The production view's Provisional word (cards/words.rs).
const PROVISIONAL: &str = "▍";
const PROVISIONAL_LINE: &str = "▍\n";
const REPLY: &[u8] = b"The answer is forty-two.";
// The ctl call that signs in and provisions the deployment's key.
const SIGN_IN: usize = 0;

#[test]
fn a_person_reads_a_chats_reply_as_it_is_written_then_its_committed_turn() {
    let scratch = Scratch::new("live-terminal");
    fixtures::standing_chats(scratch.path()).expect("standing chat fixture");
    let ran = run_on(
        scratch,
        || llm::scripted(scripts::live_scripts()),
        || {
            [sign_in(D1), terminal::open()]
                .into_iter()
                .chain(terminal::start_change(true))
                .chain([
                    terminal::see(b"What do you want done?"),
                    terminal::type_on(b"What do you want done?", scripts::WORDS.as_bytes()),
                    terminal::see(scripts::FIRST),
                    release_hold(scripts::LIVE),
                    terminal::see_without(REPLY, PROVISIONAL.as_bytes()),
                    terminal::see(b"waiting on your words"),
                    terminal::close(),
                ])
                .collect()
        },
    );
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
            // Only a committed turn has a clock after its speaker.
            && assistant.lines().next().is_some_and(|clock| clock.trim_ascii().len() == 5 && clock.contains(':'))
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
    let (_, conversation) = text.split_once("Conversation:\n")?;
    let (speaker, assistant) = conversation.split_once("Temper\n")?;
    let (_, indent) = speaker.rsplit_once('\n')?;
    let (prefix, _) = assistant.split_once(PROVISIONAL_LINE)?;
    Some(prefix.lines().map_while(|line| line.strip_prefix(indent)).collect::<Vec<_>>().join("\n"))
}
