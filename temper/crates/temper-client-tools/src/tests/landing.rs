use super::fixtures as f;
use crate::{jig, landing};
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn panel(phase: w::LandingPhase) -> w::LandingPanel {
    let mut gates = List::with_capacity(1);
    gates
        .push(
            w::GateProof::new(
                &w::CEILINGS,
                w::GateProofParts {
                    name: b"gate".as_slice().into(),
                    blocking: true,
                    key: Some([0x33; 32]),
                    proof: w::Proof::Dispatched(
                        w::Dispatched::new(&w::CEILINGS, w::DispatchedParts { attempt: 1, run: Some(7) }).unwrap(),
                    ),
                    faults: 0,
                },
            )
            .unwrap(),
        )
        .unwrap();
    let mut members = List::with_capacity(1);
    members
        .push(
            w::Member::new(
                &w::CEILINGS,
                w::MemberParts {
                    head: None,
                    left: None,
                    judgement: None,
                    change: 51,
                    forge: 1,
                    owner: b"work".as_slice().into(),
                    name: b"hello".as_slice().into(),
                    branch: b"temper/51".as_slice().into(),
                    landing: b"main".as_slice().into(),
                    pull: Some(
                        w::Pull::new(
                            &w::CEILINGS,
                            w::PullParts {
                                number: 3,
                                address: b"http://127.0.0.1:3000/work/hello/pulls/3".as_slice().into(),
                            },
                        )
                        .unwrap(),
                    ),
                    base: Some([0x22; 32]),
                    candidate: Some([0x11; 32]),
                    landed: if phase == w::LandingPhase::Landed { Some([0x11; 32]) } else { None },
                    gates,
                },
            )
            .unwrap(),
        )
        .unwrap();
    let hold = if phase == w::LandingPhase::Held {
        Some(w::LandingHold::Faults(
            w::HeldFaults::new(&w::CEILINGS, w::HeldFaultsParts { member: 0, gate: 0 }).unwrap(),
        ))
    } else {
        None
    };
    w::LandingPanel::new(
        &w::CEILINGS,
        w::LandingPanelParts {
            train: None,
            round: None,
            task: 61,
            project: 1,
            phase,
            hold,
            members,
            preparation: 1,
            stale_proofs: 0,
            turn: None,
        },
    )
    .unwrap()
}
fn deliver(root: &mut f::Harness, subject: client::Watchable, bytes: Box<[u8]>) {
    let token = root.watch_token(&subject);
    root.client(client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch: token,
            record: client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
        },
    });
}
fn read_panel(panel: &w::LandingPanel, json: bool) -> jig::Printed {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(if json {
        &["landing", "show", "61", "--project", "1", "--json"]
    } else {
        &["landing", "show", "61", "--project", "1"]
    });
    let subject = landing::subject(Some(61), 1, false, &root.limits);
    deliver(&mut root, subject, wire::landing_panel(panel));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert_eq!(root.watches.len(), 1, "only me remains watched");
    answer
}
#[test]
fn a_landing_prints_its_members_gates_and_turn() {
    f::check_golden("landing_proving", &read_panel(&panel(w::LandingPhase::Proving), false).stdout);
}
#[test]
fn a_landed_landing_prints_main_at_its_candidate() {
    f::check_golden("landing_landed", &read_panel(&panel(w::LandingPhase::Landed), false).stdout);
}
#[test]
fn a_held_landing_names_its_gate() {
    f::check_golden("landing_held", &read_panel(&panel(w::LandingPhase::Held), false).stdout);
}
fn with_gate(proof: w::Proof, key: Option<[u8; 32]>, blocking: bool, faults: u32) -> w::LandingPanel {
    let mut parts = panel(w::LandingPhase::Proving).into_parts();
    let mut member = parts.members.get(0).unwrap().clone().into_parts();
    let mut gate = member.gates.get(0).unwrap().clone().into_parts();
    gate.proof = proof;
    gate.key = key;
    gate.blocking = blocking;
    gate.faults = faults;
    member.gates = List::with_capacity(1);
    member.gates.push(w::GateProof::new(&w::CEILINGS, gate).unwrap()).unwrap();
    parts.members = List::with_capacity(1);
    parts.members.push(w::Member::new(&w::CEILINGS, member).unwrap()).unwrap();
    w::LandingPanel::new(&w::CEILINGS, parts).unwrap()
}
fn receipt() -> w::Receipted {
    w::Receipted::new(
        &w::CEILINGS,
        w::ReceiptedParts { run: 7, number: 3, address: b"http://127.0.0.1:3000/work/hello/pulls/3".as_slice().into() },
    )
    .unwrap()
}
#[test]
fn a_gate_passed_by_receipt_prints_its_run_and_address() {
    let panel = with_gate(w::Proof::Passed(receipt()), Some([0x33; 32]), true, 0);
    let answer = read_panel(&panel, false);
    assert!(contains(
        &answer.stdout,
        "  gate gate, blocking: key 3333333333333333, passed by run 7 · http://127.0.0.1:3000/work/hello/pulls/3\n"
    ));
    let json = read_panel(&panel, true);
    assert!(contains(
        &json.stdout,
        "\"proof\":{\"passed\":{\"run\":7,\"number\":3,\"address\":\"http://127.0.0.1:3000/work/hello/pulls/3\"}}"
    ));
}
#[test]
fn every_gate_proof_prints_its_state_and_runner_faults() {
    for (proof, key, expected) in [
        (w::Proof::Unkeyed, None, "no key yet"),
        (w::Proof::Keyed, Some([0x33; 32]), "key 3333333333333333, keyed"),
        (w::Proof::Waiting, Some([0x33; 32]), "key 3333333333333333, waiting on another landing's run"),
        (
            w::Proof::Dispatched(
                w::Dispatched::new(&w::CEILINGS, w::DispatchedParts { attempt: 1, run: None }).unwrap(),
            ),
            Some([0x33; 32]),
            "key 3333333333333333, dispatched (try 1)",
        ),
        (
            w::Proof::Failed(receipt()),
            Some([0x33; 32]),
            "key 3333333333333333, failed by run 7 · http://127.0.0.1:3000/work/hello/pulls/3",
        ),
    ] {
        let answer = read_panel(&with_gate(proof, key, false, 2), false);
        assert!(contains(&answer.stdout, "  gate gate, advisory: "));
        assert!(contains(&answer.stdout, ", 2 runner faults\n"));
        assert!(contains(&answer.stdout, expected));
    }
}
#[test]
fn a_landing_the_engine_does_not_know_fails_with_exit_2() {
    for end in [c::WatchEnd::Unknown, c::WatchEnd::SubjectEnded] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["landing", "show", "61", "--project", "1"]);
        root.end(call, &landing::subject(Some(61), 1, false, &root.limits), end);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        assert_eq!(answer.stderr.as_ref(), b"no landing for task 61\n");
        assert_eq!(root.watches.len(), 1);
    }
}
#[test]
fn a_landed_queue_change_has_no_landing_in_text_or_json() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        root.pending_reads.push(1);
        let call = root.submit_prepared(f::call(&["landing", "show", "3"]), json, None);
        root.serve(call, f::task_result(f::goal(3, c::WaitsOn::Nothing)));
        let step = w::Step::Landed(w::Landed::new(&w::CEILINGS, w::LandedParts { merge: [34; 32] }).unwrap());
        let mut parts = super::change::panel(step).into_parts();
        parts.task = 3;
        let subject = landing::subject(Some(3), 1, false, &root.limits);
        deliver(&mut root, subject, wire::change_panel(&w::ChangePanel::new(&w::CEILINGS, parts).unwrap()));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2, "json={json}");
        assert!(answer.stdout.is_empty());
        assert_eq!(
            answer.stderr.as_ref(),
            if json { b"{\"error\":\"no landing for task 3\"}".as_slice() } else { b"no landing for task 3\n" }
        );
        assert_eq!(root.watches.len(), 1, "only me remains watched");
    }
}
fn list(turn: Option<u64>, tasks: &[u64]) -> w::LandingsPanel {
    let mut landings = List::with_capacity(u32::try_from(tasks.len()).unwrap());
    for task in tasks {
        let mut changes = List::with_capacity(1);
        changes.push(task.checked_sub(10).unwrap()).unwrap();
        let mut repositories = List::with_capacity(1);
        repositories.push(b"work/hello".as_slice().into()).unwrap();
        let mut landed = List::with_capacity(1);
        if *task != 61 {
            landed.push([0x11; 32]).unwrap();
        }
        landings
            .push(
                w::LandingLine::new(
                    &w::CEILINGS,
                    w::LandingLineParts {
                        train: Some(
                            w::Train::new(
                                &w::CEILINGS,
                                w::TrainParts {
                                    forge: 1,
                                    repository: 9,
                                    owner: b"work".as_slice().into(),
                                    name: b"hello".as_slice().into(),
                                    branch: b"release".as_slice().into(),
                                },
                            )
                            .unwrap(),
                        ),
                        task: *task,
                        phase: if *task == 61 { w::LandingPhase::Turn } else { w::LandingPhase::Landed },
                        changes,
                        repositories,
                        landed,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::LandingsPanel::new(&w::CEILINGS, w::LandingsPanelParts { project: 1, turn, landings }).unwrap()
}
fn read_list(panel: &w::LandingsPanel, json: bool) -> jig::Printed {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(if json {
        &["landing", "list", "--project", "1", "--ended", "--json"]
    } else {
        &["landing", "list", "--project", "1", "--ended"]
    });
    let subject = landing::subject(None, 1, true, &root.limits);
    deliver(&mut root, subject, wire::landings_panel(panel));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert_eq!(root.watches.len(), 1);
    answer
}
#[test]
fn landings_print_live_first_then_newest() {
    let mut parts = list(Some(61), &[62, 60, 61, 63]).into_parts();
    let mut newest = parts.landings.get(3).unwrap().clone().into_parts();
    newest.phase = w::LandingPhase::Waiting;
    *parts.landings.get_mut(3).unwrap() = w::LandingLine::new(&w::CEILINGS, newest).unwrap();
    let answer = read_list(&w::LandingsPanel::new(&w::CEILINGS, parts).unwrap(), false);
    assert!(contains(&answer.stdout, "train: work/hello on forge 1 into release"));
    let mut previous = 0;
    for prefix in ["\nlanding 63 ·", "\nlanding 61 ·", "\nlanding 62 ·", "\nlanding 60 ·"] {
        let at = skein_lib::bytes::find(&answer.stdout, prefix.as_bytes()).unwrap();
        assert!(at > previous);
        previous = at;
    }
}
#[test]
fn landings_with_none_print_no_landings() {
    assert_eq!(
        read_list(&list(None, &[]), false).stdout.as_ref(),
        "landings of project 1 · turn: free\nno landings\n".as_bytes()
    );
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(if json {
            &["landing", "list", "--project", "1", "--json"]
        } else {
            &["landing", "list", "--project", "1"]
        });
        root.end(call, &landing::subject(None, 1, false, &root.limits), c::WatchEnd::Unknown);
        assert_eq!(root.answer(call).unwrap().stdout, read_list(&list(None, &[]), json).stdout);
    }
}
#[test]
fn landing_json_prints_the_panel_record() {
    f::check_golden("landings_live", &read_list(&list(Some(61), &[60, 61]), false).stdout);
    f::check_golden("landing_json", &read_panel(&panel(w::LandingPhase::Proving), true).stdout);
    f::check_golden("landings_json", &read_list(&list(Some(61), &[60, 61]), true).stdout);
}
#[test]
fn an_absent_project_reads_the_task_and_a_project_name_resolves_through_me() {
    let mut root = f::signed_in(f::PARTY);
    root.pending_reads.push(1);
    let call = root.submit(&["landing", "show", "61"]);
    assert_eq!(root.reads[&call].1, c::Read::Task { task: 61 });
    let task = f::goal(61, c::WaitsOn::Nothing);
    root.serve(call, c::ReadResult::Value(c::ReadValue::Task(Box::new(task))));
    let subject = landing::subject(Some(61), 1, false, &root.limits);
    deliver(&mut root, subject, wire::landing_panel(&panel(w::LandingPhase::Proving)));
    assert_eq!(root.answer(call).unwrap().status, 0);
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).unwrap().name = f::document_text(b"work");
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(&["landing", "show", "61", "--project", "work"]);
    let subject = landing::subject(Some(61), 1, false, &root.limits);
    deliver(&mut root, subject, wire::landing_panel(&panel(w::LandingPhase::Proving)));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn landing_calls_round_trip_and_reject_waits_and_zero_tasks() {
    for argv in [
        &["landing", "show", "61"][..],
        &["landing", "show", "61", "--project", "work"],
        &["landing", "list", "--project", "1", "--ended"],
    ] {
        let call = f::call(argv);
        assert_eq!(f::printed(&call), jig::call::Parsed::Call { call: call.clone(), json: false });
        assert_eq!(f::from_json(&jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
    }
    for argv in [
        &["landing", "show", "0"][..],
        &["wait", "landing", "61"],
        &["landings"],
        &["landing", "show", "61", "--key", ""],
    ] {
        f::parse(argv).unwrap_err();
    }
}
#[test]
fn help_landing() {
    f::check_golden("help_landing_show", &f::help(Some(57)));
}
#[test]
fn help_landings() {
    f::check_golden("help_landing_list", &f::help(Some(58)));
}

#[test]
fn phases_preparation_and_each_turn_holder_print_in_the_owners_words() {
    for (phase, word) in [
        (w::LandingPhase::Waiting, "waiting"),
        (w::LandingPhase::Preparing, "preparing"),
        (w::LandingPhase::Proving, "proving"),
        (w::LandingPhase::Proved, "proved"),
        (w::LandingPhase::Turn, "in its turn"),
        (w::LandingPhase::Landed, "landed"),
        (w::LandingPhase::Held, "held"),
        (w::LandingPhase::Cancelled, "cancelled"),
    ] {
        let answer = read_panel(&panel(phase), false);
        assert!(
            answer
                .stdout
                .strip_prefix("landing 61 · project 1 · ".as_bytes())
                .unwrap()
                .strip_prefix(word.as_bytes())
                .unwrap()
                .starts_with(b"\n")
        );
    }
    for (turn, words) in
        [(None, "turn: free\n"), (Some(61), "turn: this landing's\n"), (Some(60), "turn: landing 60's\n")]
    {
        let mut parts = panel(w::LandingPhase::Waiting).into_parts();
        parts.turn = turn;
        let mut member = parts.members.get(0).unwrap().clone().into_parts();
        member.candidate = None;
        member.base = None;
        member.pull = None;
        parts.members = List::with_capacity(1);
        parts.members.push(w::Member::new(&w::CEILINGS, member).unwrap()).unwrap();
        let answer = read_panel(&w::LandingPanel::new(&w::CEILINGS, parts).unwrap(), false);
        assert!(contains(&answer.stdout, "  candidate: not prepared\n"));
        assert!(contains(&answer.stdout, words));
        assert!(!contains(&answer.stdout, "pull request"));
    }
}
#[test]
fn a_line_can_name_several_changes_repositories_and_landed_commits() {
    let mut parts = list(None, &[61]).into_parts();
    let mut line = parts.landings.get(0).unwrap().clone().into_parts();
    line.changes = List::with_capacity(2);
    line.changes.push(51).unwrap();
    line.changes.push(52).unwrap();
    line.repositories = List::with_capacity(2);
    line.repositories.push(b"work/hello".as_slice().into()).unwrap();
    line.repositories.push(b"work/lib".as_slice().into()).unwrap();
    line.landed = List::with_capacity(2);
    line.landed.push([0x11; 32]).unwrap();
    line.landed.push([0x22; 32]).unwrap();
    parts.landings = List::with_capacity(1);
    parts.landings.push(w::LandingLine::new(&w::CEILINGS, line).unwrap()).unwrap();
    let answer = read_list(&w::LandingsPanel::new(&w::CEILINGS, parts).unwrap(), false);
    assert_eq!(answer.stdout.as_ref(), "landings of project 1 · turn: free\nlanding 61 · in its turn · change 51, 52 · work/hello, work/lib · train: work/hello on forge 1 into release · landed at 111111111111 · landed at 222222222222\n".as_bytes());
}

#[test]
fn unknown_task_and_malformed_panels_finish_the_read_and_close_the_watch() {
    for why in [c::ReadNone::Unknown, c::ReadNone::Gone] {
        let mut root = f::signed_in(f::PARTY);
        root.pending_reads.push(1);
        let call = root.submit(&["landing", "show", "61"]);
        root.serve(call, c::ReadResult::None(why));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        assert_eq!(answer.stderr.as_ref(), b"no landing for task 61\n");
    }
    let mut root = f::signed_in(f::PARTY);
    root.pending_reads.push(1);
    let call = root.submit(&["landing", "show", "61"]);
    root.serve(call, c::ReadResult::Value(c::ReadValue::Task(Box::new(f::goal(60, c::WaitsOn::Nothing)))));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 1);
    assert_eq!(answer.stderr.as_ref(), b"unexpected task document\n");
    for wrong_task in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["landing", "show", "61", "--project", "1"]);
        let mut parts = panel(w::LandingPhase::Proving).into_parts();
        if wrong_task {
            parts.task = 60;
        }
        let bytes = if wrong_task {
            wire::landing_panel(&w::LandingPanel::new(&w::CEILINGS, parts).unwrap())
        } else {
            Box::default()
        };
        let subject = landing::subject(Some(61), 1, false, &root.limits);
        deliver(&mut root, subject, bytes);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 1);
        assert_eq!(answer.stderr.as_ref(), b"unexpected landing document\n");
        assert_eq!(root.watches.len(), 1);
    }
}
#[test]
fn panel_printing_refuses_one_byte_past_the_text_window() {
    let panel = panel(w::LandingPhase::Proving);
    for json in [false, true] {
        let mut limits = f::limits_root();
        let answer = landing::print(&panel, json, &limits);
        limits.tools.text = u32::try_from(answer.stdout.len()).unwrap();
        assert_eq!(landing::print(&panel, json, &limits).status, 0);
        limits.tools.text = limits.tools.text.checked_sub(1).unwrap();
        let answer = landing::print(&panel, json, &limits);
        assert_eq!(answer.status, 1);
        assert!(contains(&answer.stderr, "result limit"));
    }
}

fn contains(bytes: &[u8], part: &str) -> bool {
    skein_lib::bytes::find(bytes, part.as_bytes()).is_some()
}

#[test]
fn an_invalid_direct_call_prints_invalid_landing_call() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let mut call = f::call(&["landing", "show", "61"]);
        *call.values.get_mut(0).unwrap() = Some(jig::call::Values::One(jig::call::Value::Number(0)));
        let number = root.submit_prepared(call, json, None);
        let answer = root.answer(number).unwrap();
        assert_eq!(answer.status, 1);
        assert!(answer.stdout.is_empty());
        assert_eq!(
            answer.stderr.as_ref(),
            if json { b"{\"error\":\"invalid landing call\"}".as_slice() } else { b"invalid landing call\n" }
        );
        assert_eq!(root.watches.len(), 1, "only me remains watched");
    }
}

