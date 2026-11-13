//! Restores cross the keyed request boundary once and retain the engine's answer.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary fixtures and printed-byte assertions; testing-strategy.md, step tests"
)]
use crate::{jig::rows, tests::fixtures as f};
use jig_client::contract as c;
use skein_lib::List;

const PARTY: u64 = 1;
const ARGV: &[&str] = &["revision", "restore", "4", "back to the old charter", "--key", "restore-4"];

fn omitted() -> List<c::Setting> {
    let mut settings = List::with_capacity(2);
    settings.push(c::Setting::Application { kind: 2, scope: 3, number: 2 }).unwrap();
    settings.push(c::Setting::ProjectPolicy { project: 3 }).unwrap();
    settings
}
fn restored(left_out: List<c::Setting>) -> c::Outcome {
    c::Outcome::Restored { proposal: 7, to: 0, left_out }
}

#[test]
fn revision_restore_parses_its_revision_reason_and_required_key() {
    assert_eq!(rows::RESTORE_AT, 63);
    for argv in [
        ARGV,
        &["revision", "restore", "4", "--key", "restore-4"],
        &["revision", "restore", "0", "--key", "restore-4"],
        &["revision", "restore", "4294967295", "--key", "restore-4"],
    ] {
        let call = f::call(argv);
        assert_eq!(call.tool, rows::RESTORE_AT);
        assert_eq!(f::back(&call), f::owned(argv));
        assert_eq!(f::from_json(&crate::jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
    }
    for argv in [
        vec!["revision", "restore", "--key", "restore-4"],
        vec!["revision", "restore", "four", "--key", "restore-4"],
        vec!["revision", "restore", "4294967296", "--key", "restore-4"],
        vec!["revision", "restore", "4"],
    ] {
        assert!(f::parse(&argv).is_err(), "{argv:?}");
    }
}

#[test]
fn revision_restore_sends_one_restore_and_prints_the_proposal_and_what_it_left_out() {
    for json in [false, true] {
        for left_out in [false, true] {
            let mut root = f::signed_in(PARTY);
            let number = root.submit_prepared(f::call(ARGV), json, None);
            assert_eq!(root.sends.len(), 1);
            assert!(root.reads.is_empty());
            assert_eq!(root.sent_name(number), Some(f::named("restore-4")));
            assert_eq!(
                f::act_ask(&root, number),
                &c::Ask::Restore {
                    revision: 4,
                    reason: c::Text::new(b"back to the old charter".as_slice().into(), 1024).unwrap(),
                }
            );
            root.done(number, restored(if left_out { omitted() } else { List::with_capacity(0) }));
            let printed = root.answer(number).unwrap();
            assert_eq!(printed.status, 0);
            assert!(printed.stderr.is_empty());
            if left_out {
                f::check_golden(if json { "restore_proposed_json" } else { "restore_proposed" }, &printed.stdout);
            } else if json {
                assert_eq!(printed.stdout.as_ref(), br#"{"operation":"restore","outcome":{"done":{"restored":{"proposal":7,"to":0,"left_out":[]}}},"task":null,"to":null}"#);
            } else {
                assert_eq!(printed.stdout.as_ref(), b"proposed restoring revision 4 as proposal p1/7 to role 0\n");
            }
        }
    }
    let mut root = f::signed_in(PARTY);
    let number = root.submit(&["revision", "restore", "4", "--key", "restore-4"]);
    let c::Ask::Restore { revision, reason } = f::act_ask(&root, number) else { panic!("restore") };
    assert_eq!(*revision, 4);
    assert_eq!((reason.bytes()).len(), 0);
}

#[test]
fn a_restore_replayed_under_its_key_prints_the_first_answer() {
    for json in [false, true] {
        let mut root = f::signed_in(PARTY);
        let first = root.submit_prepared(f::call(ARGV), json, None);
        let ask = f::act_ask(&root, first).clone();
        let key = root.sent_name(first);
        let outcome = restored(omitted());
        root.done(first, outcome.clone());
        let expected = root.answer(first).unwrap();
        let second = root.submit_prepared(f::call(ARGV), json, None);
        assert_eq!(f::act_ask(&root, second), &ask);
        assert_eq!(root.sent_name(second), key);
        root.done(second, outcome);
        let actual = root.answer(second).unwrap();
        assert_eq!((actual.status, actual.stdout, actual.stderr), (expected.status, expected.stdout, expected.stderr));
    }
}

#[test]
fn revision_restore_refusals_print_tools_words_and_exit_2() {
    for (revision, refusal, text, record) in [
        (
            1_u32,
            c::Refusal::Forgotten { oldest: 3 },
            "temper ctl: revision 1 is forgotten; the oldest kept is revision 3\n",
            r#"{"forgotten":{"oldest":3}}"#,
        ),
        (
            4,
            c::Refusal::Malformed { at: 0, why: super::malformed_setting(c::Setting::Rules, c::SettingFault::Count) },
            "revision restore: too large; revision 4's settings pass a proposal's bound; restore a later revision\n",
            r#"{"malformed":{"at":0,"why":{"settings":{"setting":{"kind":"rules"},"fault":"count"}}}}"#,
        ),
        (
            4,
            c::Refusal::Malformed { at: 0, why: c::Malformed::SettingsUnchanged },
            "revision restore: nothing to restore; the settings are as revision 4 left them\n",
            r#"{"malformed":{"at":0,"why":"settings_unchanged"}}"#,
        ),
        (
            8,
            c::Refusal::Stale(c::RefusalStale { newest: 7 }),
            "revision restore: no revision 8; the newest is revision 7\n",
            r#"{"stale":{"newest":7}}"#,
        ),
        (
            0,
            c::Refusal::Stale(c::RefusalStale { newest: 7 }),
            "revision restore: no revision 0; the newest is revision 7\n",
            r#"{"stale":{"newest":7}}"#,
        ),
        (
            4,
            c::Refusal::Stale(c::RefusalStale { newest: 7 }),
            "revision restore: stale; newest revision 7; read settings show again\n",
            r#"{"stale":{"newest":7}}"#,
        ),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(PARTY);
            let argv = ["revision", "restore", &revision.to_string(), "--key", "restore-4"];
            let number = root.submit_prepared(f::call(&argv), json, None);
            root.answered(number, c::Answer::Refused { refusal: refusal.clone(), at: None });
            let printed = root.answer(number).unwrap();
            assert_eq!(printed.status, 2);
            assert!(printed.stdout.is_empty());
            let expected = if json {
                format!(
                    r#"{{"operation":"restore","outcome":{{"refused":{{"refusal":{record},"at":null}}}},"task":null,"to":null}}"#
                )
            } else {
                text.into()
            };
            assert_eq!(printed.stderr.as_ref(), expected.as_bytes());
        }
    }
}

#[test]
fn a_restore_keeps_its_revision_and_party_for_recovery_and_pending() {
    let mut root = f::signed_in(PARTY);
    let number = root.submit(ARGV);
    let (_, _, request) = root.sends.get(&number).unwrap();
    let mut records = List::with_capacity(1);
    records
        .push(skein_lib::RequestRecord {
            key: f::named("restore-4"),
            scope: PARTY,
            first_sent: None,
            request: jig_client::Held {
                key: jig_client::Key {
                    asker: crate::root::RUNNER.asker(),
                    number: (u64::from(f::RESTORED_PREFIX) << 32_u32) | 1,
                },
                request: request.clone(),
            },
        })
        .unwrap();
    let pending = crate::jig::render::pending::pending(records.as_slice(), 0, false);
    assert_eq!(pending.stdout.as_ref(), b"restore, party 1, not sent, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n");
    let mut recovered = f::started_with(records, f::SEED, false);
    recovered.deliver(c::Delivery::Me(Box::new(f::me(PARTY))));
    let (_, _, resent) = recovered.sends.get(&0).unwrap();
    assert_eq!(resent, request);
    recovered.done(0, restored(omitted()));
    assert_eq!(recovered.kept_lines.as_slice(), &[Box::<[u8]>::from(b"kept: restore, proposed restoring revision 4 as proposal p1/7 to role 0\nleft out: landing-rule 2 --project 3\nleft out: project-policy 3\n".as_slice())]);
}
