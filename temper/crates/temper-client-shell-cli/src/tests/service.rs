//! Host-owned service files are private, checked before sending and never printed.
use super::fixtures::{Frame, service};
use skein_io::kernel::{Exit, Kind, OpenHow, Stat};
const MAKE: &[&str] = &["party", "make", "--project", "7", "--role", "2", "--out", "record", "scout", "--key", "scout"];
fn make() -> Frame {
    Frame::read(MAKE)
}
#[test]
fn party_make_writes_the_private_file_only_after_the_answer_is_durable() {
    let mut f = make();
    f.hold_service_erase = true;
    f.answer_sign_in_then_me();
    assert_eq!(f.made_services, 1);
    assert!(!f.record_exists, "no file before the answer is durable");
    assert!(f.record_data.is_empty());
    assert!(f.stdout().is_empty());
    f.store_token_with_address();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.stdout(), b"made: party 9\n");
    assert_eq!(f.record_how, Some(OpenHow::CreateNoFollow { mode: Some(0o600) }));
    assert!(f.record_data.starts_with(b"http://127.0.0.1:4000\n"));
    assert!(f.record_data.ends_with(&service::CREDENTIAL));
    assert_eq!(f.file_steps, ["create", "write", "sync-file", "sync-directory"]);
    assert_eq!(f.made_services, 1);
    f.no_service_leaked();
}
#[test]
fn party_make_refuses_an_existing_file_before_sending() {
    let mut f = make();
    f.record_exists = true;
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.requests(), 0);
    assert!(f.file_steps.is_empty());
    assert_eq!(f.stderr(), b"temper ctl: record exists or could not be checked\n");
}
#[test]
fn a_party_make_key_replay_prints_the_party_and_writes_nothing() {
    let mut f = make();
    f.service_replay = true;
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.stdout(), b"made: party 9\n");
    assert!(!f.record_exists);
    assert!(f.file_steps.is_empty());
    f.no_service_leaked();
}
#[test]
fn sign_in_service_uses_the_files_address_and_credential_and_keeps_the_sign_in() {
    let mut f = Frame::read(&["sign-in", "--service", "link-file"]);
    f.link = service::FILE;
    f.token = b"";
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(f.link_bearers.first().and_then(Option::as_ref).is_some_and(|bytes| bytes.as_ref() == service::CREDENTIAL));
    assert_eq!(f.stored(b"address"), Some(b"http://127.0.0.1:4100\n".as_slice()));
    assert!(f.stored(b"token").is_some_and(|token| token == super::fixtures::TOKEN_BYTES));
    assert!(!f.stdout().is_empty());
    f.no_service_leaked();
}
#[test]
fn sign_in_service_refuses_group_or_other_access_before_reading_or_sending() {
    for mode in [0o640, 0o604, 0o610, 0o601] {
        let mut f = Frame::read(&["sign-in", "--service", "link-file"]);
        f.link = service::FILE;
        f.input_stat = Some(Stat { kind: Kind::File, size: service::FILE.len() as u64, mode, owner: 7, links: 1 });
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.requests(), 0);
        assert_eq!(f.input_reads, 0);
        assert!(f.stderr().starts_with(b"temper ctl: service file mode "));
    }
}
#[test]
fn sign_in_service_refuses_other_forms_before_sending() {
    for bytes in [
        b"".as_slice(),
        super::fixtures::LINK,
        b"http://127.0.0.1:4000\nshort",
        b"not-an-address\nZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
    ] {
        let mut f = Frame::read(&["sign-in", "--service", "link-file"]);
        f.link = bytes;
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.requests(), 0);
        assert!(f.stored(b"address").is_none());
        assert!(f.stored(b"token").is_none());
        f.no_service_leaked();
    }
}
