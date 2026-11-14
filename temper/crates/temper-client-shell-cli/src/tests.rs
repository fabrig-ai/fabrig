mod deadline;
pub(crate) mod fixtures;
mod service;
use crate::{directory, limits};
use skein_io::kernel::{Error, Exit, ServiceSignal};
const SIGN_IN_FIRST: skein_lib::Duration = skein_lib::Duration::from_millis(100);
const DEADLINE_TICK: skein_lib::Duration = skein_lib::Duration::from_nanos(1); // The clock's smallest step.
const SIGN_IN_FILE_CALL: &[u8] = br#"{"tool":"sign-in","arguments":{"file":"link-file"}}"#;
#[test]
fn memory_exchanges_wait_for_closing_sockets() {
    let mut f = fixtures::Engine::socket_wait();
    let close = limits::native().io.close_timeout;
    let start = skein_lib::Time::ZERO;
    for number in 1..=5 {
        let requests = f.requests();
        f.exchange(number);
        if number % 2 == 1 {
            assert_eq!(f.probe_replies().len(), usize::try_from(number - 1).expect("reply count"));
            assert_eq!(f.requests(), requests, "no socket starts while all are closing");
            assert!(f.stream_live());
            f.advance_past(close);
        }
        assert_eq!(f.probe_replies().last().map(|(reply, _)| *reply), Some(number));
        assert!(f.stream_live());
        assert_eq!(f.exit(), None);
    }
    assert_eq!(
        f.probe_replies(),
        vec![
            (1, start.saturating_add(close)),
            (2, start.saturating_add(close)),
            (3, start.saturating_add(close.saturating_mul(2))),
            (4, start.saturating_add(close.saturating_mul(2))),
            (5, start.saturating_add(close.saturating_mul(3))),
        ]
    );
    f.signal(ServiceSignal::Terminate);
    assert_eq!(f.exit(), Some(Exit::Code(limits::TERMINATED)));
    assert!(!f.stream_live());
    assert!(skein_shell::Host::is_empty(&f.host));
}
#[test]
fn a_busy_directory_is_refused_after_60_s_of_retries() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.lock_busy();
    assert_eq!(f.next_deadline(), Some(f.started().saturating_add(limits::LOCK_TRY)));
    f.advance_past(limits::LOCK);
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is busy with another call\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn calls_that_send_nothing_take_no_lock() {
    let cases: &[&[&[u8]]] = &[&[b"me"], &[b"task", b"show", b"41", b"--until", b"changed"], &[b"pending"], &[b"help"]];
    for words in cases {
        let mut f = fixtures::Frame::act(words);
        f.run();
        assert_eq!(f.locks, 0, "{words:?}");
    }
}
#[test]
fn a_name_address_is_refused() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.address = b"http://localhost:4000\n";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn numeric_loopback_addresses_are_accepted() {
    for address in [b"http://127.3.4.5:4000\n".as_slice(), b"http://[::1]:4000\n"] {
        assert!(directory::address(address).is_ok(), "{address:?}");
    }
}
#[test]
fn a_non_loopback_address_exits_1_before_the_token() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.address = b"http://192.0.2.1:4000\n";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.token_reads, 0);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_refused_directory_exits_1() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.refused = true;
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is not private: Mode(63)\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sigterm_exits_143_printing_nothing_more() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.signal(ServiceSignal::Terminate);
    assert_eq!(f.stderr(), b"");
    assert_eq!(f.exit(), Some(Exit::Code(limits::TERMINATED)));
}
#[test]
fn sigint_exits_130() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.signal(ServiceSignal::Interrupt);
    assert_eq!(f.exit(), Some(Exit::Code(limits::INTERRUPTED)));
}
#[test]
fn partial_writes_finish_before_exit() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.absent = true;
    f.write_size = fixtures::PARTIAL;
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY does not exist\n");
    assert!(f.writes > 1);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn an_absent_directory_is_never_made() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.absent = true;
    f.run();
    assert_eq!(f.private_opens, 0);
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY does not exist\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_zero_byte_token_is_not_signed_in() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.token = b"";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is not signed in\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
/// client/ctl/README.md, 6.1: `address` is written by sign-in, so a DIRECTORY without it is not signed in.
#[test]
fn a_directory_without_an_address_is_not_signed_in() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.address_error = Some(Error::NotFound);
    f.run();
    assert_eq!(str::from_utf8(f.stderr()).expect("diagnostic"), "temper ctl: DIRECTORY is not signed in\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.requests(), 0);
}
/// client/ctl/README.md, 6.1: a step's failure names the step and the error.
#[test]
fn a_directory_step_that_fails_names_the_step_and_the_error() {
    const LONG_ADDRESS: &[u8] = &[b'a'; limits::ADDRESS as usize + 1];
    const LONG_TOKEN: &[u8] = &[b't'; fixtures::TOKEN_BYTES.len() + 1];
    for (address_error, token_error, address, token, line) in [
        (
            Some(Error::Permission),
            None,
            b"http://127.0.0.1:4000\n".as_slice(),
            fixtures::TOKEN_BYTES,
            "temper ctl: DIRECTORY could not be read: address: Permission\n",
        ),
        (
            None,
            Some(Error::Permission),
            b"http://127.0.0.1:4000\n",
            fixtures::TOKEN_BYTES,
            "temper ctl: DIRECTORY could not be read: token: Permission\n",
        ),
        (
            None,
            None,
            LONG_ADDRESS,
            fixtures::TOKEN_BYTES,
            "temper ctl: DIRECTORY could not be read: address: too large\n",
        ),
        (
            None,
            None,
            b"http://127.0.0.1:4000\n",
            LONG_TOKEN,
            "temper ctl: DIRECTORY could not be read: token: too large\n",
        ),
    ] {
        let mut f = fixtures::Frame::act(fixtures::CHAT);
        f.address_error = address_error;
        f.token_error = token_error;
        f.address = address;
        f.token = token;
        f.run();
        assert_eq!(str::from_utf8(f.stderr()).expect("diagnostic"), line);
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.requests(), 0);
        assert!(skein_lib::bytes::find(f.stderr(), fixtures::TOKEN_BYTES).is_none());
    }
}
#[test]
fn a_read_never_opens_the_store() {
    let mut f = fixtures::Engine::read(&["me", "--json"]);
    f.sign_in_then_deliver_me();
    f.drain();
    assert!(!f.opened(b"store"));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_read_waits_for_me() {
    let mut f = fixtures::Engine::read(&["me", "--json"]);
    f.drain();
    assert_eq!(f.exit(), None);
    assert_eq!(f.stdout(), b"");
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(f.stdout().starts_with(b"{\""));
}
#[test]
fn an_unheard_read_exits_1_at_the_link_deadline() {
    let mut f = fixtures::Engine::read(&["me"]);
    f.drain();
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.stderr(), b"temper ctl: the engine at http://127.0.0.1:4000 did not answer a read within 30 s\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_read_refused_before_hearing_the_engine_exits_1_at_once() {
    let mut f = fixtures::Engine::read(&["me"]);
    f.refuse_connections = true;
    f.drain();
    assert_eq!(f.refused_connections, 1);
    assert_eq!(f.stderr(), b"temper ctl: nothing listens at http://127.0.0.1:4000; is temper running?\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn an_act_refused_before_hearing_the_engine_keeps_its_store_and_releases_its_lock() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.refuse_connections = true;
    f.drain();
    assert_eq!(f.refused_connections, 1);
    assert_eq!(f.stderr(), b"temper ctl: nothing listens at http://127.0.0.1:4000; is temper running?\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.kept_records(), 1);
    assert!(!f.act_sent());
    assert_eq!(f.locks, 1);
    assert_eq!(f.released_after, Some(0));
}
#[test]
fn a_sign_in_refused_before_hearing_the_engine_exits_1_at_once() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.refuse_connections = true;
    f.drain();
    assert_eq!(f.refused_connections, 1);
    assert_eq!(f.stderr(), b"temper ctl: nothing listens at http://127.0.0.1:4000; is temper running?\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
    assert_eq!(f.released_after, Some(0));
}
#[test]
fn a_sign_in_that_heard_busy_keeps_its_deadline_when_the_retry_is_refused() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.busy_links = 1;
    f.answer_sign_in_then_me();
    f.refuse_connections = true;
    f.advance_past(SIGN_IN_FIRST);
    assert_eq!(f.refused_connections, 1);
    assert_eq!(f.exit(), None);
    assert_eq!(f.stderr(), b"");
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: temper is busy; this link stays usable until it expires\n");
}
#[test]
fn streams_full_exits_one_at_the_link_deadline() {
    for bound in [jig_client_protocol_wire::StreamBound::Component, jig_client_protocol_wire::StreamBound::Transport] {
        for words in [&["me"][..], &["task", "show", "41", "--until", "changed"][..]] {
            let mut f = fixtures::Engine::full_streams(words, bound.clone(), fixtures::FULL_STREAMS);
            f.advance_past(skein_lib::Duration::from_nanos(limits::DEADLINE.as_nanos() - DEADLINE_TICK.as_nanos()));
            assert_eq!(f.exit(), None);
            assert_eq!((f.stderr()).len(), 0);
            f.advance_past(DEADLINE_TICK);
            assert_eq!(f.exit(), Some(Exit::Code(1)));
            assert_eq!(f.stderr(), b"temper ctl: the engine's 2 streams are all in use, by waits and open clients; end one, or wait and call again\n");
            assert!(f.stderr().len() <= 240);
            assert_eq!((f.stdout()).len(), 0);
        }
    }
}
#[test]
fn streams_full_uses_the_reported_size_within_the_diagnostic_bound() {
    let mut f = fixtures::Engine::full_streams(&["me"], jig_client_protocol_wire::StreamBound::Transport, u32::MAX);
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: the engine's 4294967295 streams are all in use, by waits and open clients; end one, or wait and call again\n");
    assert!(f.stderr().len() <= 240);
}
#[test]
fn streams_full_retries_and_succeeds_when_a_stream_frees_before_the_deadline() {
    for bound in [jig_client_protocol_wire::StreamBound::Component, jig_client_protocol_wire::StreamBound::Transport] {
        let mut f = fixtures::Engine::full_streams(&["me", "--json"], bound, fixtures::FULL_STREAMS);
        f.streams_full = None;
        f.sign_in_then_deliver_me();
        f.advance_past(limits::native().root.domain.client.domain.backoff.first);
        assert_eq!(f.exit(), Some(Exit::Code(0)));
        assert!(f.stdout().starts_with(b"{\""));
        assert_eq!((f.stderr()).len(), 0);
        assert_eq!(f.host.root.client().link().streams_full, None);
    }
}
#[test]
fn a_read_sends_no_stream_gone() {
    let mut f = fixtures::Engine::read(&["me"]);
    f.sign_in_then_deliver_me();
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(!f.sent_stream_gone());
}
#[test]
fn memory_saves_are_stored_in_order_without_file_operations() {
    use jig_client::{Event, Request};
    use skein_lib::{List, Time};
    let limits = limits::native();
    let mut memory = crate::transport::Memory::new(
        directory::address(b"http://127.0.0.1:4000\n").expect("fixture address"),
        &limits.connections,
    );
    for request in [
        Request::Save { records: List::with_capacity(0) },
        Request::Erase { records: List::with_capacity(0) },
        Request::Save { records: List::with_capacity(0) },
    ] {
        memory.down(Time::ZERO, request);
        assert_eq!(memory.events.pop(), Some(Event::Stored));
    }
    assert_eq!(memory.events.pop(), None);
    memory.progress(Time::ZERO);
    assert!(memory.request().is_none());
}
#[test]
fn host_operations_cover_its_io() {
    use skein_shell::Host;
    let f = fixtures::Engine::read(&["me"]);
    let limits = limits::native();
    assert_eq!(f.host.operations(), skein_io::operations(&limits.io).expect("native operation bound") * 2 + 5);
}
#[test]
fn sign_in_keeps_its_record_before_success() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), None);
    assert_eq!(f.stored(b"party"), None, "no party before the token and address are durable");
    assert!(!f.opened(b"party"), "an empty token does not read party");
    f.store_token_with_address();
    assert_eq!(f.stored(b"address"), Some(b"http://127.0.0.1:4000\n".as_slice()));
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.stored(b"party"), Some(b"7\n".as_slice()));
    assert_eq!(f.stored_modes.get(b"party".as_slice()), Some(&0o600));
    assert!(f.link_bearers.iter().all(Option::is_none));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_signed_in_directory_refuses_a_link_without_spending_it() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.address = b"http://127.0.0.1:3999\n";
    f.token = fixtures::TOKEN_BYTES;
    f.party = Some(b"7\n");
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is signed in as party 7; this link is not spent: sign out first, temper ctl DIRECTORY sign-out, or sign in to another directory\n");
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.requests(), 0);
    assert_eq!(f.link_posts.len(), 0);
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
    assert_eq!(f.stored(b"party"), None);
    assert_eq!(f.token, fixtures::TOKEN_BYTES);
    assert_eq!(f.address, b"http://127.0.0.1:3999\n");
    assert_eq!(f.party, Some(b"7\n".as_slice()));
}

