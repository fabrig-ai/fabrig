use super::fixtures as f;
use crate::{follow, jig};
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};
#[test]
fn follow_act_commands_send_keyed_requests_and_print_kept_or_the_entrys_refusal() {
    let a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    for command in ["start", "hold", "skip", "merge"] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let mut words = List::with_capacity(16);
            for word in ["follow", command, "--project", "2", "1", a, "--key", "follow-act"] {
                words.push(word).unwrap();
            }
            match command {
                "hold" | "skip" => {
                    words.push("--reason").unwrap();
                    words.push("waits for the handoff fix").unwrap();
                }
                "merge" => {
                    words.push(b).unwrap();
                }
                "start" => {}
                _ => unreachable!(),
            }
            if json {
                words.push("--json").unwrap();
            }
            let call = root.submit(words.as_slice());
            let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
            assert_eq!(*scope, c::Scope::Project(2));
            let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
            let w::Request::FollowAct(request) = request.into_parts().request else { panic!("follow act") };
            assert_eq!((request.project(), request.follow()), (2, 1));
            match (command, request.act()) {
                ("start", w::FollowAct::Start(value)) => assert_eq!(
                    *value.after(),
                    [
                        0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                        0xaa, 0xaa, 0xaa, 0xaa, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                    ]
                ),
                ("hold", w::FollowAct::Hold(value)) | ("skip", w::FollowAct::Skip(value)) => {
                    assert_eq!(value.reason(), b"waits for the handoff fix");
                }
                ("merge", w::FollowAct::Merge(value)) => {
                    assert_eq!(value.afters().len(), 2);
                    assert_eq!(value.afters().iter().nth(1).unwrap()[0], 0xbb);
                }
                other => panic!("wrong act: {other:?}"),
            }
            root.adoption_answer(
                call,
                w::Outcome::Follow(
                    w::FollowOutcome::new(&w::CEILINGS, w::FollowOutcomeParts { refusal: None }).unwrap(),
                ),
            );
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 0);
            assert_eq!(printed.stdout.as_ref(), if json { b"{\"kept\":{}}".as_slice() } else { b"kept\n" });
            let mut refused = f::signed_in(f::PARTY);
            let call = refused.submit(words.as_slice());
            let reason = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: the change has started producing";
            refused.adoption_answer(
                call,
                w::Outcome::Follow(
                    w::FollowOutcome::new(
                        &w::CEILINGS,
                        w::FollowOutcomeParts { refusal: Some(reason.as_bytes().into()) },
                    )
                    .unwrap(),
                ),
            );
            let printed = refused.answer(call).unwrap();
            assert_eq!(printed.status, 2);
            assert!(skein_lib::bytes::find(&printed.stderr, reason.as_bytes()).is_some());
        }
    }
}
fn row() -> w::FollowView {
    let mut paths = List::with_capacity(2);
    paths.push(b"src".as_slice().into()).unwrap();
    paths.push(b"include".as_slice().into()).unwrap();
    let setting = w::Follow::new(
        &w::CEILINGS,
        w::FollowParts {
            source: w::FollowSource::new(
                &w::CEILINGS,
                w::FollowSourceParts {
                    repository: w::FollowRepository::new(
                        &w::CEILINGS,
                        w::FollowRepositoryParts { forge: 1, repository: 12 },
                    )
                    .unwrap(),
                    branch: b"main".as_slice().into(),
                    paths,
                },
            )
            .unwrap(),
            target: w::FollowTarget::new(
                &w::CEILINGS,
                w::FollowTargetParts {
                    repository: w::FollowRepository::new(
                        &w::CEILINGS,
                        w::FollowRepositoryParts { forge: 1, repository: 13 },
                    )
                    .unwrap(),
                    branch: b"main".as_slice().into(),
                },
            )
            .unwrap(),
            owner: w::FollowOwner::Person(
                w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person: 4 }).unwrap(),
            ),
            start: w::FollowStart::Owner,
            producer: Some(b"6;3.1;".as_slice().into()),
            verification: w::FollowCheck::new(
                &w::CEILINGS,
                w::FollowCheckParts {
                    workflow: b"tests.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: w::FollowFormat::Junit,
                },
            )
            .unwrap(),
        },
    )
    .unwrap();
    w::FollowView::new(
        &w::CEILINGS,
        w::FollowViewParts {
            number: 1,
            setting,
            source: b"org/vsr".as_slice().into(),
            source_branch: b"main".as_slice().into(),
            target: b"org/vsr-rs".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            task: Some(71),
            stopped: None,
            lag: 2,
            entries: List::with_capacity(0),
            skips: List::with_capacity(0),
            parity: None,
        },
    )
    .unwrap()
}
fn panel(rows: &[w::FollowView]) -> w::FollowsPanel {
    let mut follows = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        follows.push(row.clone()).unwrap();
    }
    w::FollowsPanel::new(
        &w::CEILINGS,
        w::FollowsPanelParts { project: 2, follows, dependencies: List::with_capacity(0) },
    )
    .unwrap()
}
fn read(panel: &w::FollowsPanel, show: bool, json: bool) -> jig::Printed {
    read_as(panel, show, json, f::me(f::PARTY))
}
fn read_as(panel: &w::FollowsPanel, show: bool, json: bool, me: c::Me) -> jig::Printed {
    let mut root = f::signed_in(f::PARTY);
    root.deliver(c::Delivery::Me(Box::new(me)));
    let words: &[&str] = match (show, json) {
        (false, false) => &["follow", "list", "--project", "2"],
        (true, false) => &["follow", "show", "--project", "2", "1"],
        (false, true) => &["follow", "list", "--project", "2", "--json"],
        (true, true) => &["follow", "show", "--project", "2", "1", "--json"],
    };
    let call = root.submit(words);
    let token = root.watch_token(&follow::subject(2, &root.limits));
    root.client(client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch: token,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::follows_panel(panel),
            },
        },
    });
    assert_eq!(root.watches.len(), 1, "one-shot read retires its watch");
    root.answer(call).unwrap()
}
fn parity(verdict: w::FollowVerdict) -> w::FollowParity {
    w::FollowParity::new(
        &w::CEILINGS,
        w::FollowParityParts {
            source: [0xab; 32],
            head: [0xcd; 32],
            verdict,
            run: b"https://forge.example/run/2".as_slice().into(),
        },
    )
    .unwrap()
}
#[test]
fn follow_list_prints_the_binding_lag_and_every_parity() {
    for (lag, verdict, expected) in [
        (2, None, "2 behind · no check yet"),
        (0, Some(w::FollowVerdict::Passed), "caught up · check passes at abababababab"),
        (
            1,
            Some(w::FollowVerdict::Failing(
                w::FollowFailing::new(&w::CEILINGS, w::FollowFailingParts { count: 3 }).unwrap(),
            )),
            "1 behind · 3 failing at abababababab",
        ),
        (7, Some(w::FollowVerdict::Many), "7 behind · many failing at abababababab"),
    ] {
        let mut parts = row().into_parts();
        parts.lag = lag;
        parts.parity = match verdict {
            Some(verdict) => Some(parity(verdict)),
            None => None,
        };
        let answer = read(&panel(&[w::FollowView::new(&w::CEILINGS, parts).unwrap()]), false, false);
        assert_eq!(answer.status, 0);
        assert_eq!(
            answer.stdout,
            skein_lib::bytes::concat(&[
                "follow 1: org/vsr main → org/vsr-rs main · ".as_bytes(),
                expected.as_bytes(),
                b"\n"
            ])
        );
    }
}
#[test]
fn follow_show_prints_the_whole_setting_and_oldest_first_entries() {
    let mut parts = row().into_parts();
    parts.entries = List::with_capacity(7);
    let words = w::FollowWords::new(&w::CEILINGS, w::FollowWordsParts { words: b"account".as_slice().into() }).unwrap();
    for (index, state) in [
        w::FollowState::NotMade,
        w::FollowState::WaitsWord,
        w::FollowState::HeldByOwner(words.clone()),
        w::FollowState::WaitsFor(w::FollowWait::new(&w::CEILINGS, w::FollowWaitParts { change: 80 }).unwrap()),
        w::FollowState::Producing,
        w::FollowState::Held(words.clone()),
        w::FollowState::Ended(words),
    ]
    .into_iter()
    .enumerate()
    {
        // Deliberately descending commits: order belongs to the queue, not its hashes.
        parts
            .entries
            .push(
                w::FollowEntry::new(
                    &w::CEILINGS,
                    w::FollowEntryParts {
                        after: [u8::try_from(7 - index).unwrap(); 32],
                        change: if index == 0 { None } else { Some(81) },
                        state,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    parts.lag = 7;
    parts.skips = List::with_capacity(1);
    parts
        .skips
        .push(
            w::FollowSkip::new(
                &w::CEILINGS,
                w::FollowSkipParts { after: [0x11; 32], reason: b"not ported".as_slice().into() },
            )
            .unwrap(),
        )
        .unwrap();
    parts.parity = Some(parity(w::FollowVerdict::Many));
    let panel = panel(&[w::FollowView::new(&w::CEILINGS, parts).unwrap()]);
    let answer = read(&panel, true, false);
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), "follow 1: org/vsr main → org/vsr-rs main · 7 behind · many failing at abababababab\nsource: org/vsr (1/12)\nbranch: main\npaths: src include\ntarget: org/vsr-rs (1/13)\nbranch: main\nowner: person 4\nstart: owner\nproducer: 6;3.1;\ncheck: tests.yml · artifact results · junit\ntask: 71\nlag: 7\n  070707070707 · not made yet\n  060606060606 · change 81 · waits for its owner's word\n  050505050505 · change 81 · held by its owner: account\n  040404040404 · change 81 · waits for change 80\n  030303030303 · change 81 · producing as change 81\n  020202020202 · change 81 · held: account\n  010101010101 · change 81 · ended without landing: account\nskips:\n  111111111111 · not ported\nparity: many failing at abababababab · target cdcdcdcdcdcd · https://forge.example/run/2\n".as_bytes());
    let json = read(&panel, true, true);
    assert_eq!(json.status, 0);
    for expected in [
        b"\"source\":{\"forge\":1,\"repository\":12".as_slice(),
        b"\"state\":\"producing as change 81\"",
        b"\"verdict\":\"many\"",
    ] {
        assert!(skein_lib::bytes::find(&json.stdout, expected).is_some());
    }
}
#[test]
fn follow_reads_handle_empty_missing_refused_and_named_projects() {
    assert_eq!(read(&panel(&[]), false, false).stdout.as_ref(), b"no follows\n");
    assert_eq!(read(&panel(&[]), true, false).status, 2);
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["follow", "list", "--project", "2"]);
    root.end(call, &follow::subject(2, &root.limits), c::WatchEnd::NotAllowed);
    let refused = root.answer(call).unwrap();
    assert_eq!(refused.status, 2);
    assert_eq!(refused.stderr.as_ref(), b"not_allowed\n");
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).unwrap().name = f::document_text(b"work");
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(&["follow", "list", "--project", "work"]);
    root.end(call, &follow::subject(1, &root.limits), c::WatchEnd::Unknown);
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"no follows\n");
    for argv in [&["follow", "list", "--project", "2"][..], &["follow", "show", "1", "--project", "work"][..]] {
        let call = f::call(argv);
        assert_eq!(f::from_json(&jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
    }
}

#[test]
fn follow_list_prints_dependencies_after_follows_naming_only_the_readers_projects() {
    for has_follow in [false, true] {
        for member in [false, true] {
            let mut me = f::me(f::PARTY);
            let mut project = me.projects.get(0).unwrap().clone();
            project.project = 2;
            project.name = f::document_text(b"fabrig");
            me.projects = List::with_capacity(2);
            me.projects.push(project.clone()).unwrap();
            if member {
                project.project = 7;
                project.name = f::document_text(b"vsr-rs");
                me.projects.push(project).unwrap();
            }
            let rows = [row()];
            let mut parts = panel(if has_follow { &rows } else { &[] }).into_parts();
            parts.dependencies = List::with_capacity(1);
            parts
                .dependencies
                .push(w::DependencyView::new(&w::CEILINGS, w::DependencyViewParts { number: 1, project: 7 }).unwrap())
                .unwrap();
            let panel = w::FollowsPanel::new(&w::CEILINGS, parts).unwrap();
            let answer = read_as(&panel, false, false, me.clone());
            assert_eq!(answer.status, 0);
            let prefix = if has_follow {
                "follow 1: org/vsr main → org/vsr-rs main · 2 behind · no check yet\n"
            } else {
                "no follows\n"
            };
            let expected = if member {
                "dependency 1: fabrig depends on vsr-rs\n"
            } else {
                "dependency 1: fabrig depends on project 7\n"
            };
            assert_eq!(answer.stdout, skein_lib::bytes::concat(&[prefix.as_bytes(), expected.as_bytes()]));
            let json = read_as(&panel, false, true, me);
            assert_eq!(json.status, 0);
            assert!(skein_lib::bytes::find(&json.stdout, br#""dependencies":[{"number":1,"project":7}]"#).is_some());
            assert!(
                skein_lib::bytes::find(&json.stdout, b"vsr-rs").is_none() || has_follow,
                "JSON carries no project names"
            );
        }
    }
}
