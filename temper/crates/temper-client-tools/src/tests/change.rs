mod history;
use super::fixtures as f;
use crate::{
    change,
    jig::{self, call::Parsed, digest::Digest},
};
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn panel(step: w::Step) -> w::ChangePanel {
    let used = w::Used::new(&w::CEILINGS, w::UsedParts { used: 0, limit: 3 }).unwrap();
    w::ChangePanel::new(
        &w::CEILINGS,
        w::ChangePanelParts {
            way: w::Way::Queue,
            in_landing: None,
            task: 2,
            forge: 3,
            owner: b"qa".as_slice().into(),
            name: b"pilot-20261026".as_slice().into(),
            branch: b"temper/2-readme".as_slice().into(),
            landing: b"main".as_slice().into(),
            head: Some([17; 32]),
            pull: Some(
                w::Pull::new(
                    &w::CEILINGS,
                    w::PullParts {
                        number: 1,
                        address: b"http://127.0.0.1:3000/qa/pilot-20261026/pulls/1".as_slice().into(),
                    },
                )
                .unwrap(),
            ),
            step,
            hold: None,
            runs: None,
            ci_required: true,
            ci: w::Ci::Pending,
            approvals: List::with_capacity(0),
            unmapped_reviews: List::with_capacity(0),
            gates: List::with_capacity(0),
            repairs: used.clone(),
            resolutions: used,
            updates: w::Used::new(&w::CEILINGS, w::UsedParts { used: 0, limit: 5 }).unwrap(),
            questions: w::Questions::new(
                &w::CEILINGS,
                w::QuestionsParts {
                    route: w::Route::None,
                    taken: w::Used::new(&w::CEILINGS, w::UsedParts { used: 0, limit: 4 }).expect("question limit"),
                    open: List::with_capacity(0),
                },
            )
            .expect("no questions"),
        },
    )
    .unwrap()
}
fn landed() -> w::Step {
    w::Step::Landed(w::Landed::new(&w::CEILINGS, w::LandedParts { merge: [34; 32] }).unwrap())
}
fn deliver(root: &mut f::Harness, panel: w::ChangePanel) {
    let mut own_read = None;
    for (&call, (_, read)) in &root.reads {
        if *read == (c::Read::Task { task: 2 }) {
            own_read = Some(call);
            break;
        }
    }
    if let Some(call) = own_read {
        let serving = root.serving;
        serve(root, call, false, false);
        root.serving = serving;
    }
    deliver_panel(root, panel);
}
fn deliver_panel(root: &mut f::Harness, panel: w::ChangePanel) {
    let subject = change::subject(2, 1, &root.limits);
    let token = root.watch_token(&subject);
    root.client(client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch: token,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::change_panel(&panel),
            },
        },
    });
}
fn task(ended: bool, change_result: bool) -> c::Task {
    let mut task = f::goal(2, c::WaitsOn::Nothing);
    task.chip.executor = c::ChipExecutor::Procedure { connector: wire::FORGE, kind: 2 };
    task.result_contract = c::TaskContractDocument::Change { connector: wire::FORGE, kind: 1, words: f::LONG_WORDS };
    if ended {
        task.chip.phase = c::Phase::Ended;
        task.result = Some(c::TaskResult::Document(Box::new(c::Result {
            task: task.chip.clone(),
            ending: c::Ending::Done,
            value: if change_result {
                Some(c::ResultValue::Change {
                    kind: 1,
                    part: c::Part { connector: wire::FORGE, kind: c::PartKind::Resource, key: c::PartKey::Number(1) },
                    words: f::document_text(b"2222222222222222222222222222222222222222"),
                })
            } else {
                None
            },
            at: None,
            details: None,
            acts: List::with_capacity(0),
        })));
    }
    task
}
fn serve(root: &mut f::Harness, call: u32, ended: bool, change_result: bool) {
    root.serve(call, c::ReadResult::Value(c::ReadValue::Task(Box::new(task(ended, change_result)))));
}
fn contains(bytes: &[u8], part: &[u8]) {
    assert!(skein_lib::bytes::find(bytes, part).is_some(), "{bytes:?} missing {part:?}");
}
fn end_panel(root: &mut f::Harness, call: u32, end: c::WatchEnd) {
    let subject = change::subject(2, 1, &root.limits);
    root.end(call, &subject, end);
}
const EMPTY_SEARCH: &[u8] = b"No change: widget already shows an empty search state when no results match.";
fn no_change_task() -> c::Task {
    let mut task = task(true, false);
    let Some(c::TaskResult::Document(result)) = &mut task.result else { panic!("terminal result") };
    result.value = Some(c::ResultValue::Report(f::document_text(EMPTY_SEARCH)));
    task
}
fn no_change_result() -> c::ReadResult {
    let mut task = no_change_task();
    let Some(c::TaskResult::Document(result)) = task.result.take() else { panic!("terminal result") };
    c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Task(2),
        value: c::EndedValue::Task { document: Box::new(task), result },
        at: None,
        by: None,
    })))
}
fn no_change_panel(step: w::Step) -> w::ChangePanel {
    let mut parts = panel(step).into_parts();
    parts.head = None;
    parts.pull = None;
    parts.ci = w::Ci::Unknown;
    w::ChangePanel::new(&w::CEILINGS, parts).unwrap()
}
#[test]
fn a_no_change_completion_names_its_result_and_report() {
    for initial in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--until", "landed"]);
        root.serve(call, if initial { no_change_result() } else { f::task_result(task(false, false)) });
        if initial {
            deliver_panel(&mut root, no_change_panel(w::Step::Ended));
        } else {
            deliver_panel(&mut root, no_change_panel(w::Step::Producing));
            assert!(root.answer(call).is_none());
            root.pending_reads.push(call);
            root.deliver_subject(
                &client::Watchable::Jig(c::Subject::Task { task: 2 }),
                c::Delivery::Task(Box::new(no_change_task())),
            );
            deliver_panel(&mut root, no_change_panel(w::Step::Ended));
            assert!(root.answer(call).is_none(), "the final task read supplies completion details");
            root.serve(call, f::task_result(no_change_task()));
        }
        let answer = root.answer(call).expect("completion ends an unmet landing wait");
        assert_eq!(answer.status, 3);
        contains(&answer.stdout, b"not met: ended\n");
        contains(&answer.stdout, b"result: completed, no change\n");
        contains(&answer.stdout, b"next: temper ctl DIR task show 2\n");
        let call = root.submit(&["change", "show", "2"]);
        root.serve(call, f::task_result(no_change_task()));
        deliver_panel(&mut root, no_change_panel(w::Step::Ended));
        let answer = root.answer(call).expect("retained panel read");
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, b"step: ended\n");
        contains(&answer.stdout, b"result: completed, no change\n");
        contains(&answer.stdout, b"next: temper ctl DIR task show 2\n");
        let call = root.submit(&["change", "show", "2", "--json"]);
        root.serve(call, no_change_result());
        deliver_panel(&mut root, no_change_panel(w::Step::Ended));
        let answer = root.answer(call).expect("completion in JSON");
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, br#""result":"completed, no change""#);
        let call = root.submit(&["task", "show", "2"]);
        root.serve(call, no_change_result());
        let answer = root.answer(call).expect("direct retained report");
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, EMPTY_SEARCH);
    }
}
#[test]
fn a_final_task_read_judges_the_panel_delivered_during_it() {
    for (step, status) in [(landed(), 0), (w::Step::Checking, 3)] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--until", "landed"]);
        serve(&mut root, call, true, true);
        root.pending_reads.push(call);
        root.deliver_subject(
            &client::Watchable::Jig(c::Subject::Task { task: 2 }),
            c::Delivery::Task(Box::new(task(true, true))),
        );
        deliver_panel(&mut root, panel(step));
        assert!(root.answer(call).is_none(), "the final read remains pending");
        serve(&mut root, call, true, true);
        let answer = root.answer(call).expect("the final panel decides the condition");
        assert_eq!(answer.status, status);
        contains(&answer.stdout, if status == 0 { b"met\n" } else { b"not met: ended\n" });
    }
}
#[test]
fn change_calls_round_trip_and_reject_other_conditions() {
    for argv in [
        &["change", "show", "2"][..],
        &["change", "show", "2", "--project", "shop"],
        &[
            "change",
            "show",
            "2",
            "--project",
            "1",
            "--until",
            "landed",
            "--since",
            "0123456789abcdef",
            "--timeout",
            "1s",
        ],
        &["change", "show", "2", "--until", "ci:failed"],
    ] {
        let call = f::call(argv);
        assert_eq!(f::printed(&call), Parsed::Call { call: call.clone(), json: false });
        assert_eq!(f::from_json(&jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
    }
    for argv in [
        &["change", "show", "0"][..],
        &["change", "show", "2", "--until", "ended"],
        &["change", "show", "2", "--until", "ci:unknown"],
        &["change", "show", "2", "--key", "no"],
    ] {
        f::parse(argv).unwrap_err();
    }
}
#[test]
fn checking_text_and_json_name_pull_head_gates_bounds_and_same_digest() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1"]);
    deliver(&mut root, panel(w::Step::Checking));
    let text = root.answer(call).unwrap();
    assert_eq!(text.status, 0);
    contains(&text.stdout, b"change 2 \xc2\xb7 qa/pilot-20261026 on forge 3 \xc2\xb7 temper/2-readme into main\n");
    contains(&text.stdout, b"pull request #1 \xc2\xb7 http://127.0.0.1:3000/qa/pilot-20261026/pulls/1\nstep: checking\nhead 1111111111111111111111111111111111111111 \xc2\xb7 CI pending\ngates: none\n");
    contains(&text.stdout, b"repairs 0 of 3 \xc2\xb7 resolutions 0 of 3 \xc2\xb7 updates 0 of 5\n");
    contains(&text.stdout, b"next: temper ctl DIR change show 2 --until landed\n");
    let call = root.submit(&["change", "show", "2", "--project", "1", "--json"]);
    deliver(&mut root, panel(w::Step::Checking));
    let json = root.answer(call).unwrap();
    let expected = br#"{"task":2,"forge":3,"owner":"qa","name":"pilot-20261026","branch":"temper/2-readme","landing":"main","head":"1111111111111111111111111111111111111111","pull":{"number":1,"address":"http://127.0.0.1:3000/qa/pilot-20261026/pulls/1"},"step":"checking","merge":null,"queue":null,"repair":null,"hold":null,"runs":null,"ci_required":true,"ci":"pending","gates":[],"approvals":[],"unmapped_reviews":[],"repairs":{"used":0,"limit":3},"resolutions":{"used":0,"limit":3},"updates":{"used":0,"limit":5},"questions":{"route":"none","taken":0,"limit":4,"open":[]}"#;
    assert!(json.stdout.starts_with(expected));
    let mut document = skein_lib::Writer::new(expected.len() + 1);
    document.put(expected).unwrap();
    document.put(b"}").unwrap();
    let digest = Digest::of(&document.finish()).hex();
    contains(&json.stdout, &digest);
    contains(&text.stdout, &digest);
    assert_eq!(root.watches.len(), 1, "only me remains watched");
}
#[test]
fn landed_and_held_repair_panels_keep_whole_facts() {
    for json in [false, true] {
        let limits = f::limits_root();
        let answer = change::print(Some(&panel(landed())), json, None, b"", &limits, &change::Holds::new());
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, b"2222222222222222222222222222222222222222");
        assert!(skein_lib::bytes::find(&answer.stdout, b"next:").is_none());
        let mut parts = panel(w::Step::Repairing(
            w::Repairing::new(&w::CEILINGS, w::RepairingParts { reason: w::Reason::Ci }).unwrap(),
        ))
        .into_parts();
        parts.hold = Some(w::Hold::Stalled);
        parts.runs = Some(7);
        parts.gates = List::with_capacity(1);
        parts
            .gates
            .push(
                w::Gate::new(
                    &w::CEILINGS,
                    w::GateParts { number: 1, kind: w::GateKind::Review, blocks: true, verdict: w::Verdict::Pending },
                )
                .unwrap(),
            )
            .unwrap();
        let answer = change::print(
            Some(&w::ChangePanel::new(&w::CEILINGS, parts).unwrap()),
            json,
            None,
            b"",
            &limits,
            &change::Holds::new(),
        );
        if json {
            contains(&answer.stdout, br#""repair":"ci","hold":"stalled","runs":7"#);
            contains(&answer.stdout, br#""gates":[{"number":1,"kind":"review","blocks":true,"verdict":"pending"}]"#);
        } else {
            contains(&answer.stdout, b"held: its wait stalled at repairing\nrunning: task 7\n");
            contains(&answer.stdout, b"repair: Repairing: CI failed at its head.");
        }
    }
}
#[test]
fn absent_project_reads_the_task_project_before_opening_the_panel() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2"]);
    assert_eq!(root.reads[&call].1, c::Read::Task { task: 2 });
    assert_eq!(root.watches.len(), 1);
    serve(&mut root, call, false, false);
    deliver(&mut root, panel(w::Step::Checking));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn waits_test_each_delivery_including_the_first() {
    for (condition, ci) in [
        ("pull", w::Ci::Pending),
        ("ci:passed", w::Ci::Passed),
        ("ci:pending", w::Ci::Pending),
        ("ci:failed", w::Ci::Failed),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1", "--until", condition, "--json"]);
        let mut parts = panel(w::Step::Checking).into_parts();
        parts.ci = ci;
        deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, br#""met":true"#);
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
    deliver(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none());
    deliver(&mut root, panel(landed()));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn a_wait_reopens_the_last_panel_after_the_watch_ends_at_landing() {
    for end in [c::WatchEnd::Unknown, c::WatchEnd::SubjectEnded] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed", "--json"]);
        deliver(&mut root, panel(w::Step::Checking));
        end_panel(&mut root, call, end);
        assert!(root.answer(call).is_none());
        serve(&mut root, call, true, true);
        deliver(&mut root, panel(landed()));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, br#""step":"landed"#);
    }
}
#[test]
fn live_task_waits_for_the_next_document_then_reopens_the_panel() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
    end_panel(&mut root, call, c::WatchEnd::Unknown);
    serve(&mut root, call, false, false);
    let subject = client::Watchable::Jig(c::Subject::Task { task: 2 });
    let token = root.watch_token(&subject);
    root.client(client::Event::Up {
        up: client::Up::Watched {
            watch: token,
            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task(false, false)))),
        },
    });
    assert_eq!(root.watches.len(), 2, "me and task; no premature panel retry");
    let mut next = task(false, false);
    next.activity += 1;
    root.client(client::Event::Up {
        up: client::Up::Watched { watch: token, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(next))) },
    });
    deliver(&mut root, panel(landed()));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn an_unmet_final_panel_or_a_task_without_a_change_keeps_its_last_panel() {
    for change_result in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "ci:passed", "--json"]);
        deliver(&mut root, panel(w::Step::Checking));
        end_panel(&mut root, call, c::WatchEnd::SubjectEnded);
        serve(&mut root, call, true, change_result);
        if change_result {
            deliver(&mut root, panel(landed()));
        }
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 3);
        if change_result {
            contains(&answer.stdout, br#""met":false,"why":"ended"#);
        } else {
            contains(&answer.stdout, br#""step":"checking"#);
        }
    }
}
#[test]
fn since_and_condition_both_must_hold_and_timeout_keeps_last_digest() {
    let mut root = f::signed_in(f::PARTY);
    let answer = change::print(Some(&panel(w::Step::Checking)), true, None, b"", &root.limits, &change::Holds::new());
    let marker = b"\"digest\":\"";
    let start = skein_lib::bytes::find(&answer.stdout, marker).unwrap() + marker.len();
    let digest = &answer.stdout[start..start + 16];
    let mut call = f::call(&[
        "change",
        "show",
        "2",
        "--project",
        "1",
        "--since",
        "0000000000000000",
        "--until",
        "pull",
        "--timeout",
        "1s",
    ]);
    *call.values.get_mut(2).unwrap() = Some(jig::call::Values::One(
        jig::argv::value::decode(crate::table::TOOLS, jig::table::Kind::Digest, jig::table::Bound::None, digest)
            .unwrap(),
    ));
    let call = root.submit_prepared(call, true, None);
    deliver(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none());
    root.now = root.now.checked_add(Duration::from_secs(1)).unwrap();
    root.fire();
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 3);
    contains(&answer.stdout, digest);
    contains(&answer.stdout, b"timeout");
}

