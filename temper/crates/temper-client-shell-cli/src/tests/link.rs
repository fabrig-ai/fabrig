//! Asked links use the existing fake disk and HTTP engine, never a live listener.
use super::fixtures::{FILE_PARENT, Frame, TOKEN_BYTES};
use jig_client_protocol_wire::LinkAskAnswer;
use skein_io::kernel::{Error, Exit, Kind, OpenHow};
use skein_lib::Duration;
const URL: &[u8] = b"http://127.0.0.1:4000/sign-in?secret=__________________________________________8\n";
fn link() -> Frame {
    Frame::read(&["link", "--out", "record"])
}
fn no_leak(f: &Frame) {
    for bytes in [f.stdout(), f.stderr()] {
        assert!(!bytes.windows(7).any(|part| part == b"secret="));
        assert!(!bytes.windows(8).any(|part| part == b"________"));
        assert!(!bytes.windows(7).any(|part| part == b"http://"));
    }
    assert_eq!(f.stored(b"address"), None);
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.stored(b"store"), None);
    assert!(!f.opened(b"store"));
}
#[test]
fn link_writes_the_private_url_and_waits_for_durable_success() {
    for json in [false, true] {
        let mut f = if json { Frame::read(&["link", "--out", "record", "--json"]) } else { link() };
        f.hold_stores = Some(());
        f.answer_sign_in_then_me();
        assert_eq!(f.asked_attempts.len(), 1);
        assert_eq!(f.asked_bearers.as_slice(), &[Box::from(TOKEN_BYTES)]);
        assert_eq!(f.stdout(), b"", "no success until the file and parent are synced");
        assert_eq!(f.exit(), None);
        f.store_token_with_address();
        assert_eq!(f.exit(), Some(Exit::Code(0)));
        assert_eq!(f.record_data, URL);
        assert_eq!(f.record_how, Some(OpenHow::CreateNoFollow { mode: Some(0o600) }));
        assert_eq!(f.record_root, Some(FILE_PARENT));
        assert_eq!(f.file_steps, ["create", "write", "sync-file", "sync-directory"]);
        assert_eq!(f.locks, 0, "link takes the read path, with no DIRECTORY lock");
        assert_eq!(f.stderr(), b"");
        if json {
            let record: serde_json::Value = serde_json::from_slice(f.stdout()).expect("link JSON");
            assert_eq!(record, serde_json::json!({"file": "record", "expires": "2000-02-29T12:34:56Z"}));
        } else {
            assert_eq!(f.stdout(), b"sign-in link written to record; it signs in once, until 2000-02-29T12:34:56Z\n");
        }
        no_leak(&f);
    }
}
#[test]
fn an_asked_link_signs_in_another_directory() {
    let mut asked = link();
    asked.answer_sign_in_then_me();
    assert_eq!(asked.record_data, URL);
    let mut signed = Frame::sign_in(URL);
    signed.answer_sign_in_then_me();
    signed.store_token_with_address();
    assert_eq!(signed.exit(), Some(Exit::Code(0)));
    assert_eq!(signed.stored(b"token"), Some(TOKEN_BYTES));
    assert_eq!(signed.stored(b"address"), Some(b"http://127.0.0.1:4000\n".as_slice()));
    assert_eq!(signed.link_posts.len(), 1);
    let record = jig_client_protocol_wire::LinkSignIn::decode(
        &jig_client_protocol_wire::CEILINGS,
        &mut skein_lib::Reader::new(&signed.link_posts[0]),
    )
    .expect("sign-in record");
    assert_eq!(record.secret().expose(), &[255; 32]);
}
#[test]
fn link_refuses_an_existing_output_before_sending() {
    for kind in [Kind::File, Kind::Symlink, Kind::Directory] {
        let mut f = link();
        f.record_exists = true;
        f.record_existing_kind = Some(kind);
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.requests(), 0);
        assert_eq!(f.file_steps.len(), 0);
        assert_eq!(f.stderr(), b"temper ctl: record exists or could not be checked\n");
        no_leak(&f);
    }
}
#[test]
fn link_requires_a_signed_in_loopback_directory_before_sending() {
    for case in ["no address", "no token", "empty token", "not loopback"] {
        let mut f = link();
        let line = match case {
            "no address" => {
                f.address_error = Some(Error::NotFound);
                b"temper ctl: DIRECTORY is not signed in\n".as_slice()
            }
            "no token" => {
                f.token_error = Some(Error::NotFound);
                b"temper ctl: DIRECTORY is not signed in\n"
            }
            "empty token" => {
                f.token = b"";
                b"temper ctl: DIRECTORY is not signed in\n"
            }
            "not loopback" => {
                f.address = b"http://192.0.2.1:4000\n";
                b"temper ctl: the engine's address must be on loopback\n"
            }
            _ => unreachable!(),
        };
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)), "{case}");
        assert_eq!(f.requests(), 0, "{case}");
        assert!(!f.record_exists);
        assert_eq!(f.stderr(), line, "{case}");
        no_leak(&f);
    }
}
#[test]
fn link_uses_the_directory_address_including_ipv6() {
    let mut f = link();
    f.address = b"http://[::1]:4100\n";
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert_eq!(f.record_data, b"http://[::1]:4100/sign-in?secret=__________________________________________8\n");
    no_leak(&f);
}
#[test]
fn link_engine_refusals_write_no_file() {
    for (answer, line) in [
        (LinkAskAnswer::Refused, b"temper ctl: the engine refused a sign-in link: it gives one to a person's sign-in alone, where its printed link is enabled\n".as_slice()),
        (LinkAskAnswer::SignedOut, b"temper ctl: signed out\n"),
    ] {
        let mut f = link();
        f.asked_answers.push_back(answer);
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.asked_attempts.len(), 1);
        assert_eq!(f.stderr(), line);
        assert_eq!(f.stdout(), b"");
        assert!(!f.record_exists);
        no_leak(&f);
    }
}
#[test]
fn link_retries_transient_answers_with_sign_ins_backoff() {
    let mut f = link();
    f.asked_answers.extend([LinkAskAnswer::NotReady, LinkAskAnswer::Busy]);
    f.answer_sign_in_then_me();
    assert_eq!(f.asked_attempts.len(), 1);
    f.advance_past(Duration::from_millis(99));
    assert_eq!(f.asked_attempts.len(), 1);
    f.advance_past(Duration::from_millis(1));
    assert_eq!(f.asked_attempts.len(), 2);
    f.advance_past(Duration::from_millis(199));
    assert_eq!(f.asked_attempts.len(), 2);
    f.advance_past(Duration::from_millis(1));
    assert_eq!(f.asked_attempts.len(), 3);
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(f.asked_bearers.iter().all(|bearer| bearer.as_ref() == TOKEN_BYTES));
    assert_eq!(f.record_data, URL);
    no_leak(&f);
}
#[test]
fn link_deadline_names_the_last_transient_answer() {
    for (answer, line) in [
        (LinkAskAnswer::Busy, b"temper ctl: temper is busy\n".as_slice()),
        (LinkAskAnswer::NotReady, b"temper ctl: temper is not ready: it is starting, or its core is not answering, and temper serve's log says why\n"),
    ] {
        let mut f = link();
        f.asked_answers.push_back(answer);
        f.answer_sign_in_then_me();
        f.quiet.push(b"/jig/v1/link/ask");
        f.advance_past(Duration::from_millis(100));
        assert_eq!(f.asked_attempts.len(), 1);
        assert_eq!(f.exit(), None);
        f.advance_past(Duration::from_secs(31));
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert_eq!(f.stdout(), b"");
        assert_eq!(f.stderr(), line);
        assert!(!f.record_exists);
        no_leak(&f);
    }
}
#[test]
fn link_reports_each_failed_file_step_without_success() {
    for step in ["create", "write", "sync-file", "sync-directory"] {
        let mut f = link();
        if step == "create" {
            f.record_error = Some(Error::NoSpace);
        } else {
            f.record_fault = Some(step);
        }
        f.answer_sign_in_then_me();
        assert_eq!(f.exit(), Some(Exit::Code(1)), "{step}");
        assert_eq!(f.asked_attempts.len(), 1);
        assert_eq!(f.stdout(), b"");
        assert_eq!(f.stderr(), b"temper ctl: record could not be written: NoSpace\n", "{step}");
        no_leak(&f);
    }
}