#[test]
fn a_refused_landing_or_landings_watch_prints_not_allowed() {
    for (argv, task) in [
        (&["landing", "show", "61", "--project", "1"][..], Some(61)),
        (&["landing", "list", "--project", "1"][..], None),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(argv), json, None);
            let subject = landing::subject(task, 1, false, &root.limits);
            root.end(call, &subject, c::WatchEnd::NotAllowed);
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 2);
            assert!(answer.stdout.is_empty());
            assert_eq!(
                answer.stderr.as_ref(),
                if json { b"{\"error\":\"not_allowed\"}".as_slice() } else { b"not_allowed\n" }
            );
            assert_eq!(root.watches.len(), 1, "only me remains watched");
        }
    }
}

#[test]
fn wrong_change_tasks_and_wrong_landing_projects_remain_document_errors() {
    let change = wire::change_panel(&super::change::panel(w::Step::Checking));
    let mut parts = panel(w::LandingPhase::Landed).into_parts();
    parts.project = 2;
    let wrong_project = wire::landing_panel(&w::LandingPanel::new(&w::CEILINGS, parts).unwrap());
    for json in [false, true] {
        for bytes in [change.clone(), wrong_project.clone(), Box::default()] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(&["landing", "show", "61", "--project", "1"]), json, None);
            let subject = landing::subject(Some(61), 1, false, &root.limits);
            deliver(&mut root, subject, bytes);
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 1);
            assert!(answer.stdout.is_empty());
            assert_eq!(
                answer.stderr.as_ref(),
                if json {
                    b"{\"error\":\"unexpected landing document\"}".as_slice()
                } else {
                    b"unexpected landing document\n"
                }
            );
            assert_eq!(root.watches.len(), 1);
        }
    }
}