#[test]
fn a_signed_in_directory_without_a_readable_decimal_party_still_refuses() {
    for (party, error) in [
        (None, None),
        (Some(b"not a party\n".as_slice()), None),
        (Some(b"7\n8\n".as_slice()), None),
        (Some(b"7".as_slice()), None),
        (Some(b"+7\n".as_slice()), None),
        (Some(b"\n".as_slice()), None),
        (Some(b"18446744073709551616\n".as_slice()), None),
        (Some(b"000000000000000000007\n".as_slice()), None),
        (Some(b"7\n".as_slice()), Some(skein_io::kernel::Error::Permission)),
    ] {
        let mut f = fixtures::Disk::sign_in(fixtures::LINK);
        f.token = fixtures::TOKEN_BYTES;
        f.party = party;
        f.party_error = error;
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)), "{party:?}, {error:?}");
        assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is signed in; this link is not spent: sign out first, temper ctl DIRECTORY sign-out, or sign in to another directory\n");
        assert_eq!(f.requests(), 0);
        assert_eq!(f.stored(b"token"), None);
        assert_eq!(f.stored(b"address"), None);
        assert_eq!(f.stored(b"party"), None);
    }
}
#[test]
fn a_signed_in_directory_reads_the_largest_party() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.token = fixtures::TOKEN_BYTES;
    f.party = Some(b"18446744073709551615\n");
    f.drain();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is signed in as party 18446744073709551615; this link is not spent: sign out first, temper ctl DIRECTORY sign-out, or sign in to another directory\n");
    assert_eq!(f.requests(), 0);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_party_write_failure_is_named_before_success_prints() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.fail_party = true;
    f.answer_sign_in_then_me();
    f.store_token_with_address();
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.stored(b"address"), Some(b"http://127.0.0.1:4000\n".as_slice()));
    assert_eq!(f.stored(b"party"), None);
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY could not be written: party: Permission\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn the_engine_names_each_link_refusal_without_writing_files() {
    use jig_client_protocol_wire::SignInAnswer;
    for (answer, line) in [
        (SignInAnswer::LinkUsed, b"temper ctl: the sign-in link was refused: it was already used; temper-box serve prints a fresh one\n".as_slice()),
        (SignInAnswer::LinkExpired, b"temper ctl: the sign-in link was refused: it has expired; temper-box serve prints a fresh one\n".as_slice()),
        (SignInAnswer::LinkUnknown, b"temper ctl: the sign-in link was refused: temper does not know it, as another deployment's link or one it no longer remembers; temper-box serve prints a fresh one\n".as_slice()),
        (SignInAnswer::Refused, b"temper ctl: the sign-in link was refused\n".as_slice()),
    ] {
        let mut f = fixtures::Disk::sign_in(fixtures::LINK);
        f.link_refusal = Some(answer);
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.stdout(), b"");
        assert_eq!(f.stderr(), line);
        assert_eq!(f.stored(b"token"), None);
        assert_eq!(f.stored(b"address"), None);
        assert_eq!(f.stored(b"party"), None);
    }
}
#[test]
fn a_non_loopback_link_exits_1_before_sending() {
    let mut f = fixtures::Disk::sign_in(
        b"http://192.0.2.1:4000/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    f.drain();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.requests(), 0);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_refused_link_never_prints_its_secret() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.refuse_link = Some(());
    f.answer_sign_in_then_me();
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"temper ctl: the sign-in link was refused\n");
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_full_link_exits_one_without_writing_token_or_address() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.full_link = Some((2, skein_lib::Wall::EPOCH, skein_lib::Wall::from_nanos(300_000_000_000)));
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"temper ctl: all 2 sign-ins are in use; the earliest ends at 1970-01-01T00:00:00Z; this link stays usable until 1970-01-01T00:05:00Z; after that, temper serve prints a fresh one when it is tried\n");
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
}

