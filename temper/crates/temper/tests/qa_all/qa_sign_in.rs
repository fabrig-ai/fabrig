//! Signing in by the printed link (client/README.md 6.1, 6.7, 7; client/tools.md 8.1; jig's domain/people.md 3): a QA pass run by `--profile qa`.
#![expect(clippy::unwrap_used, reason = "Test fixtures may panic (skein testing-strategy.md 4).")]
use crate::support;

use skein_io::kernel::Exit;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};
use support::{
    Forge, Scratch,
    ctl::{self, Call, Step},
};

fn deployment(name: &str) -> Scratch {
    let scratch = Scratch::new(name);
    support::configuration(scratch.path(), &Forge::Unreached);
    scratch
}

fn success(ended: &support::Ended) {
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
}

#[test]
fn ctl_printed_link_saves_a_private_usable_credential() {
    let scratch = deployment("qa-sign-in-ctl");
    let seen = ctl::run(ctl::Story::new(
        scratch.path(),
        vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))],
    ));
    for call in &seen.calls {
        success(call);
    }
    assert_eq!(seen.calls.len(), 2);
    let token = scratch.path().join("client/token");
    assert!(!fs::read(&token).expect("saved credential").is_empty());
    assert_eq!(fs::metadata(&token).unwrap().permissions().mode() & 0o777, 0o600);
    assert!(!ctl::json(&seen.calls[1]).is_null());
    success(&seen.serve);
    scratch.remove();
}

#[test]
fn open_printed_link_reaches_the_inbox() {
    let scratch = deployment("qa-sign-in-open");
    let seen =
        ctl::run(ctl::Story::new(scratch.path(), vec![Step::Open { directory: "terminal", until: Some(b"Inbox") }]));
    assert_eq!(seen.calls.len(), 1);
    success(&seen.calls[0]);
    assert!(String::from_utf8_lossy(&seen.calls[0].stdout).contains("Inbox"));
    assert!(!fs::read(scratch.path().join("terminal/token")).expect("saved credential").is_empty());
    success(&seen.serve);
    scratch.remove();
}

// domain/testing.md 2.6 uses two slots: one held and one that must remain available.
const SIGN_INS: usize = 2;

fn two_slots(name: &str) -> Scratch {
    let scratch = deployment(name);
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["engine"]["limits"]["overrides"]["people.sign_ins"] = SIGN_INS.into();
    fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    scratch
}

fn signed_in(scratch: &Scratch, steps: Vec<Step>) -> ctl::Seen {
    let seen = ctl::run(ctl::Story::new(scratch.path(), steps));
    for call in &seen.calls {
        success(call);
    }
    success(&seen.serve);
    seen
}

fn token_unchanged(path: &std::path::Path, before: &[u8], original: &fs::Metadata) {
    assert!(fs::read(path).unwrap() == before, "held credential changed");
    let metadata = fs::metadata(path).unwrap();
    assert_eq!(metadata.ino(), original.ino(), "token file replaced");
    assert_eq!(
        (metadata.mtime(), metadata.mtime_nsec()),
        (original.mtime(), original.mtime_nsec()),
        "token file written"
    );
}

#[test]
fn ctl_keeps_its_held_credential_even_when_the_pool_is_full() {
    let scratch = two_slots("qa-sign-in-kept-ctl");
    signed_in(&scratch, vec![Step::Run(Call::sign_in())]);
    let path = scratch.path().join("client/token");
    let before = fs::read(&path).unwrap();
    let metadata = fs::metadata(&path).unwrap();
    // A second fresh link must leave the second slot free for another directory.
    signed_in(&scratch, vec![Step::Run(Call::sign_in())]);
    signed_in(&scratch, vec![Step::Run(Call::sign_in().in_directory("second"))]);
    // The fourth fresh link must keep the first sign-in even at the pool's bound.
    signed_in(&scratch, vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))]);
    token_unchanged(&path, &before, &metadata);
    scratch.remove();
}

#[test]
fn open_keeps_its_held_credential_even_when_the_pool_is_full() {
    let scratch = two_slots("qa-sign-in-kept-open");
    let open = || Step::Open { directory: "terminal", until: Some(b"Inbox") };
    signed_in(&scratch, vec![open()]);
    let path = scratch.path().join("terminal/token");
    let before = fs::read(&path).unwrap();
    let metadata = fs::metadata(&path).unwrap();
    signed_in(&scratch, vec![open()]);
    signed_in(&scratch, vec![Step::Run(Call::sign_in().in_directory("second"))]);
    let seen = signed_in(&scratch, vec![open()]);
    assert!(String::from_utf8_lossy(&seen.calls[0].stdout).contains("Inbox"));
    token_unchanged(&path, &before, &metadata);
    scratch.remove();
}