#[test]
fn ended_landings_show_round_evidence_and_outcomes_in_text_and_json() {
    let mut parts = panel(w::LandingPhase::Landed).into_parts();
    parts.train = Some(
        w::Train::new(
            &w::CEILINGS,
            w::TrainParts {
                forge: 1,
                repository: 9,
                owner: b"work".as_slice().into(),
                name: b"hello".as_slice().into(),
                branch: b"release".as_slice().into(),
            },
        )
        .unwrap(),
    );
    parts.round = Some(
        w::LandingRound::new(
            &w::CEILINGS,
            w::LandingRoundParts {
                number: 2,
                branch: b"temper/61/2".as_slice().into(),
                pull: Some(
                    w::Pull::new(
                        &w::CEILINGS,
                        w::PullParts {
                            number: 8,
                            address: b"https://forge.example/work/hello/pulls/8".as_slice().into(),
                        },
                    )
                    .unwrap(),
                ),
                combined: Some(w::Combined::Squash),
                step: w::LandingStep::Landed,
                upstream: Some([0x44; 32]),
                failure: Box::default(),
            },
        )
        .unwrap(),
    );
    let mut member = parts.members.get(0).unwrap().clone().into_parts();
    member.head = Some([0x55; 32]);
    member.landing = b"release".as_slice().into();
    member.judgement = Some(
        w::PrefixJudgement::new(
            &w::CEILINGS,
            w::PrefixJudgementParts {
                prefix: [0x11; 32],
                verdict: w::Ci::Passed,
                gates: List::from_boxed(Box::new([w::PrefixGate::new(
                    &w::CEILINGS,
                    w::PrefixGateParts { number: 3, blocking: true, verdict: w::Ci::Passed },
                )
                .unwrap()])),
            },
        )
        .unwrap(),
    );
    parts.members = List::from_boxed(Box::new([w::Member::new(&w::CEILINGS, member.clone()).unwrap()]));
    let landed = w::LandingPanel::new(&w::CEILINGS, parts.clone()).unwrap();
    let text = read_panel(&landed, false);
    for words in [
        "train: work/hello on forge 1 into release",
        "round 2: temper/61/2 · pull request #8",
        " · squash · step: landed",
        "  round head: 55555555",
        "  landed: release at 11111111",
        "  candidate 11111111",
        " on release 22222222",
        "  judgement at 11111111",
        "    gate 3, blocking: passed",
    ] {
        assert!(contains(&text.stdout, words), "{words}");
    }
    let json = read_panel(&landed, true);
    for words in [
        "\"combined\":\"squash\"",
        "\"step\":\"landed\"",
        "\"landed\":\"1111111111111111111111111111111111111111111111111111111111111111\"",
        "\"number\":8",
        "\"prefix\":\"11111111",
        "\"head\":\"55555555",
        "\"branch\":\"release\"",
    ] {
        assert!(contains(&json.stdout, words), "{words}");
    }
    parts.phase = w::LandingPhase::Cancelled;
    member.landed = None;
    member.left = Some(w::LandingLeave::Failed(
        w::FailedGate::new(&w::CEILINGS, w::FailedGateParts { gate: 0, run: 7 }).unwrap(),
    ));
    parts.members = List::from_boxed(Box::new([w::Member::new(&w::CEILINGS, member).unwrap()]));
    let left = w::LandingPanel::new(&w::CEILINGS, parts).unwrap();
    assert!(contains(&read_panel(&left, false).stdout, "left: gate 0 failed in run 7"));
    assert!(contains(&read_panel(&left, true).stdout, "\"left\":{\"failed\":{\"gate\":0,\"run\":7}}"));
}