#[test]
fn a_busy_link_retries_the_same_link_and_token_then_signs_in() {
    let mut f = fixtures::Disk::act(&[b"sign-in", b"-"]);
    f.input = fixtures::LINK;
    f.busy_links = 1;
    f.token = b"";
    f.answer_sign_in_then_me();
    assert_eq!(f.next_deadline(), Some(skein_lib::Time::ZERO.saturating_add(SIGN_IN_FIRST)));
    assert_eq!(f.link_posts.len(), 1);
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
    f.advance_past(SIGN_IN_FIRST);
    assert_eq!(f.link_posts.len(), 2);
    assert_eq!(f.link_posts[0], f.link_posts[1]);
    assert_eq!(f.link_bearers[0].as_deref(), None);
    assert_eq!(f.link_bearers[0], f.link_bearers[1]);
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.stderr(), b"");
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}

#[test]
fn a_busy_link_past_the_deadline_exits_one_and_keeps_its_files() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.busy_links = u32::MAX;
    f.drain();
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: temper is busy; this link stays usable until it expires\n");
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"address"), None);
}
#[test]
fn sign_in_takes_the_lock() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.lock_busy();
    assert_eq!(f.locks, 1);
    assert!(!f.opened(b"link-file"));
    assert_eq!(f.requests(), 0);
    f.advance_past(limits::LOCK);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sign_in_waits_for_live_me() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.store_token_with_address();
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.exit(), None);
    assert_eq!(f.stdout(), b"");
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_ne!((f.stdout()).len(), 0);
}
#[test]
fn a_new_act_waits_for_the_kept_answers_erase_to_be_stored() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.answer_kept();
    assert_eq!(f.root_kept(), 0);
    assert!(!f.act_sent());
    f.store_erase();
    assert!(f.act_sent());
    assert!(f.stderr().starts_with(b"kept: chat, "));
}
#[test]
fn a_kept_line_names_no_name_or_key() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.answer_kept();
    assert_eq!(f.stderr(), b"kept: chat, done: task 41, chat\n");
    assert_eq!(f.stdout(), b"");
}
#[test]
fn a_dropped_record_prints_its_line() {
    let mut f = fixtures::Disk::act_with(fixtures::dropped());
    f.answer_kept();
    assert_eq!(f.stderr(), b"dropped: 1 kept records these tools cannot read\n");
    assert!(f.act_sent());
}
#[test]
fn another_partys_record_prints_the_parked_line() {
    let mut f = fixtures::Disk::act_with(fixtures::other_party());
    f.answer_kept();
    assert_eq!(f.stderr(), b"kept: 1 for party 8, unsent while it is not signed in here\n");
    assert!(f.act_sent());
}
#[test]
fn a_failed_erase_admits_no_act() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.answer_kept();
    f.fail_erase();
    assert!(!f.act_sent());
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"kept: chat, done: task 41, chat\ntemper ctl: not kept (DIRECTORY's store not written)\n");
}
#[test]
fn two_writes_get_two_ordered_terminals() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.answer_kept();
    let first = f.store_count();
    assert!(!f.act_sent());
    f.store_erase();
    assert!(f.store_count() >= first + 2);
    assert!(f.act_sent());
}
#[test]
fn an_unheard_act_stays_kept() {
    let mut f = fixtures::Disk::act_with(fixtures::small_store(&[]));
    f.answer_kept();
    assert!(f.act_sent());
    let kept = f.stored(b"store").expect("kept act").to_vec();
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stored(b"store"), Some(kept.as_slice()));
    assert!(f.kept_records() > 0);
}
#[test]
fn a_bad_store_exits_1_cannot_be_read() {
    let mut f = fixtures::Disk::act_with(b"not a small store".as_slice().into());
    f.drain();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY's store cannot be read\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sigterm_mid_act_keeps_its_record() {
    let mut f = fixtures::Disk::act_with(fixtures::small_store(&[]));
    f.answer_kept();
    assert!(f.act_sent());
    let kept = f.stored(b"store").expect("kept act").to_vec();
    f.signal(ServiceSignal::Terminate);
    assert_eq!(f.stored(b"store"), Some(kept.as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(limits::TERMINATED)));
}
#[test]
fn the_lock_outlasts_a_store_write_in_flight() {
    let mut f = fixtures::Disk::act_with(fixtures::small_store(&[]));
    f.answer_new_act();
    assert_eq!(f.exit(), None);
    f.signal(ServiceSignal::Terminate);
    assert_eq!(f.released_after, None, "DIRECTORY's lock ended with a store write in flight");
    let before = f.store_count();
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(limits::TERMINATED)));
    assert!(f.store_count() > before);
    assert_eq!(f.released_after, Some(f.store_count()));
}
#[test]
fn a_json_kept_line_is_unprefixed() {
    let mut words = fixtures::CHAT.to_vec();
    words.push(b"--json");
    let mut f = fixtures::Disk::act_words_with(&words, fixtures::kept_chat());
    f.answer_kept();
    assert_eq!(f.stderr(), b"{\"operation\":\"chat\",\"outcome\":{\"done\":{\"document\":{\"created\":{\"first\":41,\"last\":41,\"given\":null}}}}}\n");
    assert_eq!(f.stdout(), b"");
}
#[test]
fn a_chat_answer_waits_for_its_erase() {
    let mut f = fixtures::Disk::act_with(fixtures::small_store(&[]));
    f.answer_new_act();
    assert!(f.act_sent());
    assert_eq!(f.exit(), None);
    assert_eq!(f.stdout(), b"");
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.stdout(), b"done: task 41, chat\n");
}
#[test]
fn sign_in_reads_its_file_beneath_the_file_root() {
    let mut f = fixtures::Disk::act(&[b"sign-in", b"/elsewhere/link"]);
    f.token = b"";
    f.answer_sign_in_then_me();
    assert_eq!(f.link_root, Some(fixtures::FILE_PARENT));
    assert_eq!(f.link_leaf.as_deref(), Some(b"link".as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn file_root_splits_a_file_argument() {
    for (path, parent) in
        [(b"/link".as_slice(), Some(b"/".as_slice())), (b"a/b/link", Some(b"a/b")), (b"link", Some(b".")), (b"-", None)]
    {
        let parsed = crate::arguments::parse(&[b"sign-in", path]).expect("file call");
        assert_eq!(crate::arguments::file_root(&parsed).as_deref(), parent, "{path:?}");
    }
}
#[test]
fn pending_lists_each_kept_request() {
    let mut f = fixtures::local::pending(false, false, false);
    f.drain();
    assert_eq!(f.stdout(), b"chat, party 73, not sent, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n");
    assert_eq!((f.locks, f.requests()), (0, 0));
    assert_eq!(f.store_count(), 0);
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn pending_counts_unread_records_in_its_line() {
    let mut f = fixtures::local::pending(true, false, false);
    f.drain();
    assert_eq!(
        f.stdout(),
        b"chat, party 73, not sent, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n1 kept records these tools cannot read; a call that sends drops them\n"
    );
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn pending_skips_the_stream_record() {
    let mut f = fixtures::local::pending(false, true, false);
    f.drain();
    assert_eq!(f.stdout(), b"chat, party 73, not sent, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n");
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn pending_json_carries_its_rows_with_unread() {
    let mut f = fixtures::local::pending(true, false, true);
    f.drain();
    assert_eq!(f.stdout(), br#"{"pending":[{"operation":"chat","party":73,"first_sent":null,"unanswered":"it clears when the engine answers it, or past its retention; pending lists it"}],"unread":1}"#);
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_file_that_is_no_store_cannot_be_read() {
    let mut f = fixtures::Disk::act_words_with(&[b"pending"], b"not a store".as_slice().into());
    f.drain();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY's store cannot be read\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sign_out_waits_for_the_empty_token_stored() {
    let mut f = fixtures::Disk::act(&[b"sign-out"]);
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), None);
    f.store_token_with_address();
    assert_eq!(f.stored(b"token"), Some(b"".as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn sign_out_empties_a_refused_token_when_the_engine_answers() {
    let mut f = fixtures::Disk::act(&[b"sign-out"]);
    f.refuse_token = true;
    f.hold_stores = Some(());
    f.drain();
    assert_eq!(f.exit(), None, "the empty token must be durable");
    assert_eq!(f.stored(b"token"), None);
    f.store_token_with_address();
    assert_eq!(f.stored(b"token"), Some(b"".as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(0)), "no deadline advance needed");
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"");
}
#[test]
fn sign_out_without_an_answer_keeps_the_token() {
    for listening in [false, true] {
        let mut f = fixtures::Disk::act(&[b"sign-out"]);
        f.refuse_connections = !listening;
        if listening {
            f.quiet.push(b"/jig/v1/stream");
        }
        f.drain();
        if listening {
            assert_eq!(f.exit(), None);
            f.advance_past(skein_lib::Duration::from_secs(30));
        }
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.stored(b"token"), None);
        assert_eq!(f.token, fixtures::TOKEN_BYTES);
        assert_eq!(f.stdout(), b"");
        assert_eq!(
            f.stderr(),
            if listening {
                b"temper ctl: the engine at http://127.0.0.1:4000 did not answer within 30 s\n".as_slice()
            } else {
                b"temper ctl: nothing listens at http://127.0.0.1:4000; is temper running?\n".as_slice()
            }
        );
    }
}
#[test]
fn sign_out_not_ready_keeps_the_token() {
    let mut f = fixtures::Disk::act(&[b"sign-out"]);
    f.sign_out_answer = jig_client_protocol_wire::SignOutAnswer::NotReady;
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), None);
    f.advance_past(skein_lib::Duration::from_secs(30));
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sign_in_dash_reads_stdin() {
    let mut f = fixtures::Disk::act(&[b"sign-in", b"-"]);
    f.token = b"";
    f.input = fixtures::LINK;
    f.answer_sign_in_then_me();
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.link_leaf, None);
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn sign_in_dash_refuses_empty_stdin() {
    for input in [b"".as_slice(), b"\n"] {
        let mut f = fixtures::Disk::act(&[b"sign-in", b"-"]);
        f.input = input;
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(
            f.stderr(),
            b"temper ctl: sign-in: stdin held no link; give the box's sign-in link on stdin: temper ctl DIR sign-in -\n"
        );
        assert!(f.stderr().len() <= 240);
        assert_eq!(f.requests(), 0);
        assert_eq!(f.stdout(), b"");
        assert_eq!(f.stored(b"token"), None);
        assert_eq!(f.stored(b"address"), None);
    }
}
#[test]
fn close_kept_line() {
    let mut f = fixtures::Disk::act_with(fixtures::local::kept_close());
    f.answer_kept();
    assert_eq!(f.stderr(), b"kept: close, done: task 41, close\n");
    assert_eq!(f.stdout(), b"");
}

#[test]
fn a_drawn_key_is_resent_before_a_new_unnamed_request() {
    let words = &[b"task", b"stop".as_slice(), b"7"];
    let mut first = fixtures::Disk::act_words_with(words, fixtures::small_store(&[]));
    first.sign_in_then_deliver_me();
    first.advance_past(crate::limits::DEADLINE);
    let mut next = fixtures::Disk::act_words_with(words, first.store_snapshot());
    next.sign_in_then_deliver_me();
    assert_eq!(next.act_keys().first(), first.act_keys().first());
    assert_eq!(next.act_keys().len(), 2, "replay and a fresh unnamed act");
    assert_ne!(next.act_keys()[0], next.act_keys()[1]);
}
#[test]
fn adopt_is_an_act() {
    let mut f = fixtures::Disk::act_words_with(
        &[
            b"repository",
            b"adopt",
            b"--project",
            b"shop",
            b"--owner",
            b"o",
            b"--name",
            b"n",
            b"--landing",
            b"main",
            b"--key",
            b"adopt-repo",
        ],
        fixtures::small_store(&[]),
    );
    f.sign_in_then_deliver_me();
    f.advance_past(crate::limits::DEADLINE);
    assert_eq!(f.stderr(), b"temper ctl: the engine at http://127.0.0.1:4000 gave no outcome for this act within 30 s; it is kept, and the same --key may be tried again\n");
    assert_eq!(f.kept_records(), fixtures::local::ACT_WITH_STREAM);
    assert_eq!(f.locks, 1);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}

#[test]
fn a_stopping_engine_exits_the_wait_only_at_the_link_deadline() {
    for (span, expected) in [(fixtures::waits::BEFORE_DEADLINE, None), (limits::DEADLINE, Some(Exit::Code(1)))] {
        let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
        f.engine_stopping();
        f.advance_past(span);
        assert_eq!(f.exit(), expected, "{span:?}");
        assert_eq!(
            f.stderr(),
            if expected.is_some() {
                b"temper ctl: the link to the engine at http://127.0.0.1:4000 did not come up within 30 s\n".as_slice()
            } else {
                b""
            },
            "{span:?}"
        );
    }
}
#[test]
fn a_refused_reconnection_keeps_a_heard_wait_until_the_link_deadline() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.refuse_connections = true;
    f.drop_stream();
    f.advance_past(skein_lib::Duration::from_secs(1));
    assert!(f.refused_connections > 0, "the restart's connection was refused");
    assert_eq!(f.exit(), None);
    assert_eq!(f.stderr(), b"");
    f.advance_past(skein_lib::Duration::from_secs(28));
    assert_eq!(f.exit(), None);
    f.advance_past(skein_lib::Duration::from_secs(1));
    assert_eq!(
        f.stderr(),
        b"temper ctl: the link to the engine at http://127.0.0.1:4000 did not come up within 30 s\n"
    );
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_heard_wait_reconnects_after_a_refusal_and_observes_the_change() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.refuse_connections = true;
    f.drop_stream();
    f.advance_past(skein_lib::Duration::from_secs(1));
    assert!(f.refused_connections > 0, "the restart's connection was refused");
    assert_eq!(f.exit(), None);
    f.refuse_connections = false;
    f.sign_in_then_deliver_me();
    f.advance_past(limits::native().io.close_timeout);
    assert!(f.stream_live(), "the engine's restarted stream is live");
    f.change_task();
    assert_eq!(f.stderr(), b"");
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_live_wait_passes_30_seconds() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.advance_live(fixtures::waits::LONG_LIVE);
    assert_eq!(f.exit(), None);
}
#[test]
fn a_wait_timeout_exits_3() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.advance_live(temper_client_tools::limits::WAIT);
    assert_eq!(f.exit(), Some(Exit::Code(3)));
    assert_eq!(f.stderr(), b"");
    assert!(f.stdout().ends_with(b"not met: timeout\n"));
}
#[test]
fn a_changed_wait_exits_0() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.change_task();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_refused_wait_exits_2() {
    let mut f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    f.refuse_wait();
    assert_eq!(f.exit(), Some(Exit::Code(2)));
}
#[test]
fn help_opens_nothing() {
    for words in [&[b"help".as_slice()][..], &[b"help".as_slice(), b"me"], &[b"help".as_slice(), b"me", b"--json"]] {
        let mut f = fixtures::Frame::act(words);
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(0)), "{words:?}");
        assert!(f.no_directory(), "{words:?}");
        let parsed = crate::arguments::parse(words).expect("help call");
        let temper_client_tools::jig::call::Parsed::Call { call, json } = parsed else { unreachable!() };
        let selected = temper_client_tools::help::selection(&call).expect("help selection");
        let expected = if json {
            temper_client_tools::help::json(selected, &temper_client_tools::limits::tools()).expect("JSON help")
        } else {
            let mut out = skein_lib::Writer::new(temper_client_tools::limits::TEXT as usize);
            temper_client_tools::help::selected(selected, &mut out).expect("help");
            let mut exact = skein_lib::Writer::new(out.written());
            temper_client_tools::help::selected(selected, &mut exact).expect("measured help");
            exact.finish()
        };
        assert_eq!(f.stdout(), expected.as_ref(), "{words:?}");
    }
}
#[test]
fn help_json_prints_every_record() {
    use serde_json::Value;
    use temper_client_tools::table::{GROUPS, TOOLS};

    fn records(words: &[&str]) -> Vec<Value> {
        let mut frame = fixtures::Frame::read(words);
        frame.drain();
        assert_eq!(frame.exit(), Some(Exit::Code(0)), "{words:?}");
        assert!(frame.no_directory(), "{words:?}");
        assert!(frame.stderr().is_empty(), "{words:?}");
        serde_json::from_slice(frame.stdout()).expect("JSON array")
    }

    let groups = records(&["help", "--json"]);
    assert_eq!(groups.len(), GROUPS.len());
    for (value, group) in groups.iter().zip(GROUPS) {
        assert_eq!(value.as_object().expect("group object").len(), 2);
        assert_eq!(value["name"], std::str::from_utf8(group.name).expect("group name"));
        assert_eq!(value["line"], std::str::from_utf8(group.line).expect("group line"));
    }

    let chat = records(&["help", "chat", "--json"]);
    assert_eq!(chat.len(), 3);
    assert_eq!(chat[0]["name"], "chat show");
    assert_eq!(chat[1]["name"], "chat start");
    assert_eq!(chat[2]["name"], "chat close");
    let arguments = chat[1]["forms"][0].as_array().expect("chat start arguments");
    let names = arguments.iter().map(|arg| arg["name"].as_str().expect("argument name")).collect::<Vec<_>>();
    assert_eq!(names, ["project", "template", "text", "model", "effort", "sub_agent"]);
    assert_eq!(chat[1]["keyed"], "required");

    let settings = records(&["help", "settings", "show", "--json"]);
    assert_eq!(settings.len(), 2);
    assert_eq!(settings[0]["name"], "settings show");
    assert_eq!(settings[1]["name"], "settings show");
    let arguments = settings[1]["forms"][0].as_array().expect("settings value arguments");
    let names = arguments.iter().map(|arg| arg["name"].as_str().expect("argument name")).collect::<Vec<_>>();
    assert_eq!(names, ["kind", "number", "revision"]);

    let forge = records(&["help", "forge", "connect", "--json"]);
    assert_eq!(forge.len(), 1);
    assert_eq!(forge[0]["name"], "forge connect");
    assert_eq!(forge[0]["forms"].as_array().expect("forge connect forms").len(), 8);

    let me = records(&["help", "me", "--json"]);
    assert_eq!(me.len(), 1);
    assert_eq!(me[0]["name"], "me");

    for (records, name) in
        [(chat, b"chat".as_slice()), (settings, b"settings show"), (forge, b"forge connect"), (me, b"me")]
    {
        let mut expected = Vec::new();
        for row in TOOLS {
            if row.name == name || row.name.split(|byte| *byte == b' ').next() == Some(name) {
                let json = temper_client_tools::jig::json::write_tool(row, &temper_client_tools::limits::tools())
                    .expect("whole tool record");
                expected.push(serde_json::from_slice::<Value>(&json).expect("tool JSON"));
            }
        }
        assert_eq!(records, expected, "every selected record is whole and follows TOOLS order");
    }
}
#[test]
fn call_stdin_cannot_also_give_text() {
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = br#"{"tool":"chat start","arguments":{"project":"shop","text":"-","key":"start-chat"}}"#;
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert!(f.no_directory());
}
#[test]
fn usage_exits_1_before_directory_opens() {
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = br#"{"tool":"unknown","arguments":{}}"#;
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(
        f.stderr(),
        b"temper ctl: usage: temper ctl DIR GROUP VERB [ARGUMENT...]; temper ctl help GROUP VERB prints one tool\n"
    );
    assert!(f.no_directory());
}
#[test]
fn a_json_argument_refusal_names_the_tool_before_opening_the_directory() {
    for path in ["-", "/elsewhere/call.json"] {
        let mut f = fixtures::Frame::read(&["--call", path]);
        let call = br#"{"tool":"task activity","arguments":{"task":2,"limit":40}}"#;
        if path == "-" {
            f.input = call;
        } else {
            f.link = call;
        }
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(
            f.stderr(),
            b"temper ctl: task activity: --limit COUNT takes a count from 1 to 32; temper ctl help task activity\n"
        );
        assert_eq!((f.stdout()).len(), 0);
        assert!(!f.opened(b"directory"));
        assert_eq!(f.requests(), 0);
    }
}
#[test]
fn a_terminal_task_phase_names_until_ended_and_sends_nothing() {
    use temper_client_tools::jig::argv::Usage;
    assert_eq!(
        crate::arguments::parse(&[b"task", b"show", b"41", b"--until", b"phase:done"]),
        Err(Usage::TerminalPhase { phase: jig_client::contract::Phase::Done })
    );
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = br#"{"tool":"task show","arguments":{"task":41,"until":"phase:done"}}"#;
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(
        f.stderr(),
        b"temper ctl: --until phase:done never holds: a task's watch ends with the task; use --until ended\n"
    );
    assert_eq!((f.stdout()).len(), 0);
    assert!(f.no_directory());
    assert_eq!(f.requests(), 0);
}
#[test]
fn a_json_call_reads_its_file_beneath_the_file_root() {
    let mut f = fixtures::Frame::read(&["--call", "/elsewhere/call.json"]);
    f.link = br#"{"tool":"me","arguments":{}}"#;
    f.sign_in_then_deliver_me();
    assert_eq!(f.link_root, Some(fixtures::FILE_PARENT));
    assert_eq!(f.link_leaf.as_deref(), Some(b"call.json".as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_call_read_from_a_file_that_names_a_file_is_usage() {
    for call in [SIGN_IN_FILE_CALL, br#"{"tool":"sign-in","arguments":{"file":"-"}}"#] {
        let mut f = fixtures::Frame::read(&["--call", "/elsewhere/call.json"]);
        f.link = call;
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)), "{call:?}");
        assert_eq!(
            f.stderr(),
            b"temper ctl: usage: temper ctl DIR GROUP VERB [ARGUMENT...]; temper ctl help GROUP VERB prints one tool\n",
            "{call:?}"
        );
        assert!(!f.opened(b"directory"), "{call:?}");
    }
}
#[test]
fn a_call_read_from_stdin_that_names_a_file_is_usage() {
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = SIGN_IN_FILE_CALL;
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(
        f.stderr(),
        b"temper ctl: usage: temper ctl DIR GROUP VERB [ARGUMENT...]; temper ctl help GROUP VERB prints one tool\n"
    );
    assert!(f.no_directory());
}
#[test]
fn text_dash_reads_stdin() {
    let mut f = fixtures::Frame::act_words_with(
        &[b"chat", b"start", b"-", b"--project", b"shop", b"--key", b"start-chat"],
        fixtures::small_store(&[]),
    );
    f.input = b"hello";
    f.answer_new_act();
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.stdout(), b"done: task 41, chat\n");
}

#[test]
fn a_wait_exposes_its_root_policy_deadline() {
    use skein_shell::Host;
    let f = fixtures::Engine::wait(&["task", "show", "41", "--until", "changed"]);
    let expected = temper_client_tools::root::next_deadline(&f.host.root).expect("live wait has a deadline");
    assert_eq!(f.host.next_policy_deadline(), Some(expected));
}
#[test]
fn a_json_call_reads_stdin() {
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = br#"{"tool":"me","arguments":{}}"#;
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(!f.opened(b"call.json"));
    assert_ne!((f.stdout()).len(), 0);
}

#[test]
fn a_file_call_advertises_its_initial_work() {
    use skein_shell::Host;
    let f = fixtures::Frame::read(&["--call", "/elsewhere/call.json"]);
    assert!(f.host.work_pending(skein_lib::Time::ZERO));
}

#[test]
fn a_link_answered_again_is_sent_after_backoff() {
    let mut f = fixtures::Disk::act(&[b"sign-in", b"-"]);
    f.input = fixtures::LINK;
    f.again_links = 1;
    f.token = b"";
    f.answer_sign_in_then_me();
    assert_eq!(f.next_deadline(), Some(skein_lib::Time::ZERO.saturating_add(SIGN_IN_FIRST)));
    assert_eq!(f.link_posts.len(), 1);
    f.advance_past(SIGN_IN_FIRST);
    assert_eq!(f.link_posts.len(), 2);
    assert_eq!(f.link_posts[0], f.link_posts[1]);
    assert_eq!(f.link_bearers[0].as_deref(), None, "a link never offers a held token");
    assert_eq!(f.link_bearers[0], f.link_bearers[1], "retry offers the same token");
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.stderr(), b"");
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}