fn full(ended: &support::Ended, client: &str) -> (String, String) {
    assert_eq!(ended.exit, Some(Exit::Code(1)));
    // The terminal's frame lines (`== temper ==`, `temper: closed`) are printed at any exit (client/README.md 6.7).
    let stdout = String::from_utf8_lossy(&ended.stdout);
    assert!(!stdout.contains("Inbox"), "full must not print a successful result: {stdout:?}");
    let line = String::from_utf8(ended.stderr.clone()).unwrap();
    let prefix = format!("temper {client}: all {SIGN_INS} sign-ins are in use; the earliest ends at ");
    let rest = line.strip_prefix(&prefix).expect("full is distinct from busy and names the capacity");
    let (earliest, rest) = rest.split_once("; this link stays usable until ").expect("earliest expiry");
    let expiry = rest
        .strip_suffix("; after that, temper serve prints a fresh one when it is tried\n")
        .expect("link expiry and remedy");
    for moment in [earliest, expiry] {
        assert_eq!(moment.len(), "2026-10-23T15:00:00Z".len());
        for (at, expected) in [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':'), (19, b'Z')] {
            assert_eq!(moment.as_bytes()[at], expected, "UTC to the second");
        }
        assert!(moment.bytes().enumerate().all(|(i, c)| [4, 7, 10, 13, 16, 19].contains(&i) || c.is_ascii_digit()));
    }
    (earliest.into(), expiry.into())
}

fn fill_pool(scratch: &Scratch) {
    signed_in(scratch, vec![Step::Run(Call::sign_in())]);
    signed_in(scratch, vec![Step::Run(Call::sign_in().in_directory("second"))]);
}

fn refused_link_then_next(directory: &str) -> Vec<Step> {
    vec![Step::Run(Call::sign_in().in_directory(directory)), Step::NextLink]
}

#[test]
fn full_link_crosses_ctl_and_open_then_ctl_retries() {
    // Calls: first, spent, second, spent, ctl-full, open-full, sign-out, retry, me.
    const CTL_FULL: usize = 4;
    const OPEN_FULL: usize = 5;
    const SIGN_OUT: usize = 6;
    const RETRY: usize = 7;
    const ME: usize = 8;
    let scratch = two_slots("qa-sign-in-full-ctl");
    let mut steps = vec![Step::Run(Call::sign_in())];
    steps.extend(refused_link_then_next("second"));
    steps.push(Step::Run(Call::sign_in().in_directory("second")));
    steps.extend(refused_link_then_next("third"));
    steps.extend([
        Step::Run(Call::sign_in().in_directory("third")),
        Step::Open { directory: "terminal", until: None },
        Step::Run(Call::new(&["sign-out"])),
        Step::Run(Call::sign_in().in_directory("third")),
        Step::Run(Call::new(&["me", "--json"]).in_directory("third")),
    ]);
    let seen = ctl::run(ctl::Story::new(scratch.path(), steps).measured(&["third", "terminal"]));
    assert_eq!(seen.calls.len(), ME + 1);
    let ctl_times = full(&seen.calls[CTL_FULL], "ctl");
    let open_times = full(&seen.calls[OPEN_FULL], "open");
    assert_eq!(ctl_times, open_times, "a full attempt must preserve the original link expiry");
    for at in [SIGN_OUT, RETRY, ME] {
        success(&seen.calls[at]);
    }
    assert!(!ctl::json(&seen.calls[ME]).is_null());
    for at in [CTL_FULL, OPEN_FULL] {
        assert!(seen.tokens[at].iter().all(Option::is_none), "full wrote a token");
    }
    success(&seen.serve);
    scratch.remove();
}

#[test]
fn full_link_crosses_open_and_ctl_then_open_retries() {
    const OPEN_FULL: usize = 4;
    const CTL_FULL: usize = 5;
    const SIGN_OUT: usize = 6;
    const RETRY: usize = 7;
    let scratch = two_slots("qa-sign-in-full-open");
    let mut steps = vec![Step::Run(Call::sign_in())];
    steps.extend(refused_link_then_next("second"));
    steps.push(Step::Run(Call::sign_in().in_directory("second")));
    steps.extend(refused_link_then_next("third"));
    steps.extend([
        Step::Open { directory: "terminal", until: None },
        Step::Run(Call::sign_in().in_directory("third")),
        Step::Run(Call::new(&["sign-out"])),
        Step::Open { directory: "terminal", until: Some(b"Inbox") },
    ]);
    let seen = ctl::run(ctl::Story::new(scratch.path(), steps).measured(&["third", "terminal"]));
    assert_eq!(seen.calls.len(), RETRY + 1);
    assert_eq!(full(&seen.calls[OPEN_FULL], "open"), full(&seen.calls[CTL_FULL], "ctl"));
    success(&seen.calls[SIGN_OUT]);
    success(&seen.calls[RETRY]);
    assert!(String::from_utf8_lossy(&seen.calls[RETRY].stdout).contains("Inbox"));
    for at in [OPEN_FULL, CTL_FULL] {
        assert!(seen.tokens[at].iter().all(Option::is_none), "full wrote a token");
    }
    success(&seen.serve);
    scratch.remove();
}

