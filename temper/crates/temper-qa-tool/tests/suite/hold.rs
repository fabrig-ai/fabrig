//! The holder watches the same locked ledger as the loop (qa/places.md, 5 and 8).
use super::{
    passes,
    place::{assert_mode, under_umasks},
};
use skein_scratch::Scratch;
use std::{fs, os::unix::fs::PermissionsExt, time::Duration};
use temper_qa_tool::{
    Exit, Home,
    hold::{Drive, Holder},
};

#[test]
fn open_tightens_the_place_and_privatizes_its_log_before_launch() {
    if under_umasks("hold::open_tightens_the_place_and_privatizes_its_log_before_launch", &["022", "000"]) {
        return;
    }
    let scratch = Scratch::new("qa-open-modes");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#).expect("config");
    let home = Home::open(scratch.path()).expect("home");
    let pass = home
        .transact(|ledger| {
            passes::queue(ledger, "work", "web");
            passes::build(ledger, "A", &[1], 5);
            Ok(passes::start(ledger))
        })
        .expect("start pass");
    let place = home.directory.join("places/1");
    for path in [&place, &place.join("box"), &place.join("copies")] {
        fs::create_dir_all(path).expect("existing place");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("old directory mode");
    }
    fs::write(place.join("setup"), "fresh\n").expect("setup");
    for name in ["drive.log", "drive.lock"] {
        let path = place.join(name);
        fs::write(&path, "old").expect("old file");
        fs::set_permissions(path, fs::Permissions::from_mode(0o666)).expect("old file mode");
    }
    // Refuse the test executable's harness start, after open prepares its paths and log.
    let environment = vec![(Box::from(b"RUST_TEST_THREADS".as_slice()), Box::from(b"invalid".as_slice()))];
    let reply = Drive::for_session(&home, pass.session).expect("drive").open(&environment).expect("launch refusal");
    assert_ne!(reply.status, 0);
    for path in [&place, &place.join("box"), &place.join("copies")] {
        assert_mode(path, 0o700);
    }
    for name in ["drive.log", "drive.lock"] {
        assert_mode(&place.join(name), 0o600);
    }
}

#[test]
fn the_ledger_watch_ends_the_drive_and_releases_the_place() {
    if under_umasks("hold::the_ledger_watch_ends_the_drive_and_releases_the_place", &["022", "000"]) {
        return;
    }
    for fixed in [false, true] {
        let scratch = Scratch::new("qa-holder-watch");
        let mut config = serde_json::json!({"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}});
        if fixed {
            config["places"] = serde_json::json!([{"directory":"places/1", "ports":[31001,31002,31003,31004,31005,31006,31007,31008,31009]}]);
        }
        fs::write(scratch.path().join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("config");
        let home = Home::open(scratch.path()).expect("home");
        let pass = home
            .transact(|ledger| {
                passes::queue(ledger, "work", "web");
                passes::build(ledger, "A", &[1], 5);
                Ok(passes::start(ledger))
            })
            .expect("start pass");
        let place = home.directory.join("places/1");
        fs::create_dir_all(place.join("bin")).expect("bin");
        fs::write(place.join("setup"), "fresh\n").expect("setup");
        let fake = place.join("bin/temper-client-journeys");
        fs::write(&fake, "#!/bin/sh\nprintf 'ready\\n'\nIFS= read -r line\nprintf '%s\\n' \"$line\" > ended-by\n[ \"$line\" = end ] || exit 4\nprintf 'ended\\n'\n").expect("fake");
        fs::set_permissions(fake, fs::Permissions::from_mode(0o700)).expect("executable");
        // Use an absolute output path without changing the test process's working directory.
        let fake = place.join("bin/temper-client-journeys");
        let script = fs::read_to_string(&fake)
            .expect("script")
            .replace("> ended-by", &format!("> '{}'", place.join("ended-by").display()))
            .replace(
                "printf 'ready",
                &format!("printf '%s\\n' \"$*\" > '{}'; printf 'ready", place.join("arguments").display()),
            );
        fs::write(fake, script).expect("output path");
        let holder =
            Holder::start(Drive::for_session(&home, pass.session).expect("drive"), vec![], Duration::from_millis(1))
                .expect("holder");
        assert_mode(&place.join("drive.lock"), 0o600);
        assert_mode(&place.join("drive.sock"), 0o600);
        let lock = fs::File::open(place.join("drive.lock")).expect("lock");
        assert!(lock.try_lock().is_err(), "the holder keeps the place busy before its start is requested");
        home.transact(|ledger| ledger.end_pass_session(pass.session, Exit::Code(0), None, 6, "qa", &[]))
            .expect("end pass operation");
        let clock = skein_shell::Clock::new();
        let before = clock.now().now.as_nanos();
        holder.serve().expect("holder ends on next watch without a client request");
        let elapsed = clock.now().now.as_nanos().saturating_sub(before);
        assert!(elapsed < 1_000_000_000, "watch took {elapsed} ns with a 1ms period");
        assert_eq!(fs::read_to_string(place.join("ended-by")).expect("drive got end"), "end\n");
        assert!(!place.join("drive.sock").exists());
        lock.try_lock().expect("place free");
        let args = fs::read_to_string(place.join("arguments")).expect("drive arguments");
        assert!(args.contains(&format!("--files {}", home.directory.join("passes/1/files").display())), "{args}");
        assert_eq!(args.contains("--ports 31001,31002,31003,31004,31005,31006,31007,31008,31009"), fixed, "{args}");
        assert_eq!(args.contains("--ports"), fixed, "{args}");
    }
}