#[test]
fn repeated_again_keeps_the_original_sign_in_deadline() {
    let mut f = fixtures::Disk::act(&[b"sign-in", b"-"]);
    f.token = b"";
    f.input = fixtures::LINK;
    f.again_links = u32::MAX;
    f.drain();
    let mut elapsed = skein_lib::Duration::ZERO;
    let mut backoff = SIGN_IN_FIRST;
    while elapsed < limits::DEADLINE {
        let span = backoff
            .min(skein_lib::Duration::from_nanos(limits::DEADLINE.as_nanos().saturating_sub(elapsed.as_nanos())));
        f.advance_past(span);
        elapsed = elapsed.saturating_add(span);
        backoff = backoff.saturating_mul(2).min(skein_lib::Duration::from_secs(2));
    }
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: temper is not ready: it is starting, or its core is not answering, and temper serve's log says why; this link stays usable until it expires\n");
}

#[test]
fn a_sign_in_answered_not_ready_then_never_again_says_not_ready() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.again_links = 1;
    f.drain();
    f.quiet.push(b"/jig/v1/sign-in/link");
    f.advance_past(SIGN_IN_FIRST);
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"temper ctl: temper is not ready: it is starting, or its core is not answering, and temper serve's log says why; this link stays usable until it expires\n");
}