#[test]
fn a_held_wait_and_a_wait_without_a_condition_use_the_panel_facts() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "held"]);
    let mut parts = panel(w::Step::Checking).into_parts();
    parts.hold = Some(w::Hold::Stalled);
    deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
    assert_eq!(root.answer(call).unwrap().status, 0);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "changed"]);
    deliver(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none(), "first delivery only establishes the digest");
    deliver(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none(), "repeated digest is no change");
    deliver(&mut root, panel(w::Step::Gating));
    assert_eq!(root.answer(call).unwrap().status, 0);
}

fn cancelled() -> c::Task {
    let mut task = task(true, false);
    task.chip.phase = c::Phase::Cancelled;
    task
}
#[test]
fn a_change_wait_follows_cancelled_and_failed_tasks_beside_the_live_panel() {
    for (phase, why) in [(c::Phase::Cancelled, b"ended: cancelled".as_slice()), (c::Phase::Failed, b"ended: failed")] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let words = f::extended(
                &["change", "show", "2", "--project", "1", "--until", "landed"],
                if json { &["--json"] } else { &[] },
            );
            let call = root.submit(words.as_slice());
            deliver(&mut root, panel(w::Step::Producing));
            assert!(root.answer(call).is_none());
            let subject = client::Watchable::Jig(c::Subject::Task { task: 2 });
            let token = root.watch_token(&subject);
            let mut task = cancelled();
            task.chip.phase = phase;
            root.pending_reads.push(call);
            root.client(client::Event::Up {
                up: client::Up::Watched {
                    watch: token,
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
                },
            });
            root.serve(call, c::ReadResult::Value(c::ReadValue::Task(Box::new(task))));
            let answer = root.answer(call).expect("task end answers without a panel end or timeout");
            assert_eq!(answer.status, 3);
            contains(&answer.stdout, why);
            contains(&answer.stdout, b"producing");
            contains(&answer.stdout, b"digest");
            assert_eq!(root.watches.len(), 1);
        }
    }
}
#[test]
fn a_cancelled_task_with_no_panel_exits_two() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
    end_panel(&mut root, call, c::WatchEnd::Unknown);
    root.serve(call, c::ReadResult::Value(c::ReadValue::Task(Box::new(cancelled()))));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 2);
    contains(&answer.stderr, b"no change panel for task 2");
    assert!(root.family_reads.is_empty(), "a wait never reads history");
}

