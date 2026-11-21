//! Worker shorthands against the tools' fake engine (client/ctl/workers.md, 3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust fixtures and output comparisons"
)]
use crate::{jig, tests::fixtures as f};
use jig_client::contract as c;
use skein_lib::{List, Wall};

const DIGEST: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddee00";
const SHA256: [u8; 32] = [
    0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 17, 34, 51, 68, 85, 102, 119, 136, 153,
    170, 187, 204, 221, 238, 0,
];
fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let mut out = List::uncapped();
    for value in values {
        assert!(out.push(value).is_ok());
    }
    out
}
pub(super) fn row(number: u32, name: &str, withdrawn: bool) -> c::WorkerState {
    c::WorkerState {
        pause: c::WorkerPause::Taking,
        number,
        name: name.as_bytes().into(),
        places: 2,
        digest: Some(SHA256),
        withdrawn,
        linked: false,
        since: None,
    }
}
fn setting(number: u32) -> c::Setting {
    c::Setting::Worker(c::SettingNumber { number })
}
fn added(number: u32) -> c::SettingEdit {
    c::SettingEdit::Set(c::SettingSet {
        value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
            number,
            name: b"w2".as_slice().into(),
            places: 4,
            digest: Some(SHA256),
            withdrawn: false,
        })),
    })
}
fn add() -> Vec<&'static str> {
    f::WORKER_ADD.to_vec()
}
fn act(argv: &[&str], rows: Vec<c::WorkerState>) -> (f::Harness, u32) {
    let (mut root, call) = f::read_call(argv);
    serve(&mut root, call, rows);
    (root, call)
}
pub(super) fn serve(root: &mut f::Harness, call: u32, rows: Vec<c::WorkerState>) {
    assert_eq!(root.reads.get(&call).expect("newest revision read first").1, c::Read::Settings);
    assert_eq!(root.reads.len(), 1);
    assert!(root.extra_reads.is_empty());
    assert!(!root.sends.contains_key(&call));
    root.serve(
        call,
        c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
            newest: 17,
            settings: list([]),
            held: list([]),
            pending: list([]),
        }))),
    );
    assert_eq!(root.reads.get(&call).expect("deployment read after the revision").1, c::Read::Deployment);
    assert!(!root.sends.contains_key(&call), "both reads precede the change");
    let mut deployment = f::deployment();
    deployment.worker_settings = list(rows);
    root.serve(call, c::ReadResult::Value(c::ReadValue::Deployment(Box::new(deployment))));
}
fn change(root: &f::Harness, call: u32, number: u32, edit: c::SettingEdit) {
    assert_eq!(root.sends.len(), 1);
    assert!(root.reads.is_empty());
    assert_eq!(
        f::act_ask(root, call),
        &c::Ask::ChangeSettings(c::ChangeSettings {
            against: 17,
            items: list([c::SettingItem { setting: setting(number), edit }]),
            running: c::Running::Stays,
        })
    );
}
#[test]
fn worker_add_changes_one_free_number_against_the_newest_revision() {
    for json in [false, true] {
        let mut argv = add();
        if json {
            argv.push("--json");
        }
        let (mut root, call) = act(&argv, vec![row(1, "local", false)]);
        change(&root, call, 2, added(2));
        root.done(call, c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 18 }));
        let answer = root.answer(call).expect("worker changed");
        assert_eq!(answer.status, 0);
        assert_eq!(
            answer.stdout.as_ref(),
            if json { br#"{"revision":18}"#.as_slice() } else { b"worker w2: revision 18\n" }
        );
    }
}
#[test]
fn worker_add_revives_a_withdrawn_number_but_refuses_a_live_name() {
    let (root, call) = act(&add(), vec![row(1, "local", false), row(7, "w2", true)]);
    change(&root, call, 7, added(7));
    let (mut root, call) = act(&add(), vec![row(7, "w2", false)]);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("live name refused locally");
    assert_eq!(answer.status, 2);
    assert_eq!(answer.stderr.as_ref(), b"worker w2 exists; `worker places` changes its places\n");
}
#[test]
fn worker_places_preserves_the_value_and_remove_sends_removed_at_its_number() {
    for verb in ["places", "remove"] {
        let mut argv = vec!["worker", verb, "w1"];
        if verb == "places" {
            argv.push("8");
        }
        argv.extend(["--key", "edit-w1"]);
        let mut worker = row(7, "w1", false);
        worker.linked = true;
        let (mut root, call) = act(&argv, vec![worker]);
        let edit = if verb == "places" {
            c::SettingEdit::Set(c::SettingSet {
                value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
                    number: 7,
                    name: b"w1".as_slice().into(),
                    places: 8,
                    digest: Some(SHA256),
                    withdrawn: false,
                })),
            })
        } else {
            c::SettingEdit::Removed
        };
        change(&root, call, 7, edit);
        root.done(call, c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 18 }));
        assert_eq!(root.answer(call).expect("changed").stdout.as_ref(), b"worker w1: revision 18\n");
    }
}
#[test]
fn worker_edits_refuse_unknown_and_withdrawn_names_without_sending() {
    for argv in [
        vec!["worker", "places", "w1", "8", "--key", "places-w1"],
        vec!["worker", "remove", "w1", "--key", "remove-w1"],
    ] {
        for rows in [vec![], vec![row(7, "w1", true)]] {
            let (mut root, call) = act(&argv, rows);
            assert!(root.sends.is_empty());
            let answer = root.answer(call).expect("no live name refused");
            assert_eq!(answer.status, 2);
            assert_eq!(answer.stderr.as_ref(), b"no worker w1\n");
        }
    }
}
fn malformed(fault: c::SettingFault) -> c::Refusal {
    c::Refusal::Malformed { at: 1, why: super::malformed_setting(setting(7), fault) }
}
#[test]
fn worker_refusals_keep_the_change_status_and_say_the_remedy_without_retrying() {
    for (refusal, expected) in [
        (malformed(c::SettingFault::Linked), "w1 is linked: stop it first; its runs end interrupted"),
        (malformed(c::SettingFault::Count), "configured workers' bound reached"),
        (c::Refusal::Stale(c::RefusalStale { newest: 18 }), "stale; newest revision 18; run it again"),
        (c::Refusal::Role { held: Some(c::HeldRole::Member) }, "role"),
    ] {
        for json in [false, true] {
            let mut argv = vec!["worker", "remove", "w1", "--key", "remove-w1"];
            if json {
                argv.push("--json");
            }
            let (mut root, call) = act(&argv, vec![row(7, "w1", false)]);
            change(&root, call, 7, c::SettingEdit::Removed);
            root.answered(call, c::Answer::Refused { refusal: refusal.clone(), at: None });
            let answer = root.answer(call).expect("refusal answered");
            assert_eq!(answer.status, 2);
            assert!(root.reads.is_empty());
            assert_eq!(root.sends.len(), 1);
            if json {
                assert!(core::str::from_utf8(&answer.stderr).expect("JSON").contains("outcome"));
            } else {
                assert!(core::str::from_utf8(&answer.stderr).expect("refusal").contains(expected), "{answer:?}");
            }
        }
    }
}
#[test]
fn worker_arguments_refuse_malformed_sha256_zero_places_and_bad_names_before_sending() {
    for (name, places, digest) in [
        ("w2", "4", &DIGEST[..63]),
        ("w2", "4", "00112233445566778899AABBCCDDEEFF00112233445566778899AABBCCDDEE00"),
        ("w2", "4", "g0112233445566778899aabbccddeeff00112233445566778899aabbccddee00"),
        ("w2", "0", DIGEST),
        ("w2", "4294967296", DIGEST),
        (".w2", "4", DIGEST),
        ("w/2", "4", DIGEST),
        ("w 2", "4", DIGEST),
    ] {
        let root = f::signed_in(f::PARTY);
        f::parse(&["worker", "add", name, "--places", places, "--digest", digest, "--key", "add-w2"])
            .expect_err("invalid worker argument is usage");
        let json = format!(
            r#"{{"tool":"worker add","arguments":{{"worker":"{name}","places":{places},"digest":"{digest}","key":"add-w2"}}}}"#
        );
        f::json(json.as_bytes()).expect_err("invalid JSON worker argument is usage");
        assert!(root.sends.is_empty());
        assert!(root.reads.is_empty());
    }
    f::parse(&["worker", "places", "w1", "0", "--key", "places-w1"]).expect_err("places start at one");
    for verb in ["pause", "drain", "resume"] {
        f::parse(&["worker", verb, "w1", "--key", "not-keyed"]).expect_err("asks take no key");
    }
    for condition in ["drained:", "paused:.w1", "drained:w/1", "ended", "usable"] {
        f::parse(&["worker", "list", "--until", condition]).expect_err("not a worker condition");
    }
    f::parse(&["worker", "list", "--timeout", "1s"]).expect_err("timeout needs a wait");
    for condition in ["drained:w1", "paused:W_2.x-9", "changed"] {
        let argv = ["worker", "list", "--until", condition, "--timeout", "1"];
        let call = f::call(&argv);
        assert_eq!(f::back(&call), f::owned(&argv));
        let json = jig::json::write_call(crate::table::TOOLS, &call).expect("worker condition JSON");
        assert_eq!(f::from_json(&json), call);
    }
    for name in ["w2", "W_2.x-9", "1"] {
        let argv = ["worker", "add", name, "--places", "1", "--digest", DIGEST, "--key", "add-w2"];
        let call = f::call(&argv);
        assert_eq!(f::back(&call), f::owned(&argv));
        let json = jig::json::write_call(crate::table::TOOLS, &call).expect("worker call JSON");
        assert_eq!(f::from_json(&json), call);
        assert_eq!(f::printed(&call), jig::call::Parsed::Call { call, json: false });
    }
}
fn deployment_read(root: &mut f::Harness, call: u32, deployment: c::Deployment) {
    assert_eq!(root.reads.len(), 1, "only the deployment, no settings revision");
    assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Deployment);
    assert!(root.extra_reads.is_empty());
    root.serve(call, c::ReadResult::Value(c::ReadValue::Deployment(Box::new(deployment))));
}
fn live(pause: c::WorkerPause) -> c::Deployment {
    let mut worker = row(7, "w1", false);
    worker.linked = true;
    worker.places = 4;
    worker.pause = pause;
    worker.since = Some(Wall::from_nanos(3_660_000_000_000));
    let mut deployment = f::deployment();
    deployment.worker_settings = list([worker]);
    deployment.hosts = list([host("w1", true, 2)]);
    deployment
}
fn worker_watch() -> jig_client::Watchable {
    jig_client::Watchable::Jig(c::Subject::Deployment)
}
fn deliver(root: &mut f::Harness, pause: c::WorkerPause) {
    root.deliver_subject(&worker_watch(), c::Delivery::Deployment(Box::new(live(pause))));
}
fn waiting(flags: &[&str]) -> (f::Harness, u32) {
    let mut argv = vec!["worker", "list"];
    argv.extend(flags);
    let (mut root, call) = f::read_call(&argv);
    assert!(!root.watches.values().any(|watch| *watch == worker_watch()));
    deployment_read(&mut root, call, live(c::WorkerPause::Taking));
    (root, call)
}
#[test]
fn worker_asks_are_unkeyed_unrevisioned_and_print_their_answers() {
    for (verb, mode, text) in [
        ("pause", c::WorkerPauseMode::Pause, "worker w1: asked to pause\n"),
        ("drain", c::WorkerPauseMode::Drain, "worker w1: asked to drain\nworker list --until drained:w1\n"),
        ("resume", c::WorkerPauseMode::Resume, "worker w1: asked to take work again\n"),
    ] {
        for json in [false, true] {
            let mut argv = vec!["worker", verb, "w1"];
            if json {
                argv.push("--json");
            }
            let mut root = f::signed_in(f::PARTY);
            let mut previous = None;
            for _ in 0_u32..2 {
                let call = root.submit(&argv);
                deployment_read(&mut root, call, live(c::WorkerPause::Taking));
                assert_eq!(f::act_ask(&root, call), &c::Ask::PauseWorker { worker: 7, mode });
                assert_eq!(f::call(&["worker", verb, "w1"]).key, None, "no ctl key");
                let envelope = root.sent_name(call).unwrap();
                assert_ne!(previous, Some(envelope), "unkeyed asks get fresh client envelope keys");
                previous = Some(envelope);
                root.done(call, c::Outcome::WorkerPauseAsked { worker: 7 });
                let answer = root.answer(call).unwrap();
                assert_eq!(answer.status, 0);
                assert_eq!(
                    answer.stdout.as_ref(),
                    if json {
                        br#"{"operation":"pause-worker","outcome":{"done":{"worker_pause_asked":{"worker":7}}},"task":null,"to":null}"#.as_slice()
                    } else {
                        text.as_bytes()
                    }
                );
            }
            assert_eq!(root.sends.len(), 2, "asked again is sent again");
        }
    }
}
#[test]
fn worker_asks_refuse_unknown_or_withdrawn_names_before_sending() {
    for verb in ["pause", "drain", "resume"] {
        for rows in [vec![], vec![row(9, "w9", true)]] {
            let (mut root, call) = f::read_call(&["worker", verb, "w9"]);
            let mut deployment = f::deployment();
            deployment.worker_settings = list(rows);
            deployment_read(&mut root, call, deployment);
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 2);
            assert_eq!(answer.stderr.as_ref(), b"no worker w9\n");
            assert!(root.sends.is_empty());
        }
    }
}
#[test]
fn worker_ask_refusals_name_the_unlinked_worker_and_keep_the_other_worker_wording() {
    for verb in ["pause", "drain", "resume"] {
        for (refusal, text) in [
            (c::Refusal::Unlinked { worker: 7 }, "w1 is not linked\n".to_owned()),
            (
                c::Refusal::Role { held: Some(c::HeldRole::Member) },
                format!("worker {verb}: The held role does not permit this action\n"),
            ),
        ] {
            for json in [false, true] {
                let mut argv = vec!["worker", verb, "w1"];
                if json {
                    argv.push("--json");
                }
                let (mut root, call) = f::read_call(&argv);
                deployment_read(&mut root, call, live(c::WorkerPause::Taking));
                root.answered(call, c::Answer::Refused { refusal: refusal.clone(), at: None });
                let answer = root.answer(call).unwrap();
                assert_eq!(answer.status, 2);
                assert_eq!(root.sends.len(), 1);
                assert!(root.reads.is_empty());
                if json {
                    assert!(core::str::from_utf8(&answer.stderr).unwrap().contains("outcome"));
                } else {
                    assert_eq!(answer.stderr.as_ref(), text.as_bytes());
                }
            }
        }
    }
}
#[test]
fn worker_list_prints_pause_and_hashes_only_its_json_rows() {
    for (pause, word, suffix) in [
        (c::WorkerPause::Taking, "taking", ""),
        (c::WorkerPause::Paused, "paused", " · paused"),
        (c::WorkerPause::Draining, "draining", " · draining"),
        (c::WorkerPause::Drained, "drained", " · drained"),
    ] {
        let (mut root, call) = f::read_call(&["worker", "list", "--json"]);
        deployment_read(&mut root, call, live(pause));
        let answer = root.answer(call).unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
        assert_eq!(json["rows"][0]["pause"], word);
        let digest = json.as_object_mut().unwrap().remove("digest").unwrap();
        // Preserve the encoder's field order when checking the raw digest input.
        let document = core::str::from_utf8(&answer.stdout).unwrap();
        let rows = format!("{}}}", document.rsplit_once(",\"digest\":").unwrap().0);
        let expected = jig::digest::Digest::of(rows.as_bytes()).hex();
        assert_eq!(digest, core::str::from_utf8(&expected).unwrap());
        let call = root.submit(&["worker", "list"]);
        let mut deployment = live(pause);
        deployment.worker_settings.push(row(9, "gone", true)).unwrap();
        deployment.accounts = list([]); // Unprinted deployment facts do not affect the digest.
        deployment_read(&mut root, call, deployment);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_eq!(
            core::str::from_utf8(&answer.stdout).unwrap(),
            format!(
                "w1 · 4 places · up since 01:01 · 2 runs{suffix}\ndigest: workers {}\n",
                core::str::from_utf8(&expected).unwrap()
            )
        );
    }
}
#[test]
fn worker_drained_wait_answers_on_the_first_matching_delivery_including_the_first() {
    for first in [false, true] {
        let (mut root, call) = waiting(&["--until", "drained:w1", "--json"]);
        if !first {
            for pause in [c::WorkerPause::Taking, c::WorkerPause::Paused, c::WorkerPause::Draining] {
                deliver(&mut root, pause);
                assert!(root.answer(call).is_none(), "not drained yet");
            }
        }
        deliver(&mut root, c::WorkerPause::Drained);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert!(f::has(&answer, br#""pause":"drained""#));
        assert!(f::has(&answer, br#""met":true"#));
        assert!(!root.watches.values().any(|watch| *watch == worker_watch()));
    }
}
#[test]
fn worker_paused_wait_accepts_paused_draining_and_drained_but_not_taking() {
    for pause in [c::WorkerPause::Paused, c::WorkerPause::Draining, c::WorkerPause::Drained] {
        let (mut root, call) = waiting(&["--until", "paused:w1"]);
        deliver(&mut root, c::WorkerPause::Taking);
        assert!(root.answer(call).is_none());
        deliver(&mut root, pause);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert!(answer.stdout.ends_with(b"met\n"));
    }
}
#[test]
fn worker_wait_timeout_keeps_the_current_rows_and_is_not_met() {
    let (mut root, call) = waiting(&["--until", "drained:w1", "--timeout", "1s"]);
    deliver(&mut root, c::WorkerPause::Draining);
    root.now = root.now.saturating_add(skein_lib::Duration::from_secs(2));
    deliver(&mut root, c::WorkerPause::Drained);
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 3);
    assert!(f::has(&answer, b"not met: timeout\n"));
    assert!(f::has(&answer, b"digest: workers "));
    assert!(!root.watches.values().any(|watch| *watch == worker_watch()));
}
#[test]
fn worker_since_and_changed_wait_for_a_row_change_not_other_deployment_facts() {
    for (since, changed) in [(true, false), (false, true), (true, true)] {
        let (mut root, call) = f::read_call(&["worker", "list", "--json"]);
        deployment_read(&mut root, call, live(c::WorkerPause::Taking));
        let read = root.answer(call).unwrap();
        let digest = f::last_digest(&read.stdout);
        let mut argv = vec!["worker", "list", "--json"];
        if since {
            argv.extend(["--since", core::str::from_utf8(&digest).unwrap()]);
        }
        if changed {
            argv.extend(["--until", "changed"]);
        }
        let call = root.submit(&argv);
        deployment_read(&mut root, call, live(c::WorkerPause::Taking));
        deliver(&mut root, c::WorkerPause::Taking);
        assert!(root.answer(call).is_none(), "read and first delivery share a digest");
        let mut deployment = live(c::WorkerPause::Taking);
        deployment.worker_settings.push(row(9, "gone", true)).unwrap();
        root.deliver_subject(&worker_watch(), c::Delivery::Deployment(Box::new(deployment)));
        assert!(root.answer(call).is_none(), "withdrawn row is not printed");
        deliver(&mut root, c::WorkerPause::Paused);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        assert_ne!(f::last_digest(&answer.stdout), digest);
    }
}
#[test]
fn worker_wait_refuses_unknown_live_names_before_opening_the_watch() {
    for condition in ["drained:w9", "paused:w9"] {
        let (mut root, call) = f::read_call(&["worker", "list", "--until", condition]);
        let mut deployment = live(c::WorkerPause::Drained);
        deployment.worker_settings.push(row(9, "w9", true)).unwrap();
        deployment_read(&mut root, call, deployment);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 1);
        assert_eq!(answer.stderr.as_ref(), b"no worker w9\n");
        assert!(!root.watches.values().any(|watch| *watch == worker_watch()));
        assert!(root.sends.is_empty());
    }
}
fn host(name: &str, connected: bool, count: usize) -> c::Host {
    let mut runs = vec![];
    for attempt in 0..count {
        runs.push(c::HostRun {
            task: c::TaskLink::Live(f::goal(41, f::waiting_on_me()).chip),
            attempt: u64::try_from(attempt).unwrap(),
            kept: 0,
            listed: true,
            state: c::AttemptState::Kept,
            location: c::RunLocation::Connected { channel: 1, kind: c::HostKind::Worker },
            grace_until: None,
        });
    }
    c::Host {
        name: Some(name.as_bytes().into()),
        awaiting_logins: false,
        rule: c::HostRule::Open,
        programs: list([]),
        channel: 1,
        kind: c::HostKind::Worker,
        slots: 4,
        used: 0,
        free: 4,
        draining: false,
        connected,
        grace_until: None,
        grace_remaining: None,
        runs: list(runs),
        cut: false,
    }
}
#[test]
fn worker_list_joins_connected_hosts_keeps_number_order_and_omits_withdrawn_rows() {
    let mut up = row(2, "w1", false);
    up.linked = true;
    up.since = Some(Wall::from_nanos(3_660_000_000_000));
    let mut lost = row(4, "lost", false);
    lost.since = Some(Wall::from_nanos(7_320_000_000_000));
    lost.places = 1;
    let mut deployment = f::deployment();
    deployment.worker_settings = list([up, lost, row(7, "never", false), row(9, "gone", true)]);
    deployment.hosts =
        list([host("w1", false, 3), host("elsewhere", true, 2), host("w1", true, 1), host("lost", false, 2)]);
    for json in [false, true] {
        let (mut root, call) = f::read_call(if json { &["worker", "list", "--json"] } else { &["worker", "list"] });
        assert_eq!(root.reads.len(), 1);
        assert_eq!(root.reads.get(&call).expect("one deployment read").1, c::Read::Deployment);
        root.serve(call, c::ReadResult::Value(c::ReadValue::Deployment(Box::new(deployment.clone()))));
        let answer = root.answer(call).expect("worker list answered");
        assert_eq!(answer.status, 0);
        assert!(root.sends.is_empty());
        assert!(root.reads.is_empty());
        let output = core::str::from_utf8(&answer.stdout).unwrap();
        if json {
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(output).unwrap()["rows"],
                serde_json::from_str::<serde_json::Value>(&format!(
                    r#"[{{"number":2,"name":"w1","places":2,"digest":"{DIGEST}","withdrawn":false,"linked":true,"since":3660000000000,"pause":"taking","runs":1}},{{"number":4,"name":"lost","places":1,"digest":"{DIGEST}","withdrawn":false,"linked":false,"since":7320000000000,"pause":"taking","runs":0}},{{"number":7,"name":"never","places":2,"digest":"{DIGEST}","withdrawn":false,"linked":false,"since":null,"pause":"taking","runs":0}}]"#
                )).unwrap()
            );
        } else {
            assert_eq!(
                output.split("digest: workers ").next().unwrap(),
                "w1 · 2 places · up since 01:01 · 1 run\nlost · 1 place · down since 02:02 · no runs\nnever · 2 places · down · no runs\n"
            );
        }
    }
}