mod busy_watches;
mod keys;

mod moves;

#[test]
fn an_unanswered_adopt_holds_no_close_export_or_unrelated_adopt() {
    use fixtures::recovery::{ADOPT, unanswered_adopt};
    let mut other = ADOPT.to_vec();
    *other.last_mut().expect("adopt key") = b"another-adopt";
    for (words, status) in [
        (&[b"chat".as_slice(), b"close", b"7", b"--key", b"close"][..], 0),
        (&[b"account".as_slice(), b"export", b"7", b"/linked-parent/record"][..], 0),
        (other.as_slice(), 2),
    ] {
        let store = unanswered_adopt();
        let mut f = fixtures::Frame::act_words_with(words, store);
        let original = f.initial_records().into_iter().find(|row| row.key.len() == 16).expect("restored adoption");
        f.move_answers
            .extend([fixtures::moves::Answer::Moving(fixtures::moves::RECORD), fixtures::moves::Answer::Done]);
        f.answer_new_act();
        assert!(f.act_sent(), "new act must be sent beside kept adoption");
        f.store_erase();
        assert_eq!(f.exit(), Some(Exit::Code(status)), "{:?}", f.stderr());
        assert!(skein_lib::bytes::find(f.stderr(), b"kept: adopt, first sent ").is_some());
        assert!(
            skein_lib::bytes::find(
                f.stderr(),
                b"unanswered: it clears when the engine answers it, or past its retention; pending lists it"
            )
            .is_some()
        );
        let mut pending = fixtures::Frame::act_words_with(&[b"pending"], f.store_snapshot());
        assert!(pending.initial_records().contains(&original), "kept record remains byte identical");
        pending.drain();
        assert!(skein_lib::bytes::find(pending.stdout(), b"adopt, party 7, first sent ").is_some());
    }
}
#[test]
fn a_kept_adoption_is_named_at_finish_after_the_stream_drops() {
    let mut f = fixtures::Frame::act_words_with(
        &[b"chat", b"close", b"7", b"--key", b"close"],
        fixtures::recovery::unanswered_adopt(),
    );
    let original = f.initial_records().into_iter().find(|row| row.key.len() == 16).expect("restored adoption");
    f.answer_new_act();
    f.drop_stream();
    let me = f
        .host
        .root
        .client()
        .document(jig_client::Handle { asker: temper_client_tools::root::RUNNER.asker(), number: 0 })
        .expect("retained me");
    assert_eq!(me.state, jig_client::State::Behind);
    assert!(f.act_sent(), "close dispatched beside the replayed adoption");
    assert_eq!(f.exit(), None, "close waits for its erase acknowledgment");
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.stdout(), b"done: task 41, close\n");
    let mut pending = fixtures::Frame::act_words_with(&[b"pending"], f.store_snapshot());
    assert!(pending.initial_records().contains(&original), "kept adoption remains byte identical");
    pending.drain();
    assert_eq!(pending.exit(), Some(Exit::Code(0)));
    assert!(skein_lib::bytes::find(pending.stdout(), b"adopt, party 7, first sent ").is_some());
    let stderr = std::str::from_utf8(f.stderr()).expect("notice text");
    assert_eq!(stderr.lines().count(), 1, "{stderr:?}");
    assert_eq!(stderr.matches("kept: adopt, first sent ").count(), 1);
    assert_eq!(
        stderr
            .matches("unanswered: it clears when the engine answers it, or past its retention; pending lists it")
            .count(),
        1
    );
}