fn kept_expiry(name: &str, keep: impl Fn() -> Step) {
    let scratch = two_slots(name);
    fill_pool(&scratch);
    let probe = || vec![Step::Run(Call::sign_in().in_directory("third"))];
    let before = ctl::run(ctl::Story::new(scratch.path(), probe()));
    let (earliest, _) = full(&before.calls[0], "ctl");
    success(&before.serve);
    signed_in(&scratch, vec![keep()]);
    let after = ctl::run(ctl::Story::new(scratch.path(), probe()));
    assert_eq!(full(&after.calls[0], "ctl").0, earliest, "kept sign-in extended its saved expiry");
    success(&after.serve);
    scratch.remove();
}

#[test]
fn ctl_kept_sign_in_preserves_its_saved_expiry() {
    kept_expiry("qa-sign-in-expiry-ctl", || Step::Run(Call::sign_in()));
}

#[test]
fn open_kept_sign_in_preserves_its_saved_expiry() {
    // The native terminal must keep its own directory, so give it the first slot.
    let scratch = two_slots("qa-sign-in-expiry-open");
    signed_in(&scratch, vec![Step::Open { directory: "terminal", until: Some(b"Inbox") }]);
    signed_in(&scratch, vec![Step::Run(Call::sign_in().in_directory("second"))]);
    let probe = || vec![Step::Run(Call::sign_in().in_directory("third"))];
    let before = ctl::run(ctl::Story::new(scratch.path(), probe()));
    let (earliest, _) = full(&before.calls[0], "ctl");
    success(&before.serve);
    // Cross the diagnostic clock's one-second resolution before reusing the link.
    std::thread::sleep(std::time::Duration::from_secs(1));
    signed_in(&scratch, vec![Step::Open { directory: "terminal", until: Some(b"Inbox") }]);
    let after = ctl::run(ctl::Story::new(scratch.path(), probe()));
    assert_eq!(full(&after.calls[0], "ctl").0, earliest, "kept sign-in extended its saved expiry");
    success(&after.serve);
    scratch.remove();
}

#[test]
fn bad_held_credential_signs_in_afresh() {
    let scratch = two_slots("qa-sign-in-bad-held");
    signed_in(&scratch, vec![Step::Run(Call::sign_in())]);
    let path = scratch.path().join("client/token");
    let mut corrupted = fs::read(&path).unwrap();
    // Keep the native token's encoding and sign-in number, changing only its bearer bytes.
    let middle = corrupted.len() / 2;
    let byte = corrupted.get_mut(middle).expect("credential bytes");
    *byte = if *byte == b'a' { b'b' } else { b'a' };
    fs::write(&path, &corrupted).unwrap();
    signed_in(&scratch, vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))]);
    assert!(fs::read(path).unwrap() != corrupted, "bad credential was kept");
    scratch.remove();
}

#[test]
#[ignore = "backlog: each serve start takes a new port, and ctl's sign-out reaches the previous one by DIR/address, waiting 30 s (the ctl DIR/address finding); production's clients port is fixed"]
fn ended_held_credential_signs_in_afresh() {
    let scratch = two_slots("qa-sign-in-ended-held");
    signed_in(&scratch, vec![Step::Run(Call::sign_in())]);
    let path = scratch.path().join("client/token");
    let before = fs::read(&path).unwrap();
    signed_in(&scratch, vec![Step::Run(Call::new(&["sign-out"]))]);
    fs::write(&path, &before).unwrap();
    signed_in(&scratch, vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))]);
    assert!(fs::read(path).unwrap() != before, "ended credential was kept");
    scratch.remove();
}

#[test]
fn full_ctl_keeps_existing_token_and_address() {
    let scratch = two_slots("qa-sign-in-full-files");
    fill_pool(&scratch);
    let token = scratch.path().join("client/token");
    let address = scratch.path().join("client/address");
    let mut held = fs::read(&token).unwrap();
    let middle = held.len() / 2;
    held[middle] = if held[middle] == b'a' { b'b' } else { b'a' };
    fs::write(&token, &held).unwrap();
    let token_metadata = fs::metadata(&token).unwrap();
    let saved_address = fs::read(&address).unwrap();
    let address_metadata = fs::metadata(&address).unwrap();
    // An unrecognized credential needs a new slot; full must preserve both old files.
    let seen = ctl::run(ctl::Story::new(scratch.path(), vec![Step::Run(Call::sign_in())]));
    full(&seen.calls[0], "ctl");
    token_unchanged(&token, &held, &token_metadata);
    token_unchanged(&address, &saved_address, &address_metadata);
    success(&seen.serve);
    scratch.remove();
}

#[test]
fn spent_link_is_refused_and_serve_prints_a_fresh_one() {
    const REFUSED: usize = 1;
    const FRESH: usize = 2;
    let scratch = two_slots("qa-sign-in-spent");
    let seen = ctl::run(ctl::Story::new(
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::sign_in().in_directory("second")),
            Step::NextLink,
            Step::Run(Call::sign_in().in_directory("second")),
        ],
    ));
    assert_eq!(seen.calls[REFUSED].exit, Some(Exit::Code(1)));
    assert_eq!(seen.calls[REFUSED].stderr, b"temper ctl: the sign-in link was refused\n");
    success(&seen.calls[FRESH]);
    success(&seen.serve);
    scratch.remove();
}