#[test]
fn changed_with_since_waits_for_a_panel_change_after_the_baseline() {
    let mut root = f::signed_in(f::PARTY);
    let read = root.submit(&["change", "show", "2", "--project", "1", "--json"]);
    deliver(&mut root, panel(w::Step::Gating));
    let digest = f::last_digest(&root.answer(read).unwrap().stdout);
    root.pending_reads.push(root.next + 1);
    let call = root.wait_since(&["change", "show", "2", "--project", "1"], &["--until", "changed"], &digest);
    for step in [w::Step::Checking, w::Step::Gating] {
        deliver(&mut root, panel(step));
        assert!(root.answer(call).is_none());
    }
    deliver(&mut root, panel(w::Step::Checking));
    assert_eq!(root.answer(call).unwrap().status, 0);
}

#[test]
fn a_change_held_before_registration_names_the_hold_instead_of_a_missing_panel() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(&["change", "show", "2", "--project", "1"]), json, None);
        end_panel(&mut root, call, c::WatchEnd::Unknown);
        root.serve(call, f::task_result(f::held_task(task(false, false), c::HoldCategory::Effects)));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        contains(&answer.stderr, b"held before registration: Effects need a decision");
    }
}

#[test]
fn a_panels_held_reviewer_says_held_with_its_reason() {
    for waiting in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let argv = if waiting {
            &["change", "show", "2", "--project", "1", "--until", "landed", "--timeout", "1s"][..]
        } else {
            &["change", "show", "2", "--project", "1"][..]
        };
        let call = root.submit(argv);
        let mut parts = panel(w::Step::Gating).into_parts();
        parts.runs = Some(8);
        root.serving = Some(call);
        deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
        root.serving = None;
        assert!(root.answer(call).is_none(), "read the reviewer's state before naming it");
        root.serve(call, f::task_result(f::held_task(f::goal(8, c::WaitsOn::Nothing), c::HoldCategory::Budget)));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, if waiting { 3 } else { 0 });
        contains(&answer.stdout, b"held: task 8 (Insufficient activation budget)");
        assert!(skein_lib::bytes::find(&answer.stdout, b"running: task 8").is_none());
    }
}