#[test]
fn a_rerun_under_a_kept_name_waits_for_that_answer_and_a_different_ask_is_held() {
    use fixtures::recovery::{ADOPT, unanswered_adopt};
    let mut f = fixtures::Frame::act_words_with(ADOPT, unanswered_adopt());
    f.sign_in_then_deliver_me();
    assert_eq!(f.act_keys().len(), 1, "same ask reuses the restored request");
    assert_eq!((f.stdout()).len(), 0);
    f.answer_restored();
    f.store_erase();
    assert_eq!(f.exit(), Some(Exit::Code(2)));
    assert!(f.stderr().starts_with(b"No repository o/n"), "{:?}", f.stderr());
    assert!(skein_lib::bytes::find(f.stderr(), b"kept:").is_none());
    let mut words = ADOPT.to_vec();
    words.extend_from_slice(&[b"--prefix", b"different/"]);
    let mut f = fixtures::Frame::act_words_with(&words, unanswered_adopt());
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.act_keys().len(), 1);
    assert!(skein_lib::bytes::find(f.stderr(), b"held").is_some());
}

#[test]
fn a_jig_rerun_prints_the_kept_requests_success_as_its_own() {
    let mut first = fixtures::Frame::act_words_with(fixtures::CHAT, fixtures::small_store(&[]));
    first.sign_in_then_deliver_me();
    let mut rerun = fixtures::Frame::act_words_with(fixtures::CHAT, first.store_snapshot());
    rerun.sign_in_then_deliver_me();
    assert_eq!(rerun.act_keys(), first.act_keys());
    assert_eq!(rerun.exit(), None);
    rerun.answer_restored();
    assert_eq!(rerun.exit(), None, "answer waits for its erase to be durable");
    rerun.store_erase();
    assert_eq!(rerun.exit(), Some(Exit::Code(0)));
    assert_eq!(rerun.stdout(), b"done: task 41, chat\n");
    assert_eq!((rerun.stderr()).len(), 0);
    assert_eq!(rerun.kept_records(), 1, "only the stream record remains");
}