#[test]
fn a_stack_shows_each_head_prefix_and_outcome_in_order_in_text_and_json() {
    let mut parts = panel(w::LandingPhase::Proving).into_parts();
    let template = parts.members.get(0).unwrap().clone().into_parts();
    parts.members = List::with_capacity(3);
    for offset in 0_u8..3 {
        let mut member = template.clone();
        member.change = 51 + u64::from(offset);
        member.branch = format!("temper/{}", member.change).into_bytes().into();
        member.head = Some([0x30 + offset; 32]);
        member.candidate = Some([0x40 + offset; 32]);
        member.landed = if offset == 0 { member.candidate } else { None };
        member.left = if offset == 2 {
            Some(w::LandingLeave::HeadMoved(
                w::MovedHead::new(&w::CEILINGS, w::MovedHeadParts { head: [0x99; 32] }).unwrap(),
            ))
        } else {
            None
        };
        member.gates = List::with_capacity(0);
        parts.members.push(w::Member::new(&w::CEILINGS, member).unwrap()).unwrap();
    }
    let panel = w::LandingPanel::new(&w::CEILINGS, parts).unwrap();
    let text = read_panel(&panel, false);
    let text = core::str::from_utf8(&text.stdout).unwrap();
    let json = read_panel(&panel, true);
    let json = core::str::from_utf8(&json.stdout).unwrap();
    let mut text_position = 0;
    let mut json_position = 0;
    for offset in 0_u8..3 {
        let change = 51 + u64::from(offset);
        let at = text.find(&format!("member: change {change}")).expect("member text");
        assert!(at >= text_position);
        text_position = at;
        let at = json.find(&format!("\"change\":{change}")).expect("member JSON");
        assert!(at >= json_position);
        json_position = at;
        let head = format!("{:02x}", 0x30 + offset).repeat(32);
        let prefix = format!("{:02x}", 0x40 + offset).repeat(32);
        assert!(text.contains(&format!("round head: {head}")));
        assert!(text.contains(&format!("candidate {prefix}")));
        assert!(json.contains(&format!("\"head\":\"{head}\"")));
        assert!(json.contains(&format!("\"candidate\":\"{prefix}\"")));
    }
    assert!(text.contains("landed: main at 40404040"));
    assert!(text.contains("left: head moved to 99999999"));
    assert!(json.contains("\"landed\":\"40404040"));
    assert!(json.contains("\"head_moved\":\"99999999"));
}