#[test]
fn a_landing_wait_observes_a_reviewer_held_after_its_first_read() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed", "--timeout", "1s"]);
    let mut parts = panel(w::Step::Gating).into_parts();
    parts.runs = Some(8);
    root.serving = Some(call);
    deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
    root.serving = None;
    root.serve(call, f::task_result(f::goal(8, c::WaitsOn::Nothing)));
    root.deliver_subject(
        &client::Watchable::Jig(c::Subject::Task { task: 8 }),
        c::Delivery::Task(Box::new(f::held_task(f::goal(8, c::WaitsOn::Nothing), c::HoldCategory::Budget))),
    );
    let answer = root.answer(call).expect("a held reviewer ends the landing wait without a timeout");
    assert_eq!(answer.status, 3);
    contains(&answer.stdout, b"held: task 8 (Insufficient activation budget)");
    assert!(skein_lib::bytes::find(&answer.stdout, b"running: task 8").is_none());
}

#[test]
fn a_refused_reviewer_watch_ends_the_landing_wait() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed", "--timeout", "1s"]);
    let mut parts = panel(w::Step::Gating).into_parts();
    parts.runs = Some(8);
    root.serving = Some(call);
    deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
    root.serving = None;
    root.serve(call, f::task_result(f::goal(8, c::WaitsOn::Nothing)));
    root.end(call, &client::Watchable::Jig(c::Subject::Task { task: 8 }), c::WatchEnd::NotAllowed);
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 2);
    contains(&answer.stderr, b"not_allowed");
}

#[test]
fn an_authority_hold_prints_its_precise_cause_in_text_and_json() {
    let reason = b"its 6 tasks are made; released without more tasks; cancel: task cancel 2 REASON; or ask afresh; bare release holds again";
    let mut value = panel(w::Step::Gating).into_parts();
    value.hold = Some(w::Hold::Authority(
        w::AuthorityHold::new(&w::CEILINGS, w::AuthorityHoldParts { reason: reason.as_slice().into() }).unwrap(),
    ));
    let panel = w::ChangePanel::new(&w::CEILINGS, value).unwrap();
    for json in [false, true] {
        let printed = change::print(Some(&panel), json, None, b"", &f::limits_root(), &change::Holds::new());
        assert_eq!(printed.status, 0);
        contains(&printed.stdout, reason);
    }
}