#[test]
fn every_watched_read_is_classified_from_its_wait_flags() {
    use crate::arguments::{self, Class};
    let reads: &[&[&str]] = &[
        &["me"],
        &["inbox"],
        &["task", "show", "1"],
        &["task", "tree", "1"],
        &["task", "run", "1"],
        &["task", "list", "--project", "1"],
        &["account", "list"],
        &["forge", "list"],
        &["change", "show", "1"],
    ];
    for read in reads {
        for flags in [&[][..], &["--until", "changed"], &["--since", "0123456789abcdef"]] {
            let words: Vec<_> = read.iter().chain(flags).map(|word| word.as_bytes()).collect();
            let temper_client_tools::jig::call::Parsed::Call { call, .. } =
                arguments::parse(&words).expect("watched read")
            else {
                panic!("call")
            };
            assert_eq!(arguments::class(&call), if flags.is_empty() { Class::Read } else { Class::Wait });
            assert!(!arguments::class(&call).locks());
        }
    }
}

#[test]
fn a_cold_setup_waits_past_the_link_deadline_then_adopts() {
    for json in [false, true] {
        let mut f = fixtures::Frame::setup(json);
        assert!(f.stdout().windows(7).any(|s| s == b"setting"));
        f.advance_live(fixtures::setup::COLD_START);
        assert_eq!(f.exit(), None);
        f.finish_setup();
        assert_eq!(f.exit(), Some(Exit::Code(0)));
        assert_eq!((f.stderr()).len(), 0);
        assert!(f.stdout().windows(7).any(|s| s == if json { b"adopted" } else { b"Adopted" }));
    }
}
#[test]
fn a_cold_setup_past_its_allowance_names_the_forges_and_timeout() {
    for json in [false, true] {
        let mut f = fixtures::Frame::setup(json);
        f.advance_live(temper_client_tools::limits::WAIT);
        assert_eq!(f.exit(), Some(Exit::Code(3)));
        assert!(f.stdout().windows(7).any(|s| s == b"timeout"));
        assert!(f.stdout().windows(1).any(|s| s == b"3"));
        assert_eq!((f.stderr()).len(), 0);
    }
}

