//! Each finding act and the subject/read's printed forms.
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
    root_at(3)
}
fn root_at(project: u32) -> f::Harness {
    let mut root = f::started(List::with_capacity(0));
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).unwrap().project = project;
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
        goal: None,
        reason: None,
    };
    let answer = w::FindingAnswer { result: w::FindingResult::Finding(detail) };
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
        root.adoption_answer(call, w::Outcome::Triaged(w::Triaged { goal: None }));
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), output.as_bytes());
    }
}
fn triage(root: &f::Harness, call: u32) -> w::TriageFinding {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
    assert_eq!(*scope, c::Scope::Project(2));
    let w::Request::TriageFinding(triage) = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request
    else {
        panic!("TriageFinding")
    };
    triage
}

#[test]
fn finding_repair_sends_the_group_and_prints_the_committed_goal() {
    for json in [false, true] {
        let mut root = root_at(2);
        let call = root
            .submit_call(f::call(&["finding", "repair", "2/3", "shop/5", "repair together", "--key", "repair"]), json);
        assert_eq!(root.sends.len(), 1);
        assert_eq!(
            triage(&root, call),
            w::TriageFinding {
                project: 2,
                number: 3,
                act: w::Triage::Repair(w::FindingRepair {
                    more: List::from_boxed(Box::new([5])),
                    words: b"repair together".as_slice().into(),
                })
            }
        );
        root.adoption_answer(call, w::Outcome::Triaged(w::Triaged { goal: Some(90) }));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert!(f::has(&printed, br#""goal":90,"more":[5]"#));
        } else {
            assert_eq!(printed.stdout.as_ref(), b"repair goal 90 made for 2/3, 2/5\n");
        }
        assert!(root.family_reads.is_empty(), "triage answers need no further read");
    }
    // The one-finding form must reserve its last positional, even when TEXT looks like a finding.
    let mut root = root_at(2);
    let call = root.submit(&["finding", "repair", "2/3", "2/5", "--key", "one"]);
    assert_eq!(
        triage(&root, call).act,
        w::Triage::Repair(w::FindingRepair { more: List::uncapped(), words: b"2/5".as_slice().into() })
    );
    root.adoption_answer(call, w::Outcome::Triaged(w::Triaged { goal: Some(91) }));
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"repair goal 91 made for 2/3\n");
}

#[test]
fn finding_repair_refuses_mixed_repeated_and_excess_findings_before_sending() {
    for findings in [["2/3", "3/5"], ["2/3", "shop/3"], ["2/3", "2/3"]] {
        let mut root = root_at(2);
        let call = root.submit(&["finding", "repair", findings[0], findings[1], "repair together", "--key", "repair"]);
        assert_eq!(root.answer(call).unwrap().status, 2);
        assert!(root.nothing_sent());
    }
    let names = [
        "2/1", "2/2", "2/3", "2/4", "2/5", "2/6", "2/7", "2/8", "2/9", "2/10", "2/11", "2/12", "2/13", "2/14", "2/15",
        "2/16", "2/17",
    ];
    for count in [16, 17] {
        let mut argv = List::uncapped();
        argv.push("finding").unwrap();
        argv.push("repair").unwrap();
        for name in names.iter().take(count) {
            argv.push(*name).unwrap();
        }
        for word in ["repair together", "--key", "repair"] {
            argv.push(word).unwrap();
        }
        if count == 17 {
            f::parse(argv.as_slice()).expect_err("at most sixteen findings");
        } else {
            let mut root = root_at(2);
            let call = root.submit(argv.as_slice());
            let w::Triage::Repair(repair) = triage(&root, call).act else { panic!("repair") };
            assert_eq!(repair.more.len(), 15);
            assert_eq!(repair.more.as_slice().last(), Some(&16));
        }
    }
    assert!(f::parse(&["finding", "repair", "2/3", "repair together"]).is_err(), "key required");
    assert!(f::parse(&["finding", "repair", "repair together", "--key", "repair"]).is_err(), "finding required");
    let mut root = root_at(2);
    let mut call = f::call(&["finding", "repair", "2/3", "repair together", "--key", "repair"]);
    let mut findings = List::uncapped();
    for _ in 0_u32..17 {
        findings.push(Value::Text(b"2/3".as_slice().into())).unwrap();
    }
    *call.values.get_mut(0).unwrap() = Some(Values::Many(findings.into_boxed()));
    let call = root.submit_call(call, false);
    assert_eq!(root.answer(call).unwrap().status, 2);
    assert!(root.nothing_sent());
    for words in ["", &"x".repeat(4097)] {
        let mut root = root_at(2);
        let call = root.submit(&["finding", "repair", "2/3", words, "--key", "repair"]);
        assert_eq!(root.answer(call).unwrap().status, 2);
        assert!(root.nothing_sent());
    }
}