#[test]
fn the_change_tasks_own_hold_keeps_its_reason_and_way_out() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(&["change", "show", "2", "--project", "1"]), json, None);
        root.serve(call, f::task_result(f::held_task(task(false, false), c::HoldCategory::Deadline)));
        deliver(&mut root, panel(w::Step::Checking));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, b"Deadline passed");
        contains(&answer.stdout, b"amend the task's deadline, then task release 2 --hold 9");
        if json {
            contains(&answer.stdout, br#""task_hold":{"reason":"Deadline passed","way_out":"#);
        } else {
            contains(&answer.stdout, b"held: task 2 (Deadline passed)\nway out: ");
            contains(&answer.stdout, b"next: temper ctl DIR task show 2\n");
        }
    }
}

#[test]
fn a_landing_wait_ends_on_the_change_tasks_hold_without_a_panel_change() {
    for json in [false, true] {
        for initial in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(
                f::call(&["change", "show", "2", "--project", "1", "--until", "landed"]),
                json,
                None,
            );
            let held = f::held_task(task(false, false), c::HoldCategory::Deadline);
            if initial {
                root.serve(call, f::task_result(held.clone()));
                root.deliver_subject(
                    &client::Watchable::Jig(c::Subject::Task { task: 2 }),
                    c::Delivery::Task(Box::new(held.clone())),
                );
            }
            deliver(&mut root, panel(w::Step::Checking));
            if !initial {
                assert!(root.answer(call).is_none());
                root.deliver_subject(
                    &client::Watchable::Jig(c::Subject::Task { task: 2 }),
                    c::Delivery::Task(Box::new(held)),
                );
            }
            let answer = root.answer(call).expect("hold ends the wait at once");
            assert_eq!(answer.status, 3);
            contains(&answer.stdout, b"Deadline passed");
            contains(&answer.stdout, b"task release 2 --hold 9");
            contains(&answer.stdout, if json { br#""met":false,"why":"held"# } else { b"not met: held" });
            assert_eq!(root.watches.len(), 1, "all change watches close: {:?}", root.watches);
        }
    }
}

#[test]
fn an_early_panel_waits_for_the_change_tasks_hold_read() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1"]);
    deliver_panel(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none(), "the forge panel alone cannot attest no task hold");
    root.serve(call, f::task_result(f::held_task(task(false, false), c::HoldCategory::Deadline)));
    let answer = root.answer(call).unwrap();
    contains(&answer.stdout, b"held: task 2 (Deadline passed)");
    contains(&answer.stdout, b"task release 2 --hold 9");
}

#[test]
fn a_change_task_hold_is_shown_beside_its_held_reviewer() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(&["change", "show", "2"]), json, None);
        root.serve(call, f::task_result(f::held_task(task(false, false), c::HoldCategory::Deadline)));
        let mut parts = panel(w::Step::Gating).into_parts();
        parts.runs = Some(8);
        root.serving = Some(call);
        deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
        root.serving = None;
        root.serve(call, f::task_result(f::held_task(f::goal(8, c::WaitsOn::Nothing), c::HoldCategory::Budget)));
        let answer = root.answer(call).unwrap();
        contains(&answer.stdout, b"Deadline passed");
        contains(&answer.stdout, b"task release 2 --hold 9");
        contains(&answer.stdout, b"Insufficient activation budget");
        if json {
            contains(&answer.stdout, br#""run_hold":"Insufficient activation budget"#);
        } else {
            contains(&answer.stdout, b"held: task 8 (Insufficient activation budget)");
        }
    }
}

#[test]
fn a_change_task_hold_changes_the_digest_without_a_forge_panel_change() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "changed", "--json"]);
    deliver(&mut root, panel(w::Step::Checking));
    assert!(root.answer(call).is_none());
    root.deliver_subject(
        &client::Watchable::Jig(c::Subject::Task { task: 2 }),
        c::Delivery::Task(Box::new(f::held_task(task(false, false), c::HoldCategory::Deadline))),
    );
    let answer = root.answer(call).expect("the hold changes what change show prints and hashes");
    assert_eq!(answer.status, 0);
    contains(&answer.stdout, br#""met":true"#);
    contains(&answer.stdout, b"Deadline passed");
}

#[test]
fn a_panel_ending_during_a_delegate_read_still_reads_the_change_task() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
    let mut parts = panel(w::Step::Gating).into_parts();
    parts.runs = Some(8);
    root.serving = Some(call);
    deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
    root.serving = None;
    end_panel(&mut root, call, c::WatchEnd::SubjectEnded);
    // The delegate's old terminal may arrive before the change's final task read.
    root.serve(call, f::task_result(f::goal(8, c::WaitsOn::Nothing)));
    serve(&mut root, call, true, true);
    deliver(&mut root, panel(landed()));
    assert_eq!(root.answer(call).unwrap().status, 0);
}