#[test]
fn a_change_wait_retries_not_ready_reads_to_the_link_deadline() {
    let mut f = fixtures::Engine::change_wait("60s");
    f.change_task_not_ready();
    f.advance_live(fixtures::waits::BEFORE_DEADLINE);
    assert_eq!(f.exit(), None, "{}", String::from_utf8_lossy(f.stderr()));
    assert_eq!(f.stderr(), b"");
    f.advance_live(limits::native().root.domain.client.domain.backoff.first);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: the engine at http://127.0.0.1:4000 did not answer a read within 30 s\n");
}
#[test]
fn a_change_wait_rides_a_stopping_engine_to_its_timeout() {
    let mut f = fixtures::Engine::change_wait("12s");
    f.engine_stopping();
    f.engine_restarted();
    f.advance_live(fixtures::changes::TIMEOUT);
    assert_eq!(f.exit(), Some(Exit::Code(3)), "{}", String::from_utf8_lossy(f.stderr()));
    assert_eq!(f.stderr(), b"");
    assert!(f.stdout().ends_with(b"not met: timeout\n"));
    assert!(
        f.stdout()
            .windows(b"held: its wait stalled at gating".len())
            .any(|part| part == b"held: its wait stalled at gating")
    );
}
#[test]
fn a_recovered_change_read_releases_the_link_deadline() {
    let mut f = fixtures::Engine::change_wait("60s");
    f.change_task_not_ready();
    f.change_task_ready();
    f.advance_live(fixtures::waits::LONG_LIVE);
    assert_eq!(f.exit(), None, "{}", String::from_utf8_lossy(f.stderr()));
    assert_eq!(f.stderr(), b"");
}
#[test]
fn a_stopping_engine_ends_a_change_wait_at_the_link_deadline() {
    let mut f = fixtures::Engine::change_wait("60s");
    f.engine_stopping();
    f.advance_past(fixtures::waits::BEFORE_DEADLINE);
    assert_eq!(f.exit(), None);
    f.advance_past(limits::native().root.domain.client.domain.backoff.first);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(
        f.stderr(),
        b"temper ctl: the link to the engine at http://127.0.0.1:4000 did not come up within 30 s\n"
    );
}

#[test]
fn validity_accepts_stock_limits_and_refuses_invalid_edges() {
    let mut limits = limits::native();
    assert!(limits::valid(&limits));
    limits.connections.body_bytes = u32::MAX; // Stream record + 512 overflows.
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.connections.http.head = 1;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.connections.http.read = 0;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.connections.http.send = 0;
    assert!(!limits::valid(&limits));
}

#[test]
fn validity_refuses_unusable_io_and_overflowing_operation_counts() {
    let mut limits = limits::native();
    limits.io.refusals = 0;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.io.retry = skein_lib::Duration::ZERO;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.io.sockets = u32::MAX;
    assert!(!limits::valid(&limits));
}

#[test]
fn validity_accepts_large_representable_io_edges() {
    let mut limits = limits::native();
    limits.io.sockets = (1 << 24_u32) - 1;
    limits.io.sends = u32::MAX;
    assert!(limits::valid(&limits));
}

#[test]
fn landing_reads_keep_missing_and_invalid_document_statuses_in_both_modes() {
    for json in [false, true] {
        for (task, status, message) in [("7", 2, "no landing for task 7"), ("8", 1, "unexpected landing document")] {
            let mut words = vec!["landing", "show", task, "--project", "3"];
            if json {
                words.push("--json");
            }
            let f = fixtures::Frame::change_read(&words);
            assert_eq!(f.exit(), Some(Exit::Code(status)), "task={task}, json={json}");
            assert_eq!((f.stdout()).len(), 0);
            let expected = if json && status == 2 {
                format!("{{\"error\":\"{message}\"}}")
            } else if json {
                format!("temper ctl: {{\"error\":\"{message}\"}}\n")
            } else if status == 1 {
                format!("temper ctl: {message}\n")
            } else {
                format!("{message}\n")
            };
            assert_eq!(f.stderr(), expected.as_bytes());
        }
    }
}

#[test]
fn a_third_project_keeps_setup_calls_usable() {
    let mut setup = fixtures::Frame::project_setup();
    setup.sign_in_then_deliver_me();
    assert_eq!(setup.exit(), Some(Exit::Code(0)));
    assert_eq!(setup.stdout(), b"project 3\n");
    let mut show = setup.setup_call(&["project", "show", "3"]);
    show.sign_in_then_deliver_me();
    show.advance_past(limits::DEADLINE);
    assert_eq!(show.exit(), Some(Exit::Code(0)), "{}", String::from_utf8_lossy(show.stderr()));
    assert!(String::from_utf8_lossy(show.stdout()).contains("proof-5-clearer-words"));
    let mut adopt = show.setup_call(&[
        "repository",
        "adopt",
        "--project",
        "3",
        "--owner",
        "o",
        "--name",
        "n",
        "--landing",
        "main",
        "--key",
        "adopt-third",
    ]);
    adopt.sign_in_then_deliver_me();
    assert!(adopt.act_sent(), "adoption reaches the available engine");
    assert_eq!(adopt.exit(), Some(Exit::Code(0)));
    assert!(String::from_utf8_lossy(adopt.stdout()).contains("Adopted"));
    for _ in 0..2 {
        let mut me = adopt.setup_call(&["me"]);
        me.sign_in_then_deliver_me();
        assert_eq!(me.exit(), Some(Exit::Code(0)));
        assert!(String::from_utf8_lossy(me.stdout()).contains("proof-5-clearer-words"));
    }
    let native = crate::limits::native();
    let preset = native.root.domain.client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("shipped link limits");
    let frame = fixtures::recovery::project_me(9); // Standard's deployment capacity.
    let bytes = frame.measure();
    let decoded = jig_client_protocol_link::translate::documents::frame(frame, &link).expect("nine memberships");
    assert_eq!(decoded.projects.len(), 9);
    eprintln!(
        "nine-project me: {bytes} wire bytes; membership slots: {:?} bytes; document bound: {} bytes",
        skein_lib::List::<jig_client::contract::MeProject>::price(9),
        preset.domain.document_bytes
    );
}

#[test]
fn invalid_history_filters_and_cursors_open_no_directory_and_send_nothing() {
    for words in [
        &["landing", "history", "--project", "1", "--change", "51", "--landing", "61"][..],
        &["landing", "history", "--project", "1", "--after", "0200000000000000280000000000000020"][..],
    ] {
        let borrowed: Vec<_> = words.iter().map(|word| word.as_bytes()).collect();
        assert!(crate::arguments::parse(&borrowed).is_err());
    }
    for input in [
        br#"{"tool":"landing history","arguments":{"project":"1","change":51,"landing":61}}"#.as_slice(),
        br#"{"tool":"landing history","arguments":{"project":"1","after":"0200000000000000280000000000000020"}}"#
            .as_slice(),
    ] {
        let mut frame = fixtures::Frame::read(&["--call", "-"]);
        frame.input = input;
        frame.drain();
        assert_eq!(frame.exit(), Some(Exit::Code(1)));
        assert!(frame.no_directory());
        assert_eq!(frame.requests(), 0);
    }
}