#[test]
fn finding_blame_sends_both_commit_lengths_or_none_and_prints_the_landing() {
    for (commit, bytes, printed) in [
        (
            "1234567890abcdef1234567890abcdef12345678",
            &b"\x12\x34\x56\x78\x90\xab\xcd\xef\x12\x34\x56\x78\x90\xab\xcd\xef\x12\x34\x56\x78"[..],
            b"blamed: 2/3 on landing 1234567890ab\n".as_slice(),
        ),
        (
            "abababababababababababababababababababababababababababababababab",
            &[0xab; 32][..],
            b"blamed: 2/3 on landing abababababab\n".as_slice(),
        ),
    ] {
        let mut root = root_at(2);
        let call = root.submit(&["finding", "blame", "2/3", "--landing", commit, "this landing", "--key", "blame"]);
        assert_eq!(
            triage(&root, call),
            w::TriageFinding {
                project: 2,
                number: 3,
                act: w::Triage::Blame(w::FindingBlame {
                    landing: Some(bytes.into()),
                    why: b"this landing".as_slice().into()
                })
            }
        );
        assert_eq!(root.sends.len(), 1);
        root.adoption_answer(call, w::Outcome::Triaged(w::Triaged { goal: None }));
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), printed);
    }
    let mut root = root_at(2);
    let call = root.submit(&["finding", "blame", "2/3", "--none", "no landing", "--key", "blame"]);
    assert_eq!(
        triage(&root, call).act,
        w::Triage::Blame(w::FindingBlame { landing: None, why: b"no landing".as_slice().into() })
    );
    root.adoption_answer(call, w::Outcome::Triaged(w::Triaged { goal: None }));
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"blamed: 2/3 on no landing\n");
}

#[test]
fn finding_blame_refuses_missing_conflicting_flags_bad_commits_and_missing_keys() {
    for argv in [
        &["finding", "blame", "2/3", "why", "--key", "blame"][..],
        &[
            "finding",
            "blame",
            "2/3",
            "--landing",
            "1234567890abcdef1234567890abcdef12345678",
            "--none",
            "why",
            "--key",
            "blame",
        ][..],
        &["finding", "blame", "2/3", "--none", "why"][..],
        &["finding", "blame", "2/3", "--landing", "1234567890abcdef1234567890abcdef12345678", "why"][..],
    ] {
        assert!(f::parse(argv).is_err(), "{argv:?}");
    }
    for commit in [
        "1234",
        "z234567890abcdef1234567890abcdef12345678",
        "1234567890abcdef1234567890abcdef1234567",
        "1234567890abcdef1234567890abcdef123456789",
    ] {
        let mut root = root_at(2);
        let call = root.submit(&["finding", "blame", "2/3", "--landing", commit, "why", "--key", "blame"]);
        assert_eq!(root.answer(call).unwrap().status, 2);
        assert!(root.nothing_sent());
    }
    for why in ["", &"x".repeat(201)] {
        let mut root = root_at(2);
        let call = root.submit(&["finding", "blame", "2/3", "--none", why, "--key", "blame"]);
        assert_eq!(root.answer(call).unwrap().status, 2);
        assert!(root.nothing_sent());
    }
}

#[test]
fn finding_triage_prints_act_refusal_words_whole_but_keeps_field_refusals_named() {
    for argv in [
        &["finding", "repair", "2/3", "repair together", "--key", "repair"][..],
        &["finding", "blame", "2/3", "--none", "why", "--key", "blame"][..],
    ] {
        for words in ["x".repeat(1024), "why".into()] {
            let mut root = root_at(2);
            let call = root.submit(argv);
            root.adoption_answer(
                call,
                w::Outcome::FindingRefused(w::FindingRefused {
                    reason: w::FindingRefusal::Words(w::FindingWords { field: words.as_bytes().into() }),
                }),
            );
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 2);
            assert!(skein_lib::bytes::find(&printed.stderr, words.as_bytes()).is_some());
            assert_eq!(
                skein_lib::bytes::find(&printed.stderr, b"finding words past their bounds:").is_some(),
                words == "why"
            );
        }
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
            row: row(),
            body: b"Boot twice".as_slice().into(),
            filer: w::Filer::Party(w::HolderParty { number: 8 }),
            goal: Some(90),
            reason: Some(b"duplicate".as_slice().into()),
        };
        let answer = w::FindingAnswer { result: w::FindingResult::Finding(detail) };
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