#[test]
fn ready_and_preparing_text_names_the_way_and_landing() {
    let member = w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: Some(61) }).unwrap();
    for (name, step) in
        [("change_ready", w::Step::Ready(member.clone())), ("change_preparing", w::Step::Preparing(member))]
    {
        let mut parts = panel(step).into_parts();
        parts.way = w::Way::Turn;
        parts.in_landing = Some(61);
        let panel = w::ChangePanel::new(&w::CEILINGS, parts).unwrap();
        let answer = change::print(Some(&panel), false, None, b"", &f::limits_root(), &change::Holds::new());
        assert_eq!(answer.status, 0);
        f::check_golden(name, &answer.stdout);
    }
}
#[test]
fn a_queue_change_prints_no_way_line() {
    let answer =
        change::print(Some(&panel(w::Step::Checking)), false, None, b"", &f::limits_root(), &change::Holds::new());
    assert_eq!(answer.status, 0);
    assert!(skein_lib::bytes::find(&answer.stdout, b"way:").is_none());
}
#[test]
fn a_turn_change_waiting_for_room_prints_its_way_alone() {
    let mut parts =
        panel(w::Step::Ready(w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: None }).unwrap()))
            .into_parts();
    parts.way = w::Way::Turn;
    let panel = w::ChangePanel::new(&w::CEILINGS, parts).unwrap();
    let answer = change::print(Some(&panel), false, None, b"", &f::limits_root(), &change::Holds::new());
    contains(&answer.stdout, b"step: ready\nway: turn\nhead ");
}
#[test]
fn ready_and_preparing_wait_for_landed() {
    let member = w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: Some(61) }).unwrap();
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
    for step in [w::Step::Ready(member.clone()), w::Step::Preparing(member)] {
        let mut parts = panel(step).into_parts();
        parts.way = w::Way::Turn;
        parts.in_landing = Some(61);
        deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
        assert!(root.answer(call).is_none());
    }
    deliver(&mut root, panel(landed()));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
// The QA change's panel names a reviewer that remains held through restart.
const REVIEWER: u64 = 8;
// QA's 12 s condition timeout is shorter than the link's 30 s deadline.
const RESTART_TIMEOUT: Duration = Duration::from_secs(12);
fn reviewing(root: &mut f::Harness, call: u32) {
    let mut parts = panel(w::Step::Gating).into_parts();
    parts.runs = Some(REVIEWER);
    root.serving = Some(call);
    deliver(root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
    root.serving = None;
}
fn reviewing_call(argv: &[&str]) -> (f::Harness, u32) {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(argv);
    reviewing(&mut root, call);
    (root, call)
}
fn stop_panel(root: &mut f::Harness, call: u32) {
    let token = root.watch_token(&change::subject(2, 1, &root.limits));
    root.serving = Some(call);
    root.client(client::Event::Up { up: client::Up::WatchEnded { watch: token, end: c::WatchEnd::Stopping } });
    root.serving = None;
}
#[test]
fn a_change_wait_keeps_its_panel_when_a_restart_cuts_its_read() {
    let (mut root, call) =
        reviewing_call(&["change", "show", "2", "--project", "1", "--until", "ci:failed", "--timeout", "12s"]);
    stop_panel(&mut root, call);
    root.serve(call, c::ReadResult::None(c::ReadNone::Unreachable));
    let answer = match root.answer(call) {
        Some(answer) => Some((answer.status, answer.stderr)),
        None => None,
    };
    assert_eq!(answer, None);
    root.now = root.now.saturating_add(RESTART_TIMEOUT);
    root.serving = Some(call);
    root.fire();
    let answer = root.answer(call).expect("condition timeout answers");
    assert_eq!(answer.status, 3);
    contains(&answer.stdout, b"step: gating");
    contains(&answer.stdout, b"not met: timeout");
    assert!(answer.stderr.is_empty());
}
#[test]
fn a_stopping_panel_does_not_read_its_stale_reviewer() {
    let (mut root, call) = reviewing_call(&["change", "show", "2", "--project", "1", "--until", "ci:failed"]);
    root.serve(call, f::task_result(f::held_task(f::goal(REVIEWER, c::WaitsOn::Nothing), c::HoldCategory::Budget)));
    stop_panel(&mut root, call);
    assert!(root.reads.is_empty(), "a stopping watch is no new panel delivery");
}

#[test]
fn a_retried_reviewer_read_keeps_its_target_and_hold() {
    let (mut root, call) =
        reviewing_call(&["change", "show", "2", "--project", "1", "--until", "ci:failed", "--timeout", "12s"]);
    root.serve(call, c::ReadResult::None(c::ReadNone::Unreachable));
    root.now = root.now.saturating_add(root.limits.domain.client.domain.backoff.first);
    root.serving = Some(call);
    root.fire();
    root.serving = None;
    assert_eq!(root.reads[&call].1, c::Read::Task { task: REVIEWER });
    root.serve(call, f::task_result(f::held_task(f::goal(REVIEWER, c::WaitsOn::Nothing), c::HoldCategory::Budget)));
    root.now = root.now.saturating_add(RESTART_TIMEOUT);
    root.fire();
    let answer = root.answer(call).expect("condition timeout");
    assert_eq!(answer.status, 3);
    contains(&answer.stdout, b"held: task 8 (Insufficient activation budget)");
}
#[test]
fn reviewer_read_refusals_keep_their_ordinary_status_and_line() {
    for (why, status, line) in [
        (c::ReadNone::Unknown, 2, b"unknown\n".as_slice()),
        (c::ReadNone::Busy, 1, b"busy\n"),
        (c::ReadNone::SignedOut, 1, b"signed out\n"),
    ] {
        let (mut root, call) = reviewing_call(&["change", "show", "2", "--project", "1"]);
        root.serve(call, c::ReadResult::None(why));
        let answer = root.answer(call).expect("read refusal");
        assert_eq!(answer.status, status);
        assert_eq!(answer.stderr.as_ref(), line);
    }
}

fn after_step(which: u8) -> w::Step {
    let mut tiers = List::with_capacity(1);
    let line: &[u8] = match which {
        0 => b"fast run 2 running",
        1 => b"fast run 2 \xc2\xb7 owns 1 test: no-break",
        _ => b"fast run 3",
    };
    tiers.push(Box::from(line)).unwrap();
    match which {
        0 => w::Step::Verifying(w::Verifying::new(&w::CEILINGS, w::VerifyingParts { merge: [34; 32], tiers }).unwrap()),
        1 => w::Step::Red(w::Red::new(&w::CEILINGS, w::RedParts { merge: [34; 32], tiers }).unwrap()),
        _ => w::Step::Cleared(
            w::Cleared::new(&w::CEILINGS, w::ClearedParts { merge: [34; 32], tiers, uncleared: which == 3 }).unwrap(),
        ),
    }
}
#[test]
fn after_landing_text_and_json_keep_tier_lines_merge_and_following_call() {
    for (which, line) in [
        (0, b"step: verifying \xc2\xb7 fast run 2 running".as_slice()),
        (1, b"step: red \xc2\xb7 fast run 2 \xc2\xb7 owns 1 test: no-break".as_slice()),
        (2, b"step: cleared \xc2\xb7 fast run 3".as_slice()),
        (3, b"step: landed, uncleared\n".as_slice()),
    ] {
        let value = panel(after_step(which));
        let text = change::print(Some(&value), false, None, b"", &f::limits_root(), &change::Holds::new());
        assert_eq!(text.status, 0);
        contains(&text.stdout, line);
        contains(&text.stdout, b"2222222222222222222222222222222222222222");
        assert_eq!(skein_lib::bytes::find(&text.stdout, b"--until cleared").is_some(), which < 2);
        let json = change::print(Some(&value), true, None, b"", &f::limits_root(), &change::Holds::new());
        contains(&json.stdout, br#""tiers":["fast run "#);
        if which == 1 {
            contains(&json.stdout, b"owns 1 test: no-break");
        }
        contains(&json.stdout, br#""merge":"2222222222222222222222222222222222222222""#);
    }
}
#[test]
fn landed_is_met_by_every_later_step_and_cleared_waits_through_red() {
    for which in 0..4 {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "landed"]);
        deliver(&mut root, panel(after_step(which)));
        assert_eq!(root.answer(call).unwrap().status, 0);
    }
    for final_step in [2, 3] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1", "--until", "cleared"]);
        for pending in [0, 1] {
            deliver(&mut root, panel(after_step(pending)));
            assert!(root.answer(call).is_none());
        }
        deliver(&mut root, panel(after_step(final_step)));
        assert_eq!(root.answer(call).unwrap().status, 0);
    }
}

