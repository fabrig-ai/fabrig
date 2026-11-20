//! Each finding act and the subject/read's printed forms.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary fixture collections and UTF-8 output assertions in step tests"
)]
use super::fixtures as f;
use crate::{
    finding,
    jig::{
        call::{Value, Values},
        rows,
    },
};
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn root() -> f::Harness {
    let mut root = f::started(List::with_capacity(0));
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).unwrap().project = 3;
    root.deliver(c::Delivery::Me(Box::new(me)));
    root
}
fn request(root: &f::Harness, call: u32) -> w::Request {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
    assert_eq!(*scope, c::Scope::Project(3));
    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request
}
fn held(task: u64) -> w::Holder {
    w::Holder::Task(w::HolderTask { number: task })
}
fn row() -> w::FindingRow {
    w::FindingRow {
        number: 12,
        kind: w::FindingKind::Red,
        severity: w::Severity::Medium,
        area: Some(b"tests".as_slice().into()),
        state: w::FindingState::New,
        title: b"Flaky boot".as_slice().into(),
        holder: held(40),
        at: 77,
        why: Some(b"words, not code".as_slice().into()),
        hander: Some(w::Filer::Task(w::HolderTask { number: 39 })),
        goal: None,
        reason: None,
        cleared: None,
    }
}
fn document() -> w::Findings {
    let owner = w::AreaOwner::Task(w::AreaTask { number: 40 });
    let area = w::AreaRow {
        name: b"tests".as_slice().into(),
        words: b"Tests and their fixtures".as_slice().into(),
        owner,
        holder: Some(held(40)),
    };
    w::Findings {
        project: 3,
        rows: List::from_boxed(Box::new([row()])),
        areas: List::from_boxed(Box::new([area])),
        triager: None,
        holder: held(41),
        cut: true,
    }
}
fn deliver(root: &mut f::Harness, state: Option<w::FindingState>, mine: bool) {
    let watch = root.watch_token(&finding::subject(3, state, mine));
    root.client(client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::findings(&document()),
            },
        },
    });
}
fn serve(root: &mut f::Harness, call: u32) {
    let detail = w::FindingDetail {
        row: row(),
        body: b"Boot twice".as_slice().into(),
        filer: w::Filer::Party(w::HolderParty { number: 8 }),
        blame: None,
    };
    let answer = w::FindingAnswer { result: w::FindingResult::Finding(Box::new(detail)) };
    root.serve_readout(
        call,
        client::Readout::Family { family: wire::FORGE, record: Some(wire::finding_answer(&answer)) },
    );
}
#[test]
fn finding_acts_send_their_fields_and_print_the_committed_holder() {
    let mut root = root();
    let mut call = f::call(&[
        "finding",
        "file",
        "--project",
        "shop",
        "--kind",
        "red",
        "--area",
        "tests",
        "Flaky boot",
        "-",
        "--key",
        "file",
    ]);
    *call.values.get_mut(5).unwrap() = Some(Values::One(Value::Text(b"Boot twice\n".as_slice().into())));
    let call = root.submit_prepared(call, false, None);
    let w::Request::FileFinding(file) = request(&root, call) else { panic!("FileFinding") };
    assert_eq!(
        file,
        w::FileFinding {
            project: 3,
            kind: w::FindingKind::Red,
            severity: w::Severity::Medium,
            area: Some(b"tests".as_slice().into()),
            title: b"Flaky boot".as_slice().into(),
            body: b"Boot twice\n".as_slice().into()
        }
    );
    root.adoption_answer(call, w::Outcome::Filed(w::Filed { number: 12 }));
    assert!(root.answer(call).is_none(), "the durable outcome has no holder; read it");
    serve(&mut root, call);
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"filed: 3/12, held by task 40\n");
    let hand = root.submit(&["finding", "hand", "3/12", "--area", "docs", "words, not code", "--key", "hand"]);
    let w::Request::HandFinding(handed) = request(&root, hand) else { panic!("HandFinding") };
    assert_eq!(
        handed,
        w::HandFinding {
            project: 3,
            number: 12,
            area: b"docs".as_slice().into(),
            why: b"words, not code".as_slice().into()
        }
    );
    root.adoption_answer(hand, w::Outcome::Handed);
    serve(&mut root, hand);
    assert_eq!(root.answer(hand).unwrap().stdout.as_ref(), b"handed: 3/12, held by task 40\n");
    for (argv, act, output) in [
        (
            &["finding", "route", "3/12", "--goal", "90", "--key", "route"][..],
            w::Triage::Route(w::FindingRoute { task: 90 }),
            "routed: 3/12\n",
        ),
        (&["finding", "backlog", "3/12", "--key", "backlog"][..], w::Triage::Backlog, "backlog: 3/12\n"),
        (
            &["finding", "drop", "shop/12", "duplicate", "--key", "drop"][..],
            w::Triage::Drop(w::FindingDrop { reason: b"duplicate".as_slice().into() }),
            "dropped: 3/12\n",
        ),
    ] {
        let call = root.submit(argv);
        let w::Request::TriageFinding(triage) = request(&root, call) else { panic!("TriageFinding") };
        assert_eq!(triage, w::TriageFinding { project: 3, number: 12, act });
        root.adoption_answer(call, w::Outcome::Triaged);
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), output.as_bytes());
    }
}
#[test]
fn finding_reads_print_areas_triager_and_the_whole_finding() {
    for json in [false, true] {
        let mut root = root();
        let list =
            root.submit_call(f::call(&["finding", "list", "--project", "shop", "--state", "new", "--mine"]), json);
        deliver(&mut root, Some(w::FindingState::New), true);
        let printed = root.answer(list).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert!(f::has(&printed, br#""areas":[{"name":"tests""#));
            assert!(f::has(&printed, br#""cut":true"#));
        } else {
            assert_eq!(printed.stdout.as_ref(), "tests owner task 40, held by task 40: Tests and their fixtures\ntriager owner role 0, held by task 41\n3/12 · new · red · medium · tests · task 40 · Flaky boot\n[cut]\n".as_bytes());
        }
        let show = root.submit_prepared(f::call(&["finding", "show", "shop/12"]), json, None);
        let client::Readable::Family { scope, record, .. } = &root.family_reads.get(&show).unwrap().1 else {
            panic!("family read")
        };
        assert_eq!(*scope, c::Scope::Project(3));
        let reading = w::Reading::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record.bytes)).unwrap();
        assert_eq!(&reading.read, &w::Read::Finding(w::FindingRead { project: 3, number: 12 }));
        let detail = w::FindingDetail {
            row: {
                let mut parts = row();
                parts.goal = Some(90);
                parts.reason = Some(b"duplicate".as_slice().into());
                parts
            },
            body: b"Boot twice".as_slice().into(),
            filer: w::Filer::Party(w::HolderParty { number: 8 }),
            blame: None,
        };
        let answer = w::FindingAnswer { result: w::FindingResult::Finding(Box::new(detail)) };
        root.serve_readout(
            show,
            client::Readout::Family { family: wire::FORGE, record: Some(wire::finding_answer(&answer)) },
        );
        let printed = root.answer(show).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert!(f::has(&printed, br#""body":"Boot twice""#));
        } else {
            assert!(f::has(&printed, b"filer: party 8\nfiled: 77\nhanded on: words, not code\ngoal: task 90\ndropped: duplicate\n\nBoot twice\n"));
        }
        assert_eq!(root.watches.len(), 0, "reads close the findings watch");
    }
}
fn goal_document(rows: Vec<w::FindingRow>, cut: bool) -> w::Findings {
    let mut parts = document();
    parts.project = 2;
    parts.rows = List::from_boxed(rows.into_boxed_slice());
    parts.cut = cut;
    parts
}
fn goal_filter() -> client::Watchable {
    filtered(2, w::FindingsFilter::Goal(w::FindingsGoal { goal: 820 }))
}
fn red(number: u32, state: w::FindingState, title: &[u8]) -> w::FindingRow {
    let mut parts = row();
    parts.number = number;
    parts.state = state;
    parts.title = title.into();
    parts.goal = if parts.state == w::FindingState::Routed { Some(830) } else { None };
    parts.reason = if parts.state == w::FindingState::Dropped { Some(b"duplicate".as_slice().into()) } else { None };
    parts
}
fn goal_task() -> c::Task {
    let mut task = f::goal(820, c::WaitsOn::Nothing);
    task.project = 2;
    task
}
#[test]
fn goal_show_reads_the_task_then_its_reds_uncleared_first_in_every_place() {
    let mut cleared = red(9, w::FindingState::Routed, b"1 test fails in nightly since S7 landed");
    cleared.cleared = Some(clearing());
    let rows = vec![
        cleared,
        red(14, w::FindingState::Routed, b"3 tests fail in nightly since S7 landed"),
        red(13, w::FindingState::New, b"Waiting"),
        red(12, w::FindingState::Backlog, b"Backlogged"),
        red(11, w::FindingState::Dropped, b"Dropped"),
        {
            let mut parts = red(10, w::FindingState::Routed, b"Repairing");
            parts.goal = None;
            parts
        },
    ];
    for json in [false, true] {
        let mut root = root();
        let call = root.submit_prepared(f::call(&["goal", "show", "820"]), json, None);
        assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Task { task: 820 });
        assert!(root.watches.is_empty(), "read the task before the findings");
        root.serve(call, f::task_result(goal_task()));
        assert!(root.answer(call).is_none(), "the task alone does not complete goal show");
        deliver_document(&mut root, &goal_filter(), &goal_document(rows.clone(), true));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        let ordinary = root.submit_prepared(f::call(&["task", "show", "820"]), json, None);
        root.serve(ordinary, f::task_result(goal_task()));
        let task = root.answer(ordinary).unwrap();
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            let text = String::from_utf8(printed.stdout.into_vec()).unwrap();
            assert!(
                text.starts_with(&format!(
                    "{{\"task\":{},\"reds\":[",
                    String::from_utf8(task.stdout.into_vec()).unwrap()
                ))
            );
            for place in [
                r#""stands":{"kind":"repairing","goal":830}"#,
                r#""stands":{"kind":"waiting"}"#,
                r#""stands":{"kind":"backlog"}"#,
                r#""stands":{"kind":"dropped","reason":"duplicate"}"#,
                r#""stands":{"kind":"repairing"}"#,
                r#""stands":{"kind":"cleared","tier":"nightly","run":412}"#,
            ] {
                assert!(text.contains(place), "{place}: {text}");
            }
            let order: Vec<usize> =
                [14u32, 13, 12, 11, 10, 9].iter().map(|n| text.find(&format!("\"number\":{n},")).unwrap()).collect();
            assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(text.ends_with("],\"cut\":true}"));
        } else {
            let suffix = printed.stdout.strip_prefix(task.stdout.as_ref()).expect("task show's document, unchanged");
            assert_eq!(suffix, "reds after landing:\n  2/14 3 tests fail in nightly since S7 landed · being repaired by goal 830\n  2/13 Waiting · waiting for triage\n  2/12 Backlogged · in the backlog\n  2/11 Dropped · dropped: duplicate\n  2/10 Repairing · being repaired\n  2/9 1 test fails in nightly since S7 landed · cleared by nightly run 412\n[cut]\n".as_bytes());
        }
        assert!(root.watches.is_empty());
    }
}
#[test]
fn goal_show_prints_none_and_cut_for_a_goal_or_other_task_live_or_ended() {
    for goal in [false, true] {
        for ended in [false, true] {
            for cut in [false, true] {
                for json in [false, true] {
                    let mut root = root();
                    let mut task = goal_task();
                    if !goal {
                        task.goal = None;
                    }
                    let page = if ended { f::ended_task_result(task, c::Ending::Done) } else { f::task_result(task) };
                    let ordinary = root.submit_prepared(f::call(&["task", "show", "820"]), json, None);
                    root.serve(ordinary, page.clone());
                    let document = root.answer(ordinary).unwrap();
                    let call = root.submit_prepared(f::call(&["goal", "show", "820"]), json, None);
                    root.serve(call, page);
                    deliver_document(&mut root, &goal_filter(), &goal_document(vec![], cut));
                    let printed = root.answer(call).unwrap();
                    assert_eq!(printed.status, 0);
                    if json {
                        let expected = format!(
                            "{{\"task\":{},\"reds\":[],\"cut\":{cut}}}",
                            String::from_utf8(document.stdout.into_vec()).unwrap()
                        );
                        assert_eq!(printed.stdout.as_ref(), expected.as_bytes());
                    } else {
                        let suffix = printed.stdout.strip_prefix(document.stdout.as_ref()).unwrap();
                        assert_eq!(
                            suffix,
                            if cut {
                                b"reds after landing: none\n[cut]\n".as_slice()
                            } else {
                                b"reds after landing: none\n"
                            }
                        );
                    }
                    assert!(root.watches.is_empty());
                }
            }
        }
    }
}
#[test]
fn goal_show_keeps_supplemental_task_words_and_refuses_a_missing_task() {
    let mut root = root();
    let call = root.submit(&["goal", "show", "820"]);
    let mut task = goal_task();
    task.spec.specification.words = f::document_text(b"First");
    task.spec.specification.words.cut = true;
    root.serve(call, f::task_result(task));
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::TaskText { task: 820, field: c::TaskTextField::Specification, from: 5 }
    );
    assert!(root.watches.is_empty(), "finish the task's source before its findings");
    root.serve(call, c::ReadResult::None(c::ReadNone::Unknown));
    deliver_document(&mut root, &goal_filter(), &goal_document(vec![], false));
    let printed = root.answer(call).expect("refused supplemental words keep the prefix and continuation");
    assert_eq!(printed.status, 0);
    assert!(f::has(&printed, b"asked: First"));
    assert!(f::has(&printed, b"task text 820 --field specification --from 0"));
    assert!(printed.stdout.ends_with(b"reds after landing: none\n"));
    let missing = root.submit(&["goal", "show", "821"]);
    root.serve(missing, c::ReadResult::None(c::ReadNone::Unknown));
    assert_eq!(root.answer(missing).unwrap().status, 2);
    assert!(root.watches.is_empty());
}
fn blamed_goal(goal: u64, milestone: Option<u64>, person: Option<u64>) -> w::BlamedGoal {
    w::BlamedGoal { goal, milestone, person }
}
fn landing(goals: Vec<w::BlamedGoal>) -> w::BlamedLanding {
    let mut commit = [0; 32];
    commit[..6].copy_from_slice(&[0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]);
    w::BlamedLanding { commit, goals: List::from_boxed(goals.into_boxed_slice()) }
}
#[test]
fn finding_show_prints_blame_landings_and_clearing_before_the_body() {
    for (blame, text, json_blame) in [
        (
            Some((w::BlameStrength::Owned, vec![landing(vec![blamed_goal(820, Some(812), Some(4))])])),
            "blame: owned\n  landing 1a2b3c4d5e6f: goal 820 in milestone 812, person 4\ncleared: nightly run 412 at 2026-11-19T09:40Z\n",
            r#""blame":{"strength":"owned","landings":[{"commit":"1a2b3c4d5e6f0000000000000000000000000000000000000000000000000000","goals":[{"goal":820,"milestone":812,"person":4}]}]}"#,
        ),
        (
            Some((
                w::BlameStrength::Suspected,
                vec![
                    landing(vec![blamed_goal(820, None, Some(4)), blamed_goal(821, Some(812), None)]),
                    landing(vec![]),
                ],
            )),
            "blame: suspected\n  landing 1a2b3c4d5e6f: goal 820, person 4; goal 821 in milestone 812\n  landing 1a2b3c4d5e6f: no goal\ncleared: nightly run 412 at 2026-11-19T09:40Z\n",
            r#""blame":{"strength":"suspected","landings":[{"commit":"1a2b3c4d5e6f0000000000000000000000000000000000000000000000000000","goals":[{"goal":820,"milestone":null,"person":4},{"goal":821,"milestone":812,"person":null}]},{"commit":"1a2b3c4d5e6f0000000000000000000000000000000000000000000000000000","goals":[]}]}"#,
        ),
        (
            Some((w::BlameStrength::Unblamed, vec![landing(vec![])])),
            "blame: none\n  landing 1a2b3c4d5e6f: no goal\ncleared: nightly run 412 at 2026-11-19T09:40Z\n",
            r#""blame":{"strength":"none","landings":[{"commit":"1a2b3c4d5e6f0000000000000000000000000000000000000000000000000000","goals":[]}]}"#,
        ),
        (None, "cleared: nightly run 412 at 2026-11-19T09:40Z\n", r#""blame":null"#),
    ] {
        for json in [false, true] {
            let mut root = root();
            let call = root.submit_prepared(f::call(&["finding", "show", "3/12"]), json, None);
            let mut parts = row();
            parts.reason = Some(b"duplicate".as_slice().into());
            parts.cleared = Some(clearing());
            let blame = blame.clone().map(|(strength, landings)| w::FindingBlame {
                strength,
                landings: List::from_boxed(landings.into_boxed_slice()),
            });
            let detail = w::FindingDetail {
                row: parts,
                body: b"Boot twice".as_slice().into(),
                filer: w::Filer::Party(w::HolderParty { number: 8 }),
                blame: blame.clone(),
            };
            let answer = w::FindingAnswer { result: w::FindingResult::Finding(Box::new(detail)) };
            root.serve_readout(
                call,
                client::Readout::Family { family: wire::FORGE, record: Some(wire::finding_answer(&answer)) },
            );
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 0);
            if json {
                f::unique_keys(&printed.stdout).unwrap();
                assert!(f::has(&printed, json_blame.as_bytes()), "{}", String::from_utf8_lossy(&printed.stdout));
                assert!(f::has(&printed, br#""cleared":{"tier":"nightly","run":412,"at":1795081200000000000}"#));
            } else {
                assert!(
                    f::has(&printed, format!("dropped: duplicate\n{text}\nBoot twice\n").as_bytes()),
                    "{}",
                    String::from_utf8_lossy(&printed.stdout)
                );
                if blame.is_none() {
                    assert!(!f::has(&printed, b"blame:"));
                }
            }
        }
    }
}
fn filtered(project: u32, filter: w::FindingsFilter) -> client::Watchable {
    let subject = w::FindingsSubject { project, filter };
    let watch = w::Watching { watch: w::Watch::Findings(subject) };
    client::Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watch) },
    }
}
fn clearing() -> w::FindingClearing {
    w::FindingClearing { tier: b"nightly".as_slice().into(), run: 412, at: 1_795_081_200_000_000_000 }
}
fn deliver_document(root: &mut f::Harness, subject: &client::Watchable, document: &w::Findings) {
    let watch = root.watch_token(subject);
    root.client(client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::findings(document),
            },
        },
    });
}
#[test]
fn finding_list_filters_blame_and_prints_clearing_and_row_fields() {
    for (flag, task, filter) in [
        ("--goal", "820", w::FindingsFilter::Goal(w::FindingsGoal { goal: 820 })),
        ("--milestone", "812", w::FindingsFilter::Milestone(w::FindingsMilestone { milestone: 812 })),
    ] {
        for json in [false, true] {
            let mut root = root();
            let call = root.submit_call(f::call(&["finding", "list", "--project", "shop", flag, task]), json);
            let mut parts = document();
            let mut cleared = row();
            cleared.goal = Some(830);
            cleared.reason = Some(b"duplicate".as_slice().into());
            cleared.cleared = Some(clearing());
            parts.rows = List::from_boxed(Box::new([cleared, row()]));
            deliver_document(&mut root, &filtered(3, filter.clone()), &parts);
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 0);
            if json {
                f::unique_keys(&printed.stdout).unwrap();
                assert!(f::has(&printed, br#""goal":830,"reason":"duplicate","cleared":{"tier":"nightly","run":412,"at":1795081200000000000}"#));
                assert!(f::has(&printed, br#""goal":null,"reason":null,"cleared":null"#));
            } else {
                assert!(f::has(&printed, "Flaky boot · cleared by nightly run 412\n".as_bytes()));
            }
            assert!(root.watches.is_empty());
        }
    }
}
#[test]
fn finding_list_refuses_mixed_filters_naming_the_flags() {
    for (left, right) in [
        (&["--goal", "820"][..], &["--milestone", "812"][..]),
        (&["--goal", "820"][..], &["--state", "new"][..]),
        (&["--goal", "820"][..], &["--mine"][..]),
        (&["--milestone", "812"][..], &["--state", "new"][..]),
        (&["--milestone", "812"][..], &["--mine"][..]),
    ] {
        let mut argv = vec!["finding", "list", "--project", "2"];
        argv.extend_from_slice(left);
        argv.extend_from_slice(right);
        let why = f::parse(&argv).expect_err("mixed filters");
        let bytes: Vec<&[u8]> = argv.iter().map(|word| word.as_bytes()).collect();
        let mut measure = skein_lib::Writer::measure(512);
        crate::table::refusal(why, &bytes, &mut measure).unwrap();
        let mut text = skein_lib::Writer::new(measure.written());
        crate::table::refusal(why, &bytes, &mut text).unwrap();
        let text = String::from_utf8(text.finish().into_vec()).unwrap();
        assert!(text.contains(left[0]) && text.contains(right[0]), "{text}");
        // Prepared calls bypass argv but must still be usage failures, sending nothing.
        let mut call = f::call(&["finding", "list", "--project", "2"]);
        for words in [left, right] {
            let (index, value) = match words[0] {
                "--goal" => (3, Value::Number(820)),
                "--milestone" => (4, Value::Number(812)),
                "--state" => (1, Value::Word(0)),
                "--mine" => (2, Value::Switch),
                _ => unreachable!(),
            };
            *call.values.get_mut(index).unwrap() = Some(Values::One(value));
        }
        let mut root = root();
        let number = root.submit_call(call, false);
        let printed = root.answer(number).unwrap();
        assert_eq!(printed.status, 1);
        let text = String::from_utf8(printed.stderr.into_vec()).unwrap();
        assert!(text.contains(left[0]) && text.contains(right[0]), "{text}");
        assert!(root.nothing_sent());
    }
}
#[test]
fn finding_refuses_other_forms_and_names_the_areas_on_refusal() {
    for name in ["12", "shop", "/12", "3/", "3/0", "3/12/2", "4294967296/12", "3/no"] {
        assert!(f::parse(&["finding", "drop", name, "duplicate", "--key", "drop"]).is_err(), "{name}");
    }
    let mut root = root();
    let mut invalid = f::call(&["finding", "drop", "3/12", "duplicate", "--key", "drop"]);
    *invalid.values.get_mut(0).unwrap() = Some(Values::One(Value::Text(b"12".as_slice().into())));
    let invalid = root.submit_call(invalid, false);
    assert_eq!(root.answer(invalid).unwrap().status, 1);
    assert!(root.nothing_sent());
    let call = root.submit(&["finding", "hand", "3/12", "--area", "missing", "wrong area", "--key", "missing"]);
    let why = w::FindingRefusal::NoArea(w::FindingAreas {
        areas: List::from_boxed(Box::new([b"tests".as_slice().into(), b"docs".as_slice().into()])),
    });
    root.adoption_answer(call, w::Outcome::FindingRefused(w::FindingRefused { reason: why }));
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 2);
    assert!(skein_lib::bytes::find(&printed.stderr, b"no area missing in project 3; its areas: tests, docs").is_some());
    assert_eq!(crate::table::TOOLS[usize::from(rows::FINDING_SHOW_AT)].name, b"finding show");
}
