use super::fixtures as f;
use jig_client::{self as client, contract as c};
use temper_client_protocol_wire::v1 as w;

#[test]
fn ready_sends_one_keyed_home_request_and_prints_the_answered_row() {
    let (mut root, call) = f::draft_root("ready", &[]);
    let request = root.sent_draft(call);
    assert_eq!(
        (request.project, request.repository, request.path.as_ref()),
        (f::PROJECT, None, f::DRAFT_PATH.as_bytes())
    );
    let w::DraftChange::Ready(ready) = request.change else { panic!("ready") };
    assert_eq!(*ready.chat(), None);
    assert_eq!(root.sends.len(), 1);
    assert!(root.sent_name(call).is_some());
    root.adoption_answer(call, w::Outcome::Drafted(f::landed_draft(true, None, None, 0)));
    let printed = root.answer(call).expect("durable draft");
    assert_eq!(printed.status, 0);
    f::check_golden("draft_ready", &printed.stdout);
}
#[test]
fn repository_and_chat_reach_ready_and_not_ready_uses_the_answered_identity() {
    let (mut root, call) = f::draft_root("ready", &["--owner", "o", "--name", "n", "--chat", "7"]);
    let request = root.sent_draft(call);
    let repository = request.repository.expect("named repository");
    assert_eq!((repository.owner(), repository.name()), (b"o".as_slice(), b"n".as_slice()));
    let w::DraftChange::Ready(ready) = request.change else { panic!("ready") };
    assert_eq!(*ready.chat(), Some(7));
    root.adoption_answer(call, w::Outcome::Drafted(f::landed_draft(true, Some(7), None, 0)));
    assert_eq!(root.answer(call).expect("ready").status, 0);
    let (mut root, call) = f::draft_root("not-ready", &["--owner", "o", "--name", "n"]);
    assert_eq!(root.sent_draft(call).change, w::DraftChange::NotReady);
    root.adoption_answer(call, w::Outcome::Drafted(f::landed_draft(false, None, None, 0)));
    let printed = root.answer(call).expect("not ready");
    assert_eq!(
        (printed.status, printed.stdout.as_ref()),
        (0, b"not ready: acme/shop docs/design/draft/catch-up.md\n".as_slice())
    );
}
#[test]
fn incomplete_repository_pairs_are_usage_errors_and_send_nothing() {
    for verb in ["ready", "not-ready"] {
        for extra in [&["--owner", "o"][..], &["--name", "n"]] {
            let (mut root, call) = f::draft_root(verb, extra);
            let printed = root.answer(call).expect("local usage failure");
            assert_eq!((printed.status, printed.stderr.as_ref()), (1, b"--owner and --name go together\n".as_slice()));
            assert!(root.nothing_sent());
        }
    }
}
#[test]
fn every_draft_refusal_says_what_to_do_and_exits_two() {
    for (reason, expected) in [
        (w::DraftRefusal::NoRepository, "shop has no home repository: name one with --owner and --name\n"),
        (w::DraftRefusal::Path, "docs/design/draft/catch-up.md is no draft's path: a relative path ending .md\n"),
        (
            w::DraftRefusal::Missing,
            "no docs/design/draft/catch-up.md at the head of the landing branch: land it first\n",
        ),
        (w::DraftRefusal::NotAFile, "docs/design/draft/catch-up.md is a directory\n"),
        (w::DraftRefusal::Unreachable, "the forge did not answer: try again\n"),
        (w::DraftRefusal::Unknown, "no landed draft at docs/design/draft/catch-up.md\n"),
        (w::DraftRefusal::Full, "shop keeps 64 landed drafts, every one listed: say one not ready first\n"),
    ] {
        let (mut root, call) = f::draft_root("ready", &[]);
        root.adoption_answer(
            call,
            w::Outcome::DraftRefused(
                w::DraftRefused::new(&w::CEILINGS, w::DraftRefusedParts { reason }).expect("refusal"),
            ),
        );
        let printed = root.answer(call).expect("draft refused");
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, expected.as_bytes()));
    }
    let (mut root, call) = f::draft_root("ready", &["--owner", "o", "--name", "n"]);
    root.adoption_answer(
        call,
        w::Outcome::DraftRefused(
            w::DraftRefused::new(&w::CEILINGS, w::DraftRefusedParts { reason: w::DraftRefusal::NoRepository })
                .expect("refusal"),
        ),
    );
    assert_eq!(root.answer(call).expect("no repository").stderr.as_ref(), b"no repository o/n in shop\n");
}
#[test]
fn draft_admission_refusals_use_jigs_words() {
    for core in [false, true] {
        let (mut root, call) = f::draft_root("ready", &[]);
        if core {
            root.adoption_answer(call, f::core_key_conflict());
        } else {
            root.answered(call, c::Answer::Refused { refusal: c::Refusal::KeyConflict, at: None });
        }
        let printed = root.answer(call).expect("admission refused");
        assert_eq!(printed.status, 2);
        assert!(
            skein_lib::bytes::find(&printed.stderr, b"The request key names another act").is_some(),
            "{:?}",
            printed.stderr
        );
    }
}
#[test]
fn list_watches_the_project_and_prints_all_three_states_then_closes() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["draft", "list", "--project", "shop"]);
    assert!(root.answer(call).is_none());
    let rows = [
        f::landed_draft(true, Some(7), None, 180_000_000_000),
        f::landed_draft(false, None, None, 120_000_000_000),
        f::landed_draft(true, None, Some(40), 60_000_000_000),
    ];
    root.deliver_drafts(&f::drafts_panel(&rows));
    let printed = root.answer(call).expect("drafts panel");
    assert_eq!(printed.status, 0);
    f::check_golden("draft_list", &printed.stdout);
    assert_eq!(root.watches.len(), 1, "only me stays open");
    assert_eq!(root.closed.len(), 1);
}
#[test]
fn no_drafts_prints_the_empty_words() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["draft", "list", "--project", "shop"]);
    root.deliver_drafts(&f::drafts_panel(&[]));
    let printed = root.answer(call).expect("empty panel");
    assert_eq!((printed.status, printed.stdout.as_ref()), (0, b"No landed drafts.\n".as_slice()));
}
#[test]
fn json_preserves_the_whole_wire_row_for_both_acts_and_list() {
    let expected = br#"{"forge":1,"owner":"acme","name":"shop","path":"docs/design/draft/catch-up.md","branch":"main","commit":"0123456789abcdef0123456789abcdef01234567","title":"Catch-up barrier","writer":73,"at":0,"chat":7,"goal":null,"ready":true}"#;
    for verb in ["ready", "not-ready"] {
        let (mut root, call) = f::draft_root(verb, &["--json"]);
        root.adoption_answer(call, w::Outcome::Drafted(f::landed_draft(true, Some(7), None, 0)));
        let printed = root.answer(call).expect("JSON row");
        assert_eq!((printed.status, printed.stdout.as_ref()), (0, expected.as_slice()));
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["draft", "list", "--project", "shop", "--json"]);
    root.deliver_drafts(&f::drafts_panel(&[f::landed_draft(true, Some(7), None, 0)]));
    let printed = root.answer(call).expect("JSON rows");
    assert_eq!(printed.status, 0);
    let mut array = skein_lib::Writer::new(expected.len().checked_add(2).expect("array bytes"));
    array.put(b"[").expect("array open");
    array.put(expected).expect("row");
    array.put(b"]").expect("array close");
    assert_eq!(printed.stdout, array.finish());
}
#[test]
fn until_changed_ignores_the_first_and_identical_panels() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["draft", "list", "--project", "shop", "--until", "changed"]);
    let first = f::drafts_panel(&[f::landed_draft(true, None, None, 0)]);
    root.deliver_drafts(&first);
    assert!(root.answer(call).is_none());
    root.deliver_drafts(&first);
    assert!(root.answer(call).is_none());
    root.deliver_drafts(&f::drafts_panel(&[f::landed_draft(false, None, None, 0)]));
    let printed = root.answer(call).expect("changed");
    assert_eq!(printed.status, 0);
    assert!(skein_lib::bytes::find(&printed.stdout, b"not ready").is_some());
    assert_eq!(root.watches.len(), 1);
}
#[test]
fn changed_wait_times_out_and_closes_even_without_a_panel() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["draft", "list", "--project", "shop", "--until", "changed", "--timeout", "1s"]);
    root.now = skein_lib::Time::from_nanos(1_000_000_000);
    let env = root.env(f::limits_root());
    let mut out = skein_lib::Queue::uncapped();
    crate::root::fire(&mut root.root, &env, &mut out);
    root.outputs(out);
    let printed = root.answer(call).expect("timeout");
    assert_eq!((printed.status, printed.stdout.as_ref()), (3, b"not met: timeout\n".as_slice()));
    // jig closes an unanswered opening watch once its late answer arrives.
    root.deliver_drafts(&f::drafts_panel(&[]));
    assert_eq!(root.watches.len(), 1);
    assert!(root.answer(call).is_none());
}
#[test]
fn draft_waits_for_live_me_and_restores_a_kept_request() {
    let mut root = f::started(skein_lib::List::with_capacity(0));
    let call = root.submit(&["draft", "ready", "--project", "shop", f::DRAFT_PATH, "--key", "draft"]);
    assert!(root.nothing_sent());
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert_eq!(root.sent_draft(call).project, f::PROJECT);
    let mut kept = skein_lib::List::with_capacity(1);
    kept.push(skein_lib::RequestRecord {
        key: root.sent_name(call).expect("key"),
        scope: f::PARTY,
        first_sent: None,
        request: client::Held {
            key: client::Key { asker: f::forge_asker(), number: 1 },
            request: root.adoption_bytes(call).clone(),
        },
    })
    .expect("one restore");
    let mut restored = f::started(kept);
    restored.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    restored.adoption_answer(0, w::Outcome::Drafted(f::landed_draft(true, None, None, 0)));
    assert_eq!(restored.kept_lines.as_slice(), &[f::golden("draft_ready")]);
}