fn question_parts() -> w::ChangePanelParts {
    let mut parts = panel(w::Step::Checking).into_parts();
    let mut open = List::with_capacity(2);
    for (asker, number, line, stands) in
        [(7, 1, b"Which branch?".as_slice(), w::Stands::Asked), (8, 2, b"Which check?".as_slice(), w::Stands::Taken)]
    {
        open.push(
            w::Question::new(&w::CEILINGS, w::QuestionParts { asker, number, line: Box::from(line), stands }).unwrap(),
        )
        .unwrap();
    }
    parts.questions = w::Questions::new(
        &w::CEILINGS,
        w::QuestionsParts {
            route: w::Route::Up,
            taken: w::Used::new(&w::CEILINGS, w::UsedParts { used: 2, limit: 4 }).unwrap(),
            open,
        },
    )
    .unwrap();
    parts
}
#[test]
fn change_show_prints_its_questions() {
    let value = w::ChangePanel::new(&w::CEILINGS, question_parts()).unwrap();
    let text = change::print(Some(&value), false, None, b"", &f::limits_root(), &change::Holds::new());
    assert_eq!(text.status, 0);
    f::check_golden("change_questions", &text.stdout);
    let json = change::print(Some(&value), true, None, b"", &f::limits_root(), &change::Holds::new());
    contains(&json.stdout, br#""questions":{"route":"up","taken":2,"limit":4,"open":[{"asker":7,"number":1,"line":"Which branch?","stands":"asked"},{"asker":8,"number":2,"line":"Which check?","stands":"taken"}]}"#);
}
#[test]
fn change_show_names_asked_and_held_questions() {
    for (stands, text, json) in [
        (w::Stands::Asked, b"asked of requester".as_slice(), br#""stands":"asked""#.as_slice()),
        (w::Stands::Held, b"answered, held for dependencies", br#""stands":"held""#),
    ] {
        let mut parts = question_parts();
        let mut questions = parts.questions.into_parts();
        let mut open = List::with_capacity(1);
        open.push(
            w::Question::new(
                &w::CEILINGS,
                w::QuestionParts { asker: 7, number: 1, line: Box::from(b"Which branch?".as_slice()), stands },
            )
            .unwrap(),
        )
        .unwrap();
        questions.open = open;
        parts.questions = w::Questions::new(&w::CEILINGS, questions).unwrap();
        let value = w::ChangePanel::new(&w::CEILINGS, parts).unwrap();
        let answer = change::print(Some(&value), false, None, b"", &f::limits_root(), &change::Holds::new());
        contains(&answer.stdout, text);
        let answer = change::print(Some(&value), true, None, b"", &f::limits_root(), &change::Holds::new());
        contains(&answer.stdout, json);
        assert!(skein_lib::bytes::find(&answer.stdout, br#""foreman":"#).is_none());
    }
}

#[test]
fn ordinary_and_json_red_reads_keep_long_ids_and_many() {
    let prefix = "slow run 3 · owns 1 test: ".as_bytes();
    let mut long = skein_lib::Writer::new(prefix.len().checked_add(256).unwrap());
    long.put(prefix).unwrap();
    long.put(&[b'x'; 256]).unwrap();
    let long = long.finish();
    for line in [long.as_ref(), b"slow run 3 \xc2\xb7 suspected of many tests"] {
        let mut tiers = List::with_capacity(1);
        tiers.push(Box::from(line)).unwrap();
        let step = w::Step::Red(w::Red::new(&w::CEILINGS, w::RedParts { merge: [34; 32], tiers }).unwrap());
        let value = panel(step);
        for json in [false, true] {
            let printed = change::print(Some(&value), json, None, b"", &f::limits_root(), &change::Holds::new());
            assert_eq!(printed.status, 0);
            contains(&printed.stdout, line);
            contains(&printed.stdout, b"2222222222222222222222222222222222222222");
        }
    }
}

#[test]
fn resource_holds_keep_full_names_and_recovery_in_both_ctl_forms() {
    const WORDS: &[u8] = b"watch resources on branch temper/11/c13 in playground/widget: forge.client.resources = 8; required usage 9; reduce the named work or raise this limit in the deployment configuration, restart, then release";
    assert!(WORDS.len() > 128);
    for (command, json) in [("task", false), ("task", true), ("change", false), ("change", true)] {
        let mut root = f::signed_in(f::PARTY);
        let arguments = [command, "show", "2", "--project", "1"];
        let arguments = if command == "change" { &arguments[..] } else { &arguments[..3] };
        let call = root.submit_prepared(f::call(arguments), json, None);
        if command == "change" {
            end_panel(&mut root, call, c::WatchEnd::Unknown);
        }
        let mut held = f::held_task(task(false, false), c::HoldCategory::ResourceUnavailable);
        held.hold.as_mut().unwrap().words = Some(f::document_text(WORDS));
        root.serve(call, f::task_result(held));
        let printed = root.answer(call).expect("held task read");
        assert!(
            skein_lib::bytes::find(&printed.stdout, WORDS).is_some()
                || skein_lib::bytes::find(&printed.stderr, WORDS).is_some()
        );
    }
}

pub(super) fn approval_parts() -> w::ChangePanelParts {
    let mut parts = panel(w::Step::Queued(
        w::Queued::new(&w::CEILINGS, w::QueuedParts { place: 1, ahead: List::with_capacity(0) }).unwrap(),
    ))
    .into_parts();
    parts.approvals = List::with_capacity(1);
    parts
        .approvals
        .push(
            w::Approval::new(
                &w::CEILINGS,
                w::ApprovalParts {
                    scope: 1,
                    number: 2,
                    role: 0,
                    freshness: w::RuleFreshness::Clean,
                    required: 1,
                    read: w::ReviewRead::Complete,
                    counted: List::with_capacity(0),
                    remaining: Some(1),
                    refused: false,
                },
            )
            .unwrap(),
        )
        .unwrap();
    parts.unmapped_reviews = List::with_capacity(1);
    parts
        .unmapped_reviews
        .push(
            w::UnmappedReview::new(
                &w::CEILINGS,
                w::UnmappedReviewParts { account: 99, head: [17; 32], verdict: w::Verdict::Approved },
            )
            .unwrap(),
        )
        .unwrap();
    parts
}

#[test]
fn approval_wait_text_and_json_send_the_person_to_the_pull_request() {
    for (json, satisfied) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut root = f::signed_in(f::PARTY);
        let call = if json {
            root.submit(&["change", "show", "2", "--project", "1", "--json"])
        } else {
            root.submit(&["change", "show", "2", "--project", "1"])
        };
        let parts = if satisfied { satisfied_approval_parts() } else { approval_parts() };
        deliver(&mut root, w::ChangePanel::new(&w::CEILINGS, parts).unwrap());
        let answer = root.answer(call).expect("panel response");
        assert_eq!(answer.status, 0);
        if satisfied {
            if json {
                contains(&answer.stdout, br#""remaining":0"#);
                contains(
                    &answer.stdout,
                    br#""counted":[{"person":7,"head":"2222222222222222222222222222222222222222"}]"#,
                );
            } else {
                contains(&answer.stdout, b"counted person 7 at head 2222222222222222222222222222222222222222");
                contains(&answer.stdout, b"remaining 0");
                contains(&answer.stdout, b"next: temper ctl DIR change show 2 --until landed");
            }
        } else if json {
            contains(&answer.stdout, br#""role":0,"required":1,"freshness":"clean","read":"complete","remaining":1"#);
            contains(&answer.stdout, br#""counted":[]"#);
            contains(&answer.stdout, br#""account":99,"head":"1111111111111111111111111111111111111111","verdict":"approved","reason":"forge account mapped to no person""#);
        } else {
            contains(
                &answer.stdout,
                b"approval: scope 1 rule 2 role 0 clean required 1; reviews complete; counted 0; remaining 1",
            );
            contains(&answer.stdout, b"uncounted review: forge account 99 at head 1111111111111111111111111111111111111111 approved; mapped to no person");
            contains(&answer.stdout, b"next: review the pull request http://127.0.0.1:3000/qa/pilot-20261026/pulls/1");
            assert!(skein_lib::bytes::find(&answer.stdout, b"--until landed").is_none());
        }
    }
}

fn satisfied_approval_parts() -> w::ChangePanelParts {
    let mut parts = approval_parts();
    let mut approval = parts.approvals.get(0).unwrap().clone().into_parts();
    approval.remaining = Some(0);
    approval.counted = List::with_capacity(1);
    approval
        .counted
        .push(w::CountedReview::new(&w::CEILINGS, w::CountedReviewParts { person: 7, head: [34; 32] }).unwrap())
        .unwrap();
    *parts.approvals.get_mut(0).unwrap() = w::Approval::new(&w::CEILINGS, approval).unwrap();
    parts
}

#[test]
fn no_ci_rule_prints_not_required_and_preserves_the_observation_in_json() {
    let mut parts = panel(landed()).into_parts();
    parts.ci_required = false;
    let panel = w::ChangePanel::new(&w::CEILINGS, parts).unwrap();
    let limits = f::limits_root();
    let text = change::print(Some(&panel), false, None, b"", &limits, &change::Holds::new());
    contains(&text.stdout, b"CI not required");
    let json = change::print(Some(&panel), true, None, b"", &limits, &change::Holds::new());
    contains(&json.stdout, br#""ci_required":false,"ci":"pending""#);
}
