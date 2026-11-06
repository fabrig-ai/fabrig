//! Deadline words follow the outstanding work (client/tools.md, 6.3).
use super::fixtures::{self, Engine};
use crate::{
    limits,
    waited::{self, Act, Waited},
};
use skein_io::kernel::Exit;

const STREAM_RECORD: u32 = 1; // Only the stream predecessor is stored for an act sent once.
const LINK: &str = "the link to the engine at http://127.0.0.1:4000 did not come up within 30 s";
const STREAM: &str =
    "the engine at http://127.0.0.1:4000 took the connection but its stream did not answer within 30 s";
const READ: &str = "the engine at http://127.0.0.1:4000 did not answer a read within 30 s";
const OUTCOME: &str = "the engine at http://127.0.0.1:4000 gave no outcome for this act within 30 s";
const NOT_SENT: &str = "; it was not sent";
const KEPT_NAMED: &str = "; it is kept, and the same --key may be tried again";
const KEPT_DRAWN: &str = "; it is kept, and the next call that sends tries it again";
const ONCE_NAMED: &str = "; it is not kept, and the same --key and file may be tried again";
const ONCE_DRAWN: &str = "; it is not kept, and calling again is a new request";

fn deadline(f: &mut Engine, base: &str, suffix: &str) {
    f.advance_past(limits::DEADLINE);
    assert_eq!(str::from_utf8(f.stderr()).expect("diagnostic"), format!("temper ctl: {base}{suffix}\n"));
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
fn act(words: &[&[u8]]) -> Engine {
    Engine::act_words_with(words, fixtures::small_store(&[]))
}
#[test]
fn an_uncompleted_connection_waited_for_the_link() {
    for (words, suffix) in [(&[b"me".as_slice()][..], ""), (fixtures::CHAT, NOT_SENT)] {
        let mut f = act(words);
        f.hold_connect = true;
        f.drain();
        // A read uses Memory; an act uses the native shell.
        deadline(&mut f, LINK, suffix);
    }
}
#[test]
fn an_unanswered_stream_names_the_stream_and_unsent_act() {
    for (words, suffix) in [(&[b"me".as_slice()][..], ""), (fixtures::CHAT, NOT_SENT)] {
        let mut f = act(words);
        f.quiet.push(b"/jig/v1/stream");
        f.drain();
        deadline(&mut f, STREAM, suffix);
    }
}
#[test]
fn a_live_unanswered_read_names_the_read() {
    let mut f = Engine::read(&["task", "show", "7", "--activity", "1"]);
    f.quiet.push(b"/jig/v1/read");
    f.sign_in_then_deliver_me();
    deadline(&mut f, READ, "");
}
#[test]
fn an_unanswered_named_act_is_kept_for_same_key() {
    let mut f = act(fixtures::CHAT);
    f.sign_in_then_deliver_me();
    deadline(&mut f, OUTCOME, KEPT_NAMED);
    assert_eq!(f.kept_records(), fixtures::local::ACT_WITH_STREAM);
}
#[test]
fn an_unanswered_drawn_act_is_kept_for_next_call() {
    let mut f = act(&[b"task", b"stop", b"7"]);
    f.sign_in_then_deliver_me();
    deadline(&mut f, OUTCOME, KEPT_DRAWN);
    assert_eq!(f.kept_records(), fixtures::local::ACT_WITH_STREAM);
}
#[test]
fn a_carried_key_is_not_kept_and_names_same_file() {
    let mut f = act(&[b"account", b"add-key", b"1", b"/linked-parent/link-file", b"--deployment", b"--key", b"key"]);
    f.link = b"opaque-fake-key\n";
    f.sign_in_then_deliver_me();
    deadline(&mut f, OUTCOME, ONCE_NAMED);
    assert_eq!(f.kept_records(), STREAM_RECORD);
}
#[test]
fn an_export_is_not_kept_and_calling_again_is_new() {
    let mut f = act(&[b"account", b"export", b"7", b"/linked-parent/record"]);
    f.sign_in_then_deliver_me();
    deadline(&mut f, OUTCOME, ONCE_DRAWN);
    assert_eq!(f.kept_records(), STREAM_RECORD);
}
#[test]
fn a_sent_act_keeps_its_words_after_refused_reconnection() {
    let mut f = act(fixtures::CHAT);
    f.sign_in_then_deliver_me();
    f.refuse_connections = true;
    f.drop_stream();
    deadline(&mut f, LINK, KEPT_NAMED);
    assert_eq!(f.kept_records(), fixtures::local::ACT_WITH_STREAM);
}

const WAIT_WORDS: [(Waited, &str); 4] =
    [(Waited::Link, LINK), (Waited::Stream, STREAM), (Waited::Read, READ), (Waited::Outcome, OUTCOME)];
const ACT_WORDS: [(Act, &str); 6] = [
    (Act::None, ""),
    (Act::NotSent, NOT_SENT),
    (Act::Kept { named: true }, KEPT_NAMED),
    (Act::Kept { named: false }, KEPT_DRAWN),
    (Act::Once { named: true }, ONCE_NAMED),
    (Act::Once { named: false }, ONCE_DRAWN),
];
#[test]
fn every_link_state_maps_to_its_outstanding_work() {
    use jig_client::Link;
    const ANY: &[bool] = &[false, true];
    for (link, connected, heard, acting, expected) in [
        (Link::Offline, ANY, ANY, ANY, Waited::Link),
        (Link::SignedOut, ANY, ANY, ANY, Waited::Link),
        (Link::OutOfDate, ANY, ANY, ANY, Waited::Link),
        (Link::Starting, &[false][..], ANY, ANY, Waited::Link),
        (Link::Starting, &[true][..], &[false][..], ANY, Waited::Stream),
        (Link::Starting, &[true][..], &[true][..], ANY, Waited::Read),
        (Link::Live, ANY, ANY, &[true][..], Waited::Outcome),
        (Link::Live, ANY, ANY, &[false][..], Waited::Read),
    ] {
        for &connected in connected {
            for &heard in heard {
                for &acting in acting {
                    assert_eq!(
                        waited::waited(link, connected, heard, acting),
                        expected,
                        "{link:?}, connected {connected}, heard {heard}, acting {acting}"
                    );
                }
            }
        }
    }
}
#[test]
fn deadline_words_are_only_the_tables_and_address() {
    for (waited, base) in WAIT_WORDS {
        for (act, suffix) in ACT_WORDS {
            assert_eq!(waited::line("http://127.0.0.1:4000", waited, act), format!("{base}{suffix}"));
        }
    }
}
#[test]
fn every_deadline_line_fits_at_the_address_bound() {
    const LINE: usize = 240; // client/tools.md, 6.3, including prefix and newline.
    let address = "a".repeat(usize::try_from(limits::ADDRESS).expect("address bound"));
    for (waited, _) in WAIT_WORDS {
        for (act, _) in ACT_WORDS {
            let line = format!("temper ctl: {}\n", waited::line(&address, waited, act));
            assert!(line.len() <= LINE, "{} bytes: {waited:?}, {act:?}", line.len());
        }
    }
}
#[test]
fn an_act_with_its_save_in_flight_was_not_sent() {
    let mut f = act(fixtures::CHAT);
    f.drain();
    f.hold_stores = Some(());
    f.sign_in_then_deliver_me();
    f.advance_past(limits::DEADLINE);
    assert_eq!(str::from_utf8(f.stderr()).expect("diagnostic"), format!("temper ctl: {OUTCOME}{NOT_SENT}\n"));
    assert!(!f.act_sent());
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}

#[test]
fn a_first_repository_watch_has_no_act_suffix() {
    let mut f = Engine::setup(false);
    f.store_erase();
    f.refuse_connections = true;
    f.drop_stream();
    deadline(&mut f, LINK, "");
}
#[test]
fn sign_out_keeps_the_generic_deadline_line() {
    let mut f = Engine::read(&["sign-out"]);
    f.quiet.push(b"/jig/v1/stream");
    f.drain();
    deadline(&mut f, "the engine at http://127.0.0.1:4000 did not answer within 30 s", "");
}
